use kernel_syscall_abi::*;

use crate::{
    context_switch,
    cpu::{write_satp, Registers},
    paging::{EntryBits, Paging},
    process::{self, try_get_process},
    trap_frame::{TrapFrame, TrapFrameExt},
    trap_future_executor::block_and_return_to_userspace,
};

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
            let physical_addr = frame.general_registers[Registers::A1.idx()];
            let size = frame.general_registers[Registers::A2.idx()];
            let mut flags = frame.general_registers[Registers::A3.idx()];
            flags = (flags & !EntryBits::ADDRESS_MASK) | EntryBits::USER;

            // TODO fix aliasing issues!
            let mut root_table = unsafe { frame.satp_as_sv39_root_table() };

            let physical_addr = if physical_addr == usize::MAX {
                let new_pages = kernel_util::boxed_slice_with_alignment(size, 4096, &0u8);
                let physical_addr = &new_pages[0] as *const u8 as usize;
                core::mem::forget(new_pages);
                physical_addr
            } else {
                if try_get_process(&mut frame.pid).read().user_id == 0 {
                    physical_addr
                } else {
                    unimplemented!(
                        "Mapping virtual address space to physical address space provided by user"
                    )
                }
            };

            let virtual_address = if virtual_address == usize::MAX {
                // Find a free set of contiguous pages
                let mut run_length = 0;
                let mut first_page_in_set = None;
                for i in (0x1000..0x80000000).step_by(4096) {
                    if let Some(page) = unsafe { root_table.query(i) } {
                        // This page is used
                        if page & EntryBits::USER != 0 {
                            run_length = 0;
                            continue;
                        }
                    }

                    // This page is free and unmapped
                    if run_length >= size {
                        first_page_in_set = Some(i - run_length);
                        break;
                    }
                    run_length += 4096;
                }
                first_page_in_set.expect("No free page found!")
            } else {
                virtual_address
            };

            let size = size.div_ceil(4096) * 4096;
            let paging_flags = flags & EntryBits::RWX;

            root_table.map(
                physical_addr,
                virtual_address,
                size,
                paging_flags | EntryBits::VALID | EntryBits::USER,
            );
            core::mem::forget(root_table);
            frame.general_registers[Registers::A0.idx()] = virtual_address;
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

                let backend_instance =
                    crate::handle_backends::open(&id, &new_fd_number, options).await;

                match backend_instance {
                    Ok(backend_instance) => {
                        process.write().handles.insert(
                            new_fd_number,
                            Handle {
                                fd_id: new_fd_number,
                                backend: Arc::downgrade(&backend_instance),
                                backend_meta: 0,
                            },
                        );
                        core::mem::forget(backend_instance.clone());
                        frame.general_registers[Registers::A0.idx()] = new_fd_number;
                        frame.general_registers[Registers::A1.idx()] = 0;
                    }
                    Err(e) => {
                        frame.general_registers[Registers::A0.idx()] = e.0;
                    }
                }
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
                let result = backend.as_ref().unwrap().read(&id, buf, options).await;
                match result {
                    Ok(o) => {
                        frame.general_registers[Registers::A0.idx()] = o;
                        frame.general_registers[Registers::A1.idx()] = 0;
                    }
                    Err(e) => {
                        frame.general_registers[Registers::A0.idx()] = usize::MAX;
                        frame.general_registers[Registers::A1.idx()] = e.0;
                        // Copy the other parameters
                        frame.general_registers[Registers::A2.idx()
                            ..(Registers::A2.idx() + e.1.len()).min(Registers::A7.idx())]
                            .copy_from_slice(&e.1);
                    }
                }
            };
            block_and_return_to_userspace(current_pid, alloc::boxed::Box::pin(fut));
        }
        Close => {
            let id = frame.general_registers[Registers::A0.idx()];

            let options = &frame.general_registers[Registers::A1.idx()..Registers::A7.idx() + 1];

            let backend = {
                let process = crate::process::try_get_process(&frame.pid);
                let process = process.write();
                process.handles[&id].backend.upgrade()
            };

            backend.unwrap().close(&id, &options);
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
