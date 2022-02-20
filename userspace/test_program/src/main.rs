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
use kernel_api::{handle::open_file, println, process_egg::ProcessEgg, Handle, UserspaceAllocator};

extern crate alloc;

#[global_allocator]
static GLOBAL_ALLOCATOR: UserspaceAllocator = UserspaceAllocator::new();
global_asm!(include_str!("start.S"));

#[no_mangle]
fn main() {
    println!("{:?}", "a");
    GLOBAL_ALLOCATOR.initialize_min_size();
    let mut log_output = Handle::open(1, &[]).unwrap();
    println!("{:?}", "b");
    log_output.write(b"Hello, world from Rust\n", &[]);
    println!("{:?}", "c");

    return; // Rest of the code is for the future
    let file = open_file("/other_prog", &[]).unwrap();
    let mut fc = Vec::new();
    let mut buffer = Vec::new();
    buffer.resize(4096, 0);
    let mut buffer = buffer.into_boxed_slice();

    loop {
        let read = file.read(&mut buffer, &[]).unwrap();
        println!("read {:?}", read);
        fc.extend_from_slice(&buffer[..read]);
        if read == 0 {
            break;
        }
    }

    let mut egg_handle = ProcessEgg::new().unwrap();

    let elf_file = elf_rs::Elf::from_bytes(&fc);
    if let elf_rs::Elf::Elf64(e) = elf_file.unwrap() {
        println!("{:?}", 1);
        for p in e.program_header_iter() {
            if p.ph.memsz() as usize == 0 {
                continue;
            }
            // This is our buffer with the program's code
            //root_table.map(&segment[0] as *const u8 as usize, p.ph.vaddr() as usize, (p.ph.memsz() as usize).max(4096), EntryBits::EXECUTE | EntryBits::VALID | EntryBits::READ);
            egg_handle.set_memory(p.ph.vaddr() as usize, p.segment());
        }
        println!("{:?}", 2);
        egg_handle.set_start_address(e.header().entry_point() as usize)
    }
    egg_handle.hatch();
    println!("{:?}", "finished");
}
