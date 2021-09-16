use core::convert::TryInto;
use core::future::Future;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;
use core::ops::Deref;

use crate::{ drivers::virtio, drivers::virtio::{block::VirtioBlockDevice, VirtioDeviceType, VirtioDevice}, external_interrupt::ExternalInterruptHandler, fdt::PropertyValue};
use alloc::{vec::Vec, sync::Arc};
use core::task::Waker;
use crate::lock::shared::{RwLock, Mutex};
use core::task::Poll;

pub struct DeviceSetupDoneFuture {
	wakers: Mutex<Vec<Waker>>,
	is_done: AtomicBool,
}

#[derive(Clone)]
pub struct DeviceSetupDoneFutureShared(Arc<DeviceSetupDoneFuture>);


impl Deref for DeviceSetupDoneFutureShared {
	type Target = DeviceSetupDoneFuture;
	
	fn deref(&self) -> &Self::Target {
	    self.0.deref()
	}
}

impl Future for DeviceSetupDoneFutureShared {
    type Output = ();

    fn poll(self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> Poll<Self::Output> {
        if self.0.is_done.load(Ordering::Acquire) {
        	Poll::Ready(())
        } else {
        	self.0.wakers.lock().push(cx.waker().clone());
        	Poll::Pending
        }
    }
}

impl DeviceSetupDoneFuture {
	fn wake(&self) {
		self.is_done.store(true, Ordering::Release);
		while let Some(waker) = self.wakers.lock().pop() {
			waker.wake()
		}
	}
}

static IS_DONE: RwLock<Option<DeviceSetupDoneFutureShared>> = RwLock::new(None);

pub fn is_done_future() -> DeviceSetupDoneFutureShared {
	let mut lock = IS_DONE.write();
	match &mut *lock {
		Some(expr) => {
			expr.clone()
		},
		None => {
			let t = DeviceSetupDoneFutureShared(Arc::new(DeviceSetupDoneFuture {
				wakers: Mutex::new(Vec::new()),
				is_done: AtomicBool::new(false)
			}));
			*lock = Some(t.clone());
			t
		},
	}
}


/// This functions scans the device tree
/// and sets up devices and interrupt handlers for all devices
pub fn setup_devices() {
	//crate::fdt::root().read().pretty(0);
	
	let lock = crate::fdt::root().read();
	lock.walk_nonstatic(&mut |node: &crate::fdt::Node| {
		
		if let Some(PropertyValue::String(compatible_with)) = node.properties.get("compatible") {
			match compatible_with {
				&"virtio,mmio" => {
					let mut virtio_device = unsafe { VirtioDevice::new(node.unit_address.unwrap_or(0) as _) };
					
					
					if virtio_device.is_present() {
						
						
						
						use alloc::sync::Arc;
						
						// First, congigure the virtio device
						virtio_device.configure();
						// Put the device inside an Arc-Mutex
						let virtio_device = Arc::new(crate::lock::shared::Mutex::new(virtio_device));
						
						// If this device has interrupts, register a handle
						
						let handler;
						if let Some(PropertyValue::u32(interrupt_id)) = node.properties.get("interrupts") {
							let virtio_device = virtio_device.clone();
							handler = Some(ExternalInterruptHandler::new((*interrupt_id).try_into().unwrap(), alloc::sync::Arc::new(move |id| {
								VirtioDevice::on_interrupt(&*virtio_device);
							})));
						} else {
							handler = None;
						}
						
						
						let virtio_driver;
						if let Some(d) = VirtioDevice::make_driver(virtio_device) {
							virtio_driver = d;
						} else {
							return;
						}
						
						
						*node.kernel_struct.write() = Some(alloc::boxed::Box::new((virtio_driver, handler)));
					}
				},
				&"ns16550a" => {
					// Create UART device
					//let uart_dev = unsafe { crate::drivers::uart::Uart::new(node.unit_address.unwrap_or(0)) }.get().unwrap();
					
				}
				_ => {
					warn!("Unrecognized device 'compatible' field: {:?}", compatible_with)
				}
			}
		}
	});
	drop(lock);
	
	// fdt::root().read().pretty(0);
	
	
	let handler2 = ExternalInterruptHandler::new(10, alloc::sync::Arc::new(|id| {
		let c = unsafe { crate::drivers::uart::Uart::new(0x10000000) }.get().unwrap();
		print!("C {}", c)
	}));
	

	info!("Finished device setup");
	
	is_done_future().wake();
	
	// Exit from this processs
	unsafe { crate::asm::do_supervisor_syscall_0(1) };
}