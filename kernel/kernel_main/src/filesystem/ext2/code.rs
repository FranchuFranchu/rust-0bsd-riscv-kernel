use alloc::{boxed::Box, sync::Arc, vec::Vec};
use core::ops::{Add, Div, Sub};

use kernel_io::Read;
pub use kernel_syscall_abi::filesystem::Ext2Error;

use super::{
    inode_handle::{InodeHandle, InodeHandleState},
    structures::{BlockGroupDescriptor, DirectoryEntry, Inode, OwnedDirectoryEntry, Superblock},
};
use crate::{
    drivers::traits::block::{GenericBlockDevice, GenericBlockDeviceExt},
    lock::shared::RwLock,
};

pub struct Ext2 {
    device: crate::arc_inject::WeakInjectRwLock<
        crate::lock::shared::rwlock::RawSharedRwLock,
        dyn to_trait::ToTraitAny + Send + Sync + Unpin,
        dyn GenericBlockDevice + Send + Sync + Unpin,
    >,
    superblock: RwLock<Option<Box<Superblock>>>,
    inode_allocation_lock: crate::lock::future::Mutex<()>,
    block_allocation_lock: crate::lock::future::Mutex<()>,
}

/// Until #88581 gets into the compiler
trait DivCeil {
    #[inline]
    fn div_ceil(self, rhs: Self) -> Self
    where
        Self:
            Div<Output = Self> + Sub<Output = Self> + Add<Output = Self> + From<u8> + Sized + Copy,
    {
        (self + rhs - Self::from(1u8)) / rhs
    }
}

pub type Result<T> = core::result::Result<T, Ext2Error>;

impl<T> DivCeil for T where T: Div<Output = T> + Sub<Output = T> + Add<Output = T> + From<u8> + Sized
{}

