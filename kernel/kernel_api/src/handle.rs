use kernel_syscall_abi::filesystem::FilesystemError;

use crate::{
    syscall::{do_syscall_1, do_syscall_slice},
    syscall_return::{AsResult, SyscallErrorData},
};

#[derive(Debug)]
pub struct Handle(usize);

pub type Result<T> = core::result::Result<T, (usize, SyscallErrorData)>;

impl Handle {
    pub fn open(backend: usize, options: &[usize]) -> Result<Self> {
        let mut params = [0; 7];
        params[0..1].copy_from_slice(&[backend]);
        params[1..options.len() + 1].copy_from_slice(options);
        Ok(Self(
            unsafe { do_syscall_slice(kernel_syscall_abi::SyscallNumbers::Open as usize, &params) }
                .as_generic_result()?,
        ))
    }
    pub fn read(&self, buffer: &mut [u8], options: &[usize]) -> Result<usize> {
        let mut params = [0; 7];
        params[0..3].copy_from_slice(&[
            self.0,
            buffer.as_mut_ptr() as *mut u8 as usize,
            buffer.len(),
        ]);
        params[3..options.len() + 3].copy_from_slice(options);
        Ok(
            unsafe { do_syscall_slice(kernel_syscall_abi::SyscallNumbers::Read as usize, &params) }
                .as_generic_result()?,
        )
    }
    pub fn write(&self, buffer: &[u8], options: &[usize]) -> Result<usize> {
        let mut params = [0; 7];
        params[0..3].copy_from_slice(&[
            self.0,
            buffer.as_ptr() as *const u8 as usize,
            buffer.len(),
        ]);
        params[3..options.len() + 3].copy_from_slice(options);
        Ok(
            unsafe {
                do_syscall_slice(kernel_syscall_abi::SyscallNumbers::Write as usize, &params)
            }
            .as_generic_result()?,
        )
    }
    // private: should only be called once
    fn close(&self) -> Result<()> {
        unsafe {
            do_syscall_1(kernel_syscall_abi::SyscallNumbers::Close as usize, self.0)
                .as_generic_result()
                .map(|_| ())
        }
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

pub fn open_file(file: &str, options: &[usize]) -> core::result::Result<Handle, FilesystemError> {
    let mut params = [0; 5];
    params[0..2].copy_from_slice(&[
        file.as_bytes().as_ptr() as *const u8 as usize,
        file.as_bytes().len(),
    ]);
    params[2..options.len() + 2].copy_from_slice(options);
    Handle::open(2, &params).map_err(|s| s.as_result())
}

impl core::fmt::Write for Handle {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s.as_bytes(), &[]).unwrap();
        Ok(())
    }
}
