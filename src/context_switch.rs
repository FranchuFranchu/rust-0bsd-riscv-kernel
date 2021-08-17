use alloc::sync::Arc;

use crate::{cpu, process::{self, ProcessState}, scheduler};



/// Trigger a context switch. Must be called from an interrupt context.
pub fn context_switch(pid: &usize) -> ! {
	// Switch to the next process

	// We unsafely get the internal value of the RwLock by locking and then lowering the write count
	// If we were to normally lock the RwLock, then if an interrupt gets triggered 
	// while the process is running, the interrupt handler wouldn't be able to modify the process trap frame
	// because we would be locking the process.
	// We also do this because run_once never returns, so the lock would never get dropped otherwise.

	// We'll also decrement the read count of the Arc to prevent memory leaks
	let lock = process::try_get_process(pid);


	let mut guard = lock.write();
	
	// Unlock the write lock
	unsafe { lock.force_write_unlock() };
	// Decrement the Arc refcount
	unsafe { 
		let raw = Arc::into_raw(lock.clone());
		Arc::decrement_strong_count(raw);
		Arc::decrement_strong_count(raw);
	};

	guard.run_once()
}

pub fn make_this_process_pending() {
	match unsafe { process::weak_get_process(&(*cpu::read_sscratch()).pid) }.upgrade() {
		None => {
			// probably a boot process
			
		},
		Some(p) => {
			p.write().state = ProcessState::Pending
		}
	}
}

pub fn schedule_and_switch() -> !  {
	let new_pid = scheduler::schedule();
	if new_pid == 0 {
		// Nothing left to schedule
		// Check if it's just that all processes have yielded or that they have been deleted
		if process::PROCESS_SCHED_QUEUE.read().len() == 0 {
			panic!("No processes alive, nothing left to schedule!");
		} else {
			// Just wait for something to happen.
			warn!("All processes have yielded");
			process::idle()
		}
	}
	
	context_switch(&new_pid)
}