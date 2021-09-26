use alloc::boxed::Box;

use super::structures::Superblock;
use crate::{drivers::traits::block::GenericBlockDevice, lock::shared::RwLock};

struct Ext2 {
    device: Box<dyn GenericBlockDevice + Send + Sync + Unpin>,
    superblock: RwLock<Option<Box<Superblock>>>,
}

impl Ext2 {
    async fn read_block(&self) {}
    async fn load_superblock(&self) -> Result<(), ()> {
        let superblock = self.device.read(2, 2).await?;

        let mut guard = self.superblock.write();

        // SAFETY: There are no illegal values for struct Superblock since it's repr C
        // and the superblock will not have data outside of allocated memory
        assert!(superblock.len() >= core::mem::size_of::<Superblock>());
        // assert!(core::mem::size_of::<Box<[u8]>>() == core::mem::size_of::<Box<Superblock>>());
        let superblock: Box<Superblock> =
            unsafe { Box::from_raw(Box::into_raw(superblock) as *mut Superblock) };

        *(guard) = Some(superblock);

        Ok(())
    }
}
