#![no_std]
#![feature(asm)]

#[inline(always)]
pub unsafe fn do_syscall_0(number: usize) -> (usize, usize) {
    let mut ret: (usize, usize) = (0, 0);
    asm!(
        "ecall",
        in("a7") number,
        lateout("a0") ret.0,
        lateout("a1") ret.1,
    );
    return ret;
}


#[inline(always)]
pub unsafe fn do_syscall_1(number: usize, a0: usize) -> (usize, usize) {
    let mut ret: (usize, usize) = (0, 0);
    asm!(
        "ecall",
        in("a7") number,
        in("a0") a0,
        lateout("a0") ret.0,
        lateout("a1") ret.1,
    );
    return ret;
}

#[inline(always)]
pub unsafe fn do_syscall_2(number: usize, a0: usize, a1: usize) -> (usize, usize) {
    let mut ret: (usize, usize) = (0, 0);
    asm!(
        "ecall",
        in("a7") number,
        in("a0") a0,
        in("a1") a1,
        lateout("a0") ret.0,
        lateout("a1") ret.1,
    );
    return ret;
}

#[inline(always)]
pub unsafe fn do_syscall_3(number: usize, a0: usize, a1: usize, a2: usize,) -> (usize, usize) {
    let mut ret: (usize, usize) = (0, 0);
    asm!(
        "ecall",
        in("a7") number,
        in("a0") a0,
        in("a1") a1,
        in("a2") a2,
        lateout("a0") ret.0,
        lateout("a1") ret.1,
    );
    return ret;
}

#[inline(always)]
pub unsafe fn do_syscall_7(number: usize, a0: usize, a1: usize, a2: usize, a3: usize, a4: usize, a5: usize, a6: usize) -> (usize, usize) {
    let mut ret: (usize, usize) = (0, 0);
    asm!(
        "ecall",
        in("a7") number,
        in("a0") a0,
        in("a1") a1,
        in("a2") a2,
        in("a3") a3,
        in("a4") a4,
        in("a5") a5,
        in("a6") a6,
        lateout("a0") ret.0,
        lateout("a1") ret.1,
    );
    return ret;
}

#[inline(always)]
pub unsafe fn do_syscall_slice(number: usize, arguments: &[usize; 7]) -> (usize, usize) {
    let mut ret: (usize, usize) = (0, 0);
    asm!(
        "ecall",
        in("a7") number,
        in("a0") arguments[0],
        in("a1") arguments[1],
        in("a2") arguments[2],
        in("a3") arguments[3],
        in("a4") arguments[4],
        in("a5") arguments[5],
        in("a6") arguments[6],
        lateout("a0") ret.0,
        lateout("a1") ret.1,
    );
    return ret;
}

pub struct Handle(usize);

impl Handle {
    pub fn open(backend: usize, options: &[usize]) -> Self {
        
        let mut params = [0; 7];
        params[0..1].copy_from_slice(&[backend]);
        params[1..options.len() + 1].copy_from_slice(options);
        let (n, _) = unsafe { do_syscall_slice(kernel_syscall_abi::SyscallNumbers::Open as usize, &params) };
        Self(n)
    }
    pub fn read(&self, buffer: &mut [u8], options: &[usize]) {
        let mut params = [0; 7];
        params[0..3].copy_from_slice(&[self.0, buffer.as_mut_ptr() as *mut u8 as usize, buffer.len()]);
        params[3..options.len() + 3].copy_from_slice(options);
        let (n, _) = unsafe { do_syscall_slice(kernel_syscall_abi::SyscallNumbers::Read as usize, &params) };
    }
    pub fn write(&self, buffer: &[u8], options: &[usize]) {
        let mut params = [0; 7];
        params[0..3].copy_from_slice(&[self.0, buffer.as_ptr() as *const u8 as usize, buffer.len()]);
        params[3..options.len() + 3].copy_from_slice(options);
        let (n, _) = unsafe { do_syscall_slice(kernel_syscall_abi::SyscallNumbers::Write as usize, &params) };
    }
}
