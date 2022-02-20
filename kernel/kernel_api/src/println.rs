#[macro_export]
macro_rules! print
{
	($($args:tt)+) => (#[allow(unused_unsafe)] {
			use core::fmt::Write;
			let mut log_output = ::kernel_api::Handle::open(1, &[]).unwrap();
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

#[macro_export]
macro_rules! print_crate
{
	($($args:tt)+) => (#[allow(unused_unsafe)] {
			use core::fmt::Write;
			let mut log_output = crate::Handle::open(1, &[]).unwrap();
			let _ = write!(log_output, $($args)+);
			});
}
#[macro_export]
macro_rules! println_crate
{
	() => ({
		   crate::print_crate!("\r\n")
		   });
	($fmt:expr) => ({
			crate::print_crate!(concat!($fmt, "\r\n"))
			});
	($fmt:expr, $($args:tt)+) => ({
			crate::print_crate!(concat!($fmt, "\r\n"), $($args)+)
			});
}
