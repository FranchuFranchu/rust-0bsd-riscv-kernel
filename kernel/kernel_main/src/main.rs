//! A RISC-V kernel written in Rust

#![feature(
    asm,
    naked_functions,
    const_trait_impl,
    const_fn_trait_bound,
    default_alloc_error_handler,
    const_mut_refs,
    panic_info_message,
    option_result_unwrap_unchecked,
    unchecked_math,
    const_btree_new,
    unsized_fn_params,
    exclusive_range_pattern,
    mixed_integer_ops,
    box_into_inner,
    unsized_locals,
    async_stream,
    label_break_value,
    int_roundings,
    type_ascription,
    map_first_last,
    global_asm,
    never_type
)]
#![cfg_attr(not(test), no_std)]
#![no_main]
#![allow(incomplete_features, clippy::empty_loop)]
// On many parts of the kernel, I would like to specify the justification for each unsafe fn call, even if it is inside an unsafe fn already
#![allow(unused_unsafe)]

extern crate alloc;

use core::{
    ffi::c_void,
    panic::PanicInfo,
    sync::atomic::{AtomicUsize, Ordering},
};

use process::PROCESSES;

use crate::{
    benchmark::time_fn,
    cpu::{load_hartid, read_sscratch},
    hart::get_hart_meta,
    paging::Paging,
    plic::Plic0,
    process::{delete_process, PidSlot},
    sbi::shutdown,
    timeout::get_time_setup,
    virtual_buffers::new_virtual_buffer,
};

#[macro_use]
extern crate log;
#[macro_use]
extern crate async_trait;
#[macro_use]
extern crate zerocopy;

// Linker symbols
extern "C" {
    static _heap_start: c_void;
    static _heap_end: c_void;

    static _stack_start: c_void;
    static _stack_end: c_void;

    fn s_trap_vector();
    fn new_hart();
}

// The boot frame is the frame that is active in the boot thread
// It needs to be statically allocated because it has to be there before
// memory allocation is up and running
static mut BOOT_FRAME: trap::TrapFrame = trap::TrapFrame::zeroed_interrupt_context();

/// Macro imports

#[macro_use]
pub mod std_macros;

#[no_mangle]
pub fn main(hartid: usize, opaque: usize) -> ! {
    if unsafe { BOOT_FRAME.pid } != 0 {
        panic!("main() called more than once!");
    }

    cpu::BOOT_HART.store(hartid, Ordering::Relaxed);

    unsafe { crate::drivers::uart::Uart::new(0x1000_0000).setup() };

    // SAFETY: We're the only hart, there's no way the data gets changed by someone else meanwhile
    unsafe { BOOT_FRAME.hartid = hartid }
    unsafe { BOOT_FRAME.pid = 1 }
    unsafe { BOOT_FRAME.interrupt_stack = &_stack_start as *const _ as usize - 0x10 }

    // SAFETY: BOOT_FRAME has a valid trap frame value so this doesn't break the rest of the kernel
    unsafe { kernel_cpu::write_sscratch(&BOOT_FRAME as *const trap::TrapFrame as usize) }

    // Now, set up the logger
    log::set_logger(&logger::KERNEL_LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Trace);

    info!("Kernel reached, logging set up");

    // SAFETY: identity_map is valid when the root page is valid, which in this case is true
    // and paging is disabled now
    #[cfg(target_arch = "riscv64")]
    unsafe {
        paging::sv39::identity_map(&mut paging::ROOT_PAGE as *mut paging::Table)
    }
    use crate::paging::sv39::RootTable;

    // Initialize allocation
    allocator::init();

    // SAFETY: s_trap_vector is a valid trap vector so no problems here
    unsafe { cpu::write_stvec(s_trap_vector as usize) };

    // Initialize the interrupt waker system
    interrupt_context_waker::init();

    // Setup paging
    // SAFETY: If identity mapping did its thing right, then nothing should change
    #[cfg(target_arch = "riscv64")]
    unsafe {
        cpu::write_satp(
            (&mut paging::ROOT_PAGE as *mut paging::Table as usize) >> 12 | cpu::csr::SATP_SV39,
        );
        BOOT_FRAME.satp = cpu::read_satp();
        BOOT_FRAME.kernel_satp = cpu::read_satp();
    }

    cpu::fence_vma();

    get_time_setup();

    //let virt_buffer = new_virtual_buffer(0x1000_0000, 0x4096);

    //kernel_debugging::backtrace::backtrace();
    //kernel_debugging::pub_names::test();

    // Initialize the device tree assuming that opaque contains a pointer to the DT
    // (standard behaviour in QEMU)
    fdt::init(opaque as _);

    // Now that allocation and FDT is set up we can move the boot frame to a "proper" place
    let copied_frame = unsafe { BOOT_FRAME.clone() };
    unsafe { hart::add_boot_hart(copied_frame) };
    unsafe {
        kernel_cpu::write_sscratch(&**hart::get_this_hart_meta().unwrap().boot_frame.read()
            as *const trap::TrapFrame as usize)
    };

    // Set up the external interrupts
    let plic = Plic0::new_with_fdt();
    plic.set_threshold(0);
    plic.set_enabled(10, true);
    plic.set_priority(10, 3);
    plic.set_enabled(8, true);
    plic.set_priority(8, 3);

    //crate::fdt::root().read().pretty(0);

    timer_queue::init();
    timer_queue::init_hart();

    handle_backends::initialize_constructors();

    // Finally, enable interrupts in the cpu level
    // SAFETY: We're enabling interrupts, since we've set stvec already that's not dangerous
    unsafe {
        use cpu::csr::*;
        // Enable software, external, and timer interrupts
        cpu::write_sie(SSIE | SEIE | STIE);

        let mut sstatus: usize;
        asm!("csrr {0}, sstatus", out(reg)(sstatus),);
        sstatus |= 1 << 1 | 1 << 18;
        asm!("csrw sstatus, {0}" , in(reg) ( sstatus));
    }

    use alloc::borrow::ToOwned;
    process::new_supervisor_process_with_name(test_task::test_task_3, "disk-test".to_owned());
    process::new_supervisor_process_with_name(
        device_setup::setup_devices,
        "setup-devices".to_owned(),
    );

    /*process::new_supervisor_process_with_name(
        test_task_2,
        "test-task-2".to_owned(),
    );*/
    unsafe { hart::start_all_harts(new_hart as usize) };
    scheduler::schedule_next_slice(0);
    timer_queue::schedule_next();
    loop {
        cpu::wfi();
    }
}