impl Ext2 {
    pub fn new(
        device: crate::arc_inject::WeakInjectRwLock<
            crate::lock::shared::rwlock::RawSharedRwLock,
            dyn to_trait::ToTraitAny + Send + Sync + Unpin,
            dyn GenericBlockDevice + Send + Sync + Unpin,
        >,
    ) -> Self {
        Ext2 {
            device,
            superblock: RwLock::new(None),
            inode_allocation_lock: crate::lock::future::Mutex::new(()),
            block_allocation_lock: crate::lock::future::Mutex::new(()),
        }
    }
    pub fn block_size(&self) -> u32 {
        use core::convert::TryInto;
        (1024u32 << self.superblock.read().as_ref().unwrap().log_block_size)
            .try_into()
            .unwrap()
    }
    pub fn block_to_sector(&self, block: u32) -> u64 {
        (block as u64) * ((1024 << self.superblock.read().as_ref().unwrap().log_block_size) / 512)
    }
    pub async fn read_block(&self, block: u32) -> Result<Box<[u8]>> {
        Ok(GenericBlockDeviceExt::read(
            &self.device,
            self.block_to_sector(block),
            self.block_size() as usize,
        )
        .await?)
    }
    pub fn block_group_count(&self) -> u32 {
        let value = self
            .superblock
            .read()
            .as_ref()
            .unwrap()
            .blocks_count
            .div_ceil(self.superblock.read().as_ref().unwrap().blocks_per_group);
        debug_assert!(
            self.superblock
                .read()
                .as_ref()
                .unwrap()
                .inodes_count
                .div_ceil(self.superblock.read().as_ref().unwrap().inodes_per_group)
                == value
        );
        value
    }
    pub fn root_inode_number(&self) -> u32 {
        2
    }
    pub fn get_inode_block_group(&self, inode: u32) -> u32 {
        (inode - 1) / self.superblock.read().as_ref().unwrap().inodes_per_group
    }
    pub fn get_inode_index_in_table(&self, inode: u32) -> u32 {
        (inode - 1) % self.superblock.read().as_ref().unwrap().inodes_per_group
    }
    pub async fn read_block_group_descriptor(
        &self,
        block_group: u32,
    ) -> Result<BlockGroupDescriptor> {
        let start_block: u32 = if self.block_size() == 1024 { 2 } else { 1 };

        let block_group_block =
            block_group / (self.block_size() / core::mem::size_of::<BlockGroupDescriptor>() as u32);
        let offset =
            block_group % (self.block_size() / core::mem::size_of::<BlockGroupDescriptor>() as u32);

        let v = self.read_block(start_block + block_group_block).await?;

        // SAFETY: No use-after-free since we're cloning it after borrowing it
        let descriptor = unsafe {
            (v.split_at(offset as usize).1.as_ptr() as *const BlockGroupDescriptor)
                .as_ref()
                .unwrap()
        };
        Ok(descriptor.clone())
    }
    pub async fn free_block(&self, block: u32) -> Result<()> {
        let blocks_per_group = self.superblock.read().as_ref().unwrap().blocks_per_group;
        let block_group_descriptor = self
            .read_block_group_descriptor(block / blocks_per_group)
            .await?;
        let byte_offset = (block / blocks_per_group) / 8;
        let bit_offset = (block / blocks_per_group) % 8;

        let mut block = self.read_block(block_group_descriptor.block_bitmap).await?;

        block[byte_offset as usize] &= !(1 << bit_offset);

        self.write_block(block_group_descriptor.block_bitmap, &mut *block)
            .await?;

        Ok(())
    }
    pub async fn allocate_block(&self) -> Result<u32> {
        let _guard = self.block_allocation_lock.lock();
        for block_group_number in 0u32..u32::MAX {
            let block_group_descriptor =
                self.read_block_group_descriptor(block_group_number).await?;
            if block_group_descriptor.free_blocks_count > 0 {
                // Allocate block
                if self.superblock.read().as_ref().unwrap().blocks_per_group > self.block_size() * 8
                {
                    unimplemented!("Block usage bitmaps with multiple blocks are not supported!")
                }
                let mut block = self.read_block(block_group_descriptor.block_bitmap).await?;
                let (block_number, byte) = block
                    .iter_mut()
                    .enumerate()
                    .filter(|(_idx, byte)| **byte != 0xFF)
                    .map(|(idx, byte)| (idx * 8, byte))
                    .next()
                    .unwrap(); // If this panics it means that there were actually no free blocks here and free_blocks_count was wrong.

                let block_number = block_group_number
                    * self.superblock.read().as_ref().unwrap().blocks_per_group
                    + block_number as u32;

                // TODO prevent panics for incorrect block group metadata

                // Byte != 0xff because of the filter() call above
                let unset_bit: u32 = (0..8).into_iter().find(|i| *byte & (1 << i) == 0).unwrap();
                // Mark this block as used
                *byte |= 1 << unset_bit;

                self.write_block(block_group_descriptor.block_bitmap, &mut *block)
                    .await?;

                return Ok(block_number + unset_bit);
            }
        }
        Err(Ext2Error::NoFreeBlocks)
    }
    pub async fn allocate_inode(&self) -> Result<u32> {
        let _guard = self.inode_allocation_lock.lock();
        for block_group_number in 0u32..u32::MAX {
            let block_group_descriptor =
                self.read_block_group_descriptor(block_group_number).await?;
            if block_group_descriptor.free_inodes_count > 0 {
                // Allocate block
                if self.superblock.read().as_ref().unwrap().blocks_per_group > self.block_size() * 8
                {
                    unimplemented!("Inode usage bitmaps with multiple blocks are not supported!")
                }
                let mut block = self.read_block(block_group_descriptor.inode_bitmap).await?;
                let (inode_number, byte) = block
                    .iter_mut()
                    .filter(|byte| **byte != 0xFF)
                    .enumerate()
                    .map(|(idx, byte)| (idx * 8, byte))
                    .next()
                    .unwrap(); // If this panics it means that there were actually no free blocks here and free_blocks_count was wrong.

                let inode_number = block_group_number
                    * self.superblock.read().as_ref().unwrap().inodes_per_group
                    + inode_number as u32
                    + 1;

                // TODO prevent panics for incorrect block group metadata

                // Byte != 0xff because of the filter() call above
                let unset_bit: u32 = (0..8).into_iter().find(|i| *byte & (1 << i) == 0).unwrap();
                // Mark this inode as used
                *byte |= 1 << unset_bit;

                self.write_block(block_group_descriptor.block_bitmap, &mut *block)
                    .await?;
                return Ok(inode_number + unset_bit);
            }
        }
        Err(Ext2Error::NoFreeInodes)
    }
    pub fn inode_size(&self) -> u32 {
        self.superblock.read().as_ref().unwrap().inode_size.into()
    }
    pub async fn read_inode(&self, inode: u32) -> Result<Inode> {
        let inode_table_block = self
            .read_block_group_descriptor(self.get_inode_block_group(inode))
            .await?
            .inode_table;

        let inode_block_offset: u32 =
            self.get_inode_index_in_table(inode) / (self.block_size() / self.inode_size());
        let inode_byte_offset: usize = ((self.get_inode_index_in_table(inode)
            % (self.block_size() / self.inode_size()))
            * self.inode_size()) as usize;

        let v = self
            .read_block(inode_table_block + inode_block_offset)
            .await?;
        // SAFETY: No use-after-free since we're cloning it after borrowing it
        Ok(unsafe {
            (v[inode_byte_offset..].as_ptr() as *const Inode)
                .as_ref()
                .unwrap()
                .clone()
        })
    }
    pub async fn write_inode(&self, inode: u32, value: &Inode) -> Result<()> {
        let inode_table_block = self
            .read_block_group_descriptor(self.get_inode_block_group(inode))
            .await?
            .inode_table;

        let inode_block_offset: u32 =
            self.get_inode_index_in_table(inode) / (self.block_size() / self.inode_size());
        let inode_byte_offset: usize = ((self.get_inode_index_in_table(inode)
            % (self.block_size() / self.inode_size()))
            * self.inode_size()) as usize;

        let mut v = self
            .read_block(inode_table_block + inode_block_offset)
            .await?;

        v[inode_byte_offset..inode_byte_offset + core::mem::size_of::<Inode>()].copy_from_slice(
            unsafe {
                core::slice::from_raw_parts(
                    value as *const Inode as *const u8,
                    core::mem::size_of::<Inode>(),
                )
            },
        );

        self.write_block(inode_table_block + inode_block_offset, &mut *v)
            .await?;

        Ok(())
    }

