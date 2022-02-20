use kernel_syscall_abi::{AllocPagesError, SyscallNumbers};

use crate::{syscall::do_syscall_4, syscall_return::AsResult};

pub type Result<T> = core::result::Result<T, AllocPagesError>;

pub fn alloc_pages(
    virtual_addr: Option<usize>,
    physical_addr: Option<usize>,
    size: usize,
    flags: usize,
) -> Result<usize> {
    let virtual_addr = if let Some(v) = virtual_addr {
        v
    } else {
        usize::MAX
    };
    let physical_addr = if let Some(v) = physical_addr {
        v
    } else {
        usize::MAX
    };

    let v = unsafe {
        do_syscall_4(
            SyscallNumbers::AllocPages as usize,
            virtual_addr,
            physical_addr,
            size,
            flags,
        )
    };
    v.as_generic_result_nonnull().map_err(|s| s.as_result())
}
