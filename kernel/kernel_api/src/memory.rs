use kernel_syscall_abi::SyscallNumbers;

use crate::syscall::do_syscall_4;

pub fn alloc_pages(virtual_addr: Option<usize>, physical_addr: Option<usize>, size: usize, flags: usize) -> (usize, usize) {
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
	
	unsafe { do_syscall_4(SyscallNumbers::AllocPages as usize, virtual_addr, physical_addr, size, flags) }
}