    pub async fn get_inode_block(&self, inode: &Inode, block: u32) -> Result<u32> {
        if block < 12 {
            // Direct block
            Ok(inode.block[block as usize])
        } else if (block >= 12) && (block <= (12 + self.block_size() / 4)) {
            // Single indirect block
            let u8_slice = self.read_block(inode.block[12]).await?;
            // Transmute it to an u32 slice
            // to easily get the block number
            // SAFETY: it's safe to transmute u8s into u32
            let (begin, u32_slice, end) = unsafe { u8_slice.align_to::<u32>() };
            assert!(begin.is_empty());
            assert!(end.is_empty());
            let a = u32_slice[(block - 12) as usize];
            Ok(a)
        } else {
            todo!("Very large file!")
        }
    }

    pub async fn set_inode_block(
        &self,
        inode: &mut Inode,
        block_index: u32,
        set_to: u32,
    ) -> Result<()> {
        if block_index <= 12 {
            // Direct block
            inode.block[block_index as usize] = set_to;
            Ok(())
        } else {
            unimplemented!()
        }
    }

    pub async fn read_inode_block(&self, inode: u32, block: u32) -> Result<Box<[u8]>> {
        self.read_inode_block_cache(&self.read_inode(inode).await?, block)
            .await
    }
    pub async fn read_inode_block_cache(&self, inode: &Inode, block: u32) -> Result<Box<[u8]>> {
        Ok(self
            .read_block(self.get_inode_block(inode, block).await?)
            .await?)
    }
    pub async fn write_inode_block(
        &self,
        inode: u32,
        block: u32,
        source_buffer: &[u8],
    ) -> Result<()> {
        self.write_inode_block_cache(&self.read_inode(inode).await?, block, source_buffer)
            .await
    }
    pub async fn write_inode_block_cache(
        &self,
        inode: &Inode,
        block: u32,
        source_buffer: &[u8],
    ) -> Result<()> {
        self.write_block(self.get_inode_block(inode, block).await?, source_buffer)
            .await?;
        Ok(())
    }
    pub async fn inode_handle<'this, 'handle>(
        &'this self,
        inode: u32,
    ) -> Result<InodeHandle<'handle>>
    where
        'this: 'handle,
    {
        Ok(InodeHandle::new(
            self,
            self.inode_handle_state(inode).await?,
        ))
    }

    pub async fn inode_handle_state(&self, inode: u32) -> Result<InodeHandleState> {
        Ok(InodeHandleState::new(self.read_inode(inode).await?, inode))
    }

