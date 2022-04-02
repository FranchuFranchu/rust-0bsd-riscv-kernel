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
    task::{Context, Poll, Waker},
};

use kernel_lock::shared::{Mutex, RwLock};

use super::{SplitVirtqueue, VirtioDevice, VirtioDeviceType};
use crate::{
    drivers::traits::block::BlockRequestFutureBuffer,
    interrupt_context_waker::InterruptContextWaker, unsafe_buffer::UnsafeSlice,
};

pub struct RequestFuture<Driver: WrappedVirtioDeviceType + Send + Sync + 'static> {
    pub driver: Weak<RwLock<FutureVirtioDeviceType<Driver>>>,
    pub header: (),

    pub buffer: Option<BlockRequestFutureBuffer>,
    pub meta: Driver::RequestMeta,
    pub descriptor_id: Option<u16>,
    pub was_queued: bool,
}

impl<Driver: WrappedVirtioDeviceType + Send + Sync + 'static> Future for RequestFuture<Driver> {
    type Output = Option<BlockRequestFutureBuffer>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<<Self as Future>::Output> {
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

            self.descriptor_id = Some(device.device_type.do_request(&mut *self));
            // Register ourselves as a waker
            device.register_waker(&self.descriptor_id.unwrap(), cx.waker().clone());

            // Release lock to prevent deadlocks
            drop(device);
            self.driver
                .upgrade()
                .unwrap()
                .write()
                .device_type
                .begin_request(self.descriptor_id.unwrap());

            Poll::Pending
        }
    }
}
impl<Driver: WrappedVirtioDeviceType + Send + Sync + 'static> FutureVirtioDeviceType<Driver> {
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
            // Lock the vqueue
            let vq = self.device_type.get_virtqueue(queue_idx);
            let mut vq_lock = vq.lock();

            // Create the iterator for this descriptor chain
            if let Some(descriptor_chain_data_iterator) = vq_lock.pop_used_element_to_iterator() {
                let descriptor_id = descriptor_chain_data_iterator.pointed_chain.unwrap();

                // Todo: Reconstruct whether this was a ReadInto or WriteFrom variant
                let mut components = descriptor_chain_data_iterator.map(|s| unsafe {
                    BlockRequestFutureBuffer::WriteFrom(UnsafeSlice::new(
                        core::slice::from_raw_parts_mut(
                            s.as_ptr() as *const u8 as *mut u8,
                            s.len(),
                        ),
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
                vq_lock.free_descriptor_chain(descriptor_id);
            }
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

pub trait WrappedVirtioDeviceType: Send + Sync + 'static {
    type RequestMeta: Unpin;
    type RequestBuildingData;

    type Trait;

    fn create_request(&mut self, data: Self::RequestBuildingData) -> RequestFuture<Self>
    where
        Self: Sized;
    fn device(&self) -> &Mutex<VirtioDevice>;
    fn from_device(device: Arc<Mutex<VirtioDevice>>) -> Self;
    fn begin_request(&self, descriptor_id: u16);
    fn do_request(&self, request: &mut RequestFuture<Self>) -> u16
    where
        Self: Sized;
    fn get_virtqueue(&self, virtqueue_id: u16) -> &Mutex<SplitVirtqueue>;
    fn set_this(&mut self, this: Weak<RwLock<FutureVirtioDeviceType<Self>>>)
    where
        Self: Sized;

    fn instance_configure(&self) {
        self.device().lock().driver_ok();
    }
}

pub struct FutureVirtioDeviceType<Driver: WrappedVirtioDeviceType + Send + Sync + 'static> {
    pub this: Weak<RwLock<Self>>,

    pub device_type: Driver,

    // A map between descriptor IDs and Wakers
    waiting_requests: BTreeMap<u16, Vec<Waker>>,
    // A map between descriptor IDs and Buffers
    header_buffers: BTreeMap<u16, BlockRequestFutureBuffer>,
}

impl<Driver: to_trait::ToTrait + WrappedVirtioDeviceType + Send + Sync + 'static> to_trait::ToTrait
    for FutureVirtioDeviceType<Driver>
{
    fn cast_to_trait(self, target_type_id: core::any::TypeId) -> Option<Box<dyn to_trait::Null>> {
        self.device_type.cast_to_trait(target_type_id)
    }

    fn cast_to_trait_ref(&self, target_type_id: core::any::TypeId) -> Option<&dyn to_trait::Null> {
        self.device_type.cast_to_trait_ref(target_type_id)
    }

    fn cast_to_trait_mut(
        &mut self,
        target_type_id: core::any::TypeId,
    ) -> Option<&mut dyn to_trait::Null> {
        self.device_type.cast_to_trait_mut(target_type_id)
    }
}
impl<Driver: WrappedVirtioDeviceType + Send + Sync + 'static> to_trait::ToTrait
    for FutureVirtioDeviceType<Driver>
{
    default fn cast_to_trait(
        self,
        _target_type_id: core::any::TypeId,
    ) -> Option<Box<dyn to_trait::Null>> {
        None
    }

    default fn cast_to_trait_ref(
        &self,
        _target_type_id: core::any::TypeId,
    ) -> Option<&dyn to_trait::Null> {
        None
    }

    default fn cast_to_trait_mut(
        &mut self,
        _target_type_id: core::any::TypeId,
    ) -> Option<&mut dyn to_trait::Null> {
        None
    }
}

impl<Driver: 'static + WrappedVirtioDeviceType + Send + Sync + Unpin> VirtioDeviceType
    for FutureVirtioDeviceType<Driver>
{
    fn configure(
        device: Arc<Mutex<VirtioDevice>>,
    ) -> Result<Arc<RwLock<dyn to_trait::ToTraitAny + Send + Sync + Unpin>>, ()> {
        let inner = Driver::from_device(device);
        let device = Self {
            this: Weak::new(),
            device_type: inner,
            waiting_requests: BTreeMap::new(),
            header_buffers: BTreeMap::new(),
        };
        let device = Arc::new(RwLock::new(device));
        device.write().this = Arc::downgrade(&device);
        device.write().device_type.set_this(Arc::downgrade(&device));

        // setup wakers and similar stuff
        device.write().poll_device();
        {
            let device = device.clone();
            // Configure the instance later on (to prevent deadlocks)
            Arc::new(InterruptContextWaker(Box::new(move || {
                device.write().device_type.instance_configure();
            })))
            .wake();
        }
        Ok(device)
    }
}
