use num_enum::{FromPrimitive, IntoPrimitive};

use crate::{
    context_switch,
    cpu::{write_satp, Registers},
    paging::{EntryBits, Paging},
    process::{self},
    test_task::boxed_slice_with_alignment,
    trap_frame::{TrapFrame, TrapFrameExt},
    trap_future_executor::block_and_return_to_userspace,
};

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

pub fn do_syscall(frame: *mut TrapFrame) {
    // First, assume that the frame is a valid pointer
    // (this may break aliasing rules though!)
    let frame_raw = frame;
    let frame = unsafe { frame_raw.as_mut().unwrap_unchecked() };

    let number = SyscallNumbers::from(frame.general_registers[Registers::A7.idx()]);
    frame.pc += 4;
    unsafe { write_satp(frame.satp) };
    use SyscallNumbers::*;
    match number {
        Exit => {
            syscall_exit(frame, 0);
        }
        Yield => {
            syscall_yield(frame);
        }
        AllocPages => {
            let virtual_address = frame.general_registers[Registers::A0.idx()];
            let size = frame.general_registers[Registers::A1.idx()];
            let size = size.unstable_div_ceil(4096) * 4096;
            let flags = frame.general_registers[Registers::A2.idx()];
            let _paging_flags = flags & EntryBits::RWX;

            // TODO fix aliasing issues!
            let mut root_table = unsafe { frame.satp_as_sv39_root_table() };
            let new_pages = boxed_slice_with_alignment(size, 4096, &0);

            root_table.map(
                &new_pages[0] as *const _ as usize,
                virtual_address,
                size,
                flags | EntryBits::VALID | EntryBits::USER,
            );
            core::mem::forget(root_table);
            core::mem::forget(new_pages);
        }
        FreePages => {}

        Open => {
            let current_pid = frame.pid;
            let fut = async move {
                use alloc::sync::Arc;

                use crate::handle::Handle;

                let id = frame.general_registers[Registers::A0.idx()];
                let options =
                    &frame.general_registers[Registers::A1.idx()..Registers::A7.idx() + 1];

                let process = crate::process::try_get_process(&frame.pid);
                let new_fd_number = process
                    .write()
                    .handles
                    .last_key_value()
                    .map(|s| s.0 + 1)
                    .unwrap_or(1);

                println!("{:?}", "will wait");
                let backend_instance =
                    crate::handle_backends::open(&id, &new_fd_number, options).await;
                println!("{:?}", backend_instance.name());
                core::mem::forget(backend_instance.clone());
                process.write().handles.insert(
                    new_fd_number,
                    Handle {
                        fd_id: new_fd_number,
                        backend: Arc::downgrade(&backend_instance),
                        backend_meta: 0,
                    },
                );
                frame.general_registers[Registers::A0.idx()] = new_fd_number;
            };
            block_and_return_to_userspace(current_pid, alloc::boxed::Box::pin(fut));
        }

        Write => {
            let current_pid = frame.pid;
            let fut = async move {
                let id = frame.general_registers[Registers::A0.idx()];

                // TODO make safe

                let buf = unsafe {
                    core::slice::from_raw_parts(
                        frame.general_registers[Registers::A1.idx()] as *const u8,
                        frame.general_registers[Registers::A2.idx()],
                    )
                };

                let options =
                    &frame.general_registers[Registers::A3.idx()..Registers::A7.idx() + 1];
                let backend = {
                    let process = crate::process::try_get_process(&frame.pid);
                    let process = process.write();
                    process.handles[&id].backend.upgrade()
                };
                backend
                    .as_ref()
                    .unwrap()
                    .write(&id, buf, options)
                    .await
                    .unwrap();
            };
            block_and_return_to_userspace(current_pid, alloc::boxed::Box::pin(fut));
        }
        Read => {
            let current_pid = frame.pid;
            let fut = async move {
                let id = frame.general_registers[Registers::A0.idx()];

                // TODO make safe
                let buf = unsafe {
                    core::slice::from_raw_parts_mut(
                        frame.general_registers[Registers::A1.idx()] as *mut u8,
                        frame.general_registers[Registers::A2.idx()],
                    )
                };
                let options =
                    &frame.general_registers[Registers::A3.idx()..Registers::A7.idx() + 1];
                let backend = {
                    let process = crate::process::try_get_process(&frame.pid);
                    let process = process.write();
                    process.handles[&id].backend.upgrade()
                };
                backend
                    .as_ref()
                    .unwrap()
                    .read(&id, buf, options)
                    .await
                    .unwrap();
            };
            block_and_return_to_userspace(current_pid, alloc::boxed::Box::pin(fut));
        }

        Unknown => {
            warn!(
                "Unknown syscall {:?}",
                frame.general_registers[Registers::A7.idx()]
            );
        }
        _ => {
            warn!("Unimplemented syscall {:?}", number);
        }
    }
    unsafe { write_satp(frame.kernel_satp) };
}

pub fn syscall_exit(frame: &mut TrapFrame, _return_code: usize) {
    crate::process::delete_process(frame.pid);
    context_switch::schedule_and_switch();
}

pub fn syscall_yield(frame: &mut TrapFrame) {
    // Set this process's state to yielded
    let p = process::try_get_process(&frame.pid);
    let mut guard = p.write();
    if guard.try_yield_maybe() {
        crate::trap::use_boot_frame_if_necessary(&*guard.trap_frame as _);
    }
    if guard.yield_maybe() {
        // If a hart switches to this process before we switch to another one, our trap frame would get corrupted
        drop(guard);
        drop(p);
        context_switch::schedule_and_switch();
    }
}

#[no_mangle]
pub unsafe extern "C" fn syscall_on_interrupt_disabled() {
    crate::std_macros::OUTPUT_LOCK.force_unlock();
    error!("Can't make a syscall while interrupts are disabled! (Maybe you're holding a lock while making a syscall?)");
    loop {}
}
