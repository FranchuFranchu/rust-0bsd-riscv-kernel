use crate::syscall::do_syscall_slice;

pub struct Handle(usize);

impl Handle {
    pub fn open(backend: usize, options: &[usize]) -> Self {
        
        let mut params = [0; 7];
        params[0..1].copy_from_slice(&[backend]);
        params[1..options.len() + 1].copy_from_slice(options);
        let (n, _) = unsafe { do_syscall_slice(kernel_syscall_abi::SyscallNumbers::Open as usize, &params) };
        Self(n)
    }
    pub fn read(&self, buffer: &mut [u8], options: &[usize]) -> usize {
        let mut params = [0; 7];
        params[0..3].copy_from_slice(&[self.0, buffer.as_mut_ptr() as *mut u8 as usize, buffer.len()]);
        params[3..options.len() + 3].copy_from_slice(options);
        let (n, _) = unsafe { do_syscall_slice(kernel_syscall_abi::SyscallNumbers::Read as usize, &params) };
        n
    }
    pub fn write(&self, buffer: &[u8], options: &[usize]) -> usize {
        let mut params = [0; 7];
        params[0..3].copy_from_slice(&[self.0, buffer.as_ptr() as *const u8 as usize, buffer.len()]);
        params[3..options.len() + 3].copy_from_slice(options);
        let (n, _) = unsafe { do_syscall_slice(kernel_syscall_abi::SyscallNumbers::Write as usize, &params) };
        n
    }
}

pub fn open_file(file: &str, options: &[usize]) -> Handle {
    let mut params = [0; 5];
    params[0..2].copy_from_slice(&[file.as_bytes().as_ptr() as *const u8 as usize, file.as_bytes().len()]);
    params[2..options.len() + 2].copy_from_slice(options);
    Handle::open(2, &params)
}

impl core::fmt::Write for Handle {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s.as_bytes(), &[]);
        Ok(())
    }
}