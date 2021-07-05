// 0BSD

use crate::process::{PROCESS_SCHED_QUEUE, try_get_process};

// Return the next PID to be run
pub fn schedule() -> usize {
	let process_sched_queue = unsafe { PROCESS_SCHED_QUEUE.assume_init_mut() };
	let mut pid = 0;
	
	for this_pid in process_sched_queue.iter() {
		println!("pid {:?}", this_pid);
		
		println!("sp 0x{:x}", crate::cpu::read_sp());
		println!("tf {:?}", &unsafe { crate::process::PROCESSES.assume_init_mut() }.read().get(&this_pid).unwrap().read().trap_frame.pid as *const usize);
		let process = try_get_process(this_pid).read().trap_frame.pid;
		println!("{:?}", unsafe { crate::process::PROCESSES.assume_init_mut() }.read().get(&this_pid));
		println!("{:?}", "finished");
		assert!(*this_pid == try_get_process(this_pid).read().trap_frame.pid, "Process's internal trap frame PID and process index in the process map do not match!"); 
		if try_get_process(this_pid).read().can_be_scheduled() {
			pid = *this_pid;
			break;
		}
	}
	
	if pid == 0 {
		// Don't schedule anything
		return 0;
	}
	
	process_sched_queue.rotate_left(1);
	
	return pid;
}