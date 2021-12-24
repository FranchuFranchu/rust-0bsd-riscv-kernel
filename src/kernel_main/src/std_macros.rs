use crate::lock::shared::Mutex;

// from osblog

pub static OUTPUT_LOCK: Mutex<()> = Mutex::new(());

#[macro_export]
macro_rules! print
{
	($($args:tt)+) => (#[allow(unused_unsafe)] {
			// Lock the output to prevent lines mixing between each other
			use core::fmt::Write;
			//let l = crate::std_macros::OUTPUT_LOCK.lock();
			let _ = write!(unsafe {crate::drivers::uart::Uart::new(0x1000_0000)}, $($args)+);
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
