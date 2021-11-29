use alloc::sync::Arc;

use crate::{drivers::virtio::VirtioDriver, external_interrupt::ExternalInterruptHandler, fdt, handle::HandleBackend, lock::shared::RwLock};
use crate::drivers::traits::block::GenericBlockDevice;

pub struct FilesystemHandleBackend {
	block_device: Arc<RwLock<dyn GenericBlockDevice + Send + Sync + Unpin>>,
}

impl HandleBackend for FilesystemHandleBackend {
	fn create_singleton() -> alloc::sync::Arc<dyn HandleBackend + Send + Sync>
	where Self: Sized {
        let block_device: Arc<RwLock<dyn GenericBlockDevice + Send + Sync + Unpin>> = {
            let guard = fdt::root().read();
            let block_device_node = guard.get("soc/virtio_mmio@10008000").unwrap();
            let lock = block_device_node.kernel_struct.read();
            let bd = lock
                .as_ref()
                .unwrap()
                .downcast_ref::<(VirtioDriver, Option<ExternalInterruptHandler>)>();

            let block_device = if let VirtioDriver::Block(bd) = &bd.as_ref().unwrap().0 {
                bd
            } else {
                panic!("Block device not found!");
            };
            block_device.clone()
        };
	    alloc::sync::Arc::new(Self { block_device })
	}
	
	fn open(&self, fd_id: &usize, options: &[usize]) {
	}
	
	fn name(&self) -> &'static str { 
		"LogOutputHandleBackend"
	}
	
	fn write(&self, fd_id: &usize, buf: &[u8]) -> Result<usize, usize> {
		Ok(0)
	}
}