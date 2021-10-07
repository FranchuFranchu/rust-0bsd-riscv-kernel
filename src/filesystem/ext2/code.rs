use core::ops::{Add, Div, Sub};

use alloc::boxed::Box;
use kernel_io::Read;
use super::structures::{BlockGroupDescriptor, DirectoryEntry, Inode, OwnedDirectoryEntry, Superblock};
use crate::{drivers::traits::block::{GenericBlockDevice, GenericBlockDeviceExt, GenericBlockDeviceError}, lock::shared::{RwLock, Mutex}};

use alloc::sync::Arc;

pub struct Ext2 {
    device: Arc<RwLock<dyn GenericBlockDevice + Send + Sync + Unpin>>,
    superblock: RwLock<Option<Box<Superblock>>>,
}

/// Until #88581 gets into the compiler
trait DivCeil {
    #[inline]
    fn div_ceil(self, rhs: Self) -> Self 
        where Self: Div<Output = Self> + Sub<Output = Self> + Add<Output = Self> + From<u8> + Sized + Copy {
        (self + rhs - Self::from(1u8)) / rhs
    }
}

#[derive(Debug)]
pub enum Ext2Error {
    OutOfBounds(usize),
    BlockDeviceError(GenericBlockDeviceError),
    IoError(kernel_io::Error),
    Unknown,
}

impl From<kernel_io::Error> for Ext2Error {
    // add code here
    fn from(other: kernel_io::Error) -> Self {
        Self::IoError(other)
    }
}

impl From<()> for Ext2Error {
    fn from(arg: ()) -> Ext2Error {
        Ext2Error::Unknown
    }
}

pub type Result<T> = core::result::Result<T, Ext2Error>;

impl<T> DivCeil for T where T: Div<Output = T> + Sub<Output = T> + Add<Output = T> + From<u8> + Sized {}