    pub async fn write_block(&self, block: u32, buffer: &[u8]) -> Result<()> {
        let ret =
            GenericBlockDeviceExt::write(&self.device, self.block_to_sector(block), buffer).await;

        ret.map_err(|s| (s.into(): Ext2Error))
    }
    pub async fn find_entry_in_directory(
        &self,
        directory: u32,
        name: &str,
    ) -> Result<Option<OwnedDirectoryEntry>> {
        let mut handle = self.inode_handle(directory).await?;

        while handle.will_read_all(core::mem::size_of::<DirectoryEntry>()) {
            // Read the first part of the directory entry to get the size
            let pos = handle.tell();
            let mut buf = alloc::vec![0; core::mem::size_of::<DirectoryEntry>()];

            handle.read(&mut buf).await?;

            // SAFETY: This assumes that the rest of the directory entry is in bounds

            let entry = unsafe { (buf.as_ptr() as *const DirectoryEntry).as_ref().unwrap() };

            // Re-read the whole directory entry, with the name too this time
            handle.seek(pos);

            if !handle.will_read_all((entry.rec_len - 1).into()) {
                break;
            }

            let mut buf = alloc::vec![0; entry.rec_len as usize];
            handle.read(&mut buf).await?;

            let entry = unsafe { (buf.as_ptr() as *const DirectoryEntry).as_ref().unwrap() };

            let this_name = unsafe { core::str::from_utf8(entry.get_name()).unwrap() };

            if this_name == name {
                return Ok(Some(OwnedDirectoryEntry::from((entry, this_name))));
            }
        }
        Ok(None)
    }
    pub async fn list_directory(&self, directory: u32) -> Result<Vec<OwnedDirectoryEntry>> {
        let mut handle = self.inode_handle(directory).await?;
        let mut v = Vec::new();

        while handle.will_read_all(core::mem::size_of::<DirectoryEntry>()) {
            // Read the first part of the directory entry to get the size
            let pos = handle.tell();
            let mut buf = alloc::vec![0; core::mem::size_of::<DirectoryEntry>()];

            handle.read(&mut buf).await?;

            // SAFETY: This assumes that the rest of the directory entry is in bounds

            let entry = unsafe { (buf.as_ptr() as *const DirectoryEntry).as_ref().unwrap() };

            // Re-read the whole directory entry, with the name too this time
            handle.seek(pos);

            if !handle.will_read_all((entry.rec_len - 1).into()) {
                break;
            }

            let mut buf = alloc::vec![0; entry.rec_len as usize];
            handle.read(&mut buf).await?;

            let entry = unsafe { (buf.as_ptr() as *const DirectoryEntry).as_ref().unwrap() };
            let this_name = unsafe { core::str::from_utf8(entry.get_name()).unwrap() };
            v.push(OwnedDirectoryEntry::from((entry, this_name)));
        }
        Ok(v)
    }
    pub async fn get_relative_path(&self, parent: u32, path: &str) -> Result<Option<u32>> {
        let mut current_inode = parent;
        for (index, component) in path.split('/').enumerate() {
            // TODO how to improve this?
            let entry = self
                .find_entry_in_directory(current_inode, component)
                .await?;
            let entry = match entry {
                Some(expr) => expr,
                None => return Ok(None),
            };

            // If this is the last component, return the inode
            if path.chars().filter(|s| s == &'/').count() == index {
                return Ok(Some(entry.inode));
            }
            if entry.file_type != 2 {
                // Not a directory, error out
                return Ok(None);
            }
            current_inode = entry.inode;
        }
        Ok(None)
    }
    pub async fn get_path(&self, path: &str) -> Result<Option<u32>> {
        let path = if path.starts_with('/') {
            &path[1..]
        } else {
            path
        };
        self.get_relative_path(2, path).await
    }
    pub async fn load_superblock(&self) -> Result<()> {
        let superblock: Box<[u8]> = GenericBlockDeviceExt::read(&self.device, 2, 512 * 2).await?;
        let mut guard = self.superblock.write();
        // SAFETY: There are no illegal values for struct Superblock since it's repr C
        // and the superblock will not have data outside of allocated memory
        assert!(superblock.len() >= core::mem::size_of::<Superblock>());
        assert!((&superblock[0] as *const _ as usize) % core::mem::align_of_val(&*superblock) == 0);
        // assert!(core::mem::size_of::<Box<[u8]>>() == core::mem::size_of::<Box<Superblock>>());
        let superblock: Box<Superblock> =
            unsafe { Box::from_raw(Box::into_raw(superblock) as *mut Superblock) };

        *(guard) = Some(superblock);

        Ok(())
    }
    /// Either expands or shortens the bytes of a file
    pub async fn truncate_inode(&self, inode: &mut Inode, length: u32) -> Result<()> {
        inode.size = length;

        for block_number in 0.. {
            if self.get_inode_block(inode, block_number).await? == 0
                && block_number <= (length / self.block_size())
            {
                self.set_inode_block(inode, block_number, self.allocate_block().await?)
                    .await?;
            } else if self.get_inode_block(inode, block_number).await? != 0
                && block_number > (length / self.block_size())
            {
                self.free_block(self.get_inode_block(inode, block_number).await?)
                    .await?;
                self.set_inode_block(inode, block_number, 0).await?;
            } else if self.get_inode_block(inode, block_number).await? == 0
                && block_number > (length / self.block_size())
            {
                break;
            }
        }
        Ok(())
    }
}
