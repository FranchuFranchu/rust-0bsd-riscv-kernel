use core::convert::TryInto;

use crate::{ drivers::virtio, drivers::virtio::{block::VirtioBlockDevice, VirtioDeviceType, VirtioDevice}, external_interrupt::ExternalInterruptHandler, fdt::PropertyValue};


/// This functions scans the device tree
/// and sets up devices and interrupt handlers for all devices
pub fn setup_devices() {
	// Create the 	virtio device

	crate::fdt::root().read().pretty(0);
	
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
						let virtio_device = Arc::new(spin::Mutex::new(virtio_device));
						
						// If this device has interrupts, register a handle
						
						let handler;
						if let Some(PropertyValue::u32(interrupt_id)) = node.properties.get("interrupts") {
							let virtio_device = virtio_device.clone();
							handler = Some(ExternalInterruptHandler::new((*interrupt_id).try_into().unwrap(), alloc::sync::Arc::new(move |id| {
								println!("{:?}", "lock");
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
						
						let virtio_driver = Arc::new(virtio_driver);
						
						
						
						
						*node.kernel_struct.write() = Some(alloc::boxed::Box::new((virtio_driver, handler)));
						println!("{:?}", "written");
					}
					/*
					VirtioBlockDevice::negotiate_features(&mut virtio);
					VirtioBlockDevice::configure(virtio);
					*/
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
		print!("{}", c)
	}));
	
	
	// Exit from this processs
	unsafe { crate::asm::do_supervisor_syscall_0(1) };
}