impl Ext2 {
	pub fn new(device: &Arc<RwLock<dyn GenericBlockDevice + Send + Sync + Unpin>>) -> Self {
		Ext2 {
			device: device.clone(),
			superblock: RwLock::new(None)
		}
	}
	pub fn block_size(&self) -> u32 {
        use core::convert::TryInto;
		(1024u32 << self.superblock.read().as_ref().unwrap().log_block_size).try_into().unwrap()
	}
	pub fn block_to_sector(&self, block: u32) -> u64 {
		(block as u64) * ((1024 << self.superblock.read().as_ref().unwrap().log_block_size) / 512)
	}
    pub async fn read_block(&self, block: u32) -> Result<Box<[u8]>> {
    	Ok(GenericBlockDeviceExt::read(&*self.device, self.block_to_sector(block), self.block_size() as usize).await?)
    }
    pub fn block_group_count(&self) -> u32 {
    	let value = self.superblock.read().as_ref().unwrap().blocks_count.div_ceil(self.superblock.read().as_ref().unwrap().blocks_per_group);
    	debug_assert!(self.superblock.read().as_ref().unwrap().inodes_count.div_ceil(self.superblock.read().as_ref().unwrap().inodes_per_group) == value);
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
    pub async fn read_block_group_descriptor(&self, block_group: u32) -> Result<BlockGroupDescriptor> {
        let start_block: u32 = if self.block_size() == 1024 { 2 } else { 1 };
        
        info!("Start block {}", start_block);
        
        let block_group_block = block_group / (self.block_size() / core::mem::size_of::<BlockGroupDescriptor>() as u32);
        let offset = block_group % (self.block_size() / core::mem::size_of::<BlockGroupDescriptor>() as u32);
        
        
        let v = self.read_block(start_block + block_group_block).await?;
        
        /// SAFETY: No use-after-free since we're cloning it after borrowing it
        let descriptor = unsafe { (v.split_at(offset as usize).1.as_ptr() as *const BlockGroupDescriptor).as_ref().unwrap() };
        Ok(descriptor.clone())
    }
    pub fn inode_size(&self) -> u32 {
        self.superblock.read().as_ref().unwrap().inode_size.into()
    }
    pub async fn read_inode(&self, inode: u32) -> Result<Inode> {
        let inode_table_block = self.read_block_group_descriptor(self.get_inode_block_group(inode)).await?.inode_table;
        
        let inode_block_offset: u32 = self.get_inode_index_in_table(inode) / (self.block_size() / self.inode_size());
        let inode_byte_offset: usize = ((self.get_inode_index_in_table(inode) % (self.block_size() / self.inode_size())) * self.inode_size()) as usize;
        
        info!("b offset {:?}", inode_byte_offset);
        
        let v = self.read_block(inode_table_block + inode_block_offset).await?;
        /// SAFETY: No use-after-free since we're cloning it after borrowing it
        Ok(unsafe { (v[inode_byte_offset..].as_ptr() as *const Inode).as_ref().unwrap().clone() })   
    }
    
    pub async fn read_inode_block(&self, inode: u32, block: u32) -> Result<Box<[u8]>> {
        self.read_inode_block_cache(&self.read_inode(inode).await?, block).await
    }
    pub async fn read_inode_block_cache(&self, inode: &Inode, block: u32) -> Result<Box<[u8]>>  {
        if inode.size < (block / self.block_size()) {
            Err(().into())
        } else if block <= 12 {
            // Direct block
            Ok(self.read_block(inode.block[block as usize]).await?)
        } else {
            unimplemented!()
        }
    }
    pub async fn inode_handle<'this>(&'this self, inode: u32) -> Result<InodeHandle<'this>> {
        Ok(InodeHandle { inode: self.read_inode(inode).await?, fs: self, position: 0 })
    }
    
    pub async fn write_block(&self, block: u32, buffer: Box<[u8]>) -> (Box<[u8]>, Result<()>) {
    	let mut ret = GenericBlockDeviceExt::write(&*self.device, self.block_to_sector(block), buffer).await;
        let mut ret = (ret.0, ret.1.map_err(|s| (s.into(): Ext2Error)));
        ret
    }
    pub async fn find_entry_in_directory(&self, directory: u32, name: &str) -> Result<Option<OwnedDirectoryEntry>> {
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
            
            if !handle.will_read_all((entry.rec_len - 1).into()){
                break;
            }
            
            let mut buf = alloc::vec![0; entry.rec_len as usize];
            handle.read(&mut buf).await?;
            
            
            let entry = unsafe { (buf.as_ptr() as *const DirectoryEntry).as_ref().unwrap() };
            
            let this_name =  unsafe { core::str::from_utf8(entry.get_name()).unwrap() };
            info!("{:?} {:?}", this_name, name);
            
            if this_name == name {
                return Ok(Some(OwnedDirectoryEntry::from((entry, this_name))));
            }
            
        }
        Ok(None)
    }
    pub async fn get_relative_path(&self, parent: u32, path: &str) -> Result<Option<u32>> {
        let mut current_inode = parent;
        for (index, component) in path.split("/").enumerate() {
            // TODO how to improve this?
            let entry = self.find_entry_in_directory(current_inode, component).await?;
            info!("{:?}", component);
            let entry = match entry {
                Some(expr) => expr,
                None => return Ok(None),
            };
            
            println!("{:?}", index);
            
            // If this is the last component, return the inode
            if path.chars().filter(|s| s == &'/').count() == index {
                return Ok(Some(entry.inode))
            }
            if entry.file_type != 2 {
                // Not a directory, error out
                return Ok(None)
            }
            current_inode = entry.inode;
        }
        Ok(None)
    }
    pub async fn get_path(&self, path: &str) -> Result<Option<u32>> {
        let path = if path.chars().nth(0).unwrap() == '/' {
            &path[1..]
        } else {
            path
        };
        self.get_relative_path(2, path).await
    }
    pub async fn load_superblock(&self) -> Result<()> {
        let superblock: Box<[u8]> = GenericBlockDeviceExt::read(&*self.device, 2, 512*2).await?;
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

pub struct InodeHandle<'a> {
    fs: &'a Ext2,
    inode: Inode,
    position: usize,
}

#[async_trait]
impl<'a> Read for InodeHandle<'a> {
    type Error = Ext2Error;
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let block_size: usize = self.fs.block_size() as usize;
        
        let mut position_in_buffer = 0;
        
        
        while position_in_buffer < buf.len() && self.position < self.inode.size as usize {
            use core::convert::TryInto;
            let current_block: u32 = (self.position / block_size).try_into().unwrap();
            let current_block_offset = self.position % block_size;
            let read_in_block_up_to = (block_size).min(self.inode.size as usize).min(buf.len() - position_in_buffer + current_block_offset);
            
            let block = self.fs.read_inode_block_cache(&self.inode, current_block).await?;
            let copy_to_buffer = &block[current_block_offset..read_in_block_up_to];
            
            buf[position_in_buffer..position_in_buffer+copy_to_buffer.len()].copy_from_slice(copy_to_buffer);
            self.position += copy_to_buffer.len();
            position_in_buffer += copy_to_buffer.len();
        }
        Ok(position_in_buffer)
    }
}

impl<'a> InodeHandle<'a> {
    pub fn will_read_all(&mut self, length: usize) -> bool {
        (self.position + length) <= (self.inode.size as usize)
    }
    pub fn seek(&mut self, position: usize) {
        self.position = position;
    }
    pub fn tell(&self) -> usize {
        self.position
    }
}


