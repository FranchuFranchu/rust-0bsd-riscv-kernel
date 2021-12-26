use alloc::{boxed::Box, collections::BTreeMap, sync::Arc};

use crate::{
    drivers::{traits::block::GenericBlockDevice, virtio::VirtioDriver},
    external_interrupt::ExternalInterruptHandler,
    fdt,
    filesystem::ext2::{Ext2, InodeHandleState},
    handle::HandleBackend,
    lock::shared::RwLock,
};

pub struct FilesystemHandleBackend {
    block_device: Ext2,
    handle_inodes: crate::lock::future::rwlock::RwLock<BTreeMap<usize, InodeHandleState>>,
}

#[async_trait]
impl<'this> HandleBackend for FilesystemHandleBackend {
    fn create_singleton() -> alloc::sync::Arc<dyn HandleBackend + Send + Sync + 'static>
    where
        Self: Sized,
    {
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
        let block_device = Ext2::new(&block_device);
        alloc::sync::Arc::new(Self {
            block_device: block_device,
            handle_inodes: crate::lock::future::rwlock::RwLock::new(BTreeMap::new()),
        })
    }

    async fn open(&self, fd_id: &usize, options: &[usize]) {
        // a1 (Option #0) = start of filename
        // a2 (Option #1) = length of filename
        let filename = unsafe { core::slice::from_raw_parts(options[0] as *const u8, options[1]) };
        let filename = core::str::from_utf8(filename).unwrap();

        self.block_device.load_superblock().await.unwrap();

        info!("Opening file: {:?}", filename);

        let h = self
            .block_device
            .inode_handle_state(self.block_device.get_path(filename).await.unwrap().unwrap())
            .await
            .unwrap();

        self.handle_inodes.write().await.insert(*fd_id, h);
    }

    fn name(&self) -> &'static str {
        "FilesystemBackend"
    }

    async fn write(&self, _fd_id: &usize, _buf: &[u8], _options: &[usize]) -> Result<usize, usize> {
        Ok(0)
    }
    async fn read(
        &self,
        fd_id: &usize,
        buf: &mut [u8],
        _options: &[usize],
    ) -> Result<usize, usize> {
        let mut inode_handle = self.handle_inodes.write().await;
        inode_handle
            .get_mut(fd_id)
            .unwrap()
            .read(&self.block_device, buf)
            .await;
        Ok(0)
    }
}
