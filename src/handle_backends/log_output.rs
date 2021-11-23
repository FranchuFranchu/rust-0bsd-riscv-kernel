use crate::handle::HandleBackend;

pub struct LogOutputHandleBackend {
	addr: usize,
}

impl HandleBackend for LogOutputHandleBackend {
	fn create_singleton() -> alloc::sync::Arc<dyn HandleBackend + Send + Sync>
	where Self: Sized {
	    alloc::sync::Arc::new(Self { addr: 0x1000_0000 })
	}
	
	fn open(&self, fd_id: &usize, options: &[usize]) {
		
	}
	
	fn name(&self) -> &'static str { 
		"LogOutputHandleBackend"
	}
	
	fn write(&self, fd_id: &usize, buf: &[u8]) -> Result<usize, usize> {
		let t = unsafe { crate::drivers::uart::Uart::new(self.addr).write_bytes(buf).unwrap() };
		Ok(buf.len())
	}
}