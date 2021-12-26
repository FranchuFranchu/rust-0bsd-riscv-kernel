use alloc::boxed::Box;
use core::{any::Any, future::Future};

use crate::unsafe_buffer::{UnsafeSlice, UnsafeSliceMut};

#[derive(Debug)]
pub enum BlockRequestFutureBuffer {
    WriteFrom(UnsafeSlice<u8>),
    ReadInto(UnsafeSliceMut<u8>),
}

pub trait BlockDevice {
    fn _create_request(
        &self,
        sector: u64,
        buffer: BlockRequestFutureBuffer,
    ) -> Box<dyn AnyRequestFuture + Send + Sync + Unpin + 'static>;
}

pub trait AnyBlockDevice: BlockDevice + Any {}
pub trait AnyRequestFuture: Future<Output = Option<BlockRequestFutureBuffer>> + Any {}

impl<T> AnyBlockDevice for T where T: BlockDevice + Any {}
impl<T> AnyRequestFuture for T where T: Future<Output = Option<BlockRequestFutureBuffer>> + Any {}

use crate::lock::shared::RwLock;

#[async_trait]
pub trait GenericBlockDevice: BlockDevice + Send + Sync {
    fn create_request(
        &self,
        sector: u64,
        buffer: BlockRequestFutureBuffer,
    ) -> Box<dyn AnyRequestFuture + Unpin + Send + Sync> {
        Box::new(Box::pin(self._create_request(sector, buffer)))
    }

    async fn read(&self, sector: u64, length: usize) -> Result<Box<[u8]>, ()> {
        let mut buffer = alloc::vec![0; length].into_boxed_slice();
        self.create_request(
            sector,
            BlockRequestFutureBuffer::ReadInto(unsafe { UnsafeSliceMut::new(&mut *buffer) }),
        )
        .await;
        Ok(buffer)
    }
    async fn read_buffer(&self, sector: u64, buffer: &mut [u8]) -> Result<(), ()> {
        self.create_request(
            sector,
            BlockRequestFutureBuffer::ReadInto(unsafe { UnsafeSliceMut::new(buffer) }),
        )
        .await;
        Ok(())
    }
    async fn write(&self, sector: u64, buffer: &[u8]) -> Result<(), ()> {
        self.create_request(
            sector,
            BlockRequestFutureBuffer::WriteFrom(unsafe { UnsafeSlice::new(buffer) }),
        )
        .await;
        Ok(())
    }
}

impl<T> GenericBlockDevice for T where T: BlockDevice + Send + Sync {}

/// Used to implement GenericBlockDevice for types with GenericBlockDevice inside them
#[async_trait]
pub trait GenericBlockDeviceExt {
    async fn read(&self, sector: u64, length: usize) -> Result<Box<[u8]>, ()>;
    async fn read_buffer(&self, sector: u64, buffer: &mut [u8]) -> Result<(), ()>;
    async fn write(&self, sector: u64, buffer: &[u8]) -> Result<(), ()>;
}

#[async_trait]
impl<T> GenericBlockDeviceExt for RwLock<T>
where
    T: GenericBlockDevice + Send + Sync + ?Sized,
{
    async fn read(&self, sector: u64, length: usize) -> Result<Box<[u8]>, ()> {
        let mut buffer = alloc::vec![0; length].into_boxed_slice();
        let f = RwLock::read(self).create_request(
            sector,
            BlockRequestFutureBuffer::ReadInto(unsafe { UnsafeSliceMut::new(&mut buffer) }),
        );
        f.await;
        Ok(buffer)
    }
    async fn read_buffer(&self, sector: u64, buffer: &mut [u8]) -> Result<(), ()> {
        let f = RwLock::read(self).create_request(
            sector,
            BlockRequestFutureBuffer::ReadInto(unsafe { UnsafeSliceMut::new(buffer) }),
        );
        f.await;
        Ok(())
    }
    async fn write(&self, sector: u64, buffer: &[u8]) -> Result<(), ()> {
        let f = RwLock::read(self).create_request(
            sector,
            BlockRequestFutureBuffer::WriteFrom(unsafe { UnsafeSlice::new(buffer) }),
        );
        f.await;
        Ok(())
    }
}

use crate::as_register::AsRegister;
#[derive(Debug, AsRegister)]
pub enum GenericBlockDeviceError {
    OutOfBounds,
}
