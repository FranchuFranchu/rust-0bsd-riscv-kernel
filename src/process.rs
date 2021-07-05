use core::{mem::MaybeUninit, pin::Pin};
use alloc::{boxed::Box, collections::{BTreeMap, VecDeque}, sync::{Arc, Weak}};
use spin::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::trap::TrapFrame;
use crate::cpu::Registers;

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
	pub kernel_allocated_stack: Option<Box<[u8; 4096]>>
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
		
		println!("Switch to frame at \x1b[32m{:?}\x1b[0m", frame_pointer);
		
		// Switch to the trap frame
		unsafe { switch_to_supervisor_frame(frame_pointer) };
		unreachable!();
	}
}


pub static mut PROCESSES: MaybeUninit<RwLock<BTreeMap<usize, Arc<RwLock<Process>>>>> = MaybeUninit::uninit();
// PIDs each hart is executing
static mut HART_PIDS: MaybeUninit<RwLock<VecDeque<usize>>> = MaybeUninit::uninit();
pub static mut PROCESS_SCHED_QUEUE: MaybeUninit<VecDeque<usize>> = MaybeUninit::uninit();

pub fn init() {
	unsafe { PROCESSES = MaybeUninit::new(RwLock::new(BTreeMap::new())) };
	unsafe { PROCESS_SCHED_QUEUE = MaybeUninit::new(VecDeque::new()) };
	unsafe { HART_PIDS = MaybeUninit::new(RwLock::new(VecDeque::new())) };
}

// All functions after this are only safe when init() has been called


// Marks the process executed as the current hart as pending
pub fn finish_executing_process(pid: usize) {
	if pid == 0 || pid == 1 {
		// Boot process can't be stopped
		return;
	}
	try_get_process(&pid).write().state = ProcessState::Pending;
	println!("Made process pending");
}

/// Creates a supervisor process and returns PID
/// SAFETY: Only when function is a valid function pointer (with)
pub fn new_supervisor_process_int(function: usize, a0: usize) -> usize {
	let mut pid = 2;
	for this_pid in pid.. {
		if !(unsafe { PROCESSES.assume_init_mut().read().contains_key(&this_pid) }) {
			pid = this_pid;
			break;
		}
	}
	println!("chosen pid: {}", pid);
	
	let mut process = Process {
		is_supervisor: true,
		file_descriptors: BTreeMap::new(),
		trap_frame: Pin::new(Box::new(TrapFrame::zeroed())),
		state: ProcessState::Pending,
		kernel_allocated_stack: None,
	};
	
	// Set the initial state for the process
	process.trap_frame.general_registers[Registers::A0.idx()] = a0;
	process.trap_frame.general_registers[Registers::Ra.idx()] = process_return_address as usize; 
	process.trap_frame.pc = function;
	
	process.trap_frame.pid = pid;
	
	
	// Create a small stack for this process
	let process_stack = [0u8; 4096];
	// Move it to a Box
	let process_stack = Box::new(process_stack);
	
	process.trap_frame.general_registers[Registers::Sp.idx()] = process_stack.as_ptr() as usize + 4096; 
	
	// Wrap the process in a lock
	let process = RwLock::new(process);
	// Move the process into an Arc
	let process = Arc::new(process);
	
	
	
	unsafe { PROCESSES.assume_init_mut() }.write().insert(pid, process);
	// Schedule the process
	unsafe { PROCESS_SCHED_QUEUE.assume_init_mut().push_back(pid) }
	
	println!("{:?}", unsafe { PROCESSES.assume_init_mut() }.read());
	
	pid
}

#[no_mangle]
pub extern "C" fn process_return_address() {
	// TODO tidy up this
	println!("A process has returned. This shouldn't really happen");
	//finish_executing_process();
	
	
	
	loop {
		crate::cpu::wfi();
	}
}

pub fn new_supervisor_process(function: fn()) -> usize {
	new_supervisor_process_int(function as usize, 0 /* a0 doesn't matter */ )
}

pub fn new_supervisor_process_argument(function: fn(usize), a0: usize) -> usize {
	new_supervisor_process_int(function as usize, a0 )
}


pub fn delete_process(pid: usize) {
	println!("Remove {:?}", pid);
	unsafe { PROCESSES.assume_init_mut() }.write().remove(&pid);
}

// This returns an empty Weak if the process doesn't exist
pub fn weak_get_process(pid: &usize) -> Weak<RwLock<Process>>  {
	unsafe { PROCESSES.assume_init_mut() }.read().get(pid).map(|arc| Arc::downgrade(arc)).unwrap_or(Weak::new())
}

// This assumes that the process exists and panics if it doesn't
// Also acts as a strong reference to the process
pub fn try_get_process(pid: &usize) -> Arc<RwLock<Process>>  {
	unsafe { PROCESSES.assume_init_mut() }.read().get(pid).unwrap().clone()
}