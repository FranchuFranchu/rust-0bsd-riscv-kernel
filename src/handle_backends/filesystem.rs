use alloc::sync::Arc;

use crate::filesystem::ext2::{Ext2, InodeHandleState};
use crate::{drivers::virtio::VirtioDriver, external_interrupt::ExternalInterruptHandler, fdt, handle::HandleBackend, lock::shared::RwLock};
use crate::drivers::traits::block::GenericBlockDevice;
use alloc::collections::BTreeMap;
use alloc::boxed::Box;

pub struct FilesystemHandleBackend {
	block_device: Ext2,
	handle_inodes: RwLock<BTreeMap<usize, InodeHandleState>>,
}

#[async_trait]
impl<'this> HandleBackend for FilesystemHandleBackend {
	fn create_singleton() -> alloc::sync::Arc<dyn HandleBackend + Send + Sync + 'static>
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
	    alloc::sync::Arc::new(Self { 
	    	block_device: Ext2::new(&block_device),
	    	handle_inodes: RwLock::new(BTreeMap::new())
	    })
	}
	
	async fn open(&self, fd_id: &usize, options: &[usize]) {
		// a1 (Option #0) = start of filename
		// a2 (Option #1) = length of filename
		let filename = unsafe { core::slice::from_raw_parts(options[0] as *const u8, options[1]) };
		let filename = core::str::from_utf8(filename).unwrap();
		
		info!("Opening file: {:?}", filename);
		
		let h = self.block_device.inode_handle_state(self.block_device.get_path(filename).await.unwrap().unwrap()).await.unwrap();
		
		self.handle_inodes.write().insert(*fd_id, h);
	}
	
	fn name(&self) -> &'static str { 
		"LogOutputHandleBackend"
	}
	
	async fn write(&self, fd_id: &usize, buf: &[u8], options: &[usize]) -> Result<usize, usize> {
		Ok(0)
	}
}