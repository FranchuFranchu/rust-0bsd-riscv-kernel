/// Start and setup new harts
use alloc::{boxed::Box, collections::BTreeMap, sync::Arc};
use core::{
    arch::asm,
    pin::Pin,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

use aligned::{Aligned, A16};

use crate::{
    cpu::{self, load_hartid},
    lock::shared::RwLock,
    plic::Plic0,
    process::{self, try_get_process, TASK_STACK_SIZE},
    s_trap_vector, sbi,
    scheduler::schedule_next_slice,
    timer_queue,
    trap::TrapFrame,
};

// Data associated with a hart
pub struct HartMeta {
    pub plic: Plic0,
    pub boot_stack: Option<Box<Aligned<A16, [u8; TASK_STACK_SIZE]>>>,
    pub boot_frame: RwLock<Pin<Box<TrapFrame>>>,
    pub is_panicking: AtomicBool,
    pub idle_process: AtomicUsize,
}

impl HartMeta {
    pub fn get_idle_process(&self) -> Option<usize> {
        match self.idle_process.load(Ordering::Acquire) {
            0 => None,
            pid => Some(pid),
        }
    }
    pub fn set_idle_process(&self, id: Option<usize>) {
        self.idle_process.store(id.unwrap_or(0), Ordering::Release);
    }
}

pub static HART_META: RwLock<BTreeMap<usize, Arc<HartMeta>>> = RwLock::new(BTreeMap::new());

pub fn get_hart_meta(hartid: usize) -> Option<Arc<HartMeta>> {
    HART_META.read().get(&hartid).cloned()
}

// Only run this from the boot hart
/// # Safety
/// When sscratch contains a valid trap frame
pub unsafe fn add_boot_hart(trap_frame: TrapFrame) {
    let meta = HartMeta {
        plic: Plic0::new_with_fdt(),
        boot_stack: None,
        boot_frame: RwLock::new(Pin::new(Box::new(trap_frame))),
        is_panicking: AtomicBool::new(false),
        idle_process: AtomicUsize::new(0),
    };
    HART_META.write().insert(load_hartid(), Arc::new(meta));
}

/// Must be run from a recently created hart
pub fn add_this_secondary_hart(hartid: usize, interrupt_sp: usize) {
    info!("Started hart");
    // Create the trap frame
    let mut trap_frame = Pin::new(Box::new(TrapFrame::zeroed_interrupt_context()));

    trap_frame.pid = 0;
    trap_frame.hartid = hartid;
    trap_frame.interrupt_stack = interrupt_sp;

    // SAFETY: trap_frame is a valid trap frame and will live as long as this hart exists
    // so sscratch will be valid and this will not invoke UB
    unsafe { cpu::write_sscratch(Pin::as_ref(&trap_frame).get_ref() as *const TrapFrame as usize) };

    // Now that we have a valid, working trap frame, we can run process::allocate_pid
    trap_frame.pid = process::allocate_pid();

    HART_META.write().insert(
        load_hartid(),
        Arc::new(HartMeta {
            plic: Plic0::new_with_fdt(),
            boot_stack: None,
            boot_frame: RwLock::new(trap_frame),
            is_panicking: AtomicBool::new(false),
            idle_process: AtomicUsize::new(0),
        }),
    );
}

pub fn get_this_hart_meta() -> Option<Arc<HartMeta>> {
    get_hart_meta(load_hartid())
}

/// # Safety
/// start_addr must be a function that is sound and sets up harts correctly
pub unsafe fn start_all_harts(start_addr: usize) {
    for hartid in 0.. {
        match sbi::hart_get_status(hartid) {
            Err(_e) => {
                info!("Max hart: {}", hartid);
                // This hart is invalid
                break;
            }
            Ok(status) => {
                if status == 1 {
                    // This hart is stopped
                    // Create a stack for it and pass it in a1
                    let process_stack = kernel_util::boxed_slice_with_alignment(4096 * 8, 4096, &0);
                    sbi::start_hart(
                        hartid,
                        start_addr,
                        process_stack.as_ptr() as usize + (4096 * 8) - 0x10,
                    )
                    .expect("Starting hart failed!");
                    Box::leak(process_stack);
                } else {
                    info!("hart status: {}", status)
                }
            }
        }
    }
}

#[no_mangle]
fn hart_entry(hartid: usize, interrupt_stack: usize) -> ! {
    add_this_secondary_hart(hartid, interrupt_stack);

    timer_queue::init_hart();

    // SAFETY: s_trap_vector is a valid trap vector so no problems
    unsafe { cpu::write_stvec(s_trap_vector as usize) };

    // Finally, enable interrupts in the cpu level
    // SAFETY: We're enabling interrupts, since we've set stvec already that's not dangerous
    unsafe {
        use cpu::csr::*;
        // Enable software, external, and timer interrupts
        cpu::write_sie(SSIE | STIE | SEIE);

        let mut sstatus: usize;
        asm!("csrr {0}, sstatus", out(reg)(sstatus),);
        sstatus |= 1 << 1;
        asm!("csrw sstatus, {0}" , in(reg) ( sstatus));
    }

    schedule_next_slice(0);
    timer_queue::schedule_next();

    loop {
        cpu::wfi()
    }
}
