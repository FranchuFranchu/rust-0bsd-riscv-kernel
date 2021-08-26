use core::num::NonZeroUsize;

#[repr(usize)]
pub enum StandardHandleErrors {
	Unimplemented = 1,
}

pub trait HandleBackend {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize, usize> {
		Err(StandardHandleErrors::Unimplemented as usize)
	}
	fn write(&mut self, buf: &[u8]) -> Result<usize, usize> {
		Err(StandardHandleErrors::Unimplemented as usize)
	}
	fn size_hint(&mut self) -> (usize, Option<usize>) {
		(0, None)
	}
	fn seek(&mut self, position: &usize) -> Result<(), usize> {
		Err(StandardHandleErrors::Unimplemented as usize)
	}
	fn tell(&mut self) -> Result<usize, usize> {
		Err(StandardHandleErrors::Unimplemented as usize)
	}
	fn split(&mut self) -> Option<NonZeroUsize> {
		None
	}
}
