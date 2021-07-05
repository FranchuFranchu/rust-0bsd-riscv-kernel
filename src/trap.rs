use crate::{process::try_get_process, sbi};

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
	println!("Trap from PID {:x}", unsafe { (*frame).pid });
	println!("\x1b[1;35mV ENTER TRAP\x1b[0m");
	
	
	if is_interrupt {
		match cause {
			// See table 3.6 of the Privileged specification
			
			// Supervisor software interrupt
			1 => {
				
			}
			// Supervisor timer interrupt
			5 => {
				// First, we set the next timer infitely far into the future so that it doesn't get triggered again
				sbi::set_absolute_timer(2_u64.pow(63)).unwrap();
				
				println!("scheduling...");
				let new_pid = crate::scheduler::schedule();
				
				if new_pid == 0 {
					panic!("Nothing to schedule!");
				}
				
				
				println!("New pid {:x}", new_pid);
				let lock = try_get_process(&new_pid);
				
				println!("Lock pid {:x}", unsafe { lock.read().trap_frame.pid });
				
				println!("Lock pid {:x}", unsafe { lock.read().trap_frame.pid });
				// println!("Locking..");
				let mut guard = lock.write();
				// println!("Locked");
				unsafe { lock.force_write_unlock() };
			
				
				
				sbi::set_relative_timer(1000000).unwrap();
				println!("\x1b[1;36m^ RUN TRAP\x1b[0m");
				guard.run_once();
			}
			// Supervisor external interrupt
			9 => {
				
			}
			_ => {
				println!("Unknown interrupt {}", cause);
			}
		}
	} else {
		match cause {
			8 | 9 | 10 | 11 => {
				println!("Envionment call to us happened!");
			}
			_ => {
				println!("Error with cause: {:?}", cause);
				panic!("Non-interrupt trap");
			}
		}
	}
	println!("\x1b[1;36m^ EXIT TRAP\x1b[0m");
	return epc;
}