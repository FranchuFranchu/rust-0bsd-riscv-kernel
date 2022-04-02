//! Userspace crate with utilities to make interacting with the kernel more ergonomic

#![no_std]
#![feature(
    allocator_api,
    ptr_metadata,
    nonnull_slice_from_raw_parts,
    panic_info_message,
    register_tool
)]
#![register_tool(rust_analyzer)]
extern crate alloc;

pub mod allocator;
pub mod elf;
pub mod handle;
pub mod interrupt;
pub mod memory;
pub mod panic;
pub mod println;
pub mod process_egg;
pub mod syscall;
pub mod syscall_return;

pub fn exit() -> ! {
    unsafe { syscall::do_syscall_0(SyscallNumbers::Exit as usize) };
    // Otherwise, loop forever
    loop {}
}

pub use allocator::UserspaceAllocator;
pub use handle::Handle;
use kernel_syscall_abi::SyscallNumbers;
