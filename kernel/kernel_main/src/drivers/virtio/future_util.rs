use alloc::{sync::Weak, sync::Arc};
use core::task::{Poll, Context};
use core::future::Future;
use core::task::Waker;
use core::pin::Pin;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use kernel_lock::shared::RwLock;

use crate::drivers::traits::block::BlockRequestFutureBuffer;
use crate::interrupt_context_waker::InterruptContextWaker;
use crate::unsafe_buffer::UnsafeSlice;
use alloc::boxed::Box;

use super::{VirtioDevice, VirtioDeviceType};

struct RequestFuture<RequestMeta, Driver: WrappedVirtioDeviceType> {
    driver: Weak<RwLock<FutureVirtioDeviceType<Driver>>>,
    header: (),

    pub buffer: Option<BlockRequestFutureBuffer>,
    pub meta: RequestMeta,
    pub descriptor_id: Option<u16>,
    pub was_queued: bool,
}


impl<RequestMeta, Driver: WrappedVirtioDeviceType> Future for RequestFuture<RequestMeta, Driver> {
    type Output = Option<BlockRequestFutureBuffer>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<<Self as Future>::Output>{
        let device = self.driver.upgrade().unwrap();
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
impl<T: WrappedVirtioDeviceType> FutureVirtioDeviceType<T> {
    /// Sets up a callback future for when the device has finished processing a request we made
    fn poll_device(&mut self) {
        let result = {
            let mut device_ref = self.device_type.device().lock();
            let this_weak = self.this.clone();
            // Note that here we aren't polling the BlockDeviceRequest, but rather the VirtioDevice
            // (where polling means waiting for a used ring to be available)
            let waker = Arc::new(InterruptContextWaker(Box::new(move || {
                // Now we can recreate self based on the weak pointer we moved
                // and then poll it again.
                // The value should be Ready now
                let _this = this_weak.upgrade().unwrap().write().poll_device();
            })));
            Pin::new(&mut *device_ref).poll(&mut Context::from_waker(&waker.into()))
        };

        if let Poll::Ready(queue_idx) = result {
            assert!(queue_idx == 0);

            // Lock the vqueue
            let mut vq_lock = self.request_virtqueue.lock();

            // Create the iterator for this descriptor chain
            let descriptor_chain_data_iterator = vq_lock.pop_used_element_to_iterator();
            let descriptor_id = descriptor_chain_data_iterator.pointed_chain.unwrap();
            
            // Todo: Reconstruct whether this was a ReadInto or WriteFrom variant
            let mut components = descriptor_chain_data_iterator.map(|s| unsafe {
                BlockRequestFutureBuffer::WriteFrom(UnsafeSlice::new(
                    core::slice::from_raw_parts_mut(s.as_ptr() as *const u8, s.len()),
                ))
            });
            let buffer_box = components.nth(1).unwrap();

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
            vq_lock.free_descriptor_chain(descriptor_id)
        } else {
            // It's pending, but we will be woken up eventually
        }
    }

    /// Returns None if buffer doesn't exist (which meanst that the request was never done OR that it has completed)
    pub fn take_buffer(&mut self, descriptor_id: &u16) -> Option<BlockRequestFutureBuffer> {
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


pub trait WrappedVirtioDeviceType {
	type RequestMeta;
	type RequestBuildingData;
    
    type Trait;
	
	fn create_request(&mut self, data: Self::RequestBuildingData) -> RequestFuture<Self::RequestMeta, Self> where Self: Sized;
    fn device(&mut self) -> &mut VirtioDevice;
	fn build_descriptor_chain(&mut self);
}

struct FutureVirtioDeviceType<T: WrappedVirtioDeviceType> {
    pub this: Weak<RwLock<Self>>,
    
    pub device_type: T,

    // A map between descriptor IDs and Wakers
    waiting_requests: BTreeMap<u16, Vec<Waker>>,
    // A map between descriptor IDs and Buffers
    header_buffers: BTreeMap<u16, BlockRequestFutureBuffer>,
}

impl<T: WrappedVirtioDeviceType> VirtioDeviceType for FutureVirtioDeviceType<T> {
    type Trait = T::Trait;
    
    fn configure(device: Arc<kernel_lock::shared::Mutex<VirtioDevice>>) -> Result<Arc<RwLock<Self::Trait>>, ()>
    where
            Self: Sized {
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
    }
}
