use core::num::NonZeroUsize;

#[repr(usize)]
pub enum StandardStreamErrors {
	Unimplemented = 1,
}

pub trait StreamBackend {
	fn read(&mut self, buf: &mut [u8]) -> Result<usize, usize> {
		Err(StandardStreamErrors::Unimplemented as usize)
	}
	fn write(&mut self, buf: &[u8]) -> Result<usize, usize> {
		Err(StandardStreamErrors::Unimplemented as usize)
	}
	fn size_hint(&mut self) -> (usize, Option<usize>) {
		(0, None)
	}
	fn seek(&mut self, position: &usize) -> Result<(), usize> {
		Err(StandardStreamErrors::Unimplemented as usize)
	}
	fn tell(&mut self) -> Result<usize, usize> {
		Err(StandardStreamErrors::Unimplemented as usize)
	}
	fn split(&mut self) -> Option<NonZeroUsize> {
		None
	}
}
