use num_enum::{FromPrimitive, IntoPrimitive};

use crate::{cpu::Registers, trap::TrapFrame};

#[repr(usize)]
#[derive(IntoPrimitive, FromPrimitive)]
#[derive(Debug)]
pub enum SyscallNumbers {
	// Kills the task
    Exit = 1,
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

fn do_syscall(frame: *mut TrapFrame) {
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
        Unknown => {
            debug!("Unknown syscall{:?}", frame.general_registers[Registers::A7.idx()]);
        },
        _ => {
            debug!("Unimplemented syscall {:?}", number);
        }
    }
}

pub fn syscall_exit(frame: &mut TrapFrame, return_code: usize) {
    // TODO also tell the scheduler to switch processes
    crate::process::delete_process(frame.pid);
    loop {
        crate::cpu::wfi();
    }
}