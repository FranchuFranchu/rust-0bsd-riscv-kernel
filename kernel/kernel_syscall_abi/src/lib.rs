#![no_std]

use num_enum::*;
#[repr(usize)]
#[derive(IntoPrimitive, FromPrimitive, Debug)]
pub enum SyscallNumbers {
    // Kills the task
    Exit = 1,
    // Marks this task as "yielded" until it gets woken up by a Waker
    Yield = 2,

    // Page operations
    AllocPages = 3,
    FreePages = 4,
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