#![feature(
    lang_items,
    global_asm,
    default_alloc_error_handler,
    panic_info_message
)]
#![no_std]
#![no_main]

use alloc::{string::ToString, vec::Vec};
use core::{arch::global_asm, panic::PanicInfo};

use flat_bytes::Flat;
use kernel_api::{
    elf::process_egg_from_elf_file, handle::open_file, println, process_egg::ProcessEgg, Handle,
    UserspaceAllocator,
};

extern crate alloc;

#[global_allocator]
static GLOBAL_ALLOCATOR: UserspaceAllocator = UserspaceAllocator::new();

global_asm!(include_str!("start.S"));

#[no_mangle]
fn main() {
    GLOBAL_ALLOCATOR.initialize_min_size();
    let mut log_output = Handle::open(1, &[]).unwrap();
    log_output.write(b"Hello from shell_program (/main)\n", &[]);

    return;
    // let mut process_egg = process_egg_from_elf_file(&open_file("/other_prog", &[]).unwrap()).unwrap();
    // process_egg.hatch();
}
