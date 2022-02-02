use kernel_cpu::is_paging_enabled;

use crate::{lock::shared::Mutex, virtual_buffers::new_virtual_buffer};

// from osblog

pub static UART_ADDRESS: Mutex<Option<usize>> = Mutex::new(None);
pub static OUTPUT_LOCK: Mutex<()> = Mutex::new(());

pub fn get_uart() -> crate::drivers::uart::Uart {
    let mut addr_lock = UART_ADDRESS.lock();
    let addr = match is_paging_enabled() {
        true => match &*addr_lock {
            Some(addr) => *addr,
            None => {
                let addr = new_virtual_buffer(0x1000_0000, 4096);
                *addr_lock = Some(addr);
                addr
            }
        },
        false => 0x1000_0000,
    };
    unsafe { crate::drivers::uart::Uart::new(addr) }
}

#[macro_export]
macro_rules! print
{
	($($args:tt)+) => (#[allow(unused_unsafe)] {
			// Lock the output to prevent lines mixing between each other
			use core::fmt::Write;
			//let l = crate::std_macros::OUTPUT_LOCK.lock();
			let _ = write!(crate::std_macros::get_uart(), $($args)+);
			});
}
#[macro_export]
macro_rules! println
{
	() => ({
		   print!("\r\n")
		   });
	($fmt:expr) => ({
			print!(concat!($fmt, "\r\n"))
			});
	($fmt:expr, $($args:tt)+) => ({
			print!(concat!($fmt, "\r\n"), $($args)+)
			});
}
