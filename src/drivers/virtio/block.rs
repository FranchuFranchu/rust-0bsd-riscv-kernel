use core::pin::Pin;
use core::task::{Context, Waker};
use core::future::Future;

use spin::Mutex;

use crate::{cpu, interrupt_context_waker::InterruptContextWaker, repr_c_serde::ReprCSerializer};
use alloc::{
	sync::{Arc, Weak},
	task::Wake,
	boxed::Box,
	vec::Vec,
	collections::{BTreeMap},
};

use super::{SplitVirtqueue, VirtioDevice, VirtioDeviceType};


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
	this: Weak<Mutex<Self>>,
	
	// A map between descriptor IDs and Wakers
	waiting_requests: BTreeMap<u16, Vec<Waker>>,
	// A map between descriptor IDs and Buffers
	header_buffers: BTreeMap<u16, Box<[u8]>>,
}

pub struct BlockRequestFuture {
	device: Weak<Mutex<VirtioBlockDevice>>,
	header: RequestHeader,
	// The buffer is moved out when block operation is being carried out, and then it's moved
	// back in when it's done (AFTER the future is poll()'d). 
	buffer: Option<Box<[u8]>>,
	descriptor_id: Option<u16>,
	was_queued: bool,
}


use core::task::Poll;

impl Future for BlockRequestFuture {
	type Output = ();
	
	
	fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<<Self as Future>::Output> { 
		if self.buffer.is_none() {
			// Check if the driver has been done yet
			if let Some(buffer) = self.device.upgrade().unwrap().lock().take_buffer(&self.descriptor_id.unwrap()) {
				self.buffer = Some(buffer);
				Poll::Ready(())
			} else {
				self.device.upgrade().unwrap().lock().register_waker(&self.descriptor_id.unwrap(), cx.waker().clone());
				Poll::Pending
			}
		} else if self.was_queued {
			// the block operation is done
			Poll::Ready(())
		} else {
			// the block operation hasn't been started yet
			self.was_queued = true;
			
			self.descriptor_id = Some(self.device.upgrade().unwrap().lock().do_request(&mut self));
			// Register ourselves as a waker
			self.device.upgrade().unwrap().lock().register_waker(&self.descriptor_id.unwrap(), cx.waker().clone());
			
			Poll::Pending
		}
	}
}

impl VirtioBlockDevice {
	fn instance_configure(&self) {
		self.device.lock().driver_ok();
	}
	
	fn create_request<'a>(&self, sector: u64, buffer: Box<[u8]>, write: bool) -> BlockRequestFuture {
		BlockRequestFuture {
			device: self.this.clone(),
			header: RequestHeader {
				r#type: if write { 1 } else { 0 },
				reserved: 0,
				sector,
			},
			buffer: Some(buffer),
			descriptor_id: None,
			was_queued: false,
		}
	}
	
	fn do_request(&mut self, future: &mut BlockRequestFuture) -> u16 {
		
		let mut vq_lock = self.request_virtqueue.lock();
		
		let status = alloc::vec![0xFFu8; 1].into_boxed_slice();
		
		let mut last = vq_lock.new_descriptor_from_boxed_slice(status, true, None);
		
		if future.header.r#type == 1 {
			last = vq_lock.new_descriptor_from_boxed_slice(future.buffer.take().unwrap(), false, Some(last));			
		} else {
			last = vq_lock.new_descriptor_from_boxed_slice(future.buffer.take().unwrap(), true, Some(last));
		}
		last = vq_lock.new_descriptor_from_sized(&future.header, false, Some(last));
		
		// Make the descriptor chain available and notify the device that the virtqueue is ready
		vq_lock.make_available(last);
		self.device.lock().queue_ready(0);
		
		// Return the head of the descriptor chain
		last
	}
	
	/// Sets up a callback future for when the device has finished processing a request we made
	fn poll_device(&mut self) {
		let mut device_ref = self.device.lock();
		let this_weak = self.this.clone();
		// Note that here we aren't polling the BlockDeviceRequest, but rather the VirtioDevice
		// (where polling means waiting for a used ring to be available)
		let result = Pin::new(&mut *device_ref).poll(&mut Context::from_waker(&Arc::new(InterruptContextWaker(Box::new(move || {
			// Now we can recreate self based on the weak pointer we moved
			// and then poll it again.
			// The value should be Ready now
			let this = this_weak.upgrade().unwrap().lock().poll_device();
		}))).into()));
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
				// Copy the iterator data
				.map(|s| *s)
				// Create a Vec<u8>
				.collect();
			
			let request_body = &data[core::mem::size_of::<RequestHeader>()..data.len()-1];
			
			
			// Now, try to recreate the Box<[u8]> that was used to create this
			// Reconstruct the buffer box
			let buffer_start_ptr = descriptor_chain_data_iterator.nth(1).unwrap().as_ptr() as *mut u8;
			let buffer_len = request_body.len();
			// SAFETY: This is constructed on do_request, and I think this is the "correct" way to restore it
			let buffer_box = unsafe { Box::from_raw(core::slice::from_raw_parts_mut(buffer_start_ptr, buffer_len)) };
			
			
			let items = self.waiting_requests.get_mut(&descriptor_id).map(|vec| vec.iter_mut());
			
			
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
	pub fn take_buffer(&mut self, descriptor_id: &u16) -> Option<Box<[u8]>>{
		self.header_buffers.remove(descriptor_id)
	}
	pub fn register_waker(&mut self, descriptor_id: &u16, waker: Waker) {
		if let Some(v) = self.waiting_requests.get_mut(descriptor_id) {
			v.push(waker)
		} else {
			self.waiting_requests.insert(*descriptor_id, alloc::vec![waker]);
		}
	}
}

impl VirtioDeviceType for VirtioBlockDevice {
	fn configure(device: Arc<Mutex<VirtioDevice>>) -> Result<Arc<Mutex<Self>>, ()> {
		let q = device.lock().configure_queue(0);
		let dev = VirtioBlockDevice { request_virtqueue: Mutex::new(q), device, this: Weak::new(), waiting_requests: BTreeMap::new(), header_buffers: BTreeMap::new() };
		let dev = Arc::new(Mutex::new(dev));
		dev.lock().this = Arc::downgrade(&dev);
		
		// setup wakers and similar stuff 
		dev.lock().poll_device();
		
		let dev_clone = dev.clone();
		// Configure the instance later on (to prevent deadlocks)
		Arc::new(InterruptContextWaker(Box::new(move || {
			dev_clone.lock().instance_configure();
			
			let block = async {
				let mut data = alloc::vec![0u8; 512].into_boxed_slice();
				
				// Write "ABCDE"
				data[0] = 65;
				data[1] = 66;
				data[2] = 67;
				data[3] = 68;
				data[4] = 69;
				
				let request = dev_clone.lock().create_request(0, data, true);
				request.await
			};
			let dev_clone = dev_clone.clone();
			
			let rc = Arc::new(InterruptContextWaker(Box::new(move || {
				// Now we can recreate self based on the weak pointer we moved
				// and then poll it again.
				// The value should be Ready now
				
				println!("{:?}", "Waken");
			})));
			let waker = &(rc).into();
			let mut cx = Context::from_waker(waker);
			let mut pinned = Box::pin(block);
			println!("Write result: {:?}", pinned.as_mut().poll(&mut cx));
			;
			/**/
			
		}))).wake();
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