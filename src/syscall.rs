use num_enum::{FromPrimitive, IntoPrimitive};

use crate::{context_switch, cpu::Registers, process::{self, ProcessState}, trap::TrapFrame};

#[repr(usize)]
#[derive(IntoPrimitive, FromPrimitive)]
#[derive(Debug)]
pub enum SyscallNumbers {
	// Kills the task
    Exit = 1,
    // Marks this task as "yielded" until it gets woken up by a Waker
    Yield = 2,
    // File descriptor operations
    Open = 0x10,
    Read,
    Write,
    Close,
    // Gets the min and max amount of bytes which are available in the fd queue for reading without blocking
    Available,
    Seek,
    Truncate,
    Tell,
    
    // Future operations (for asynchronous tasks in the kernel or in other processes)
    // Creates a new future for use in other processes
    FutureCreate = 0x20,
    // Marks a future you made as complete
    FutureComplete,
    // Checks if a current future is done
    FutureIsDone,
    // Yields until the given future is completed
    FutureAwait,
    // Clones a future many times, so that completing the "orignal" future causes all the other futures to complete too
    FutureClone,
    // Creates a future that completes when any of the given futures complete
    FutureOr,
    
    
    
    #[num_enum(default)]
    Unknown,
}

pub fn do_syscall(frame: *mut TrapFrame) {
    // First, assume that the frame is a valid pointer
    // (this may break aliasing rules though!)
    let frame_raw = frame;
    let frame = unsafe { frame_raw.as_mut().unwrap_unchecked() };
    
    let number = SyscallNumbers::from(frame.general_registers[Registers::A7.idx()]);
    use SyscallNumbers::*;
    match number {
        Exit => {
            syscall_exit(frame, 0);
        },
        Yield => {
            syscall_yield(frame);
        },
        Unknown => {
            warn!("Unknown syscall {:?}", frame.general_registers[Registers::A7.idx()]);
        },
        _ => {
            warn!("Unimplemented syscall {:?}", number);
        }
    }
}

pub fn syscall_exit(frame: &mut TrapFrame, return_code: usize) {
    crate::process::delete_process(frame.pid);
    context_switch::schedule_and_switch();
}

pub fn syscall_yield(frame: &mut TrapFrame) {
    // Set this process's state to yielded
    process::try_get_process(&frame.pid).write().state = ProcessState::Yielded;
    // Immediately cause a context switch for this hart
    context_switch::schedule_and_switch();
}

#[no_mangle]
pub extern "C" fn syscall_on_interrupt_disabled() {
    error!("Can't make a syscall while interrupts are disabled! (Maybe you're holding a lock while making a syscall?)")
}