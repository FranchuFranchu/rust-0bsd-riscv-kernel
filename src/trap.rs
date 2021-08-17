use crate::{context_switch, cpu::{self, load_hartid, read_sscratch}, external_interrupt, hart::get_this_hart_meta, interrupt_context_waker, sbi, scheduler::schedule_next_slice, syscall, timeout, timer_queue};

/// A pointer to this struct is placed in sscratch
#[derive(Default, Debug, Clone)] // No copy because they really shouldn't be copied and used without changing the PID
#[repr(C)]
pub struct TrapFrame {
	pub general_registers: [usize; 32],
	pub pc: usize, // 32
	pub hartid: usize, // 33
	pub pid: usize, // 34
	/// This may be shared between different processes executing the same hart
	pub interrupt_stack: usize,
	pub flags: usize,
}

impl TrapFrame {
	pub const fn zeroed() -> Self {
		Self { general_registers: [0; 32], hartid: 0, pid: 0, pc: 0, interrupt_stack: 0, flags: 0}
	}
	// Inherit hartid, interrupt_stack, and flags from the other trap frame
	pub fn inherit_from(&mut self, other: &TrapFrame) -> &mut TrapFrame {
		self.hartid = other.hartid;
		self.interrupt_stack = other.interrupt_stack;
		self.flags = other.flags;
		self
	}
	pub fn print(&self) {
		println!("{:?}", "trap");
		for (idx, i) in self.general_registers[1..].iter().enumerate() {
			print!("0x{:0<8x} ", i);
			if idx % 4 == 0 {
				println!();
			}
		}
	}
	pub fn is_interrupt_context(&self) -> bool {
		self.flags & 1 != 0
	}
}

impl Drop for TrapFrame {
	fn drop(&mut self) {
		warn!("Trap frame for pid {} dropped", self.pid);
		if self as *const Self == read_sscratch() {
			warn!("sscratch contains a dropped trap frame! Use-after-free is likely to happen");
		}
	}
}

#[inline]
pub fn in_interrupt_context() -> bool {
	// TODO make this sound (aliasing rules?)
	unsafe { read_sscratch().as_ref().unwrap().is_interrupt_context() }
}

#[inline]
pub(crate) fn set_interrupt_context() {
	unsafe { (*read_sscratch()).flags |= 1 }
}


#[inline]
pub(crate) fn clear_interrupt_context() {
	unsafe { (*read_sscratch()).flags &= !1 }
}

/// # Safety
/// This should never really be called directly from Rust. There's just too many invariants that need to be satisfied
#[no_mangle]
pub unsafe extern "C"  fn trap_handler(
	epc: usize,
	tval: usize,
	cause: usize,
	hartid: usize,
	sstatus: usize,
	frame: *mut TrapFrame,
) -> usize {
	set_interrupt_context();
	let is_interrupt = (cause & (usize::MAX / 2 + 1)) != 0;
	let cause = cause & 0xFFF;
	debug!("Trap from PID {:x}", unsafe { (*frame).pid });
	debug!("\x1b[1;35mV ENTER TRAP\x1b[0m");
	
	
	interrupt_context_waker::wake_all();
	if is_interrupt {
		match cause {
			// See table 3.6 of the Privileged specification
			
			// Supervisor software interrupt
			1 => {
				// We use this as an smode-to-smode system call
				// First, clear the STIP bit
				unsafe { cpu::write_sip(cpu::read_sip() & !2) };
				 
				debug!("\x1b[1;36m^ SYSCALL TRAP\x1b[0m");
				syscall::do_syscall(frame);
			}
			// Supervisor timer interrupt
			5 => {
				// First, we set the next timer infitely far into the future so that it doesn't get triggered again
				sbi::set_absolute_timer(2_u64.pow(63)).unwrap();
			
				
				let event = timer_queue::last_cause();
				use timer_queue::TimerEventCause::*;
				
				match event.cause {
					TimeoutFuture => {
						timeout::on_timer_event(event.instant);
						
						timer_queue::schedule_next();
					},
					ContextSwitch => {
						debug!("scheduling...");
						
						schedule_next_slice(1);
						
						timer_queue::schedule_next();
						
						context_switch::make_this_process_pending();
						debug!("\x1b[1;36m^ RUN TRAP\x1b[0m");
						
						context_switch::schedule_and_switch();
					},
				}
				
			}
			// Supervisor external interrupt
			9 => {
				info!("Extenral interrupt");
				// Assume it's because of the PLIC0
				let meta = get_this_hart_meta().unwrap();
				let interrupt_id = meta.plic.claim_highest_priority();
				
				external_interrupt::external_interrupt(interrupt_id);
				
				meta.plic.complete(interrupt_id);
				
				println!("{:?}", "end");
			}
			_ => {
				debug!("Unknown interrupt {}", cause);
			}
		}
	} else {
		match cause {
			8 | 9 | 10 | 11 => {
				debug!("Envionment call to us happened!");
				loop {};
			},
			_ => {
				info!("Error with cause: {:?} pc: {:X} *pc: {:X}", cause, unsafe { (*frame).pc }, unsafe { *((*frame).pc as *const u32)});
				loop {} //panic!("Non-interrupt trap");
			}
		}
	}
	interrupt_context_waker::wake_all();
	
	
	debug!("\x1b[1;36m^ EXIT TRAP {}\x1b[0m", load_hartid());
	clear_interrupt_context();
	epc
}