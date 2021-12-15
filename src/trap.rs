//! A function that runs when a trap happens

pub use crate::trap_frame::TrapFrame;
use crate::{
    context_switch,
    cpu::{self, load_hartid, read_satp, read_sscratch, read_sstatus},
    external_interrupt,
    hart::get_this_hart_meta,
    interrupt_context_waker,
    paging::{sv39::RootTable, Table},
    process::{delete_process, try_get_process},
    sbi,
    scheduler::schedule_next_slice,
    syscall, timeout, timer_queue, HART_PANIC_COUNT,
};

/// If sscratch equals original_trap_frame, then set sscratch to the boot frame for this hart
pub fn use_boot_frame_if_necessary(original_trap_frame: *const TrapFrame) {
    if core::ptr::eq(read_sscratch(), original_trap_frame) {
        debug!("Changed frame");
        unsafe {
            get_this_hart_meta()
                .unwrap()
                .boot_frame
                .write()
                .make_current()
        };
    }
}

impl Drop for TrapFrame {
    fn drop(&mut self) {
        debug!("Trap frame for pid {} dropped", self.pid);
        if self as *const Self == read_sscratch() {
            warn!("sscratch contains a dropped trap frame! Use-after-free is likely to happen");
        }
    }
}

#[inline]
pub fn in_interrupt_context() -> bool {
    // TODO make this sound (aliasing rules?)
    unsafe { read_sscratch().as_ref().unwrap().is_interrupt_context() }
}

#[inline]
pub(crate) fn set_interrupt_context() {
    unsafe { (*read_sscratch()).flags |= 1 }
}

#[inline]
pub(crate) fn clear_interrupt_context() {
    unsafe { (*read_sscratch()).flags &= !1 }
}

struct PanicGuard {}

impl Drop for PanicGuard {
    fn drop(&mut self) {}
}

/// # Safety
/// This should never really be called directly from Rust. There's just too many invariants that need to be satisfied
#[no_mangle]
pub unsafe extern "C" fn trap_handler(
    epc: usize,
    tval: usize,
    cause: usize,
    hartid: usize,
    sstatus: usize,
    frame: *mut TrapFrame,
) -> usize {
    if HART_PANIC_COUNT.load(core::sync::atomic::Ordering::Acquire) != 0 {
        panic!("{}", "other hart panicked!");
    }

    let panic_guard = PanicGuard {};

    let is_interrupt = (cause & (usize::MAX / 2 + 1)) != 0;
    let cause = cause & 0xFFF;

    // If this is not the first trap and we're in an interrupt context, then it means this is a double fault
    // A double fault is when the fault handler faults
    if !is_interrupt
        && in_interrupt_context()
        && read_sscratch().as_ref().unwrap().is_in_fault_trap()
        && read_sscratch().as_ref().unwrap().has_trapped_before()
    {
        read_sscratch().as_mut().unwrap().set_double_faulting();
        panic!("Double fault");
    }
    read_sscratch().as_mut().unwrap().set_trapped_before();

    set_interrupt_context();
    debug!("Trap from PID {:x}", unsafe { (*frame).pid });
    debug!("\x1b[1;35mV ENTER TRAP\x1b[0m");

    interrupt_context_waker::wake_all();
    if is_interrupt {
        match cause {
            // See table 3.6 of the Privileged specification

            // Supervisor software interrupt
            1 => {
                // We use this as an smode-to-smode system call
                // First, clear the SSIP bit
                unsafe { cpu::write_sip(cpu::read_sip() & !2) };

                debug!("\x1b[1;36m^ SYSCALL TRAP\x1b[0m");
                syscall::do_syscall(frame);
            }
            // Supervisor timer interrupt
            5 => {
                // First, we set the next timer infitely far into the future so that it doesn't get triggered again
                sbi::set_absolute_timer(2_u64.pow(63)).unwrap();

                let event = timer_queue::last_cause();
                use timer_queue::TimerEventCause::*;

                match event.cause {
                    TimeoutFuture => {
                        timeout::on_timer_event(event.instant);

                        timer_queue::schedule_next();
                    }
                    ContextSwitch => {
                        debug!("scheduling...");

                        schedule_next_slice(1);

                        timer_queue::schedule_next();

                        context_switch::make_this_process_pending();

                        unsafe {
                            get_this_hart_meta()
                                .unwrap()
                                .boot_frame
                                .write()
                                .make_current()
                        };

                        context_switch::schedule_and_switch();
                    }
                }
            }
            // Supervisor external interrupt
            9 => {
                // Assume it's because of the PLIC0
                let meta = get_this_hart_meta().unwrap();
                let interrupt_id = meta.plic.claim_highest_priority();

                external_interrupt::external_interrupt(interrupt_id);

                meta.plic.complete(interrupt_id);

                // Clear the SEIP bit
                unsafe { cpu::write_sip(cpu::read_sip() & !(1 << 9)) };
            }
            _ => {
                debug!("Unknown interrupt {}", cause);
            }
        }
    } else {
        read_sscratch().as_mut().unwrap().set_in_fault_trap();
        match cause {
            8 | 9 | 10 | 11 => {
                info!("Environment call to us happened!");
                syscall::do_syscall(frame);
            }
            _ => {
                error!(
                    "Error with cause: {:?} pc: {:X} *pc: {:X} tval: {:X} mode: {}",
                    cause,
                    unsafe { (*frame).pc },
                    unsafe { *((*frame).pc as *const u32) },
                    unsafe { tval as *const u32 as usize },
                    if unsafe { read_sstatus() & 1 << 8 != 0 } {
                        "supervisor"
                    } else {
                        "user"
                    }
                );
                // Kill the process
                delete_process((*frame).pid);
                loop {} //panic!("Non-interrupt trap");
            }
        }
    }
    read_sscratch().as_mut().unwrap().clear_in_fault_trap();
    interrupt_context_waker::wake_all();

    if epc > 0x8000000 as usize && !try_get_process(&(*frame).pid).read().is_supervisor {
        panic!("{:?}", "user process aren't really meant to do this");
    }

    debug!("\x1b[1;36m^ EXIT TRAP {}\x1b[0m", load_hartid());
    clear_interrupt_context();
    epc
}
