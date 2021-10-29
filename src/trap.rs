use crate::{HART_PANIC_COUNT, context_switch, cpu::{self, load_hartid, read_satp, read_sscratch}, external_interrupt, hart::get_this_hart_meta, interrupt_context_waker, process::delete_process, sbi, scheduler::schedule_next_slice, syscall, timeout, timer_queue};

/// A pointer to this struct is placed in sscratch
#[derive(Default, Debug, Clone)] // No copy because they really shouldn't be copied and used without changing the PID
#[repr(C)]
pub struct TrapFrame {
    pub general_registers: [usize; 32],
    pub pc: usize,     // 32
    pub hartid: usize, // 33
    pub pid: usize,    // 34
    pub interrupt_stack: usize, // 35. This may be shared between different processes executing the same hart
    pub flags: usize, // 36
    pub satp: usize, // 37
    pub kernel_satp: usize, // 38
}

impl TrapFrame {
    pub const fn zeroed() -> Self {
        Self {
            general_registers: [0; 32],
            hartid: 0,
            pid: 0,
            pc: 0,
            interrupt_stack: 0,
            flags: 0,
            satp: 0,
            kernel_satp: 0,
        }
    }
    pub const fn zeroed_interrupt_context() -> Self {
        Self {
            general_registers: [0; 32],
            hartid: 0,
            pid: 0,
            pc: 0,
            interrupt_stack: 0,
            flags: 1,
            satp: 0,
            kernel_satp: 0,
        }
    }
    pub fn use_current_satp_as_kernel_satp(&mut self)  {
        self.kernel_satp = read_satp();
    }
    // Inherit hartid, interrupt_stack, and flags from the other trap frame
    pub fn inherit_from(&mut self, other: &TrapFrame) -> &mut TrapFrame {
        self.hartid = other.hartid;
        self.interrupt_stack = other.interrupt_stack;
        self.flags = other.flags;
        self.satp = other.satp;
        self
    }
    pub fn print(&self) {
        println!("{:?}", "trap");
        for (idx, i) in self.general_registers[1..].iter().enumerate() {
            print!("0x{:0<8x} ", i);
            if idx % 4 == 0 {
                println!();
            }
        }
    }
    pub fn is_interrupt_context(&self) -> bool {
        self.flags & 1 != 0
    }
    pub fn has_trapped_before(&self) -> bool {
        self.flags & 2 != 0
    }
    pub fn is_double_faulting(&self) -> bool {
        self.flags & 4 != 0
    }
    pub fn set_trapped_before(&mut self) {
        self.flags |= 2
    }
    pub fn set_double_faulting(&mut self) {
        self.flags |= 4
    }
    /// You need to be the only owner of the trap frame to make it the current one
    pub unsafe fn make_current(&mut self) {
        self.flags = (*cpu::read_sscratch()).flags;
        self.flags |= 8;
        (*cpu::read_sscratch()).flags &= !8;
        cpu::write_sscratch(self as *const TrapFrame as usize)
    }
}

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
        match cause {
            8 | 9 | 10 | 11 => {
                info!("Envionment call to us happened!");
                
                syscall::do_syscall(frame);
            }
            _ => {
                error!(
                    "Error with cause: {:?} pc: {:X} *pc: {:X}",
                    cause,
                    unsafe { (*frame).pc },
                    unsafe { *((*frame).pc as *const u32) }
                );
                // Kill the process
                delete_process((*frame).pid);
                loop {} //panic!("Non-interrupt trap");
            }
        }
    }
    interrupt_context_waker::wake_all();

    debug!("\x1b[1;36m^ EXIT TRAP {}\x1b[0m", load_hartid());
    clear_interrupt_context();
    epc
}
