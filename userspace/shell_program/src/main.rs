#![feature(
    lang_items,
    global_asm,
    default_alloc_error_handler,
    panic_info_message,
)]
#![no_std]
#![no_main]

use alloc::{string::ToString, vec::Vec};
use core::{arch::global_asm, panic::PanicInfo, ptr::read_volatile};

use flat_bytes::Flat;
use kernel_api::{
    println, Handle,
    UserspaceAllocator, memory::alloc_pages, interrupt::wait_for_interrupt,
};

extern crate alloc;

#[global_allocator]
static GLOBAL_ALLOCATOR: UserspaceAllocator = UserspaceAllocator::new();

global_asm!(include_str!("start.S"));

// For a future program
const SONG_TIMINGS: [u8; 15] = [4, 2, 2, 2, 4, 4, 8, 4, 2, 4, 2, 2, 2, 4, 2];
const SONG_BPM: u8 = 145;


struct UartInput {
    address: *mut u8,
}

impl UartInput {
    fn new() -> Self {
        Self {
            address: alloc_pages(Some(0x1000_0000), Some(0x1000_0000), 0x1000, 7).unwrap() as *mut _
        }
    }
    fn get_byte(&mut self) -> u8 {
        wait_for_interrupt(0xa).unwrap();
        unsafe { self.address.read_volatile() }
    }
    fn put_byte(&mut self, byte: u8) {
        unsafe { self.address.write_volatile(byte) }
    }
}

#[no_mangle]
fn main() {
    loop {};
    GLOBAL_ALLOCATOR.initialize_min_size().unwrap();
    let log_output = Handle::open(1, &[]).unwrap();
    log_output.write(b"Hello from shell_program (/main)\n", &[]);
    let mut input = UartInput::new();
    let mut s = alloc::vec![];
    loop {
        let c = input.get_byte();
        input.put_byte(c);
        s.push(c as char);
        if c as char == '\n' || c as char == '\r' {
            let s = s.into_iter().collect::<alloc::string::String>();
            println!("You typed {}", s);
            break;
        }
    }
    return;
    // let mut process_egg = process_egg_from_elf_file(&open_file("/other_prog", &[]).unwrap()).unwrap();
    // process_egg.hatch();
}
