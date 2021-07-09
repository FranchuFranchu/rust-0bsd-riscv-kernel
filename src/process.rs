use core::{mem::MaybeUninit, pin::Pin};
use alloc::{boxed::Box, collections::{BTreeMap}, sync::{Arc, Weak}, vec::Vec};
use spin::{RwLock};

use crate::{cpu, trap::TrapFrame};
use crate::cpu::Registers;
use aligned::{A16, Aligned};

// 0BSD
#[derive(Debug)]
pub struct FileDescriptor {
	fd_id: usize,
	backend: usize,
	backend_meta: usize,
}

#[derive(Debug)]
pub enum ProcessState {
	// Currently running (right now)
	Running,
	// Waiting for a future (or any other blocking action)
	Yielded,
	// Not scheduled and not waiting for any future
	Pending,
}

#[derive(Debug)]
pub struct Process {
	/// The process ID of the process can be fetched by getting trap_frame.pid
	pub is_supervisor: bool,
	pub state: ProcessState,
	pub file_descriptors: BTreeMap<usize, FileDescriptor>,
	pub trap_frame: Pin<Box<TrapFrame>>,
	
	/// For supervisor mode the kernel initially creates a small stack page for this process
	/// This is where it's stored
	pub kernel_allocated_stack: Option<Box<Aligned<A16, [u8; 4096]>>>
}

extern "C" {
	// Never returns (outside from interrupts)
	fn switch_to_supervisor_frame(trap_frame: *mut TrapFrame) -> !;
}

impl Process {
	pub fn has_read_access(&self, address: usize, size: usize) -> bool {
		if self.is_supervisor {
			return true;
		}
		return false;
	}
	pub fn has_write_access(&self, address: usize, size: usize) -> bool {
		if self.is_supervisor {
			return true;
		}
		return false;
	}
	pub fn can_be_scheduled(&self) -> bool {
		match self.state {
			ProcessState::Pending => true,
			_ => false,
		}
	}
	// Uses this hart to execute this process until the next context switch happens
	// This function essentially never returns because it runs until an interrupt happens
	pub fn run_once(&mut self) -> ! {
		// Get a raw pointer to the Box's data (which is the trap frame)
		let frame_pointer = Pin::as_ref(&self.trap_frame).get_ref() as *const TrapFrame as *mut TrapFrame;
		
		debug!("Switch to frame at \x1b[32m{:?}\x1b[0m", frame_pointer);
		
		// Switch to the trap frame
		unsafe { switch_to_supervisor_frame(frame_pointer) };
		unreachable!();
	}
}


pub static PROCESSES: RwLock<BTreeMap<usize, Arc<RwLock<Process>>>> = RwLock::new(BTreeMap::new());
pub static PROCESS_SCHED_QUEUE: RwLock<Vec<Weak<RwLock<Process>>>> = RwLock::new(Vec::new());

pub fn init() {
}

// All functions after this are only safe when init() has been called


// Marks the process executed as the current hart as pending
pub fn finish_executing_process(pid: usize) {
	if pid == 0 || pid == 1 {
		// Boot process can't be stopped
		return;
	}
	try_get_process(&pid).write().state = ProcessState::Pending;
	debug!("Made process pending");
}

/// Creates a supervisor process and returns PID
/// SAFETY: Only when function is a valid function pointer (with)
pub fn new_supervisor_process_int(function: usize, a0: usize) -> usize {
	let mut pid = 2;
	for this_pid in pid.. {
		if !PROCESSES.read().contains_key(&this_pid) {
			pid = this_pid;
			break;
		}
	}
	debug!("chosen pid: {}", pid);
	
	let mut process = Process {
		is_supervisor: true,
		file_descriptors: BTreeMap::new(),
		trap_frame: Pin::new(Box::new(TrapFrame::zeroed())),
		state: ProcessState::Pending,
		kernel_allocated_stack: None,
	};
	
	// Set the initial state for the process
	process.trap_frame.general_registers[Registers::A0.idx()] = a0;
	// NOTE change the function for user mode
	process.trap_frame.general_registers[Registers::Ra.idx()] = process_return_address_supervisor as usize; 
	process.trap_frame.pc = function;
	
	process.trap_frame.pid = pid;
	
	
	// Create a small stack for this process
	let process_stack = [0u8; 4096];
	// Move it to a Box
	let process_stack = Box::new(process_stack);
	
	process.trap_frame.general_registers[Registers::Sp.idx()] = process_stack.as_ptr() as usize + 4096 - 0x10; 
	
	// Wrap the process in a lock
	let process = RwLock::new(process);
	// Move the process into an Arc
	let process = Arc::new(process);
	
	
	
	// Schedule the process
	PROCESS_SCHED_QUEUE.write().push(Arc::downgrade(&process));
	
	PROCESSES.write().insert(pid, process);
	
	debug!("{:?}", PROCESSES.read());
	
	pid
}

#[no_mangle]
pub extern "C" fn process_return_address_supervisor() {
	debug!("{:?}", "Process return address");
	// Run a syscall that deletes the process
	crate::syscall::syscall_exit(unsafe { cpu::read_sscratch().as_mut().unwrap_unchecked() }, 0);
}

pub fn new_supervisor_process(function: fn()) -> usize {
	new_supervisor_process_int(function as usize, 0 /* a0 doesn't matter */ )
}

pub fn new_supervisor_process_argument(function: fn(usize), a0: usize) -> usize {
	new_supervisor_process_int(function as usize, a0 )
}


pub fn delete_process(pid: usize) {
	debug!("Remove {:?}", pid);
	PROCESSES.write().remove(&pid);
}

// This returns an empty Weak if the process doesn't exist
pub fn weak_get_process(pid: &usize) -> Weak<RwLock<Process>>  {
	PROCESSES.read().get(pid).map(|arc| Arc::downgrade(arc)).unwrap_or(Weak::new())
}

// This assumes that the process exists and panics if it doesn't
// Also acts as a strong reference to the process
pub fn try_get_process(pid: &usize) -> Arc<RwLock<Process>>  {
	PROCESSES.read().get(pid).unwrap().clone()
}