use alloc::boxed::Box;
use core::future::Future;
use core::any::Any;

pub trait BlockDevice {
    fn _create_request(&self, sector: u64, buffer: Box<[u8]>, write: bool) -> Box<dyn AnyRequestFuture + Send + Sync + Unpin + 'static>;
}

pub trait AnyBlockDevice: BlockDevice + Any {}
pub trait AnyRequestFuture: Future<Output = Option<Box<[u8]>>> + Any {}

impl<T> AnyBlockDevice for T where T: BlockDevice + Any {}
impl<T> AnyRequestFuture for T where T: Future<Output = Option<Box<[u8]>>> + Any {}

#[async_trait]
pub trait GenericBlockDevice: BlockDevice + Send + Sync {
    fn create_request(&self, sector: u64, buffer: Box<[u8]>, write: bool) -> Box<dyn AnyRequestFuture + Unpin + Send + Sync> {
        Box::new(Box::pin(self._create_request(sector, buffer, write)))
    }
	
	async fn read(&self, sector: u64, length: usize) -> Result<Box<[u8]>, ()> {
		let buffer = alloc::vec![0; length].into_boxed_slice();
		Ok(self.create_request(sector, buffer, false).await.unwrap())
	}
	async fn read_buffer(&self, sector: u64, buffer: Box<[u8]>) -> Result<Box<[u8]>, ()> {
		Ok(self.create_request(sector, buffer, false).await.unwrap())
	}
	async fn write(&self, sector: u64, buffer: Box<[u8]>) -> (Box<[u8]>, Result<(), ()>) {
		(self.create_request(sector, buffer, true).await.unwrap(), Ok(()))
	}
}

impl<T> GenericBlockDevice for T where T: BlockDevice + Send + Sync {}

