use spin::Mutex;

// from osblog

pub static OUTPUT_LOCK: Mutex<()> = Mutex::new(());

#[macro_export]
macro_rules! print
{
	($($args:tt)+) => ({
			// Lock the output to prevent lines mixing between each other
			let lock = crate::std_macros::OUTPUT_LOCK.lock();
			use core::fmt::Write;
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