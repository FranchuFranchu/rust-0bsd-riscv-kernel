use core::{pin::Pin, future::Future};
use alloc::{boxed::Box, collections::{BTreeMap}, sync::{Arc, Weak}, vec::Vec};
use spin::{RwLock, RwLockWriteGuard, RwLockReadGuard};

use core::task::{Waker, RawWaker, RawWakerVTable};
use crate::{context_switch::{self, context_switch}, cpu, scheduler::schedule_next_slice, syscall::syscall_exit, trap::TrapFrame};
use crate::cpu::Registers;
use aligned::{A16, Aligned};

pub const TASK_STACK_SIZE: usize = 4096 * 8;
pub const PROCESS_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(
			/* clone */ Process::waker_clone,
			/* wake */ Process::waker_wake,
			/* wake_by_ref */ Process::waker_wake_by_ref,
			/* drop */ Process::waker_drop,
		);
pub static PROCESSES: RwLock<BTreeMap<usize, Arc<RwLock<Process>>>> = RwLock::new(BTreeMap::new());
pub static PROCESS_SCHED_QUEUE: RwLock<Vec<Weak<RwLock<Process>>>> = RwLock::new(Vec::new());


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
	pub kernel_allocated_stack: Option<Box<Aligned<A16, [u8; TASK_STACK_SIZE]>>>
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
		
		debug!("Switch to frame at \x1b[32m{:?}\x1b[0m (PC {:x})", frame_pointer, unsafe {(*frame_pointer).pc});
		
		// Switch to the trap frame
		unsafe { switch_to_supervisor_frame(frame_pointer) };
		unreachable!();
	}
	
	// These are the waker methods
	// They turn a process in Yielded state to a process in Pending state
	// The data parameter is the return value of into_raw for a Box<Weak<Process>>
	pub unsafe fn waker_clone(data: *const ()) -> RawWaker {
		let obj = Box::from_raw(data as *mut Weak<RwLock<Self>>);
		let new_waker = RawWaker::new(Box::into_raw(obj.clone()) as _, &PROCESS_WAKER_VTABLE);
		Box::leak(obj);
		new_waker
	}
	pub unsafe fn waker_wake(data: *const ()) {
		Self::waker_wake_by_ref(data)
	}
	pub unsafe fn waker_wake_by_ref(data: *const ()) {
		// The box re-acquires ownership of the RwLock<Self>
		let process: Box<Weak<RwLock<Self>>> = Box::from_raw(data as _);
		let process_internal = process.upgrade().expect("Waited process is gone!");
		process_internal.write().state = ProcessState::Pending;
		// Make the box lose ownership of the RwLock<Self>
		Box::leak(process);
	}
	pub unsafe fn waker_drop(data: *const ()) {
		// Re-create the box for this waker and then drop it to prevent memory leaks
		drop(Box::from_raw(data as *mut Weak<RwLock<Self>>));
	}
	
	/// This creates a Waker that makes this process a Pending process when woken
	pub fn construct_waker(&self) -> Waker {
		// Create a weak pointer to a RwLock<Self> and then erase its type
		let raw_pointer = Box::into_raw(Box::new(weak_get_process(&self.trap_frame.pid))) as *const ();
		// Create a waker with the pointer as the data
		unsafe { Waker::from_raw(RawWaker::new(raw_pointer, &PROCESS_WAKER_VTABLE)) }
	}
	
	/// Polls a future from this process. The waker is this processes' waker
	pub fn poll_future<T: Future>(&mut self, future: Pin<&mut T>) -> core::task::Poll<<T as Future>::Output> {
		use core::task::Poll;
		let poll_result = future.poll(&mut core::task::Context::from_waker(&self.construct_waker()));
		
		match poll_result { 
			Poll::Pending => {
				// Mark the task as yielded
				self.state = ProcessState::Yielded;
				schedule_next_slice(0);
			},
			_ => {},
		}
		
		return poll_result
	}
	pub fn this() -> Arc<RwLock<Process>> {
		unsafe { try_get_process(&cpu::read_sscratch().as_ref().expect("Not running on a process!").pid) }
	}
}
pub fn init() {
}

// All functions after this are only safe when init() has been called
// (but init doesn't do anything yet, so nothing bad happens)


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
	let process_stack = alloc::vec![0; TASK_STACK_SIZE].into_boxed_slice();
	
	
	process.trap_frame.general_registers[Registers::Sp.idx()] = process_stack.as_ptr() as usize + TASK_STACK_SIZE - 0x10; 
	
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
	unsafe {
		llvm_asm!(r"
			li a7, 1
			# Trigger a timer interrupt
			csrr t0, sip
			# Set SSIP
			ori t0, t0, 2
			csrw sip, t0
		"::: "a7", "t0")
	}
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

fn idle_entry_point() {
	cpu::wfi();
}

pub fn idle_forever_entry_point() {
	loop {
		cpu::wfi();
	}
}

/// Starts a process that wfi()s once, immediately switches to the process, then exits. 
/// Must be called from an interrupt context.
pub fn idle() -> ! {
	let pid = new_supervisor_process(idle_entry_point);
	context_switch::context_switch(&pid)
}