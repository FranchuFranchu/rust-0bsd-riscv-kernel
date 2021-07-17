use alloc::sync::Arc;

use crate::{context_switch, cpu, drivers::uart, hart::get_this_hart_meta, process::try_get_process, sbi, scheduler::schedule_next_slice, syscall, timeout, timer_queue};

/// A pointer to this struct is placed in sscratch
#[derive(Default, Debug, Clone)] // No copy because they really shouldn't be copied and used without changing the PID
#[repr(C)]
pub struct TrapFrame {
	pub general_registers: [usize; 32],
	pub pc: usize,
	pub hartid: usize,
	pub pid: usize,
}

impl TrapFrame {
	pub const fn zeroed() -> Self {
		Self { general_registers: [0; 32], hartid: 0, pid: 0, pc: 0}
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
}

impl Drop for TrapFrame {
	fn drop(&mut self) {
		warn!("Trap frame for pid {} dropped", self.pid);
	}
}


#[no_mangle]
pub extern "C" fn trap_handler(
	epc: usize,
	tval: usize,
	cause: usize,
	hartid: usize,
	sstatus: usize,
	frame: *mut TrapFrame,
) -> usize {
	let is_interrupt = (cause & (usize::MAX / 2 + 1)) != 0;
	let cause = cause & 0xFFF;
	debug!("Trap from PID {:x}", unsafe { (*frame).pid });
	debug!("\x1b[1;35mV ENTER TRAP\x1b[0m");
	
	if is_interrupt {
		match cause {
			// See table 3.6 of the Privileged specification
			
			// Supervisor software interrupt
			1 => {
				// We use this as an smode-to-smode system call
				// First, clear the STIP bit
				unsafe { cpu::write_sip(cpu::read_sip() & !2) };
				 
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
						
						
						debug!("\x1b[1;36m^ RUN TRAP\x1b[0m");
						
						context_switch::schedule_and_switch();
					},
				}
				
			}
			// Supervisor external interrupt
			9 => {
				// Assume it's because of the PLIC0
				let meta = get_this_hart_meta().unwrap();
				let interrupt_id = meta.plic.claim_highest_priority();
				meta.plic.complete(interrupt_id);
				unsafe { uart::Uart::new(0x10000000) }.get().unwrap();
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
				debug!("Error with cause: {:?} *pc: {}", cause, unsafe { *((*frame).pc as *const u32)});
				panic!("Non-interrupt trap");
			}
		}
	}
	debug!("\x1b[1;36m^ EXIT TRAP\x1b[0m");

	return epc;
}