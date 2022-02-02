//! Userspace crate with utilities to make interacting with the kernel more ergonomic

#![no_std]
#![feature(asm, allocator_api, ptr_metadata,nonnull_slice_from_raw_parts)]

extern crate alloc;

pub mod handle;
pub mod allocator;
pub mod syscall;
pub mod memory;
pub mod process_egg;
pub mod println;

pub use allocator::UserspaceAllocator;
pub use handle::Handle;