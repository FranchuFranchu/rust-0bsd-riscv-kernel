// Include assembly files

global_asm!(include_str!("asm/boot.S"));
global_asm!(include_str!("asm/trap.S"));
global_asm!(include_str!("asm/syscall.S"));
global_asm!(include_str!("asm/hart boot.S"));


// Link some assembly symbols
#[allow(clashing_extern_declarations)]
extern "C" {
	#[link_name = "do_supervisor_syscall"]
	pub fn do_supervisor_syscall_0(number: usize);
	#[link_name = "do_supervisor_syscall"]
	pub fn do_supervisor_syscall_1(number: usize, a0: usize);
	#[link_name = "do_supervisor_syscall"]
	pub fn do_supervisor_syscall_2(number: usize, a0: usize, a1: usize);
	// todo complete this
	#[link_name = "do_supervisor_syscall"]
	pub fn do_supervisor_syscall_3(number: usize, a0: usize, a1: usize);
	#[link_name = "do_supervisor_syscall"]
	pub fn do_supervisor_syscall_4(number: usize, a0: usize, a1: usize);
	#[link_name = "do_supervisor_syscall"]
	pub fn do_supervisor_syscall_5(number: usize, a0: usize, a1: usize);
	#[link_name = "do_supervisor_syscall"]
	pub fn do_supervisor_syscall_6(number: usize, a0: usize, a1: usize);
	#[link_name = "do_supervisor_syscall"]
	pub fn do_supervisor_syscall_7(number: usize, a0: usize, a1: usize);
}