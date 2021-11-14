use core::num::NonZeroUsize;
use core::fmt::Debug;
use alloc::sync::Weak;

#[repr(usize)]
pub enum StandardHandleErrors {
    Unimplemented = 1,
}

pub trait HandleBackend {
    
    fn open(options: &[usize]) -> alloc::sync::Arc<dyn HandleBackend + Send + Sync> where Self: Sized;
    
    fn name(&self) -> &'static str;
    
    
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, usize> {
        Err(StandardHandleErrors::Unimplemented as usize)
    }
    fn write(&self, buf: &[u8]) -> Result<usize, usize> {
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

// 0BSD
pub struct Handle {
    pub fd_id: usize,
    pub backend: Weak<dyn HandleBackend + Send + Sync>,
    pub backend_meta: usize,
}

impl Debug for Handle {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::result::Result<(), core::fmt::Error> { 
        fmt.debug_struct("Handle")
            .field("fd_id", &self.fd_id)
            .field("backend", &self.backend.upgrade().map(|s| s.name()))
            .finish();
        Ok(())
    }
}