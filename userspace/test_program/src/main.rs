#![feature(lang_items, global_asm)]
#![no_std]
#![no_main]

use core::panic::PanicInfo;
use kernel_api::Handle;

#[no_mangle]
fn main() {
	let log_output = Handle::open(1, &[]);
	log_output.write(b"Hello, world from Rust\n", &[]);
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
	loop {}
}

global_asm!(include_str!("start.S"));