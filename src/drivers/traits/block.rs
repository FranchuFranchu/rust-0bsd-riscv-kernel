use alloc::boxed::Box;
use core::future::Future;
use core::any::Any;

pub trait BlockDevice {
    type Request: Future<Output = Option<Box<[u8]>>>;
    fn _create_request(&self, sector: u64, buffer: Box<[u8]>, write: bool) -> Self::Request;
}

pub trait AnyBlockDevice: BlockDevice + Any {}
pub trait AnyRequestFuture: Future<Output = Option<Box<[u8]>>> + Any {}

impl<T> AnyBlockDevice for T where T: BlockDevice + Any {}
impl<T> AnyRequestFuture for T where T: Future<Output = Option<Box<[u8]>>> + Any {}

pub trait GenericBlockDevice: BlockDevice {
    fn create_request(&self, sector: u64, buffer: Box<[u8]>, write: bool) -> Box<dyn AnyRequestFuture + Unpin + Send + Sync> where Self::Request: Send + Sync + 'static {
        Box::new(Box::pin(self._create_request(sector, buffer, write)))
    }
}

impl<T> GenericBlockDevice for T where T: BlockDevice, T::Request: Send + Sync {}

