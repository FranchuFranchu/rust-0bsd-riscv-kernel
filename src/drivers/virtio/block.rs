use alloc::{
    boxed::Box,
    collections::BTreeMap,
    sync::{Arc, Weak},
    task::Wake,
    vec::Vec,
};
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Waker},
};

use super::{SplitVirtqueue, VirtioDevice, VirtioDeviceType};
use crate::{interrupt_context_waker::InterruptContextWaker, lock::shared::{Mutex, RwLock}};

// See https://docs.oasis-open.org/virtio/virtio/v1.1/cs01/virtio-v1.1-cs01.html#x1-2440004
// section 5.2.6

// This is followed by data and a status bit

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct RequestHeader {
    r#type: u32,
    reserved: u32,
    sector: u64,
}

#[derive(Debug)]
pub struct VirtioBlockDevice {
    request_virtqueue: Mutex<SplitVirtqueue>,
    device: Arc<Mutex<VirtioDevice>>,

    /// A weak pointer to itself. This has to be used when callbacks need to use self later on (when the &mut self has expired)
    pub this: Weak<RwLock<Self>>,

    // A map between descriptor IDs and Wakers
    waiting_requests: BTreeMap<u16, Vec<Waker>>,
    // A map between descriptor IDs and Buffers
    header_buffers: BTreeMap<u16, Box<[u8]>>,
}

pub struct BlockRequestFuture {
    device: Weak<RwLock<VirtioBlockDevice>>,
    header: RequestHeader,
    // The buffer is moved out when block operation is being carried out, and then it's moved
    // back in when it's done (AFTER the future is poll()'d).
    pub buffer: Option<Box<[u8]>>,
    pub descriptor_id: Option<u16>,
    pub was_queued: bool,
}

use core::task::Poll;

impl Future for BlockRequestFuture {
    type Output = Option<Box<[u8]>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<<Self as Future>::Output> {
        let device = self.device.upgrade().unwrap();
        let mut device = device.write();
        if self.buffer.is_none() {
            // Check if the driver has been done yet
            if let Some(buffer) = device.take_buffer(&self.descriptor_id.unwrap()) {
                self.buffer = Some(buffer);
                Poll::Ready(self.buffer.take())
            } else {
                device.register_waker(&self.descriptor_id.unwrap(), cx.waker().clone());
                Poll::Pending
            }
        } else if self.was_queued {
            // the block operation is done
            Poll::Ready(self.buffer.take())
        } else {
            // the block operation hasn't been started yet
            self.was_queued = true;

            self.descriptor_id = Some(device.do_request(&mut self));
            // Register ourselves as a waker
            device.register_waker(&self.descriptor_id.unwrap(), cx.waker().clone());

            // Release lock to prevent deadlocks
            drop(device);
            self.device
                .upgrade()
                .unwrap()
                .write()
                .begin_request(&self.descriptor_id.unwrap());

            Poll::Pending
        }
    }
}

use crate::drivers::traits::block::{AnyRequestFuture, BlockDevice};

impl BlockDevice for VirtioBlockDevice {
    fn _create_request(
        &self,
        sector: u64,
        buffer: Box<[u8]>,
        write: bool,
    ) -> Box<dyn AnyRequestFuture + Unpin + Send + Sync> {
        Box::new(BlockRequestFuture {
            device: self.this.clone(),
            header: RequestHeader {
                r#type: if write { 1 } else { 0 },
                reserved: 0,
                sector,
            },
            buffer: Some(buffer),
            descriptor_id: None,
            was_queued: false,
        })
    }
}

impl VirtioBlockDevice {
    fn instance_configure(&self) {
        self.device.lock().driver_ok();
    }

    pub fn do_request(&mut self, future: &mut BlockRequestFuture) -> u16 {
        let mut vq_lock = self.request_virtqueue.lock();

        let status = alloc::vec![0xFFu8; 1].into_boxed_slice();

        let mut last = vq_lock.new_descriptor_from_boxed_slice(status, true, None);

        if future.header.r#type == 1 {
            last = vq_lock.new_descriptor_from_boxed_slice(
                future.buffer.take().unwrap(),
                false,
                Some(last),
            );
        } else {
            last = vq_lock.new_descriptor_from_boxed_slice(
                future.buffer.take().unwrap(),
                true,
                Some(last),
            );
        }
        last = vq_lock.new_descriptor_from_sized(&future.header, false, Some(last));

