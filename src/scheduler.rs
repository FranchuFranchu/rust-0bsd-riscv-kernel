// 0BSD

use crate::{process::{PROCESS_SCHED_QUEUE}, sbi};

// Return the next PID to be run
pub fn schedule() -> usize {
	let mut process_sched_queue = PROCESS_SCHED_QUEUE.write();
	let mut pid = 0;
	
	// Generally speaking, we're going to have at most one process deleted each time schedule() is called
	// so we don't need a vector to store removed indexes
	let mut removed_index = 0;
	
	for (idx, this_process) in process_sched_queue.iter().enumerate() {
		debug!("{:?}", this_process.strong_count());
		match this_process.upgrade()   {
			// The process still exists
			Some(strong) => {
				debug!("{:?}", "Still exists");
				if strong.read().can_be_scheduled() {
					pid = strong.read().trap_frame.pid;
					break;
				}
			},
			// The process doesn't exist anymore. Remove it from the sched queue
			None => {
				removed_index = idx;
			},
		}
		
	}
	
	if removed_index != 0 {
		process_sched_queue.remove(removed_index);
	}
	
	if pid == 0 {
		// Don't schedule anything
		return 0;
	}
	
	process_sched_queue.rotate_left(1);
	
	return pid;
}

pub fn schedule_next_slice(slices: u64) {
	sbi::set_relative_timer(slices * 50_000_0).unwrap();
}