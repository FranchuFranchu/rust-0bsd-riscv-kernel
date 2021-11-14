use crate::handle::HandleBackend;

pub struct LogOutputHandleBackend {
	addr: usize,
}

impl HandleBackend for LogOutputHandleBackend {
	fn open(options: &[usize]) -> alloc::sync::Arc<dyn HandleBackend + Send + Sync> where Self: Sized {
		alloc::sync::Arc::new(Self { addr: 0x1000_0000 })
	}
	
	fn name(&self) -> &'static str { 
		"LogOutputHandleBackend"
	}
	
	fn write(&self, buf: &[u8]) -> Result<usize, usize> {
		let t = unsafe { crate::drivers::uart::Uart::new(self.addr).write_bytes(buf).unwrap() };
		Ok(buf.len())
	}
}