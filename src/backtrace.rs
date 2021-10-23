use crate::{cpu::Registers, trap::TrapFrame};

extern "C" {
	fn store_to_trap_frame(frame: *const TrapFrame);
	static __eh_frame_end: u32;
	static __eh_frame_start: u32;
}

pub fn backtrace() {
	
	unsafe { println!("{:p}", &__eh_frame_start); }
	unsafe { println!("{:p}", &__eh_frame_end); }
	let frame = TrapFrame::zeroed();
	unsafe { store_to_trap_frame(&frame as *const TrapFrame) };
	let sp = frame.general_registers[Registers::S0.idx()];
	println!("{:x}", sp);
	println!("{:x}", unsafe {*((sp-8) as *const usize)});
	//let ra = frame.general_registers[Registers::Ra.idx()];
	loop {};
}