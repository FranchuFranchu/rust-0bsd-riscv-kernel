use alloc::{boxed::Box, sync::Weak};
use core::{fmt::Debug, num::NonZeroUsize};

use kernel_as_register::EncodedError;

use crate::handle_backends::call_as_register_function;

#[repr(usize)]
pub enum StandardHandleErrors {
    Unimplemented = 1,
}

#[async_trait]
#[allow(where_clauses_object_safety)]
pub trait HandleBackend {
    fn create_singleton() -> alloc::sync::Arc<dyn HandleBackend + Send + Sync + 'static>
    where
        Self: Sized;

    async fn open(&self, id: &usize, options: &[usize]) -> Result<usize, EncodedError>;

    fn name(&self) -> &'static str;

    async fn read(
        &self,
        _id: &usize,
        _buf: &mut [u8],
        _options: &[usize],
    ) -> Result<usize, EncodedError> {
        call_as_register_function(async || Err(StandardHandleErrors::Unimplemented as usize)).await
    }
    async fn write(
        &self,
        _id: &usize,
        _buf: &[u8],
        _options: &[usize],
    ) -> Result<usize, EncodedError> {
        call_as_register_function(async || Err(StandardHandleErrors::Unimplemented as usize)).await
    }
    async fn size_hint(&self, _id: &usize, _options: &[usize]) -> (usize, Option<usize>) {
        (0, None)
    }
    async fn seek(
        &self,
        _id: &usize,
        _position: &usize,
        _options: &[usize],
    ) -> Result<usize, EncodedError> {
        call_as_register_function(async || Err(StandardHandleErrors::Unimplemented as usize)).await
    }
    async fn tell(&self, _id: &usize, _options: &[usize]) -> Result<usize, EncodedError> {
        call_as_register_function(async || Err(StandardHandleErrors::Unimplemented as usize)).await
    }
    async fn split(&self, _id: &usize, _options: &[usize]) -> Option<NonZeroUsize> {
        None
    }

    fn close(&self, _id: &usize, _options: &[usize]) -> Result<(), EncodedError> {
        Ok(())
    }
}

// 0BSD
pub struct Handle {
    pub fd_id: usize,
    pub backend: Weak<dyn HandleBackend + Send + Sync>,
    pub backend_meta: usize,
}

impl Debug for Handle {
    fn fmt(
        &self,
        fmt: &mut core::fmt::Formatter<'_>,
    ) -> core::result::Result<(), core::fmt::Error> {
        fmt.debug_struct("Handle")
            .field("fd_id", &self.fd_id)
            .field("backend", &self.backend.upgrade().map(|s| s.name()))
            .finish()?;
        Ok(())
    }
}