        // Return the head of the descriptor chain
        last
    }

    pub fn begin_request(&mut self, descriptor_id: &u16) {
        let mut vq_lock = self.request_virtqueue.lock();
        // Make the descriptor chain available and notify the device that the virtqueue is ready
        vq_lock.make_available(*descriptor_id);
        self.device.lock().queue_ready(0);
    }

    /// Sets up a callback future for when the device has finished processing a request we made
    fn poll_device(&mut self) {
        let mut device_ref = self.device.lock();
        let this_weak = self.this.clone();
        // Note that here we aren't polling the BlockDeviceRequest, but rather the VirtioDevice
        // (where polling means waiting for a used ring to be available)
        let result = Pin::new(&mut *device_ref).poll(&mut Context::from_waker(
            &Arc::new(InterruptContextWaker(Box::new(move || {
                // Now we can recreate self based on the weak pointer we moved
                // and then poll it again.
                // The value should be Ready now
                let this = this_weak.upgrade().unwrap().write().poll_device();
            })))
            .into(),
        ));
        drop(device_ref);

        if let Poll::Ready(queue_idx) = result {
            assert!(queue_idx == 0);

            // Lock the vqueue
            let mut vq_lock = self.request_virtqueue.lock();

            // Create the iterator for this descriptor chain
            let mut descriptor_chain_data_iterator = vq_lock.pop_used_element_to_iterator();
            let descriptor_id = descriptor_chain_data_iterator.pointed_chain.unwrap();

            let data: Vec<u8> = descriptor_chain_data_iterator
                // Join all the &[u8]s together into one iterator
                .flatten()
                .copied()
                // Create a Vec<u8>
                .collect();

            let request_body = &data[core::mem::size_of::<RequestHeader>()..data.len() - 1];

            // Now, try to recreate the Box<[u8]> that was used to create this
            // Reconstruct the buffer box
            let buffer_start_ptr =
                descriptor_chain_data_iterator.nth(1).unwrap().as_ptr() as *mut u8;
            let buffer_len = request_body.len();
            // SAFETY: This is constructed on do_request, and I think this is the "correct" way to restore it
            let buffer_box = unsafe {
                Box::from_raw(core::slice::from_raw_parts_mut(
                    buffer_start_ptr,
                    buffer_len,
                ))
            };

            self.header_buffers.insert(descriptor_id, buffer_box);

            let items = self
                .waiting_requests
                .get_mut(&descriptor_id)
                .map(|vec| vec.iter_mut());

            if items.is_none() {
                info!("No one was waiting for this!");
                return;
            }

            let items = items.unwrap();

            for i in items.into_iter() {
                i.wake_by_ref();
            }

            self.waiting_requests.remove(&descriptor_id);
        } else {
            // It's pending, but we will be woken up eventually
        }
    }

    /// Returns None if buffer doesn't exist (which meanst that the request was never done OR that it has completed)
    pub fn take_buffer(&mut self, descriptor_id: &u16) -> Option<Box<[u8]>> {
        self.header_buffers.remove(descriptor_id)
    }
    pub fn register_waker(&mut self, descriptor_id: &u16, waker: Waker) {
        if let Some(v) = self.waiting_requests.get_mut(descriptor_id) {
            v.push(waker)
        } else {
            self.waiting_requests
                .insert(*descriptor_id, alloc::vec![waker]);
        }
    }
}

use crate::drivers::traits::block::GenericBlockDevice;

impl VirtioDeviceType for VirtioBlockDevice {
	type Trait = dyn GenericBlockDevice + Send + Sync + Unpin;

    fn configure(device: Arc<Mutex<VirtioDevice>>) -> Result<Arc<RwLock<Self::Trait>>, ()> {
        let q = device.lock().configure_queue(0);
        let dev = VirtioBlockDevice {
            request_virtqueue: Mutex::new(q),
            device,
            this: Weak::new(),
            waiting_requests: BTreeMap::new(),
            header_buffers: BTreeMap::new(),
        };
        let dev = Arc::new(RwLock::new(dev));
        dev.write().this = Arc::downgrade(&dev);

        // setup wakers and similar stuff
        dev.write().poll_device();

        let dev_clone = dev.clone();
        // Configure the instance later on (to prevent deadlocks)
        Arc::new(InterruptContextWaker(Box::new(move || {
            dev_clone.write().instance_configure();
        })))
        .wake();
        Ok(dev)
    }
    fn negotiate_features(device: &mut VirtioDevice) {
        device.get_device_features(); // ignore their features
        device.set_driver_features(0);
        device.accept_features().unwrap();
    }
    fn on_interrupt(&self) {
        //self.device.lock().on_interrupt();
    }
}
