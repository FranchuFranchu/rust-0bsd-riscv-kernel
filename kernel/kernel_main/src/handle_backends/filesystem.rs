use alloc::{boxed::Box, collections::BTreeMap, sync::Arc};

use kernel_as_register::{AsRegister, EncodedError};
use kernel_syscall_abi::filesystem::FilesystemError;

use super::call_as_register_function;
use crate::{
    arc_inject::WeakInjectRwLock,
    drivers::{traits::block::GenericBlockDevice, virtio::VirtioDriver},
    external_interrupt::ExternalInterruptHandler,
    fdt,
    filesystem::ext2::{inode_handle, Ext2, Ext2Error, InodeHandleState},
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
        let block_device = {
            let guard = fdt::root().read();
            let block_device_node = guard.get("soc/virtio_mmio@10008000").unwrap();
            let lock = block_device_node.kernel_struct.read();
            let bd = lock
                .as_ref()
                .unwrap()
                .downcast_ref::<(
                    Arc<RwLock<dyn to_trait::ToTraitAny + Send + Sync + Unpin>>,
                    Option<ExternalInterruptHandler>,
                )>()
                .unwrap();

            use to_trait::ToTraitExt;
            crate::arc_inject::ArcInject::downgrade(&crate::arc_inject::ArcInject::new_std(
                &bd.0,
                |p| {
                    unsafe { p.data_ptr().as_ref().unwrap() }
                        .to_trait_ref::<dyn GenericBlockDevice + Send + Sync + Unpin>()
                        .unwrap()
                },
            ))
        };
        let block_device = WeakInjectRwLock { weak: block_device };
        let block_device = Ext2::new(block_device);
        alloc::sync::Arc::new(Self {
            block_device: block_device,
            handle_inodes: crate::lock::future::rwlock::RwLock::new(BTreeMap::new()),
        })
    }

    async fn open(&self, fd_id: &usize, options: &[usize]) -> Result<usize, EncodedError> {
        call_as_register_function::<FilesystemError, _, _, _>(async move || {
            // a1 (Option #0) = start of filename
            // a2 (Option #1) = length of filename
            let filename =
                unsafe { core::slice::from_raw_parts(options[0] as *const u8, options[1]) };
            let filename = core::str::from_utf8(filename).unwrap();

            self.block_device.load_superblock().await.unwrap();

            let f = self
                .block_device
                .get_path(filename)
                .await?
                .ok_or(FilesystemError::FileNotFound)?;

            let h = self.block_device.inode_handle_state(f).await.unwrap();

            self.handle_inodes.write().await.insert(*fd_id, h);

            Ok(0)
        })
        .await
    }

    fn name(&self) -> &'static str {
        "FilesystemBackend"
    }

    async fn write(
        &self,
        _fd_id: &usize,
        _buf: &[u8],
        _options: &[usize],
    ) -> Result<usize, EncodedError> {
        Ok(0)
    }
    async fn read(
        &self,
        fd_id: &usize,
        buf: &mut [u8],
        _options: &[usize],
    ) -> Result<usize, EncodedError> {
        let mut inode_handle = self.handle_inodes.write().await;
        Ok(inode_handle
            .get_mut(fd_id)
            .unwrap()
            .read(&self.block_device, buf)
            .await
            .unwrap())
    }
}