pub static HART_PANIC_COUNT: AtomicUsize = AtomicUsize::new(0);

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Disable ALL interrupts
    unsafe {
        cpu::write_sie(0);
    }
    HART_PANIC_COUNT.fetch_add(1, Ordering::SeqCst);
    unsafe { std_macros::OUTPUT_LOCK.force_unlock() };
    if let Some(e) = unsafe { read_sscratch().as_ref() } {
        if e.is_double_faulting() {
            use core::fmt::Write;
            write!(
                unsafe { crate::drivers::uart::Uart::new(0x1000_0000) },
                "\n\x1b[1;31mDouble fault, hart {}\x1b[0m\n",
                load_hartid()
            );
            loop {}
        }
    }

    /*let lock = match PROCESSES.try_write() {
        Some(e) => e,
        None => unsafe {
            PROCESSES.data_ptr().as_ref()
            PROCESSES.force_unlock_write();
            PROCESSES.write()
        },
    };*/

    if false
    //lock.contains_key(&crate::process::Process::this_pid())
    {
        /*drop(lock);*/
        delete_process(crate::process::Process::this_pid());
    }

    if let Some(meta) = get_hart_meta(load_hartid()) {
        if meta.is_panicking.load(Ordering::Relaxed) {
            println!("\x1b[31mDouble Panic\x1b[0m");
            loop {}
        } else {
            meta.is_panicking.store(true, Ordering::Relaxed)
        }
    }

    //let mut host_stderr = HStderr::new();

    // logs "panicked at '$reason', src/main.rs:27:4" to the host stderr
    //writeln!(host_stderr, "{}", info).ok();

    let fnomsg = format_args!("<no message>");
    let message = info.message().unwrap_or(&fnomsg);

    let trap_frame = cpu::read_sscratch();

    // Check if trap frame is out of bounds (which means we can't read data from it)
    if (trap_frame as usize) > 0x80200000
        && (trap_frame as usize) < (unsafe { &_heap_end } as *const c_void as usize)
    {
        // Assume that the trap frame is valid
        // (we already checked for trap_frame being null, so we can safely use unwrap_unchecked)
        let trap_frame = unsafe { trap_frame.as_ref().unwrap_unchecked() };

        print!("Hart \x1b[94m#{}\x1b[0m \x1b[31mpanicked\x1b[0m while running process \x1b[94m#{}\x1b[0m: ", (*trap_frame).hartid, (*trap_frame).pid);
    } else {
        print!("\x1b[31mPanic\x1b[0m with unknown context: ")
    }

    if let Some(location) = info.location() {
        println!("\"{}\" at \x1b[94m{}\x1b[0m", message, location);
    } else {
        println!("\"{}\" at unknown location", message);
    }

    // Shutdown immediately
    sbi::shutdown(0);

    loop {
        // Now (if we haven't shut down for some reason), poll the UART until we get a Ctrl+C
        // and then shutdown
        match unsafe { crate::drivers::uart::Uart::new(0x1000_0000).get() } {
            Some(3) => crate::sbi::shutdown(0),
            _ => {}
        }
    }
}

#[no_mangle]
pub fn status_summary() {
    println!("{:?}", "Processes: ");
    PROCESSES
        .read()
        .iter()
        .filter(|(_k, v)| PidSlot::is_used(v))
        .for_each(|(_k, v)| {
            let v = v.unwrap_ref().unwrap().read();
            println!(
                "{}:",
                v.name.as_ref().map(|s| s.as_ref()).unwrap_or("<unnamed>")
            );
            println!("	{:?}", v.state);
        });
}

pub mod allocator;
pub mod as_register;
pub mod asm;
pub mod context_switch;
pub use kernel_cpu as cpu;
pub mod device_setup;
pub mod drivers;
pub mod external_interrupt;
pub mod fdt;
pub mod filesystem;
pub mod future;
pub mod handle;
pub mod handle_backends;
pub mod hart;
pub mod interrupt_context_waker;
pub use kernel_io as io;
pub mod kernel_debugging;
pub use kernel_lock as lock;
pub mod benchmark;
pub mod logger;
pub mod paging;
pub mod plic;
pub mod process;
pub mod sbi;
pub mod scheduler;
pub mod syscall;
pub mod test_task;
pub mod timeout;
pub mod timer_queue;
pub mod trap;
pub mod trap_frame;
pub mod trap_future_executor;
pub mod unsafe_buffer;
pub mod virtual_buffers;
