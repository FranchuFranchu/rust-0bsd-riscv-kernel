#[macro_export]
macro_rules! print
{
	($($args:tt)+) => (#[allow(unused_unsafe)] {
			use core::fmt::Write;
			let mut log_output = Handle::open(1, &[]);
			let _ = write!(log_output, $($args)+);
			});
}
#[macro_export]
macro_rules! println
{
	() => ({
		   ::kernel_api::print!("\r\n")
		   });
	($fmt:expr) => ({
			::kernel_api::print!(concat!($fmt, "\r\n"))
			});
	($fmt:expr, $($args:tt)+) => ({
			::kernel_api::print!(concat!($fmt, "\r\n"), $($args)+)
			});
}
