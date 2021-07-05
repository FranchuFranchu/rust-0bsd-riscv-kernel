pub fn test_task() {
	println!("{:?}","Task 1");
	loop {
		crate::cpu::wfi();
		println!("{:?}", "Re-entered");
	};
}