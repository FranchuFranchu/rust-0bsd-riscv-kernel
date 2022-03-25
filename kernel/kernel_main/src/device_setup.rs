//! A kernel task that reads the device tree and initializes drivers

use alloc::{sync::Arc, vec::Vec};
use core::{
    convert::TryInto,
    future::Future,
    ops::Deref,
    sync::atomic::{AtomicBool, Ordering},
    task::{Poll, Waker},
};

use crate::{
    drivers::virtio::{VirtioDevice, VirtioDeviceType},
    external_interrupt::ExternalInterruptHandler,
    fdt::PropertyValue,
    lock::shared::{Mutex, RwLock},
    virtual_buffers::new_virtual_buffer,
};

/// Future that can be awaited to call the waker when device setup is done
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

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> Poll<Self::Output> {
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

/// Returns a DeviceSetupIsDoneFuture instance
pub fn is_done_future() -> DeviceSetupDoneFutureShared {
    let mut lock = IS_DONE.write();
    match &mut *lock {
        Some(expr) => expr.clone(),
        None => {
            let t = DeviceSetupDoneFutureShared(Arc::new(DeviceSetupDoneFuture {
                wakers: Mutex::new(Vec::new()),
                is_done: AtomicBool::new(false),
            }));
            *lock = Some(t.clone());
            t
        }
    }
}

/// This functions scans the device tree
/// and sets up devices and interrupt handlers for all devices
pub fn setup_devices() {
    //crate::fdt::root().read().pretty(0);
    info!("setting up devices");
    let lock = crate::fdt::root().read();
    lock.walk_nonstatic(&mut |node: &crate::fdt::Node| {
        if let Some(PropertyValue::String(compatible_with)) = node.properties.get("compatible") {
            match compatible_with {
                &"virtio,mmio" => {
                    let mut virtio_device = unsafe {
                        VirtioDevice::new(
                            new_virtual_buffer(node.unit_address.unwrap_or(0), 0x1000) as _,
                        )
                    };

                    if virtio_device.is_present() {
                        use alloc::sync::Arc;

                        // First, congigure the virtio device
                        virtio_device.configure();
                        // Put the device inside an Arc-Mutex
                        let virtio_device =
                            Arc::new(crate::lock::shared::Mutex::new(virtio_device));

                        // If this device has interrupts, register a handle

                        let handler;
                        if let Some(PropertyValue::u32(interrupt_id)) =
                            node.properties.get("interrupts")
                        {
                            let virtio_device = virtio_device.clone();
                            handler = Some(ExternalInterruptHandler::new(
                                (*interrupt_id).try_into().unwrap(),
                                alloc::sync::Arc::new(move |_id| {
                                    VirtioDevice::on_interrupt(&*virtio_device);
                                }),
                            ));
                        } else {
                            handler = None;
                        }

                        let virtio_driver;
                        if let Some(d) = VirtioDevice::make_driver(virtio_device) {
                            virtio_driver = d;
                        } else {
                            return;
                        }

                        *node.kernel_struct.write() =
                            Some(alloc::boxed::Box::new((virtio_driver, handler)));
                    }
                }
                &"ns16550a" => {
                    // Create UART device
                    //let uart_dev = unsafe { crate::drivers::uart::Uart::new(node.unit_address.unwrap_or(0)) }.get().unwrap();
                }
                _ => {
                    warn!(
                        "Unrecognized device 'compatible' field: {:?}",
                        compatible_with
                    )
                }
            }
        }
    });
    drop(lock);

    // fdt::root().read().pretty(0);
    /*
    let _handler2 = ExternalInterruptHandler::new(
        10,
        alloc::sync::Arc::new(|_id| {
            let c = unsafe { crate::drivers::uart::Uart::new(0x10000000) }
                .get()
                .unwrap();
            print!("C {}", c)
        }),
    );*/

    info!("Finished device setup");

    is_done_future().wake();

    // Exit from this processs
    unsafe { crate::asm::do_supervisor_syscall_0(1) };
}
