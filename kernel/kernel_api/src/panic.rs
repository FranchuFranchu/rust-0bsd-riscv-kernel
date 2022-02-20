#![allow(rust_analyzer::inactive_code)]

use alloc::{
    alloc::Global,
    string::{String, ToString},
};
use core::{
    any::TypeId,
    clone,
    fmt::{Arguments, Write},
    panic::PanicInfo,
};

use crate::Handle;

fn panic_handler_with_stream(info: &PanicInfo, out_stream: &mut impl core::fmt::Write) {
    let none_args = format_args!("unknown");
    let msg = info.message().unwrap_or(&none_args);

    if let Some(e) = info.location() {
        write!(
            out_stream,
            "thread '{}' panicked, '{}' at '{}'\n",
            "<userspace thread>", msg, e
        );
    } else {
        write!(
            out_stream,
            "thread '{}' panicked, '{}'\n",
            "<userspace thread>", msg,
        );
    }
}

// https://doc.rust-lang.org/src/std/panicking.rs.html#183
#[cfg(all(feature = "panic", not(test)))]
#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    crate::println_crate!("{:?}", info.message());
    match Handle::open(1, &[]) {
        Ok(mut log_output) => {
            panic_handler_with_stream(info, &mut log_output);
        }
        Err(e) => crate::exit(),
    }
    crate::exit()
}
