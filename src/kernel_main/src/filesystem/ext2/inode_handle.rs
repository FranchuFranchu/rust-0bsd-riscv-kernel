use alloc::boxed::Box;

use kernel_io::{Read, Write};

use super::{
    code::{Ext2, Ext2Error, Result},
    structures::Inode,
};

pub struct InodeHandleState {
    inode: Inode,
    inode_number: u32,
    position: usize,
}

pub struct InodeHandle<'a> {
    fs: &'a Ext2,
    state: InodeHandleState,
}

impl InodeHandleState {
    pub fn new(inode: Inode, inode_number: u32) -> Self {
        InodeHandleState {
            inode,
            inode_number,
            position: 0,
        }
    }
    pub async fn read(&mut self, fs: &Ext2, buf: &mut [u8]) -> Result<usize> {
        let block_size: usize = fs.block_size() as usize;

        let mut position_in_buffer = 0;

        while position_in_buffer < buf.len() && self.position < self.inode.size as usize {
            use core::convert::TryInto;
            let current_block: u32 = (self.position / block_size).try_into().unwrap();
            let current_block_offset = self.position % block_size;
            let read_in_block_up_to = (block_size)
                .min(self.inode.size as usize)
                .min(buf.len() - position_in_buffer + current_block_offset);

            let block = fs
                .read_inode_block_cache(&self.inode, current_block)
                .await?;
            let source_buffer = &block[current_block_offset..read_in_block_up_to];

            buf[position_in_buffer..position_in_buffer + source_buffer.len()]
                .copy_from_slice(source_buffer);
            self.position += source_buffer.len();
            position_in_buffer += source_buffer.len();
        }
        Ok(position_in_buffer)
    }
    pub async fn write(&mut self, fs: &Ext2, source_buffer: &[u8]) -> Result<usize> {
        let block_size: usize = fs.block_size() as usize;
        fs.truncate_inode(
            &mut self.inode,
            (self.position + source_buffer.len()) as u32,
        )
        .await?;
        let mut position_in_buffer = 0;

        while position_in_buffer < source_buffer.len() && self.position < self.inode.size as usize {
            use core::convert::TryInto;
            let current_block: u32 = (self.position / block_size).try_into().unwrap();
            let current_block_offset = self.position % block_size;
            let read_in_block_up_to = (block_size)
                .min(self.inode.size as usize)
                .min(source_buffer.len() - position_in_buffer + current_block_offset);

            println!("{:?}", read_in_block_up_to);

            let mut block = fs
                .read_inode_block_cache(&self.inode, current_block)
                .await?;
            let destination_buffer = &mut block[current_block_offset..read_in_block_up_to];

            println!("Bufs {:?} {:?}", destination_buffer, source_buffer);

            destination_buffer[position_in_buffer..position_in_buffer + source_buffer.len()]
                .copy_from_slice(source_buffer);
            self.position += destination_buffer.len();
            position_in_buffer += destination_buffer.len();
            drop(destination_buffer);
            fs.write_inode_block_cache(&self.inode, current_block, &block)
                .await?;
        }

        println!("{:?}", self.inode);
        fs.write_inode(self.inode_number, &self.inode).await?;

        Ok(position_in_buffer)
    }

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

impl<'a> InodeHandle<'a> {
    pub fn new(fs: &'a Ext2, state: InodeHandleState) -> Self {
        Self { fs, state }
    }
}

#[async_trait]
impl<'a> Read for InodeHandle<'a> {
    type Error = Ext2Error;
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.state.read(self.fs, buf).await
    }
}

#[async_trait]
impl<'a> Write for InodeHandle<'a> {
    type Error = Ext2Error;
    async fn write(&mut self, source_buffer: &[u8]) -> Result<usize> {
        self.state.write(self.fs, source_buffer).await
    }
}

impl<'a> InodeHandle<'a> {
    pub fn will_read_all(&mut self, length: usize) -> bool {
        self.state.will_read_all(length)
    }
    pub fn seek(&mut self, position: usize) {
        self.state.seek(position)
    }
    pub fn tell(&self) -> usize {
        self.state.tell()
    }
}
