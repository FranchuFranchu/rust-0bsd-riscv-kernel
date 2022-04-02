use alloc::boxed::Box;

use kernel_as_register::EncodedError;

use crate::handle::HandleBackend;

pub struct LogOutputHandleBackend {
    addr: usize,
}

#[async_trait]
impl HandleBackend for LogOutputHandleBackend {
    fn create_singleton() -> alloc::sync::Arc<dyn HandleBackend + Send + Sync>
    where
        Self: Sized,
    {
        alloc::sync::Arc::new(Self { addr: 0x1000_0000 })
    }

    async fn open(&self, _fd_id: &usize, _options: &[usize]) -> Result<usize, EncodedError> {
        Ok(0)
    }

    fn name(&self) -> &'static str {
        "LogOutputHandleBackend"
    }

    async fn write(
        &self,
        _fd_id: &usize,
        buf: &[u8],
        _options: &[usize],
    ) -> Result<usize, EncodedError> {
        // print!("[userspace process] ");
        unsafe { crate::std_macros::get_uart().write_bytes(buf).unwrap() };

        // Make sure it ends in a newline
        // if buf.last() != None && *buf.last().unwrap() != 0xa {
        //     println!();
        // }
        Ok(buf.len())
    }
}
