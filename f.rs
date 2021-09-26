#![feature(prelude_import)]
#![feature(llvm_asm, asm, naked_functions, const_trait_impl,
           const_fn_trait_bound, default_alloc_error_handler, const_mut_refs,
           panic_info_message, maybe_uninit_ref,
           option_result_unwrap_unchecked, unchecked_math, const_btree_new,
           unsized_fn_params, box_into_inner, unsized_locals, async_stream,
           label_break_value, type_ascription, global_asm)]
#![no_std]
#![no_main]
#![allow(incomplete_features)]
#![allow(dead_code)]
#![allow(unused_variables)]
#[prelude_import]
use core::prelude::rust_2018::*;
#[macro_use]
extern crate core;
#[macro_use]
extern crate compiler_builtins;

extern crate alloc;

use core::{panic::PanicInfo, ffi::c_void};
use core::sync::atomic::AtomicUsize;
use process::PROCESSES;

use crate::paging::{MEGAPAGE_SIZE, Paging};
use crate::process::{ProcessState, delete_process};
use crate::{cpu::{load_hartid, read_sscratch}, hart::get_hart_meta,
            plic::Plic0};
use core::sync::atomic::Ordering;

#[macro_use]
extern crate log;
#[macro_use]
extern crate async_trait;
#[macro_use]
extern crate zerocopy;


// Linker symbols
extern "C" {
    static _heap_start: c_void ;
    static _heap_end: c_void ;

    static _stack_start: c_void ;
    static _stack_end: c_void ;

    fn s_trap_vector();
    fn new_hart();
}


// The boot frame is the frame that is active in the boot thread
// It needs to be statically allocated because it has to be there before
// memory allocation is up and running
static mut BOOT_FRAME: trap::TrapFrame =
    trap::TrapFrame::zeroed_interrupt_context();


/// Macro imports
#[macro_use]
pub mod cpu {








    // SAFETY: We're the only hart, there's no way the data gets changed by someone else meanwhile

    // SAFETY: BOOT_FRAME has a valid trap frame value so this doesn't break the rest of the kernel


    // Now, set up the logger


    // SAFETY: identity_map is valid when the root page is valid, which in this case is true
    // and paging is disabled now


    // Initialize allocation



    // SAFETY: s_trap_vector is a valid trap vector so no problems here

    // Initialize the interrupt waker system

    // Setup paging
    // SAFETY: If identity mapping did its thing right, then nothing should change




    // Initialize the device tree assuming that opaque contains a pointer to the DT
    // (standard behaviour in QEMU)

    // Now that allocation and FDT is set up we can move the boot frame to a "proper" place


    // Set up the external interrupts

    //crate::fdt::root().read().pretty(0);




    // Finally, enable interrupts in the cpu level
    // SAFETY: We're enabling interrupts, since we've set stvec already that's not dangerous
    // Enable software, external, and timer interrupts














    // Disable ALL interrupts




    //let mut host_stderr = HStderr::new();

    // logs "panicked at '$reason', src/main.rs:27:4" to the host stderr
    //writeln!(host_stderr, "{}", info).ok();




    // Check if trap frame is out of bounds (which means we can't read data from it)
    // Assume that the trap frame is valid
    // (we already checked for trap_frame being null, so we can safely use unwrap_unchecked) 



    // Shutdown immediately

    // Now (if we haven't shut down for some reason), poll the UART until we get a Ctrl+C
    // and then shutdown




    #[repr(usize)]
    pub enum Registers {
        Zero = 0,
        Ra,
        Sp,
        Gp,
        Tp,
        T0,
        T1,
        T2,
        S0,
        S1,
        A0,
        A1,
        A2,
        A3,
        A4,
        A5,
        A6,
        A7,
        S2,
        S3,
        S4,
        S5,
        S6,
        S7,
        S8,
        S9,
        S10,
        S11,
        T3,
        T4,
        T5,
        T6,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for Registers { }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for Registers {
        #[inline]
        fn clone(&self) -> Registers { { *self } }
    }
    impl Registers {
        pub const fn idx(&self) -> usize { *self as usize }
    }
    /// # Safety
    /// This can cause hangups and other things that aren't very good
    #[inline(always)]
    pub unsafe fn write_sie(value: usize) {
        llvm_asm!("csrw sie, $0":  : "r"(value) :  : "volatile")
    }
    /// # Safety
    /// When setting interrupts, the proper context needs to be created for the trap handler
    #[inline(always)]
    pub unsafe fn write_sip(value: usize) {
        llvm_asm!("csrw sip, $0":  : "r"(value) :  : "volatile")
    }
    /// # Safety
    /// Must be s_trap
    #[inline(always)]
    pub unsafe fn write_stvec(value: usize) {
        llvm_asm!("csrw stvec, $0":  : "r"(value) :  : "volatile")
    }
    /// # Safety
    /// Must uphold SATP assumptions in the rest of the kernel. Mainly, that it's a valid page table
    #[inline(always)]
    pub unsafe fn write_satp(value: usize) {
        llvm_asm!("\n\t\tcsrw satp, $0\n\t\tsfence.vma\n\t\t":  : "r"(value) :
             : "volatile")
    }
    /// # Safety
    /// Too many constraints to document. Shouldn't be changed very frecuently.
    #[inline(always)]
    pub unsafe fn write_sstatus(value: usize) {
        llvm_asm!("csrw sstatus, $0":  : "r"(value) :  : "volatile")
    }
    /// This is unsafe because other parts of the kernel rely on sscratch being a valid pointer
    /// # Safety
    /// Must be a valid trap frame and must make sense with what the hart is executing
    #[inline(always)]
    pub unsafe fn write_sscratch(value: usize) {
        llvm_asm!("csrw sscratch, $0":  : "r"(value) :  : "volatile")
    }
    #[inline]
    pub fn read_sscratch() -> *mut crate::trap::TrapFrame {
        let value: usize;
        unsafe {
            llvm_asm!("csrr $0, sscratch": "=r"(value) :  :  : "volatile")
        };
        value as _
    }
    #[inline(always)]
    pub fn read_sp() -> usize {
        let value: usize;
        unsafe { llvm_asm!("mv $0, sp": "=r"(value) :  :  : "volatile") };
        value
    }
    #[inline(always)]
    pub fn read_sip() -> usize {
        let value: usize;
        unsafe { llvm_asm!("csrr $0, sip": "=r"(value) :  :  : "volatile") };
        value
    }
    #[inline(always)]
    pub fn read_satp() -> usize {
        let value: usize;
        unsafe { llvm_asm!("csrr $0, satp": "=r"(value) :  :  : "volatile") };
        value
    }
    #[inline(always)]
    pub fn read_sie() -> usize {
        let value: usize;
        unsafe { llvm_asm!("csrr $0, sie": "=r"(value) :  :  : "volatile") };
        value
    }
    #[inline(always)]
    pub fn read_sstatus() -> usize {
        let value: usize;
        unsafe {
            llvm_asm!("csrr $0, sstatus": "=r"(value) :  :  : "volatile")
        };
        value
    }
    #[inline(always)]
    pub fn read_time() -> usize {
        let value: usize;
        unsafe { llvm_asm!("csrr $0, time": "=r"(value) :  :  : "volatile") };
        value
    }
    #[inline(always)]
    pub fn read_cycle() -> usize {
        let value: usize;
        unsafe {
            llvm_asm!("csrr $0, cycle": "=r"(value) :  :  : "volatile")
        };
        value
    }
    /// Gets hartid from sscratch
    /// This assumes that sscratch holds a valid value
    pub fn load_hartid() -> usize { unsafe { (*read_sscratch()).hartid } }
    use core::sync::atomic::AtomicUsize;
    pub static BOOT_HART: AtomicUsize = AtomicUsize::new(0);
    #[inline(always)]
    pub fn wfi() { unsafe { llvm_asm!("wfi":  :  :  : "volatile"); } }
    pub fn fence_vma() {
        unsafe { llvm_asm!("sfence.vma zero, zero":  :  :  : "volatile") };
    }
    /// This is provided by the CLINT
    const MMIO_MTIME: *const u64 = 0x0200_BFF8 as *const u64;
    pub fn get_time() -> u64 { unsafe { *MMIO_MTIME } }
    pub mod csr {
        pub const USIP: usize = 1 << 0;
        pub const SSIP: usize = 1 << 1;
        pub const MSIP: usize = 1 << 3;
        pub const UTIP: usize = 1 << 4;
        pub const STIP: usize = 1 << 5;
        pub const MTIP: usize = 1 << 7;
        pub const UEIP: usize = 1 << 8;
        pub const SEIP: usize = 1 << 9;
        pub const MEIP: usize = 1 << 11;
        pub const USIE: usize = 1 << 0;
        pub const SSIE: usize = 1 << 1;
        pub const MSIE: usize = 1 << 3;
        pub const UTIE: usize = 1 << 4;
        pub const STIE: usize = 1 << 5;
        pub const MTIE: usize = 1 << 7;
        pub const UEIE: usize = 1 << 8;
        pub const SEIE: usize = 1 << 9;
        pub const MEIE: usize = 1 << 11;
        pub const SATP_BARE: usize = 0;
        pub const SATP_SV32: usize = 1 << 30;
        #[cfg(target_arch = "riscv64")]
        pub const SATP_SV39: usize = 8 << 60;
        #[cfg(target_arch = "riscv64")]
        pub const SATP_SV48: usize = 9 << 60;
    }
}
#[macro_use]
pub mod std_macros {
    use crate::lock::shared::Mutex;
    pub static OUTPUT_LOCK: Mutex<()> = Mutex::new(());
    #[macro_export]
    macro_rules! print {
        ($ ($ args : tt) +) =>
        (#[allow(unused_unsafe)]
         {
             use core :: fmt :: Write ; let l = crate :: std_macros ::
             OUTPUT_LOCK.lock() ; let _ = write!
             (unsafe { crate :: drivers :: uart :: Uart :: new(0x1000_0000) },
              $ ($ args) +) ;
         }) ;
    }
    #[macro_export]
    macro_rules! println {
        () => ({ print! ("\r\n") }) ; ($ fmt : expr) =>
        ({ print! (concat! ($ fmt, "\r\n")) }) ;
        ($ fmt : expr, $ ($ args : tt) +) =>
        ({ print! (concat! ($ fmt, "\r\n"), $ ($ args) +) }) ;
    }
}
#[no_mangle]
pub fn main(hartid: usize, opaque: usize) -> ! {
    if unsafe { BOOT_FRAME.pid } != 0 {
        ::core::panicking::panic("main() called more than once!");
    }
    cpu::BOOT_HART.store(hartid, Ordering::Relaxed);
    unsafe { crate::drivers::uart::Uart::new(0x1000_0000).setup() };
    unsafe { BOOT_FRAME.hartid = hartid }
    unsafe { BOOT_FRAME.pid = 1 }
    unsafe { BOOT_FRAME.interrupt_stack = &_stack_start as *const _ as usize }
    unsafe {
        crate::cpu::write_sscratch(&BOOT_FRAME as *const trap::TrapFrame as
                                       usize)
    }
    log::set_logger(&logger::KERNEL_LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Trace);
    {
        let lvl = ::log::Level::Info;
        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
            ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["Kernel reached, logging set up"],
                                                                    &match ()
                                                                         {
                                                                         () =>
                                                                         [],
                                                                     }), lvl,
                                     &("rust_0bsd_riscv_kernel",
                                       "rust_0bsd_riscv_kernel",
                                       "src/main.rs", 97u32));
        }
    };

    #[cfg(target_arch = "riscv64")]
    unsafe {
        paging::sv39::identity_map(&mut paging::ROOT_PAGE as
                                       *mut paging::Table)
    }
    use crate::paging::sv39::RootTable;
    allocator::init();
    unsafe { cpu::write_stvec(s_trap_vector as usize) };
    interrupt_context_waker::init();

    #[cfg(target_arch = "riscv64")]
    unsafe {
        cpu::write_satp((&mut paging::ROOT_PAGE as *mut paging::Table as
                             usize) >> 12 | cpu::csr::SATP_SV39)
    }
    cpu::fence_vma();
    fdt::init(opaque as _);
    let copied_frame = unsafe { BOOT_FRAME.clone() };
    unsafe { hart::add_boot_hart(copied_frame) };
    unsafe {
        crate::cpu::write_sscratch(&**hart::get_this_hart_meta().unwrap().boot_frame.read()
                                       as *const trap::TrapFrame as usize)
    };
    let plic = Plic0::new_with_fdt();
    plic.set_threshold(0);
    plic.set_enabled(10, true);
    plic.set_priority(10, 3);
    plic.set_enabled(8, true);
    plic.set_priority(8, 3);
    timer_queue::init();
    timer_queue::init_hart();
    unsafe {
        use cpu::csr::*;
        cpu::write_sie(SSIE | SEIE | STIE);
        let mut sstatus: usize;
        llvm_asm!("csrr $0, sstatus": "=r"(sstatus) :  : );
        sstatus |= 1 << 1;
        llvm_asm!("csrw sstatus, $0":  : "r"(sstatus) :  : "volatile");
    }
    let mut tab = RootTable(unsafe { &mut paging::ROOT_PAGE });
    tab.map(0x20000000, 0x20001000, 0x200000, 15);
    loop  { };
    use alloc::borrow::ToOwned;
    process::new_supervisor_process_with_name(test_task::test_task_3,
                                              "disk-test".to_owned());
    process::new_supervisor_process_with_name(device_setup::setup_devices,
                                              "setup-devices".to_owned());
    unsafe { hart::start_all_harts(new_hart as usize) };
    scheduler::schedule_next_slice(0);
    timer_queue::schedule_next();
    loop  { cpu::wfi(); }
}
pub static HART_PANIC_COUNT: AtomicUsize = AtomicUsize::new(0);
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe { cpu::write_sie(0); }
    HART_PANIC_COUNT.fetch_add(1, Ordering::SeqCst);
    if let Some(e) = unsafe { read_sscratch().as_ref() } {
        if e.is_double_faulting() {
            use core::fmt::Write;
            unsafe {
                crate::drivers::uart::Uart::new(0x1000_0000)
            }.write_fmt(::core::fmt::Arguments::new_v1(&["\n\u{1b}[1;31mDouble fault, hart ",
                                                         "\u{1b}[0m\n"],
                                                       &match (&load_hartid(),)
                                                            {
                                                            (arg0,) =>
                                                            [::core::fmt::ArgumentV1::new(arg0,
                                                                                          ::core::fmt::Display::fmt)],
                                                        }));
            loop  { };
        }
    }
    {

        #[allow(unused_unsafe)]
        {
            use core::fmt::Write;
            let l = crate::std_macros::OUTPUT_LOCK.lock();
            let _ =
                unsafe {
                    crate::drivers::uart::Uart::new(0x1000_0000)
                }.write_fmt(::core::fmt::Arguments::new_v1(&["", "\r\n"],
                                                           &match (&info.message(),)
                                                                {
                                                                (arg0,) =>
                                                                [::core::fmt::ArgumentV1::new(arg0,
                                                                                              ::core::fmt::Debug::fmt)],
                                                            }));
        }
    };
    if PROCESSES.read().contains_key(&crate::process::Process::this_pid()) {
        delete_process(crate::process::Process::this_pid());
    }
    if let Some(meta) = get_hart_meta(load_hartid()) {
        if meta.is_panicking.load(Ordering::Relaxed) {
            {

                #[allow(unused_unsafe)]
                {
                    use core::fmt::Write;
                    let l = crate::std_macros::OUTPUT_LOCK.lock();
                    let _ =
                        unsafe {
                            crate::drivers::uart::Uart::new(0x1000_0000)
                        }.write_fmt(::core::fmt::Arguments::new_v1(&["\u{1b}[31mDouble Panic\u{1b}[0m\r\n"],
                                                                   &match () {
                                                                        () =>
                                                                        [],
                                                                    }));
                }
            };
            loop  { }
        } else { meta.is_panicking.store(true, Ordering::Relaxed) }
    }
    let fnomsg =
        ::core::fmt::Arguments::new_v1(&["<no message>"],
                                       &match () { () => [], });
    let message = info.message().unwrap_or(&fnomsg);
    let trap_frame = cpu::read_sscratch();
    {
        let lvl = ::log::Level::Debug;
        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
            ::log::__private_api_log(::core::fmt::Arguments::new_v1(&[""],
                                                                    &match (&trap_frame,)
                                                                         {
                                                                         (arg0,)
                                                                         =>
                                                                         [::core::fmt::ArgumentV1::new(arg0,
                                                                                                       ::core::fmt::Debug::fmt)],
                                                                     }), lvl,
                                     &("rust_0bsd_riscv_kernel",
                                       "rust_0bsd_riscv_kernel",
                                       "src/main.rs", 230u32));
        }
    };
    if (trap_frame as usize) > 0x80200000 &&
           (trap_frame as usize) <
               (unsafe { &_heap_end } as *const c_void as usize) {
        let trap_frame = unsafe { trap_frame.as_ref().unwrap_unchecked() };

        #[allow(unused_unsafe)]
        {
            use core::fmt::Write;
            let l = crate::std_macros::OUTPUT_LOCK.lock();
            let _ =
                unsafe {
                    crate::drivers::uart::Uart::new(0x1000_0000)
                }.write_fmt(::core::fmt::Arguments::new_v1(&["Hart \u{1b}[94m#",
                                                             "\u{1b}[0m \u{1b}[31mpanicked\u{1b}[0m while running process \u{1b}[94m#",
                                                             "\u{1b}[0m: "],
                                                           &match (&(*trap_frame).hartid,
                                                                   &(*trap_frame).pid)
                                                                {
                                                                (arg0, arg1)
                                                                =>
                                                                [::core::fmt::ArgumentV1::new(arg0,
                                                                                              ::core::fmt::Display::fmt),
                                                                 ::core::fmt::ArgumentV1::new(arg1,
                                                                                              ::core::fmt::Display::fmt)],
                                                            }));
        };
    } else {

        #[allow(unused_unsafe)]
        {
            use core::fmt::Write;
            let l = crate::std_macros::OUTPUT_LOCK.lock();
            let _ =
                unsafe {
                    crate::drivers::uart::Uart::new(0x1000_0000)
                }.write_fmt(::core::fmt::Arguments::new_v1(&["\u{1b}[31mPanic\u{1b}[0m with unknown context: "],
                                                           &match () {
                                                                () => [],
                                                            }));
        }
    }
    if let Some(location) = info.location() {
        {

            #[allow(unused_unsafe)]
            {
                use core::fmt::Write;
                let l = crate::std_macros::OUTPUT_LOCK.lock();
                let _ =
                    unsafe {
                        crate::drivers::uart::Uart::new(0x1000_0000)
                    }.write_fmt(::core::fmt::Arguments::new_v1(&["\"",
                                                                 "\" at \u{1b}[94m",
                                                                 "\u{1b}[0m\r\n"],
                                                               &match (&message,
                                                                       &location)
                                                                    {
                                                                    (arg0,
                                                                     arg1) =>
                                                                    [::core::fmt::ArgumentV1::new(arg0,
                                                                                                  ::core::fmt::Display::fmt),
                                                                     ::core::fmt::ArgumentV1::new(arg1,
                                                                                                  ::core::fmt::Display::fmt)],
                                                                }));
            }
        };
    } else {
        {

            #[allow(unused_unsafe)]
            {
                use core::fmt::Write;
                let l = crate::std_macros::OUTPUT_LOCK.lock();
                let _ =
                    unsafe {
                        crate::drivers::uart::Uart::new(0x1000_0000)
                    }.write_fmt(::core::fmt::Arguments::new_v1(&["\"",
                                                                 "\" at unknown location\r\n"],
                                                               &match (&message,)
                                                                    {
                                                                    (arg0,) =>
                                                                    [::core::fmt::ArgumentV1::new(arg0,
                                                                                                  ::core::fmt::Display::fmt)],
                                                                }));
            }
        };
    }
    sbi::shutdown(0);
    loop  {
        match unsafe { crate::drivers::uart::Uart::new(0x1000_0000).get() } {
            Some(3) => crate::sbi::shutdown(0),
            _ => { }
        }
    }
}
#[no_mangle]
pub fn status_summary() {
    {

        #[allow(unused_unsafe)]
        {
            use core::fmt::Write;
            let l = crate::std_macros::OUTPUT_LOCK.lock();
            let _ =
                unsafe {
                    crate::drivers::uart::Uart::new(0x1000_0000)
                }.write_fmt(::core::fmt::Arguments::new_v1(&["", "\r\n"],
                                                           &match (&"Processes: ",)
                                                                {
                                                                (arg0,) =>
                                                                [::core::fmt::ArgumentV1::new(arg0,
                                                                                              ::core::fmt::Debug::fmt)],
                                                            }));
        }
    };
    PROCESSES.read().iter().for_each(|(k, v)|
                                         {
                                             use alloc::borrow::ToOwned;
                                             let v = v.read();
                                             {

                                                 #[allow(unused_unsafe)]
                                                 {
                                                     use core::fmt::Write;
                                                     let l =
                                                         crate::std_macros::OUTPUT_LOCK.lock();
                                                     let _ =
                                                         unsafe {
                                                             crate::drivers::uart::Uart::new(0x1000_0000)
                                                         }.write_fmt(::core::fmt::Arguments::new_v1(&["",
                                                                                                      ":\r\n"],
                                                                                                    &match (&v.name.as_ref().map(|s|
                                                                                                                                     s.as_ref()).unwrap_or("<unnamed>"),)
                                                                                                         {
                                                                                                         (arg0,)
                                                                                                         =>
                                                                                                         [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                       ::core::fmt::Display::fmt)],
                                                                                                     }));
                                                 }
                                             };
                                             {

                                                 #[allow(unused_unsafe)]
                                                 {
                                                     use core::fmt::Write;
                                                     let l =
                                                         crate::std_macros::OUTPUT_LOCK.lock();
                                                     let _ =
                                                         unsafe {
                                                             crate::drivers::uart::Uart::new(0x1000_0000)
                                                         }.write_fmt(::core::fmt::Arguments::new_v1(&["\t",
                                                                                                      "\r\n"],
                                                                                                    &match (&v.state,)
                                                                                                         {
                                                                                                         (arg0,)
                                                                                                         =>
                                                                                                         [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                       ::core::fmt::Debug::fmt)],
                                                                                                     }));
                                                 }
                                             };
                                         });
}
pub mod asm {
    global_asm! (".set XLEN, 8   # Register size (in bytes)\n.global XLEN\n\n# To make this file compatible with both rv32 and rv64\n# Store XLEN bytes\n.macro lx a, b\nld \\a, \\b\n.endm\n.macro sx a, b\nsd \\a, \\b\n.endm\n\n.macro flx a, b\nfld \\a, \\b\n.endm\n.macro fsx a, b\nfsd \\a, \\b\n.endm\n")
        global_asm! ("# This is reached when the M-mode software (bootloader) has loaded us here\n.section .text.init\n.global boot\nboot:\n\tla sp, _stack_start\n\tj main\n\t\n.global ret_\nret_:\n\tret\n\t\n.section .text")
            global_asm! ("# trap.S\n# Trap handler and global context\n# Steve Operating System\n# Stephen Marz\n# 24 February 2019\n.option norvc\n.altmacro\n.set NUM_GP_REGS, 32  # Number of registers per context\n\n\n# Use macros for saving and restoring multiple registers\n.macro save_gp i, basereg=t6\n\tsx\tx\\i, ((\\i)*XLEN)(\\basereg)\n.endm\n.macro load_gp i, basereg=t6\n\tlx\tx\\i, ((\\i)*XLEN)(\\basereg)\n.endm\n.macro save_fp i, basereg=t6\n\tfsx\tf\\i, ((NUM_GP_REGS+(\\i))*XLEN)(\\basereg)\n.endm\n.macro load_fp i, basereg=t6\n\tflx\tf\\i, ((NUM_GP_REGS+(\\i))*XLEN)(\\basereg)\n.endm\n\n.macro store_volatile_registers\n\n\n\t# All registers are volatile here, we need to save them\n\t# before we do anything.\n\tcsrrw\tt6, sscratch, t6\n\t# csrrw will atomically swap t6 into sscratch and the olx\n\t# value of sscratch into t6. This is nice because we just\n\t# switched values and didn\'t destroy anything -- all atomically!\n\t# in cpu.rs we have a structure of:\n\t#  32 gp regs\t\t0\n\t#  32 fp regs\t\t256\n\t# We use t6 as the temporary register because it is the very\n\t# bottom register (x31)\n\t.set \ti, 0\n\t.rept\t31\n\t\tsave_gp\t%i\n\t\t.set\ti, i+1\n\t.endr\n\n\t# Save the actual t6 register, which we swapped into\n\t# sscratch\n\tmv\t\tt5, t6\n\tcsrr\tt6, sscratch\n\tsave_gp 31, t5\n\n\t# Restore the kernel trap frame into sscratch\n\tcsrw\tsscratch, t5\n\n\t# TODO add fp registers to trap frame\n\tj 1f\n\n\tcsrr\tt1, sstatus\n\tsrli\tt0, t1, 13\n\tandi\tt0, t0, 3\n\tli\t\tt3, 3\n\tbne\t\tt0, t3, 1f\n\t# Save floating point registers\n\t.set \ti, 0\n\t.rept\t32\n\t\tsave_fp\t%i, t5\n\t\t.set\ti, i+1\n\t.endr\n\t\n\t1:\n.endm\n\n# IN: t6: Trap frame pointer\n.macro load_from_trap_frame\n\tj 1f\n\tcsrr\tt1, sstatus\n\tsrli\tt0, t1, 13\n\tandi\tt0, t0, 3\n\tli\t\tt3, 3\n\tbne\t\tt0, t3, 1f\n\t.set\ti, 0\n\t.rept\t32\n\t\tload_fp %i\n\t\t.set i, i+1\n\t.endr\n1: # no_f_extension:\n\t# Restore all GP registers\n\t.set\ti, 1\n\t.rept\t31\n\t\tload_gp %i\n\t\t.set\ti, i+1\n\t.endr\n.endm\n\n.macro restore_volatile_registers\n\t# Now load the trap frame back into t6\n\tcsrr\tt6, sscratch\n\n\tload_from_trap_frame\n\t# Since we ran this loop 31 times starting with i = 1,\n\t# the last one loaded t6 back to its original value.\n.endm\n\n.section .text\n.global s_trap_vector\n# This must be aligned by 4 since the last two bits\n# of the mtvec register do not contribute to the address\n# of this vector.\n.align 4\ns_trap_vector:\n\tstore_volatile_registers\n\t# Now t5 and sscratch have the trap frame pointer\n\t\n\t# Get ready to go into Rust (trap.rs)\n\t# csrw\tsie, zero\n\t\n\tlx \t\tsp, XLEN*35(t5)\n\tcsrr\ta0, sepc\n\tsx\t\ta0, XLEN*32(t5)\n\tcsrr\ta1, stval\n\tcsrr\ta2, scause\n\tlx\t\ta0, XLEN*32(t5)\n\tlx\t\ta3, XLEN*33(t5)\n\tcsrr\ta4, sstatus\n\tcsrr\ta5, sscratch\n\t\n\t# Usually, We don\'t want to write into the user\'s stack or whomever\n\t# messed with us here.\n\t# We haven\'t gotten userspace to work yet, so we can assume that this interrupt was triggered in kernel mode\n\t\n\t# la\t\tt0, KERNEL_STACK_END\n\t# lx\t\tsp, 0(t0)\n\tcall\ttrap_handler\n\t\n\t# When we get here, we\'ve returned from m_trap, restore registers\n\t# and return.\n\t# m_trap will return the return address via a0.\n\n\tcsrw\tsepc, a0\n\t\n\trestore_volatile_registers\n\ntrap_exit:\n\tsret\n\n.global switch_to_supervisor_frame\t\nswitch_to_supervisor_frame:\n\t// Load the trap frame\n\tcsrw sscratch, a0\n\t// When SRET is executed, set PC to the old PC\n\tlx t0, XLEN*32(a0)\n\tcsrw sepc, t0\n\t// If this process got interrupted, it means interrupts were enabled\n\tli t0, 0x222\n\tcsrw sie, t0\n\t\n\trestore_volatile_registers\n.global switch_to_supervisor_frame_end\nswitch_to_supervisor_frame_end:\n\tsret\n\n\n# Essentially like s_trap_vector, but smode-to-smode\ndo_syscall_internal:\n\t\n\tstore_volatile_registers\n\t# Now t5 and sscratch have the trap frame pointer\n\t\n\t# Get ready to go into Rust (trap.rs)\n\t# csrw\tsie, zero\n\t\n\tla \t\tsp, _stack_start\n\tmv\ta0, ra\n\tsx\t\ta0, XLEN*32(t5)\n\tli\t\ta1, 0\n\tcsrr\ta2, 9\n\tlx\t\ta0, XLEN*32(t5)\n\tlx\t\ta3, XLEN*33(t5)\n\tcsrr\ta4, sstatus\n\tcsrr\ta5, sscratch\n\t\n\t# Usually, We don\'t want to write into the user\'s stack or whomever\n\t# messed with us here.\n\t# We haven\'t gotten userspace to work yet, so we can assume that this interrupt was triggered in kernel mode\n\t\n\t# la\t\tt0, KERNEL_STACK_END\n\t# lx\t\tsp, 0(t0)\n\tcall\ttrap_handler\n\t\n\t# When we get here, we\'ve returned from m_trap, restore registers\n\t# and return.\n\t# m_trap will return the return address via a0.\n\n\tmv\tra, a0\n\t\n\trestore_volatile_registers\n\tret\n")
                global_asm! (".global this_hart_lock_count_2\nthis_hart_lock_count_2:\n\tj this_hart_lock_count\n\n.global do_supervisor_syscall\ndo_supervisor_syscall:\n\taddi sp, sp, -16\n\tsx ra, (sp)\n\t\n\tcsrr t0, sie\n\tbeqz t0, .error_syscall_interrupt_disabled\n\t\n\tmv t0, a7\n\tmv a7, a0\n\tmv a0, a1\n\tmv a1, a2\n\tmv a2, a3\n\tmv a3, a4\n\tmv a4, a5\n\tmv a6, t0\n\t\n\t# Set the supervisor software interrupt pending bit (SSIP)\n\tcsrr t0, sip\n\tori t0, t0, 1 << 1\n\tcsrw sip, t0\n\twfi\n\t\n\t\n\ndo_supervisor_syscall_end:\n\tlx ra, (sp)\n\taddi sp, sp, 16\n\tret\n\t\n.error_syscall_interrupt_disabled:\n\tj syscall_on_interrupt_disabled\n\t\n\tlx ra, (sp)\n\taddi sp, sp, 16\n\tret\n\t\n.global do_supervisor_syscall_end")
                    global_asm! ("# Entry point for harts other than the boot hart\n# a1 = opaque = kernel-allocated stack for this hart\n.global new_hart\nnew_hart:\n\tmv sp, a1\n\tj hart_entry")
                        #[allow(clashing_extern_declarations)]
                        extern "C" {
                            #[link_name = "do_supervisor_syscall"]
                            pub fn do_supervisor_syscall_0(number: usize);
                            #[link_name = "do_supervisor_syscall"]
                            pub fn do_supervisor_syscall_1(number: usize,
                                                           a0: usize);
                            #[link_name = "do_supervisor_syscall"]
                            pub fn do_supervisor_syscall_2(number: usize,
                                                           a0: usize,
                                                           a1: usize);
                            #[link_name = "do_supervisor_syscall"]
                            pub fn do_supervisor_syscall_3(number: usize,
                                                           a0: usize,
                                                           a1: usize);
                            #[link_name = "do_supervisor_syscall"]
                            pub fn do_supervisor_syscall_4(number: usize,
                                                           a0: usize,
                                                           a1: usize);
                            #[link_name = "do_supervisor_syscall"]
                            pub fn do_supervisor_syscall_5(number: usize,
                                                           a0: usize,
                                                           a1: usize);
                            #[link_name = "do_supervisor_syscall"]
                            pub fn do_supervisor_syscall_6(number: usize,
                                                           a0: usize,
                                                           a1: usize);
                            #[link_name = "do_supervisor_syscall"]
                            pub fn do_supervisor_syscall_7(number: usize,
                                                           a0: usize,
                                                           a1: usize);
                        }
                    }
                    pub mod allocator {
                        use slab_allocator_rs::LockedHeap as
                            LockedSlabAllocator;
                        use proxy::ProxyAllocator;
                        use core::ffi::c_void;
                        pub static ALLOCATOR:
                         ProxyAllocator<LockedSlabAllocator> =
                            ProxyAllocator(LockedSlabAllocator::empty());
                        const _: () =
                            {
                                #[rustc_std_internal_symbol]
                                unsafe fn __rg_alloc(arg0: usize, arg1: usize)
                                 -> *mut u8 {
                                    ::core::alloc::GlobalAlloc::alloc(&ALLOCATOR,
                                                                      ::core::alloc::Layout::from_size_align_unchecked(arg0,
                                                                                                                       arg1))
                                        as *mut u8
                                }
                                #[rustc_std_internal_symbol]
                                unsafe fn __rg_dealloc(arg0: *mut u8,
                                                       arg1: usize,
                                                       arg2: usize) -> () {
                                    ::core::alloc::GlobalAlloc::dealloc(&ALLOCATOR,
                                                                        arg0
                                                                            as
                                                                            *mut u8,
                                                                        ::core::alloc::Layout::from_size_align_unchecked(arg1,
                                                                                                                         arg2))
                                }
                                #[rustc_std_internal_symbol]
                                unsafe fn __rg_realloc(arg0: *mut u8,
                                                       arg1: usize,
                                                       arg2: usize,
                                                       arg3: usize)
                                 -> *mut u8 {
                                    ::core::alloc::GlobalAlloc::realloc(&ALLOCATOR,
                                                                        arg0
                                                                            as
                                                                            *mut u8,
                                                                        ::core::alloc::Layout::from_size_align_unchecked(arg1,
                                                                                                                         arg2),
                                                                        arg3)
                                        as *mut u8
                                }
                                #[rustc_std_internal_symbol]
                                unsafe fn __rg_alloc_zeroed(arg0: usize,
                                                            arg1: usize)
                                 -> *mut u8 {
                                    ::core::alloc::GlobalAlloc::alloc_zeroed(&ALLOCATOR,
                                                                             ::core::alloc::Layout::from_size_align_unchecked(arg0,
                                                                                                                              arg1))
                                        as *mut u8
                                }
                            };
                        extern "C" {
                            static _heap_start: c_void ;
                            static _heap_end: c_void ;
                        }
                        pub fn init() {
                            let heap_end =
                                unsafe {
                                    &_heap_end as *const c_void as usize
                                };
                            let heap_start =
                                unsafe {
                                    &_heap_start as *const c_void as usize
                                };
                            let mut heap_size: usize = heap_end - heap_start;
                            heap_size /= slab_allocator_rs::MIN_HEAP_SIZE;
                            heap_size *= slab_allocator_rs::MIN_HEAP_SIZE;
                            unsafe {
                                ALLOCATOR.0.init(heap_start, heap_size)
                            };
                        }
                        pub mod proxy {
                            //! Wraps around an allocator. Useful for debugging.
                            use core::alloc::{GlobalAlloc, Layout};
                            pub struct ProxyAllocator<T: GlobalAlloc>(pub T);
                            unsafe impl <T: GlobalAlloc> GlobalAlloc for
                             ProxyAllocator<T> {
                                #[inline]
                                unsafe fn alloc(&self, layout: Layout)
                                 -> *mut u8 {
                                    self.0.alloc(layout)
                                }
                                #[inline]
                                unsafe fn dealloc(&self, ptr: *mut u8,
                                                  layout: Layout) {
                                    self.0.dealloc(ptr, layout)
                                }
                            }
                        }
                    }
                    pub mod context_switch {
                        use alloc::sync::Arc;
                        use crate::{cpu, process::{self, ProcessState},
                                    scheduler};
                        /// Trigger a context switch. Must be called from an interrupt context.
                        pub fn context_switch(pid: &usize) -> ! {
                            let lock = process::try_get_process(pid);
                            let mut guard = lock.write();
                            unsafe { lock.force_unlock_write() };
                            unsafe {
                                let raw = Arc::into_raw(lock.clone());
                                Arc::decrement_strong_count(raw);
                                Arc::decrement_strong_count(raw);
                            };
                            guard.run_once()
                        }
                        pub fn make_this_process_pending() {
                            match unsafe {
                                      process::weak_get_process(&(*cpu::read_sscratch()).pid)
                                  }.upgrade() {
                                None => { }
                                Some(p) => {
                                    p.write().state = ProcessState::Pending
                                }
                            }
                        }
                        pub fn schedule_and_switch() -> ! {
                            let new_pid = scheduler::schedule();
                            if new_pid == 0 {
                                if process::useful_process_count() == 0 {
                                    {
                                        let lvl = ::log::Level::Info;
                                        if lvl <= ::log::STATIC_MAX_LEVEL &&
                                               lvl <= ::log::max_level() {
                                            ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["No processes alive, nothing left to schedule!"],
                                                                                                    &match ()
                                                                                                         {
                                                                                                         ()
                                                                                                         =>
                                                                                                         [],
                                                                                                     }),
                                                                     lvl,
                                                                     &("rust_0bsd_riscv_kernel::context_switch",
                                                                       "rust_0bsd_riscv_kernel::context_switch",
                                                                       "src/context_switch.rs",
                                                                       53u32));
                                        }
                                    };
                                    crate::sbi::shutdown(0);
                                } else {
                                    {
                                        let lvl = ::log::Level::Warn;
                                        if lvl <= ::log::STATIC_MAX_LEVEL &&
                                               lvl <= ::log::max_level() {
                                            ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["All processes have yielded"],
                                                                                                    &match ()
                                                                                                         {
                                                                                                         ()
                                                                                                         =>
                                                                                                         [],
                                                                                                     }),
                                                                     lvl,
                                                                     &("rust_0bsd_riscv_kernel::context_switch",
                                                                       "rust_0bsd_riscv_kernel::context_switch",
                                                                       "src/context_switch.rs",
                                                                       57u32));
                                        }
                                    };
                                    process::idle()
                                }
                            }
                            context_switch(&new_pid)
                        }
                    }
                    pub mod timer_queue {
                        use alloc::collections::{BinaryHeap, BTreeMap};
                        use crate::lock::shared::RwLock;
                        use crate::cpu::load_hartid;
                        use crate::{sbi};
                        /// SBI only allows us to have 1 timer set at a time
                        /// So instead we have to keep track of all points in time we want to get interrupted on
                        /// and only set the lowest
                        pub enum TimerEventCause {
                            ContextSwitch,
                            TimeoutFuture,
                        }
                        #[automatically_derived]
                        #[allow(unused_qualifications)]
                        impl ::core::fmt::Debug for TimerEventCause {
                            fn fmt(&self, f: &mut ::core::fmt::Formatter)
                             -> ::core::fmt::Result {
                                match (&*self,) {
                                    (&TimerEventCause::ContextSwitch,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "ContextSwitch");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&TimerEventCause::TimeoutFuture,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "TimeoutFuture");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                }
                            }
                        }
                        impl ::core::marker::StructuralPartialEq for
                         TimerEventCause {
                        }
                        #[automatically_derived]
                        #[allow(unused_qualifications)]
                        impl ::core::cmp::PartialEq for TimerEventCause {
                            #[inline]
                            fn eq(&self, other: &TimerEventCause) -> bool {
                                {
                                    let __self_vi =
                                        ::core::intrinsics::discriminant_value(&*self);
                                    let __arg_1_vi =
                                        ::core::intrinsics::discriminant_value(&*other);
                                    if true && __self_vi == __arg_1_vi {
                                        match (&*self, &*other) { _ => true, }
                                    } else { false }
                                }
                            }
                        }
                        impl ::core::marker::StructuralEq for TimerEventCause
                         {
                        }
                        #[automatically_derived]
                        #[allow(unused_qualifications)]
                        impl ::core::cmp::Eq for TimerEventCause {
                            #[inline]
                            #[doc(hidden)]
                            #[no_coverage]
                            fn assert_receiver_is_total_eq(&self) -> () {
                                { }
                            }
                        }
                        #[automatically_derived]
                        #[allow(unused_qualifications)]
                        impl ::core::cmp::PartialOrd for TimerEventCause {
                            #[inline]
                            fn partial_cmp(&self, other: &TimerEventCause)
                             ->
                                 ::core::option::Option<::core::cmp::Ordering> {
                                {
                                    let __self_vi =
                                        ::core::intrinsics::discriminant_value(&*self);
                                    let __arg_1_vi =
                                        ::core::intrinsics::discriminant_value(&*other);
                                    if true && __self_vi == __arg_1_vi {
                                        match (&*self, &*other) {
                                            _ =>
                                            ::core::option::Option::Some(::core::cmp::Ordering::Equal),
                                        }
                                    } else {
                                        ::core::cmp::PartialOrd::partial_cmp(&__self_vi,
                                                                             &__arg_1_vi)
                                    }
                                }
                            }
                        }
                        #[automatically_derived]
                        #[allow(unused_qualifications)]
                        impl ::core::cmp::Ord for TimerEventCause {
                            #[inline]
                            fn cmp(&self, other: &TimerEventCause)
                             -> ::core::cmp::Ordering {
                                {
                                    let __self_vi =
                                        ::core::intrinsics::discriminant_value(&*self);
                                    let __arg_1_vi =
                                        ::core::intrinsics::discriminant_value(&*other);
                                    if true && __self_vi == __arg_1_vi {
                                        match (&*self, &*other) {
                                            _ => ::core::cmp::Ordering::Equal,
                                        }
                                    } else {
                                        ::core::cmp::Ord::cmp(&__self_vi,
                                                              &__arg_1_vi)
                                    }
                                }
                            }
                        }
                        pub struct TimerEvent {
                            pub instant: u64,
                            pub cause: TimerEventCause,
                        }
                        #[automatically_derived]
                        #[allow(unused_qualifications)]
                        impl ::core::fmt::Debug for TimerEvent {
                            fn fmt(&self, f: &mut ::core::fmt::Formatter)
                             -> ::core::fmt::Result {
                                match *self {
                                    TimerEvent {
                                    instant: ref __self_0_0,
                                    cause: ref __self_0_1 } => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_struct(f,
                                                                                      "TimerEvent");
                                        let _ =
                                            ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                            "instant",
                                                                            &&(*__self_0_0));
                                        let _ =
                                            ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                            "cause",
                                                                            &&(*__self_0_1));
                                        ::core::fmt::DebugStruct::finish(debug_trait_builder)
                                    }
                                }
                            }
                        }
                        impl ::core::marker::StructuralEq for TimerEvent { }
                        #[automatically_derived]
                        #[allow(unused_qualifications)]
                        impl ::core::cmp::Eq for TimerEvent {
                            #[inline]
                            #[doc(hidden)]
                            #[no_coverage]
                            fn assert_receiver_is_total_eq(&self) -> () {
                                {
                                    let _: ::core::cmp::AssertParamIsEq<u64>;
                                    let _:
                                            ::core::cmp::AssertParamIsEq<TimerEventCause>;
                                }
                            }
                        }
                        impl ::core::marker::StructuralPartialEq for
                         TimerEvent {
                        }
                        #[automatically_derived]
                        #[allow(unused_qualifications)]
                        impl ::core::cmp::PartialEq for TimerEvent {
                            #[inline]
                            fn eq(&self, other: &TimerEvent) -> bool {
                                match *other {
                                    TimerEvent {
                                    instant: ref __self_1_0,
                                    cause: ref __self_1_1 } =>
                                    match *self {
                                        TimerEvent {
                                        instant: ref __self_0_0,
                                        cause: ref __self_0_1 } =>
                                        (*__self_0_0) == (*__self_1_0) &&
                                            (*__self_0_1) == (*__self_1_1),
                                    },
                                }
                            }
                            #[inline]
                            fn ne(&self, other: &TimerEvent) -> bool {
                                match *other {
                                    TimerEvent {
                                    instant: ref __self_1_0,
                                    cause: ref __self_1_1 } =>
                                    match *self {
                                        TimerEvent {
                                        instant: ref __self_0_0,
                                        cause: ref __self_0_1 } =>
                                        (*__self_0_0) != (*__self_1_0) ||
                                            (*__self_0_1) != (*__self_1_1),
                                    },
                                }
                            }
                        }
                        impl PartialOrd for TimerEvent {
                            fn partial_cmp(&self, other: &Self)
                             -> Option<core::cmp::Ordering> {
                                Some(self.cmp(other))
                            }
                        }
                        impl Ord for TimerEvent {
                            fn cmp(&self, other: &Self)
                             -> core::cmp::Ordering {
                                if self.instant == other.instant {
                                    self.cause.cmp(&other.cause)
                                } else {
                                    self.instant.cmp(&other.instant).reverse()
                                }
                            }
                        }
                        static TIMER_QUEUE:
                         RwLock<BTreeMap<usize,
                                         RwLock<BinaryHeap<TimerEvent>>>> =
                            RwLock::new(BTreeMap::new());
                        pub fn init() { }
                        /// Does initialization local to this hart
                        pub fn init_hart() {
                            let mut l = TIMER_QUEUE.write();
                            let hid = load_hartid();
                            l.insert(hid, RwLock::new(BinaryHeap::new()));
                        }
                        /// Removes the earliest time event and returns the time it happened and its cause
                        /// Note that the time it happened might actually be _after_ the current time, in which case this functions shouldn't have been called
                        pub fn last_cause() -> TimerEvent {
                            TIMER_QUEUE.read()[&load_hartid()].write().pop().unwrap()
                        }
                        pub fn schedule_next() {
                            let next_time =
                                TIMER_QUEUE.read().get(&load_hartid()).expect("Hartid queue not found!").read().peek().expect("Deadlock: Timer queue was drained to zero This should never happen!").instant;
                            sbi::set_absolute_timer(next_time).unwrap();
                        }
                        pub fn schedule_at(event: TimerEvent) {
                            let t = TIMER_QUEUE.read();
                            let e =
                                t.get(&load_hartid()).expect("Hartid queue not found! (2)");
                            e.write().push(event);
                            drop(t);
                        }
                    }
                    pub mod trap {
                        use crate::{HART_PANIC_COUNT, context_switch,
                                    cpu::{self, Registers, load_hartid,
                                          read_sscratch}, external_interrupt,
                                    hart::get_this_hart_meta,
                                    interrupt_context_waker,
                                    process::delete_process, sbi,
                                    scheduler::schedule_next_slice, syscall,
                                    timeout, timer_queue};
                        /// A pointer to this struct is placed in sscratch
                        #[repr(C)]
                        pub struct TrapFrame {
                            pub general_registers: [usize; 32],
                            pub pc: usize,
                            pub hartid: usize,
                            pub pid: usize,
                            /// This may be shared between different processes executing the same hart
                            pub interrupt_stack: usize,
                            pub flags: usize,
                        }
                        #[automatically_derived]
                        #[allow(unused_qualifications)]
                        impl ::core::default::Default for TrapFrame {
                            #[inline]
                            fn default() -> TrapFrame {
                                TrapFrame{general_registers:
                                              ::core::default::Default::default(),
                                          pc:
                                              ::core::default::Default::default(),
                                          hartid:
                                              ::core::default::Default::default(),
                                          pid:
                                              ::core::default::Default::default(),
                                          interrupt_stack:
                                              ::core::default::Default::default(),
                                          flags:
                                              ::core::default::Default::default(),}
                            }
                        }
                        #[automatically_derived]
                        #[allow(unused_qualifications)]
                        impl ::core::fmt::Debug for TrapFrame {
                            fn fmt(&self, f: &mut ::core::fmt::Formatter)
                             -> ::core::fmt::Result {
                                match *self {
                                    TrapFrame {
                                    general_registers: ref __self_0_0,
                                    pc: ref __self_0_1,
                                    hartid: ref __self_0_2,
                                    pid: ref __self_0_3,
                                    interrupt_stack: ref __self_0_4,
                                    flags: ref __self_0_5 } => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_struct(f,
                                                                                      "TrapFrame");
                                        let _ =
                                            ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                            "general_registers",
                                                                            &&(*__self_0_0));
                                        let _ =
                                            ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                            "pc",
                                                                            &&(*__self_0_1));
                                        let _ =
                                            ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                            "hartid",
                                                                            &&(*__self_0_2));
                                        let _ =
                                            ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                            "pid",
                                                                            &&(*__self_0_3));
                                        let _ =
                                            ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                            "interrupt_stack",
                                                                            &&(*__self_0_4));
                                        let _ =
                                            ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                            "flags",
                                                                            &&(*__self_0_5));
                                        ::core::fmt::DebugStruct::finish(debug_trait_builder)
                                    }
                                }
                            }
                        }
                        #[automatically_derived]
                        #[allow(unused_qualifications)]
                        impl ::core::clone::Clone for TrapFrame {
                            #[inline]
                            fn clone(&self) -> TrapFrame {
                                match *self {
                                    TrapFrame {
                                    general_registers: ref __self_0_0,
                                    pc: ref __self_0_1,
                                    hartid: ref __self_0_2,
                                    pid: ref __self_0_3,
                                    interrupt_stack: ref __self_0_4,
                                    flags: ref __self_0_5 } =>
                                    TrapFrame{general_registers:
                                                  ::core::clone::Clone::clone(&(*__self_0_0)),
                                              pc:
                                                  ::core::clone::Clone::clone(&(*__self_0_1)),
                                              hartid:
                                                  ::core::clone::Clone::clone(&(*__self_0_2)),
                                              pid:
                                                  ::core::clone::Clone::clone(&(*__self_0_3)),
                                              interrupt_stack:
                                                  ::core::clone::Clone::clone(&(*__self_0_4)),
                                              flags:
                                                  ::core::clone::Clone::clone(&(*__self_0_5)),},
                                }
                            }
                        }
                        impl TrapFrame {
                            pub const fn zeroed() -> Self {
                                Self{general_registers: [0; 32],
                                     hartid: 0,
                                     pid: 0,
                                     pc: 0,
                                     interrupt_stack: 0,
                                     flags: 0,}
                            }
                            pub const fn zeroed_interrupt_context() -> Self {
                                Self{general_registers: [0; 32],
                                     hartid: 0,
                                     pid: 0,
                                     pc: 0,
                                     interrupt_stack: 0,
                                     flags: 1,}
                            }
                            pub fn inherit_from(&mut self, other: &TrapFrame)
                             -> &mut TrapFrame {
                                self.hartid = other.hartid;
                                self.interrupt_stack = other.interrupt_stack;
                                self.flags = other.flags;
                                self
                            }
                            pub fn print(&self) {
                                {

                                    #[allow(unused_unsafe)]
                                    {
                                        use core::fmt::Write;
                                        let l =
                                            crate::std_macros::OUTPUT_LOCK.lock();
                                        let _ =
                                            unsafe {
                                                crate::drivers::uart::Uart::new(0x1000_0000)
                                            }.write_fmt(::core::fmt::Arguments::new_v1(&["",
                                                                                         "\r\n"],
                                                                                       &match (&"trap",)
                                                                                            {
                                                                                            (arg0,)
                                                                                            =>
                                                                                            [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                          ::core::fmt::Debug::fmt)],
                                                                                        }));
                                    }
                                };
                                for (idx, i) in
                                    self.general_registers[1..].iter().enumerate()
                                    {

                                    #[allow(unused_unsafe)]
                                    {
                                        use core::fmt::Write;
                                        let l =
                                            crate::std_macros::OUTPUT_LOCK.lock();
                                        let _ =
                                            unsafe {
                                                crate::drivers::uart::Uart::new(0x1000_0000)
                                            }.write_fmt(::core::fmt::Arguments::new_v1_formatted(&["0x",
                                                                                                   " "],
                                                                                                 &match (&i,)
                                                                                                      {
                                                                                                      (arg0,)
                                                                                                      =>
                                                                                                      [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                    ::core::fmt::LowerHex::fmt)],
                                                                                                  },
                                                                                                 &[::core::fmt::rt::v1::Argument{position:
                                                                                                                                     0usize,
                                                                                                                                 format:
                                                                                                                                     ::core::fmt::rt::v1::FormatSpec{fill:
                                                                                                                                                                         '0',
                                                                                                                                                                     align:
                                                                                                                                                                         ::core::fmt::rt::v1::Alignment::Left,
                                                                                                                                                                     flags:
                                                                                                                                                                         0u32,
                                                                                                                                                                     precision:
                                                                                                                                                                         ::core::fmt::rt::v1::Count::Implied,
                                                                                                                                                                     width:
                                                                                                                                                                         ::core::fmt::rt::v1::Count::Is(8usize),},}]));
                                    };
                                    if idx % 4 == 0 {
                                        {

                                            #[allow(unused_unsafe)]
                                            {
                                                use core::fmt::Write;
                                                let l =
                                                    crate::std_macros::OUTPUT_LOCK.lock();
                                                let _ =
                                                    unsafe {
                                                        crate::drivers::uart::Uart::new(0x1000_0000)
                                                    }.write_fmt(::core::fmt::Arguments::new_v1(&["\r\n"],
                                                                                               &match ()
                                                                                                    {
                                                                                                    ()
                                                                                                    =>
                                                                                                    [],
                                                                                                }));
                                            }
                                        };
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
                                cpu::write_sscratch(self as *const TrapFrame
                                                        as usize)
                            }
                        }
                        /// If sscratch equals original_trap_frame, then set sscratch to the boot frame for this hart
                        pub fn use_boot_frame_if_necessary(original_trap_frame:
                                                               *const TrapFrame) {
                            if core::ptr::eq(read_sscratch(),
                                             original_trap_frame) {
                                {
                                    let lvl = ::log::Level::Info;
                                    if lvl <= ::log::STATIC_MAX_LEVEL &&
                                           lvl <= ::log::max_level() {
                                        ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["Changed frame"],
                                                                                                &match ()
                                                                                                     {
                                                                                                     ()
                                                                                                     =>
                                                                                                     [],
                                                                                                 }),
                                                                 lvl,
                                                                 &("rust_0bsd_riscv_kernel::trap",
                                                                   "rust_0bsd_riscv_kernel::trap",
                                                                   "src/trap.rs",
                                                                   66u32));
                                    }
                                };
                                unsafe {
                                    get_this_hart_meta().unwrap().boot_frame.write().make_current()
                                };
                            }
                        }
                        impl Drop for TrapFrame {
                            fn drop(&mut self) {
                                {
                                    let lvl = ::log::Level::Warn;
                                    if lvl <= ::log::STATIC_MAX_LEVEL &&
                                           lvl <= ::log::max_level() {
                                        ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["Trap frame for pid ",
                                                                                                  " dropped"],
                                                                                                &match (&self.pid,)
                                                                                                     {
                                                                                                     (arg0,)
                                                                                                     =>
                                                                                                     [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                   ::core::fmt::Display::fmt)],
                                                                                                 }),
                                                                 lvl,
                                                                 &("rust_0bsd_riscv_kernel::trap",
                                                                   "rust_0bsd_riscv_kernel::trap",
                                                                   "src/trap.rs",
                                                                   75u32));
                                    }
                                };
                                if self as *const Self == read_sscratch() {
                                    {
                                        let lvl = ::log::Level::Warn;
                                        if lvl <= ::log::STATIC_MAX_LEVEL &&
                                               lvl <= ::log::max_level() {
                                            ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["sscratch contains a dropped trap frame! Use-after-free is likely to happen"],
                                                                                                    &match ()
                                                                                                         {
                                                                                                         ()
                                                                                                         =>
                                                                                                         [],
                                                                                                     }),
                                                                     lvl,
                                                                     &("rust_0bsd_riscv_kernel::trap",
                                                                       "rust_0bsd_riscv_kernel::trap",
                                                                       "src/trap.rs",
                                                                       77u32));
                                        }
                                    };
                                }
                            }
                        }
                        #[inline]
                        pub fn in_interrupt_context() -> bool {
                            unsafe {
                                read_sscratch().as_ref().unwrap().is_interrupt_context()
                            }
                        }
                        #[inline]
                        pub(crate) fn set_interrupt_context() {
                            unsafe { (*read_sscratch()).flags |= 1 }
                        }
                        #[inline]
                        pub(crate) fn clear_interrupt_context() {
                            unsafe { (*read_sscratch()).flags &= !1 }
                        }
                        struct PanicGuard {
                        }
                        impl Drop for PanicGuard {
                            fn drop(&mut self) { }
                        }
                        /// # Safety
                        /// This should never really be called directly from Rust. There's just too many invariants that need to be satisfied
                        #[no_mangle]
                        pub unsafe extern "C" fn trap_handler(epc: usize,
                                                              tval: usize,
                                                              cause: usize,
                                                              hartid: usize,
                                                              sstatus: usize,
                                                              frame:
                                                                  *mut TrapFrame)
                         -> usize {
                            if HART_PANIC_COUNT.load(core::sync::atomic::Ordering::Acquire)
                                   != 0 {
                                ::core::panicking::panic_fmt(::core::fmt::Arguments::new_v1(&[""],
                                                                                            &match (&"other hart panicked!",)
                                                                                                 {
                                                                                                 (arg0,)
                                                                                                 =>
                                                                                                 [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                               ::core::fmt::Display::fmt)],
                                                                                             }));
                            }
                            let panic_guard = PanicGuard{};
                            let is_interrupt =
                                (cause & (usize::MAX / 2 + 1)) != 0;
                            let cause = cause & 0xFFF;
                            if !is_interrupt && in_interrupt_context() &&
                                   read_sscratch().as_ref().unwrap().has_trapped_before()
                               {
                                read_sscratch().as_mut().unwrap().set_double_faulting();
                                ::core::panicking::panic("Double fault");
                            }
                            read_sscratch().as_mut().unwrap().set_trapped_before();
                            set_interrupt_context();
                            {
                                let lvl = ::log::Level::Debug;
                                if lvl <= ::log::STATIC_MAX_LEVEL &&
                                       lvl <= ::log::max_level() {
                                    ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["Trap from PID "],
                                                                                            &match (&unsafe
                                                                                                     {
                                                                                                         (*frame).pid
                                                                                                     },)
                                                                                                 {
                                                                                                 (arg0,)
                                                                                                 =>
                                                                                                 [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                               ::core::fmt::LowerHex::fmt)],
                                                                                             }),
                                                             lvl,
                                                             &("rust_0bsd_riscv_kernel::trap",
                                                               "rust_0bsd_riscv_kernel::trap",
                                                               "src/trap.rs",
                                                               136u32));
                                }
                            };
                            {
                                let lvl = ::log::Level::Debug;
                                if lvl <= ::log::STATIC_MAX_LEVEL &&
                                       lvl <= ::log::max_level() {
                                    ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["\u{1b}[1;35mV ENTER TRAP\u{1b}[0m"],
                                                                                            &match ()
                                                                                                 {
                                                                                                 ()
                                                                                                 =>
                                                                                                 [],
                                                                                             }),
                                                             lvl,
                                                             &("rust_0bsd_riscv_kernel::trap",
                                                               "rust_0bsd_riscv_kernel::trap",
                                                               "src/trap.rs",
                                                               137u32));
                                }
                            };
                            interrupt_context_waker::wake_all();
                            if is_interrupt {
                                match cause {
                                    1 => {
                                        unsafe {
                                            cpu::write_sip(cpu::read_sip() &
                                                               !2)
                                        };
                                        {
                                            let lvl = ::log::Level::Debug;
                                            if lvl <= ::log::STATIC_MAX_LEVEL
                                                   &&
                                                   lvl <= ::log::max_level() {
                                                ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["\u{1b}[1;36m^ SYSCALL TRAP\u{1b}[0m"],
                                                                                                        &match ()
                                                                                                             {
                                                                                                             ()
                                                                                                             =>
                                                                                                             [],
                                                                                                         }),
                                                                         lvl,
                                                                         &("rust_0bsd_riscv_kernel::trap",
                                                                           "rust_0bsd_riscv_kernel::trap",
                                                                           "src/trap.rs",
                                                                           152u32));
                                            }
                                        };
                                        syscall::do_syscall(frame);
                                    }
                                    5 => {
                                        sbi::set_absolute_timer(2_u64.pow(63)).unwrap();
                                        let event = timer_queue::last_cause();
                                        use timer_queue::TimerEventCause::*;
                                        match event.cause {
                                            TimeoutFuture => {
                                                timeout::on_timer_event(event.instant);
                                                timer_queue::schedule_next();
                                            }
                                            ContextSwitch => {
                                                {
                                                    let lvl =
                                                        ::log::Level::Debug;
                                                    if lvl <=
                                                           ::log::STATIC_MAX_LEVEL
                                                           &&
                                                           lvl <=
                                                               ::log::max_level()
                                                       {
                                                        ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["scheduling..."],
                                                                                                                &match ()
                                                                                                                     {
                                                                                                                     ()
                                                                                                                     =>
                                                                                                                     [],
                                                                                                                 }),
                                                                                 lvl,
                                                                                 &("rust_0bsd_riscv_kernel::trap",
                                                                                   "rust_0bsd_riscv_kernel::trap",
                                                                                   "src/trap.rs",
                                                                                   171u32));
                                                    }
                                                };
                                                schedule_next_slice(1);
                                                timer_queue::schedule_next();
                                                context_switch::make_this_process_pending();
                                                unsafe {
                                                    get_this_hart_meta().unwrap().boot_frame.write().make_current()
                                                };
                                                context_switch::schedule_and_switch();
                                            }
                                        }
                                    }
                                    9 => {
                                        let meta =
                                            get_this_hart_meta().unwrap();
                                        let interrupt_id =
                                            meta.plic.claim_highest_priority();
                                        {
                                            let lvl = ::log::Level::Info;
                                            if lvl <= ::log::STATIC_MAX_LEVEL
                                                   &&
                                                   lvl <= ::log::max_level() {
                                                ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["Extenral interrupt "],
                                                                                                        &match (&interrupt_id,)
                                                                                                             {
                                                                                                             (arg0,)
                                                                                                             =>
                                                                                                             [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                           ::core::fmt::Display::fmt)],
                                                                                                         }),
                                                                         lvl,
                                                                         &("rust_0bsd_riscv_kernel::trap",
                                                                           "rust_0bsd_riscv_kernel::trap",
                                                                           "src/trap.rs",
                                                                           192u32));
                                            }
                                        };
                                        external_interrupt::external_interrupt(interrupt_id);
                                        meta.plic.complete(interrupt_id);
                                        unsafe {
                                            cpu::write_sip(cpu::read_sip() &
                                                               !(1 << 9))
                                        };
                                    }
                                    _ => {
                                        {
                                            let lvl = ::log::Level::Debug;
                                            if lvl <= ::log::STATIC_MAX_LEVEL
                                                   &&
                                                   lvl <= ::log::max_level() {
                                                ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["Unknown interrupt "],
                                                                                                        &match (&cause,)
                                                                                                             {
                                                                                                             (arg0,)
                                                                                                             =>
                                                                                                             [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                           ::core::fmt::Display::fmt)],
                                                                                                         }),
                                                                         lvl,
                                                                         &("rust_0bsd_riscv_kernel::trap",
                                                                           "rust_0bsd_riscv_kernel::trap",
                                                                           "src/trap.rs",
                                                                           202u32));
                                            }
                                        };
                                    }
                                }
                            } else {
                                match cause {
                                    8 | 9 | 10 | 11 => {
                                        {
                                            let lvl = ::log::Level::Debug;
                                            if lvl <= ::log::STATIC_MAX_LEVEL
                                                   &&
                                                   lvl <= ::log::max_level() {
                                                ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["Envionment call to us happened!"],
                                                                                                        &match ()
                                                                                                             {
                                                                                                             ()
                                                                                                             =>
                                                                                                             [],
                                                                                                         }),
                                                                         lvl,
                                                                         &("rust_0bsd_riscv_kernel::trap",
                                                                           "rust_0bsd_riscv_kernel::trap",
                                                                           "src/trap.rs",
                                                                           208u32));
                                            }
                                        };
                                        loop  { };
                                    }
                                    _ => {
                                        {
                                            let lvl = ::log::Level::Error;
                                            if lvl <= ::log::STATIC_MAX_LEVEL
                                                   &&
                                                   lvl <= ::log::max_level() {
                                                ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["Error with cause: ",
                                                                                                          " pc: ",
                                                                                                          " *pc: "],
                                                                                                        &match (&cause,
                                                                                                                &unsafe
                                                                                                                 {
                                                                                                                     (*frame).pc
                                                                                                                 },
                                                                                                                &unsafe
                                                                                                                 {
                                                                                                                     *((*frame).pc
                                                                                                                           as
                                                                                                                           *const u32)
                                                                                                                 })
                                                                                                             {
                                                                                                             (arg0,
                                                                                                              arg1,
                                                                                                              arg2)
                                                                                                             =>
                                                                                                             [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                           ::core::fmt::Debug::fmt),
                                                                                                              ::core::fmt::ArgumentV1::new(arg1,
                                                                                                                                           ::core::fmt::UpperHex::fmt),
                                                                                                              ::core::fmt::ArgumentV1::new(arg2,
                                                                                                                                           ::core::fmt::UpperHex::fmt)],
                                                                                                         }),
                                                                         lvl,
                                                                         &("rust_0bsd_riscv_kernel::trap",
                                                                           "rust_0bsd_riscv_kernel::trap",
                                                                           "src/trap.rs",
                                                                           212u32));
                                            }
                                        };
                                        delete_process((*frame).pid);
                                        loop  { }
                                    }
                                }
                            }
                            interrupt_context_waker::wake_all();
                            {
                                let lvl = ::log::Level::Debug;
                                if lvl <= ::log::STATIC_MAX_LEVEL &&
                                       lvl <= ::log::max_level() {
                                    ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["\u{1b}[1;36m^ EXIT TRAP ",
                                                                                              "\u{1b}[0m"],
                                                                                            &match (&load_hartid(),)
                                                                                                 {
                                                                                                 (arg0,)
                                                                                                 =>
                                                                                                 [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                               ::core::fmt::Display::fmt)],
                                                                                             }),
                                                             lvl,
                                                             &("rust_0bsd_riscv_kernel::trap",
                                                               "rust_0bsd_riscv_kernel::trap",
                                                               "src/trap.rs",
                                                               221u32));
                                }
                            };
                            clear_interrupt_context();
                            epc
                        }
                    }
                    pub mod paging {
                        use core::ops::{Index, IndexMut};
                        pub mod sv32 { }
                        #[cfg(target_arch = "riscv64")]
                        pub mod sv39 {
                            use crate::sbi;
                            use super::*;
                            /// SAFETY: It's safe if root is a valid pointer
                            /// and paging is disabled
                            /// Otherwise, it can remap things the wrong way and break everything
                            pub unsafe fn identity_map(root: *mut Table) {
                                for (idx, i) in
                                    ((*root).entries).iter_mut().enumerate() {
                                    i.value =
                                        EntryBits::VALID | EntryBits::RWX |
                                            (GIGAPAGE_SIZE / 4 * idx);
                                }
                            }
                            pub struct RootTable<'a>(pub &'a mut Table);
                            #[automatically_derived]
                            #[allow(unused_qualifications)]
                            impl <'a> ::core::fmt::Debug for RootTable<'a> {
                                fn fmt(&self, f: &mut ::core::fmt::Formatter)
                                 -> ::core::fmt::Result {
                                    match *self {
                                        RootTable(ref __self_0_0) => {
                                            let debug_trait_builder =
                                                &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                         "RootTable");
                                            let _ =
                                                ::core::fmt::DebugTuple::field(debug_trait_builder,
                                                                               &&(*__self_0_0));
                                            ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                        }
                                    }
                                }
                            }
                            impl <'a> Paging for RootTable<'a> {
                                fn map(&mut self, virtual_addr: usize,
                                       physical_addr: usize, length: usize,
                                       flags: usize) {
                                    let vpn2_min =
                                        ((virtual_addr >> 28) &
                                             (PAGE_ALIGN - 1)) / 4;
                                    let vpn1_min =
                                        ((virtual_addr >> 19) &
                                             (PAGE_ALIGN - 1)) / 4;
                                    let vpn0_min =
                                        ((virtual_addr >> 9) &
                                             (PAGE_ALIGN - 1)) / 4;
                                    let vpn2_max =
                                        (((virtual_addr + length) >> 28) &
                                             (PAGE_ALIGN - 1)) / 4;
                                    let vpn1_max =
                                        (((virtual_addr + length) >> 19) &
                                             (PAGE_ALIGN - 1)) / 4;
                                    let vpn0_max =
                                        (((virtual_addr + length) >> 9) &
                                             (PAGE_ALIGN - 1)) / 4;
                                    {

                                        #[allow(unused_unsafe)]
                                        {
                                            use core::fmt::Write;
                                            let l =
                                                crate::std_macros::OUTPUT_LOCK.lock();
                                            let _ =
                                                unsafe {
                                                    crate::drivers::uart::Uart::new(0x1000_0000)
                                                }.write_fmt(::core::fmt::Arguments::new_v1(&["",
                                                                                             "\r\n"],
                                                                                           &match (&vpn0_max,)
                                                                                                {
                                                                                                (arg0,)
                                                                                                =>
                                                                                                [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                              ::core::fmt::Debug::fmt)],
                                                                                            }));
                                        }
                                    };
                                    let offset: usize =
                                        physical_addr.wrapping_sub(virtual_addr)
                                            >> 2;
                                    for vpn2 in vpn2_min..vpn2_max + 1 {
                                        let mut entry =
                                            &mut self.0.entries[vpn2];
                                        {

                                            #[allow(unused_unsafe)]
                                            {
                                                use core::fmt::Write;
                                                let l =
                                                    crate::std_macros::OUTPUT_LOCK.lock();
                                                let _ =
                                                    unsafe {
                                                        crate::drivers::uart::Uart::new(0x1000_0000)
                                                    }.write_fmt(::core::fmt::Arguments::new_v1(&["vp2 ",
                                                                                                 " ",
                                                                                                 "\r\n"],
                                                                                               &match (&vpn2,
                                                                                                       &&entry)
                                                                                                    {
                                                                                                    (arg0,
                                                                                                     arg1)
                                                                                                    =>
                                                                                                    [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                  ::core::fmt::Display::fmt),
                                                                                                     ::core::fmt::ArgumentV1::new(arg1,
                                                                                                                                  ::core::fmt::Pointer::fmt)],
                                                                                                }));
                                            }
                                        };
                                        if (vpn2 == vpn2_max ||
                                                vpn2 == vpn2_min) &&
                                               entry.is_leaf() {
                                            unsafe {
                                                entry.split(MEGAPAGE_SIZE)
                                            };
                                            {
                                                let lvl = ::log::Level::Info;
                                                if lvl <=
                                                       ::log::STATIC_MAX_LEVEL
                                                       &&
                                                       lvl <=
                                                           ::log::max_level()
                                                   {
                                                    ::log::__private_api_log(::core::fmt::Arguments::new_v1(&[""],
                                                                                                            &match (&"Split",)
                                                                                                                 {
                                                                                                                 (arg0,)
                                                                                                                 =>
                                                                                                                 [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                               ::core::fmt::Display::fmt)],
                                                                                                             }),
                                                                             lvl,
                                                                             &("rust_0bsd_riscv_kernel::paging::sv39",
                                                                               "rust_0bsd_riscv_kernel::paging::sv39",
                                                                               "src/paging/sv39.rs",
                                                                               35u32));
                                                }
                                            }
                                        };
                                        if let Some(table) =
                                               unsafe {
                                                   entry.try_as_table_mut()
                                               } {
                                            {

                                                #[allow(unused_unsafe)]
                                                {
                                                    use core::fmt::Write;
                                                    let l =
                                                        crate::std_macros::OUTPUT_LOCK.lock();
                                                    let _ =
                                                        unsafe {
                                                            crate::drivers::uart::Uart::new(0x1000_0000)
                                                        }.write_fmt(::core::fmt::Arguments::new_v1(&["",
                                                                                                     "\r\n"],
                                                                                                   &match (&"Table",)
                                                                                                        {
                                                                                                        (arg0,)
                                                                                                        =>
                                                                                                        [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                      ::core::fmt::Debug::fmt)],
                                                                                                    }));
                                                }
                                            };
                                            for vpn1 in vpn1_min..vpn1_max + 1
                                                {
                                                let mut entry =
                                                    &mut table[vpn1];
                                                {

                                                    #[allow(unused_unsafe)]
                                                    {
                                                        use core::fmt::Write;
                                                        let l =
                                                            crate::std_macros::OUTPUT_LOCK.lock();
                                                        let _ =
                                                            unsafe {
                                                                crate::drivers::uart::Uart::new(0x1000_0000)
                                                            }.write_fmt(::core::fmt::Arguments::new_v1(&["vp1 ",
                                                                                                         " ",
                                                                                                         "\r\n"],
                                                                                                       &match (&vpn1,
                                                                                                               &&entry)
                                                                                                            {
                                                                                                            (arg0,
                                                                                                             arg1)
                                                                                                            =>
                                                                                                            [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                          ::core::fmt::Display::fmt),
                                                                                                             ::core::fmt::ArgumentV1::new(arg1,
                                                                                                                                          ::core::fmt::Pointer::fmt)],
                                                                                                        }));
                                                    }
                                                };
                                                if vpn1 == vpn1_max &&
                                                       entry.is_leaf() {
                                                    unsafe {
                                                        entry.split(PAGE_SIZE)
                                                    };
                                                    {
                                                        let lvl =
                                                            ::log::Level::Info;
                                                        if lvl <=
                                                               ::log::STATIC_MAX_LEVEL
                                                               &&
                                                               lvl <=
                                                                   ::log::max_level()
                                                           {
                                                            ::log::__private_api_log(::core::fmt::Arguments::new_v1(&[""],
                                                                                                                    &match (&"Split",)
                                                                                                                         {
                                                                                                                         (arg0,)
                                                                                                                         =>
                                                                                                                         [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                                       ::core::fmt::Display::fmt)],
                                                                                                                     }),
                                                                                     lvl,
                                                                                     &("rust_0bsd_riscv_kernel::paging::sv39",
                                                                                       "rust_0bsd_riscv_kernel::paging::sv39",
                                                                                       "src/paging/sv39.rs",
                                                                                       44u32));
                                                        }
                                                    }
                                                };
                                                if let Some(table) =
                                                       unsafe {
                                                           entry.try_as_table_mut()
                                                       } {
                                                    for vpn0 in
                                                        vpn0_min..vpn0_max {
                                                        let mut entry =
                                                            &mut table[vpn0];
                                                        {

                                                            #[allow(unused_unsafe)]
                                                            {
                                                                use core::fmt::Write;
                                                                let l =
                                                                    crate::std_macros::OUTPUT_LOCK.lock();
                                                                let _ =
                                                                    unsafe {
                                                                        crate::drivers::uart::Uart::new(0x1000_0000)
                                                                    }.write_fmt(::core::fmt::Arguments::new_v1(&["vp0 ",
                                                                                                                 " ",
                                                                                                                 "\r\n"],
                                                                                                               &match (&vpn0,
                                                                                                                       &&entry)
                                                                                                                    {
                                                                                                                    (arg0,
                                                                                                                     arg1)
                                                                                                                    =>
                                                                                                                    [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                                  ::core::fmt::Display::fmt),
                                                                                                                     ::core::fmt::ArgumentV1::new(arg1,
                                                                                                                                                  ::core::fmt::Pointer::fmt)],
                                                                                                                }));
                                                            }
                                                        };
                                                        {

                                                            #[allow(unused_unsafe)]
                                                            {
                                                                use core::fmt::Write;
                                                                let l =
                                                                    crate::std_macros::OUTPUT_LOCK.lock();
                                                                let _ =
                                                                    unsafe {
                                                                        crate::drivers::uart::Uart::new(0x1000_0000)
                                                                    }.write_fmt(::core::fmt::Arguments::new_v1(&["oldval ",
                                                                                                                 "\r\n"],
                                                                                                               &match (&entry.value,)
                                                                                                                    {
                                                                                                                    (arg0,)
                                                                                                                    =>
                                                                                                                    [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                                  ::core::fmt::LowerHex::fmt)],
                                                                                                                }));
                                                            }
                                                        };
                                                        {

                                                            #[allow(unused_unsafe)]
                                                            {
                                                                use core::fmt::Write;
                                                                let l =
                                                                    crate::std_macros::OUTPUT_LOCK.lock();
                                                                let _ =
                                                                    unsafe {
                                                                        crate::drivers::uart::Uart::new(0x1000_0000)
                                                                    }.write_fmt(::core::fmt::Arguments::new_v1(&["virt ",
                                                                                                                 "\r\n"],
                                                                                                               &match (&(vpn2
                                                                                                                             <<
                                                                                                                             30
                                                                                                                             |
                                                                                                                             vpn1
                                                                                                                                 <<
                                                                                                                                 21
                                                                                                                             |
                                                                                                                             vpn0
                                                                                                                                 <<
                                                                                                                                 12),)
                                                                                                                    {
                                                                                                                    (arg0,)
                                                                                                                    =>
                                                                                                                    [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                                  ::core::fmt::LowerHex::fmt)],
                                                                                                                }));
                                                            }
                                                        };
                                                        entry.value =
                                                            (vpn2 << 28 |
                                                                 vpn1 << 19 |
                                                                 vpn0 << 10 |
                                                                 flags).wrapping_add(offset);
                                                        {

                                                            #[allow(unused_unsafe)]
                                                            {
                                                                use core::fmt::Write;
                                                                let l =
                                                                    crate::std_macros::OUTPUT_LOCK.lock();
                                                                let _ =
                                                                    unsafe {
                                                                        crate::drivers::uart::Uart::new(0x1000_0000)
                                                                    }.write_fmt(::core::fmt::Arguments::new_v1(&["newval ",
                                                                                                                 "\r\n"],
                                                                                                               &match (&entry.value,)
                                                                                                                    {
                                                                                                                    (arg0,)
                                                                                                                    =>
                                                                                                                    [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                                  ::core::fmt::LowerHex::fmt)],
                                                                                                                }));
                                                            }
                                                        };
                                                    }
                                                } else {
                                                    {

                                                        #[allow(unused_unsafe)]
                                                        {
                                                            use core::fmt::Write;
                                                            let l =
                                                                crate::std_macros::OUTPUT_LOCK.lock();
                                                            let _ =
                                                                unsafe {
                                                                    crate::drivers::uart::Uart::new(0x1000_0000)
                                                                }.write_fmt(::core::fmt::Arguments::new_v1(&["oldval ",
                                                                                                             "\r\n"],
                                                                                                           &match (&entry.value,)
                                                                                                                {
                                                                                                                (arg0,)
                                                                                                                =>
                                                                                                                [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                              ::core::fmt::LowerHex::fmt)],
                                                                                                            }));
                                                        }
                                                    };
                                                    {

                                                        #[allow(unused_unsafe)]
                                                        {
                                                            use core::fmt::Write;
                                                            let l =
                                                                crate::std_macros::OUTPUT_LOCK.lock();
                                                            let _ =
                                                                unsafe {
                                                                    crate::drivers::uart::Uart::new(0x1000_0000)
                                                                }.write_fmt(::core::fmt::Arguments::new_v1(&["virt ",
                                                                                                             "\r\n"],
                                                                                                           &match (&(vpn2
                                                                                                                         <<
                                                                                                                         30
                                                                                                                         |
                                                                                                                         vpn1
                                                                                                                             <<
                                                                                                                             21),)
                                                                                                                {
                                                                                                                (arg0,)
                                                                                                                =>
                                                                                                                [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                              ::core::fmt::LowerHex::fmt)],
                                                                                                            }));
                                                        }
                                                    };
                                                    entry.value =
                                                        (vpn2 << 28 |
                                                             vpn1 << 19 |
                                                             flags).wrapping_add(offset);
                                                    {

                                                        #[allow(unused_unsafe)]
                                                        {
                                                            use core::fmt::Write;
                                                            let l =
                                                                crate::std_macros::OUTPUT_LOCK.lock();
                                                            let _ =
                                                                unsafe {
                                                                    crate::drivers::uart::Uart::new(0x1000_0000)
                                                                }.write_fmt(::core::fmt::Arguments::new_v1(&["newval ",
                                                                                                             "\r\n"],
                                                                                                           &match (&entry.value,)
                                                                                                                {
                                                                                                                (arg0,)
                                                                                                                =>
                                                                                                                [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                              ::core::fmt::LowerHex::fmt)],
                                                                                                            }));
                                                        }
                                                    };
                                                }
                                            }
                                        } else {
                                            {

                                                #[allow(unused_unsafe)]
                                                {
                                                    use core::fmt::Write;
                                                    let l =
                                                        crate::std_macros::OUTPUT_LOCK.lock();
                                                    let _ =
                                                        unsafe {
                                                            crate::drivers::uart::Uart::new(0x1000_0000)
                                                        }.write_fmt(::core::fmt::Arguments::new_v1(&["oldval ",
                                                                                                     "\r\n"],
                                                                                                   &match (&entry.value,)
                                                                                                        {
                                                                                                        (arg0,)
                                                                                                        =>
                                                                                                        [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                      ::core::fmt::LowerHex::fmt)],
                                                                                                    }));
                                                }
                                            };
                                            {

                                                #[allow(unused_unsafe)]
                                                {
                                                    use core::fmt::Write;
                                                    let l =
                                                        crate::std_macros::OUTPUT_LOCK.lock();
                                                    let _ =
                                                        unsafe {
                                                            crate::drivers::uart::Uart::new(0x1000_0000)
                                                        }.write_fmt(::core::fmt::Arguments::new_v1(&["virt ",
                                                                                                     "\r\n"],
                                                                                                   &match (&(vpn2
                                                                                                                 <<
                                                                                                                 30),)
                                                                                                        {
                                                                                                        (arg0,)
                                                                                                        =>
                                                                                                        [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                      ::core::fmt::LowerHex::fmt)],
                                                                                                    }));
                                                }
                                            };
                                            entry.value =
                                                (vpn2 << 28 |
                                                     flags).wrapping_add(offset);
                                            {

                                                #[allow(unused_unsafe)]
                                                {
                                                    use core::fmt::Write;
                                                    let l =
                                                        crate::std_macros::OUTPUT_LOCK.lock();
                                                    let _ =
                                                        unsafe {
                                                            crate::drivers::uart::Uart::new(0x1000_0000)
                                                        }.write_fmt(::core::fmt::Arguments::new_v1(&["newval ",
                                                                                                     "\r\n"],
                                                                                                   &match (&entry.value,)
                                                                                                        {
                                                                                                        (arg0,)
                                                                                                        =>
                                                                                                        [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                      ::core::fmt::LowerHex::fmt)],
                                                                                                    }));
                                                }
                                            };
                                        }
                                    };
                                    {

                                        #[allow(unused_unsafe)]
                                        {
                                            use core::fmt::Write;
                                            let l =
                                                crate::std_macros::OUTPUT_LOCK.lock();
                                            let _ =
                                                unsafe {
                                                    crate::drivers::uart::Uart::new(0x1000_0000)
                                                }.write_fmt(::core::fmt::Arguments::new_v1(&["",
                                                                                             "\r\n"],
                                                                                           &match (&"finish",)
                                                                                                {
                                                                                                (arg0,)
                                                                                                =>
                                                                                                [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                              ::core::fmt::Debug::fmt)],
                                                                                            }));
                                        }
                                    };
                                    unsafe {
                                        llvm_asm!("sfence.vma":  :  :  :
                                            "volatile")
                                    };
                                    unsafe {
                                        llvm_asm!("fence rw, rw":  :  :  :
                                            "volatile")
                                    };
                                    {
                                        let lvl = ::log::Level::Info;
                                        if lvl <= ::log::STATIC_MAX_LEVEL &&
                                               lvl <= ::log::max_level() {
                                            ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["entry"],
                                                                                                    &match ()
                                                                                                         {
                                                                                                         ()
                                                                                                         =>
                                                                                                         [],
                                                                                                     }),
                                                                     lvl,
                                                                     &("rust_0bsd_riscv_kernel::paging::sv39",
                                                                       "rust_0bsd_riscv_kernel::paging::sv39",
                                                                       "src/paging/sv39.rs",
                                                                       75u32));
                                        }
                                    };
                                }
                            }
                        }
                        #[cfg(target_arch = "riscv64")]
                        pub mod sv48 { }
                        pub mod EntryBits {
                            pub const VALID: usize = 1 << 0;
                            pub const READ: usize = 1 << 1;
                            pub const WRITE: usize = 1 << 2;
                            pub const EXECUTE: usize = 1 << 3;
                            pub const USER: usize = 1 << 4;
                            pub const GLOBAL: usize = 1 << 5;
                            pub const ACCESSED: usize = 1 << 6;
                            pub const DIRTY: usize = 1 << 7;
                            pub const ADDRESS_MASK: usize =
                                usize::MAX ^ ((1 << 8) - 1);
                            pub const RWX: usize = 2 | 4 | 8;
                            pub const CODE_SUPERVISOR: usize =
                                (1 << 1 | 1 << 3 | 1);
                            pub const DATA_SUPERVISOR: usize =
                                (1 << 1 | 1 << 2 | 1);
                        }
                        pub struct Entry {
                            pub value: usize,
                        }
                        #[automatically_derived]
                        #[allow(unused_qualifications)]
                        impl ::core::default::Default for Entry {
                            #[inline]
                            fn default() -> Entry {
                                Entry{value:
                                          ::core::default::Default::default(),}
                            }
                        }
                        #[automatically_derived]
                        #[allow(unused_qualifications)]
                        impl ::core::marker::Copy for Entry { }
                        #[automatically_derived]
                        #[allow(unused_qualifications)]
                        impl ::core::clone::Clone for Entry {
                            #[inline]
                            fn clone(&self) -> Entry {
                                {
                                    let _:
                                            ::core::clone::AssertParamIsClone<usize>;
                                    *self
                                }
                            }
                        }
                        #[automatically_derived]
                        #[allow(unused_qualifications)]
                        impl ::core::fmt::Debug for Entry {
                            fn fmt(&self, f: &mut ::core::fmt::Formatter)
                             -> ::core::fmt::Result {
                                match *self {
                                    Entry { value: ref __self_0_0 } => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_struct(f,
                                                                                      "Entry");
                                        let _ =
                                            ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                            "value",
                                                                            &&(*__self_0_0));
                                        ::core::fmt::DebugStruct::finish(debug_trait_builder)
                                    }
                                }
                            }
                        }
                        impl Entry {
                            pub const fn zeroed() -> Self { Entry{value: 0,} }
                        }
                        impl Entry {
                            /// # Safety
                            /// The entry's value must be a valid physical address pointer 
                            pub unsafe fn as_table_mut(&mut self)
                             -> &mut Table {
                                (((self.value & EntryBits::ADDRESS_MASK) << 2)
                                     as *mut Table).as_mut().unwrap()
                            }
                            /// # Safety
                            /// The entry's value must be a valid physical address pointer 
                            pub unsafe fn as_table(&self) -> &Table {
                                (((self.value & EntryBits::ADDRESS_MASK) << 2)
                                     as *mut Table).as_ref().unwrap()
                            }
                            pub unsafe fn try_as_table_mut(&mut self)
                             -> Option<&mut Table> {
                                if self.is_leaf() {
                                    None
                                } else { Some(self.as_table_mut()) }
                            }
                            pub unsafe fn try_as_table(&self)
                             -> Option<&Table> {
                                if self.is_leaf() {
                                    None
                                } else { Some(self.as_table()) }
                            }
                            pub fn is_leaf(&self) -> bool {
                                (self.value & EntryBits::RWX) != 0
                            }
                            /// This takes a leaf entry and turns it into a reference to a page table with the same effect.
                            /// Increment should be one of the PAGE_SIZE, MEGAPAGE_SIZE, GIGAPAGE_SIZE, etc constants
                            /// If this entry is a megapage, for example, the increment should be PAGE_SIZE
                            pub unsafe fn split(&mut self, increment: usize) {
                                {

                                    #[allow(unused_unsafe)]
                                    {
                                        use core::fmt::Write;
                                        let l =
                                            crate::std_macros::OUTPUT_LOCK.lock();
                                        let _ =
                                            unsafe {
                                                crate::drivers::uart::Uart::new(0x1000_0000)
                                            }.write_fmt(::core::fmt::Arguments::new_v1(&["S ",
                                                                                         "\r\n"],
                                                                                       &match (&self,)
                                                                                            {
                                                                                            (arg0,)
                                                                                            =>
                                                                                            [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                          ::core::fmt::Pointer::fmt)],
                                                                                        }));
                                    }
                                };
                                use alloc::boxed::Box;
                                let mut table = Box::new(Table::zeroed());
                                let mut current_address =
                                    self.value & EntryBits::ADDRESS_MASK;
                                let flags =
                                    self.value & !(EntryBits::ADDRESS_MASK);
                                for entry in table.entries.iter_mut() {
                                    entry.value = flags | current_address;
                                    current_address += increment >> 2;
                                }
                                self.value =
                                    1 |
                                        ((&*table as *const Table as usize) >>
                                             2);
                                Box::leak(table);
                                {

                                    #[allow(unused_unsafe)]
                                    {
                                        use core::fmt::Write;
                                        let l =
                                            crate::std_macros::OUTPUT_LOCK.lock();
                                        let _ =
                                            unsafe {
                                                crate::drivers::uart::Uart::new(0x1000_0000)
                                            }.write_fmt(::core::fmt::Arguments::new_v1(&["",
                                                                                         "\r\n"],
                                                                                       &match (&self.value,)
                                                                                            {
                                                                                            (arg0,)
                                                                                            =>
                                                                                            [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                          ::core::fmt::LowerHex::fmt)],
                                                                                        }));
                                    }
                                };
                                if true {
                                    if !!self.is_leaf() {
                                        ::core::panicking::panic("assertion failed: !self.is_leaf()")
                                    };
                                };
                            }
                        }
                        #[repr(C)]
                        #[repr(align(4096))]
                        pub struct Table {
                            pub entries: [Entry; 512],
                        }
                        #[automatically_derived]
                        #[allow(unused_qualifications)]
                        impl ::core::fmt::Debug for Table {
                            fn fmt(&self, f: &mut ::core::fmt::Formatter)
                             -> ::core::fmt::Result {
                                match *self {
                                    Table { entries: ref __self_0_0 } => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_struct(f,
                                                                                      "Table");
                                        let _ =
                                            ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                            "entries",
                                                                            &&(*__self_0_0));
                                        ::core::fmt::DebugStruct::finish(debug_trait_builder)
                                    }
                                }
                            }
                        }
                        impl Table {
                            pub const fn zeroed() -> Self {
                                Table{entries: [Entry{value: 0,}; 512],}
                            }
                        }
                        impl Index<usize> for Table {
                            type Output = Entry;
                            fn index(&self, idx: usize) -> &Entry {
                                &self.entries[idx]
                            }
                        }
                        impl IndexMut<usize> for Table {
                            fn index_mut(&mut self, idx: usize)
                             -> &mut Entry {
                                &mut self.entries[idx]
                            }
                        }
                        pub trait Paging {
                            fn map(&mut self, physical_addr: usize,
                                   virtual_addr: usize, length: usize,
                                   flags: usize) {
                            }
                        }
                        pub unsafe fn enable(root_table_physical: usize) { }
                        pub static mut PAGE_TABLE_TABLE: Table =
                            Table::zeroed();
                        #[link_section = ".data"]
                        pub static mut ROOT_PAGE: Table = Table::zeroed();
                        pub const ENTRY_COUNT: usize = 512;
                        pub const PAGE_ALIGN: usize = 4096;
                        pub const PAGE_SIZE: usize = PAGE_ALIGN;
                        pub const MEGAPAGE_SIZE: usize =
                            PAGE_ALIGN * ENTRY_COUNT;
                        #[cfg(target_arch = "riscv64")]
                        pub const GIGAPAGE_SIZE: usize =
                            PAGE_ALIGN * ENTRY_COUNT * ENTRY_COUNT;
                        #[cfg(target_arch = "riscv64")]
                        pub const TERAPAGE_SIZE: usize =
                            PAGE_ALIGN * ENTRY_COUNT * ENTRY_COUNT *
                                ENTRY_COUNT;
                    }
                    pub mod external_interrupt {
                        //! Includes code for delegating different interrupts to different handlers
                        //! 
                        use alloc::{collections::BTreeMap, vec::Vec};
                        use alloc::sync::Arc;
                        use crate::lock::shared::RwLock;
                        static EXTERNAL_INTERRUPT_HANDLERS:
                         RwLock<BTreeMap<u32,
                                         Vec<Arc<dyn Fn(u32) + Send + Sync>>>>
                         =
                            RwLock::new(BTreeMap::new());
                        pub fn external_interrupt(id: u32) {
                            if let Some(fns) =
                                   EXTERNAL_INTERRUPT_HANDLERS.read().get(&id)
                               {
                                for function in fns.iter() { function(id); }
                            }
                        }
                        fn add_handler(id: u32,
                                       function:
                                           Arc<dyn Fn(u32) + Send + Sync>) {
                            let mut lock =
                                EXTERNAL_INTERRUPT_HANDLERS.write();
                            match lock.get_mut(&id) {
                                Some(expr) => expr.push(function),
                                None => {
                                    lock.insert(id,
                                                <[_]>::into_vec(box
                                                                    [function]));
                                }
                            };
                        }
                        fn remove_handler(id: u32,
                                          function:
                                              &Arc<dyn Fn(u32) + Send + Sync>)
                         -> Result<(), ()> {
                            let mut guard =
                                EXTERNAL_INTERRUPT_HANDLERS.write();
                            let v = guard.get_mut(&id).unwrap();
                            let index =
                                v.iter().position(|r|
                                                      {

                                                          #[allow(clippy ::
                                                                  vtable_address_comparisons)]
                                                          Arc::ptr_eq(&r,
                                                                      &function)
                                                      }).unwrap();
                            v.remove(index);
                            Ok(())
                        }
                        /// This acts as a guard; the handler is removed when this object is removed
                        pub struct ExternalInterruptHandler {
                            id: u32,
                            function: Arc<dyn Fn(u32) + Send + Sync>,
                        }
                        impl ExternalInterruptHandler {
                            pub fn new(id: u32,
                                       function:
                                           Arc<dyn Fn(u32) + Send + Sync>)
                             -> Self {
                                add_handler(id, function.clone());
                                Self{id, function,}
                            }
                        }
                        impl Drop for ExternalInterruptHandler {
                            fn drop(&mut self) {
                                remove_handler(self.id,
                                               &self.function).unwrap();
                            }
                        }
                    }
                    pub mod device_setup {
                        use core::convert::TryInto;
                        use core::future::Future;
                        use core::sync::atomic::AtomicBool;
                        use core::sync::atomic::Ordering;
                        use core::ops::Deref;
                        use crate::{drivers::virtio,
                                    drivers::virtio::{block::VirtioBlockDevice,
                                                      VirtioDeviceType,
                                                      VirtioDevice},
                                    external_interrupt::ExternalInterruptHandler,
                                    fdt::PropertyValue};
                        use alloc::{vec::Vec, sync::Arc};
                        use core::task::Waker;
                        use crate::lock::shared::{RwLock, Mutex};
                        use core::task::Poll;
                        pub struct DeviceSetupDoneFuture {
                            wakers: Mutex<Vec<Waker>>,
                            is_done: AtomicBool,
                        }
                        pub struct DeviceSetupDoneFutureShared(Arc<DeviceSetupDoneFuture>);
                        #[automatically_derived]
                        #[allow(unused_qualifications)]
                        impl ::core::clone::Clone for
                         DeviceSetupDoneFutureShared {
                            #[inline]
                            fn clone(&self) -> DeviceSetupDoneFutureShared {
                                match *self {
                                    DeviceSetupDoneFutureShared(ref __self_0_0)
                                    =>
                                    DeviceSetupDoneFutureShared(::core::clone::Clone::clone(&(*__self_0_0))),
                                }
                            }
                        }
                        impl Deref for DeviceSetupDoneFutureShared {
                            type Target = DeviceSetupDoneFuture;
                            fn deref(&self) -> &Self::Target {
                                self.0.deref()
                            }
                        }
                        impl Future for DeviceSetupDoneFutureShared {
                            type Output = ();
                            fn poll(self: core::pin::Pin<&mut Self>,
                                    cx: &mut core::task::Context<'_>)
                             -> Poll<Self::Output> {
                                if self.0.is_done.load(Ordering::Acquire) {
                                    Poll::Ready(())
                                } else {
                                    self.0.wakers.lock().push(cx.waker().clone());
                                    Poll::Pending
                                }
                            }
                        }
                        impl DeviceSetupDoneFuture {
                            fn wake(&self) {
                                self.is_done.store(true, Ordering::Release);
                                while let Some(waker) =
                                          self.wakers.lock().pop() {
                                    waker.wake()
                                }
                            }
                        }
                        static IS_DONE:
                         RwLock<Option<DeviceSetupDoneFutureShared>> =
                            RwLock::new(None);
                        pub fn is_done_future()
                         -> DeviceSetupDoneFutureShared {
                            let mut lock = IS_DONE.write();
                            match &mut *lock {
                                Some(expr) => { expr.clone() }
                                None => {
                                    let t =
                                        DeviceSetupDoneFutureShared(Arc::new(DeviceSetupDoneFuture{wakers:
                                                                                                       Mutex::new(Vec::new()),
                                                                                                   is_done:
                                                                                                       AtomicBool::new(false),}));
                                    *lock = Some(t.clone());
                                    t
                                }
                            }
                        }
                        /// This functions scans the device tree
                        /// and sets up devices and interrupt handlers for all devices
                        pub fn setup_devices() {
                            {
                                let lvl = ::log::Level::Info;
                                if lvl <= ::log::STATIC_MAX_LEVEL &&
                                       lvl <= ::log::max_level() {
                                    ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["setting up devices"],
                                                                                            &match ()
                                                                                                 {
                                                                                                 ()
                                                                                                 =>
                                                                                                 [],
                                                                                             }),
                                                             lvl,
                                                             &("rust_0bsd_riscv_kernel::device_setup",
                                                               "rust_0bsd_riscv_kernel::device_setup",
                                                               "src/device_setup.rs",
                                                               76u32));
                                }
                            };
                            let lock = crate::fdt::root().read();
                            lock.walk_nonstatic(&mut (|node:
                                                           &crate::fdt::Node|
                                                          {
                                                              if let Some(PropertyValue::String(compatible_with))
                                                                     =
                                                                     node.properties.get("compatible")
                                                                 {
                                                                  match compatible_with
                                                                      {
                                                                      &"virtio,mmio"
                                                                      => {
                                                                          let mut virtio_device =
                                                                              unsafe
                                                                              {
                                                                                  VirtioDevice::new(node.unit_address.unwrap_or(0)
                                                                                                        as
                                                                                                        _)
                                                                              };
                                                                          if virtio_device.is_present()
                                                                             {
                                                                              use alloc::sync::Arc;
                                                                              virtio_device.configure();
                                                                              let virtio_device =
                                                                                  Arc::new(crate::lock::shared::Mutex::new(virtio_device));
                                                                              let handler;
                                                                              if let Some(PropertyValue::u32(interrupt_id))
                                                                                     =
                                                                                     node.properties.get("interrupts")
                                                                                 {
                                                                                  let virtio_device =
                                                                                      virtio_device.clone();
                                                                                  handler
                                                                                      =
                                                                                      Some(ExternalInterruptHandler::new((*interrupt_id).try_into().unwrap(),
                                                                                                                         alloc::sync::Arc::new(move
                                                                                                                                                   |id|
                                                                                                                                                   {
                                                                                                                                                       VirtioDevice::on_interrupt(&*virtio_device);
                                                                                                                                                   })));
                                                                              } else {
                                                                                  handler
                                                                                      =
                                                                                      None;
                                                                              }
                                                                              let virtio_driver;
                                                                              if let Some(d)
                                                                                     =
                                                                                     VirtioDevice::make_driver(virtio_device)
                                                                                 {
                                                                                  virtio_driver
                                                                                      =
                                                                                      d;
                                                                              } else {
                                                                                  return;
                                                                              }
                                                                              *node.kernel_struct.write()
                                                                                  =
                                                                                  Some(alloc::boxed::Box::new((virtio_driver,
                                                                                                               handler)));
                                                                          }
                                                                      }
                                                                      &"ns16550a"
                                                                      => {
                                                                      }
                                                                      _ => {
                                                                          {
                                                                              let lvl =
                                                                                  ::log::Level::Warn;
                                                                              if lvl
                                                                                     <=
                                                                                     ::log::STATIC_MAX_LEVEL
                                                                                     &&
                                                                                     lvl
                                                                                         <=
                                                                                         ::log::max_level()
                                                                                 {
                                                                                  ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["Unrecognized device \'compatible\' field: "],
                                                                                                                                          &match (&compatible_with,)
                                                                                                                                               {
                                                                                                                                               (arg0,)
                                                                                                                                               =>
                                                                                                                                               [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                             ::core::fmt::Debug::fmt)],
                                                                                                                                           }),
                                                                                                           lvl,
                                                                                                           &("rust_0bsd_riscv_kernel::device_setup",
                                                                                                             "rust_0bsd_riscv_kernel::device_setup",
                                                                                                             "src/device_setup.rs",
                                                                                                             127u32));
                                                                              }
                                                                          }
                                                                      }
                                                                  }
                                                              }
                                                          }));
                            drop(lock);
                            let handler2 =
                                ExternalInterruptHandler::new(10,
                                                              alloc::sync::Arc::new(|id|
                                                                                        {
                                                                                            let c =
                                                                                                unsafe
                                                                                                {
                                                                                                    crate::drivers::uart::Uart::new(0x10000000)
                                                                                                }.get().unwrap();

                                                                                            #[allow(unused_unsafe)]
                                                                                            {
                                                                                                use core::fmt::Write;
                                                                                                let l =
                                                                                                    crate::std_macros::OUTPUT_LOCK.lock();
                                                                                                let _ =
                                                                                                    unsafe
                                                                                                    {
                                                                                                        crate::drivers::uart::Uart::new(0x1000_0000)
                                                                                                    }.write_fmt(::core::fmt::Arguments::new_v1(&["C "],
                                                                                                                                               &match (&c,)
                                                                                                                                                    {
                                                                                                                                                    (arg0,)
                                                                                                                                                    =>
                                                                                                                                                    [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                                  ::core::fmt::Display::fmt)],
                                                                                                                                                }));
                                                                                            }
                                                                                        }));
                            {
                                let lvl = ::log::Level::Info;
                                if lvl <= ::log::STATIC_MAX_LEVEL &&
                                       lvl <= ::log::max_level() {
                                    ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["Finished device setup"],
                                                                                            &match ()
                                                                                                 {
                                                                                                 ()
                                                                                                 =>
                                                                                                 [],
                                                                                             }),
                                                             lvl,
                                                             &("rust_0bsd_riscv_kernel::device_setup",
                                                               "rust_0bsd_riscv_kernel::device_setup",
                                                               "src/device_setup.rs",
                                                               143u32));
                                }
                            };
                            is_done_future().wake();
                            unsafe { crate::asm::do_supervisor_syscall_0(1) };
                        }
                    }
                    pub mod interrupt_context_waker {
                        //! This module provides a function to create Wakers that
                        //! run a specific function on interrupt-mode when woken
                        //! Note that the function will only be run when an interrupt happens
                        //! (or if it's woken from an interrupt context, then it's run at the end of the interrupt handler)
                        //! Functions in interrupt contexts can wake up other InterruptContextWakers with no risk of stack overflow or deadlocks
                        //! This is useful for giving functions the same context whether they're ran from a kernel thread or from an interrupt context
                        //! *Interrupt tasks should NOT have blocking operations!*
                        use alloc::{collections::VecDeque, boxed::Box,
                                    task::Wake, sync::Arc};
                        use crate::lock::shared::Mutex;
                        use crate::trap::in_interrupt_context;
                        static WAITING_WAKERS:
                         Mutex<Option<VecDeque<Arc<InterruptContextWaker>>>> =
                            Mutex::new(None);
                        pub struct InterruptContextWaker(pub Box<dyn Fn() +
                                                                 Send +
                                                                 Sync>);
                        impl Wake for InterruptContextWaker {
                            fn wake(self: Arc<Self>) {
                                WAITING_WAKERS.lock().as_mut().unwrap().push_back(self)
                            }
                        }
                        /// When running in an interrupt context, wake up all the interrupt context wakers
                        /// that are waiting to be woken up
                        pub(crate) fn wake_all() {
                            if !in_interrupt_context() {
                                ::core::panicking::panic("assertion failed: in_interrupt_context()")
                            };
                            while let Some(i) =
                                      {
                                          let l =
                                              WAITING_WAKERS.lock().as_mut().unwrap().pop_front();
                                          l
                                      } {
                                (i.0)();
                            }
                        }
                        pub fn init() {
                            *WAITING_WAKERS.lock() = Some(VecDeque::new());
                        }
                    }
                    pub mod syscall {
                        use num_enum::{FromPrimitive, IntoPrimitive};
                        use crate::{context_switch, cpu::Registers,
                                    process::{self, ProcessState,
                                              try_get_process},
                                    trap::TrapFrame};
                        #[repr(usize)]
                        pub enum SyscallNumbers {
                            Exit = 1,
                            Yield = 2,
                            Open = 0x10,
                            Read,
                            Write,
                            Close,
                            Available,
                            Seek,
                            Truncate,
                            Tell,
                            FutureCreate = 0x20,
                            FutureComplete,
                            FutureIsDone,
                            FutureAwait,
                            FutureClone,
                            FutureOr,

                            #[num_enum(default)]
                            Unknown,
                        }
                        #[automatically_derived]
                        #[allow(unused_qualifications)]
                        impl ::core::fmt::Debug for SyscallNumbers {
                            fn fmt(&self, f: &mut ::core::fmt::Formatter)
                             -> ::core::fmt::Result {
                                match (&*self,) {
                                    (&SyscallNumbers::Exit,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "Exit");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&SyscallNumbers::Yield,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "Yield");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&SyscallNumbers::Open,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "Open");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&SyscallNumbers::Read,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "Read");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&SyscallNumbers::Write,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "Write");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&SyscallNumbers::Close,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "Close");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&SyscallNumbers::Available,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "Available");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&SyscallNumbers::Seek,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "Seek");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&SyscallNumbers::Truncate,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "Truncate");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&SyscallNumbers::Tell,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "Tell");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&SyscallNumbers::FutureCreate,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "FutureCreate");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&SyscallNumbers::FutureComplete,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "FutureComplete");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&SyscallNumbers::FutureIsDone,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "FutureIsDone");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&SyscallNumbers::FutureAwait,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "FutureAwait");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&SyscallNumbers::FutureClone,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "FutureClone");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&SyscallNumbers::FutureOr,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "FutureOr");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&SyscallNumbers::Unknown,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "Unknown");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                }
                            }
                        }
                        impl From<SyscallNumbers> for usize {
                            #[inline]
                            fn from(enum_value: SyscallNumbers) -> Self {
                                enum_value as Self
                            }
                        }
                        impl ::num_enum::FromPrimitive for SyscallNumbers {
                            type Primitive = usize;
                            fn from_primitive(number: Self::Primitive)
                             -> Self {
                                #![allow(non_upper_case_globals)]
                                const Exit__num_enum_0__: usize = 1;
                                const Yield__num_enum_0__: usize = 2;
                                const Open__num_enum_0__: usize = 0x10;
                                const Read__num_enum_0__: usize =
                                    usize::wrapping_add(0x10, 1);
                                const Write__num_enum_0__: usize =
                                    usize::wrapping_add(usize::wrapping_add(0x10,
                                                                            1),
                                                        1);
                                const Close__num_enum_0__: usize =
                                    usize::wrapping_add(usize::wrapping_add(usize::wrapping_add(0x10,
                                                                                                1),
                                                                            1),
                                                        1);
                                const Available__num_enum_0__: usize =
                                    usize::wrapping_add(usize::wrapping_add(usize::wrapping_add(usize::wrapping_add(0x10,
                                                                                                                    1),
                                                                                                1),
                                                                            1),
                                                        1);
                                const Seek__num_enum_0__: usize =
                                    usize::wrapping_add(usize::wrapping_add(usize::wrapping_add(usize::wrapping_add(usize::wrapping_add(0x10,
                                                                                                                                        1),
                                                                                                                    1),
                                                                                                1),
                                                                            1),
                                                        1);
                                const Truncate__num_enum_0__: usize =
                                    usize::wrapping_add(usize::wrapping_add(usize::wrapping_add(usize::wrapping_add(usize::wrapping_add(usize::wrapping_add(0x10,
                                                                                                                                                            1),
                                                                                                                                        1),
                                                                                                                    1),
                                                                                                1),
                                                                            1),
                                                        1);
                                const Tell__num_enum_0__: usize =
                                    usize::wrapping_add(usize::wrapping_add(usize::wrapping_add(usize::wrapping_add(usize::wrapping_add(usize::wrapping_add(usize::wrapping_add(0x10,
                                                                                                                                                                                1),
                                                                                                                                                            1),
                                                                                                                                        1),
                                                                                                                    1),
                                                                                                1),
                                                                            1),
                                                        1);
                                const FutureCreate__num_enum_0__: usize =
                                    0x20;
                                const FutureComplete__num_enum_0__: usize =
                                    usize::wrapping_add(0x20, 1);
                                const FutureIsDone__num_enum_0__: usize =
                                    usize::wrapping_add(usize::wrapping_add(0x20,
                                                                            1),
                                                        1);
                                const FutureAwait__num_enum_0__: usize =
                                    usize::wrapping_add(usize::wrapping_add(usize::wrapping_add(0x20,
                                                                                                1),
                                                                            1),
                                                        1);
                                const FutureClone__num_enum_0__: usize =
                                    usize::wrapping_add(usize::wrapping_add(usize::wrapping_add(usize::wrapping_add(0x20,
                                                                                                                    1),
                                                                                                1),
                                                                            1),
                                                        1);
                                const FutureOr__num_enum_0__: usize =
                                    usize::wrapping_add(usize::wrapping_add(usize::wrapping_add(usize::wrapping_add(usize::wrapping_add(0x20,
                                                                                                                                        1),
                                                                                                                    1),
                                                                                                1),
                                                                            1),
                                                        1);
                                const Unknown__num_enum_0__: usize =
                                    usize::wrapping_add(usize::wrapping_add(usize::wrapping_add(usize::wrapping_add(usize::wrapping_add(usize::wrapping_add(0x20,
                                                                                                                                                            1),
                                                                                                                                        1),
                                                                                                                    1),
                                                                                                1),
                                                                            1),
                                                        1);

                                #[deny(unreachable_patterns)]
                                match number {
                                    Exit__num_enum_0__ => Self::Exit,
                                    Yield__num_enum_0__ => Self::Yield,
                                    Open__num_enum_0__ => Self::Open,
                                    Read__num_enum_0__ => Self::Read,
                                    Write__num_enum_0__ => Self::Write,
                                    Close__num_enum_0__ => Self::Close,
                                    Available__num_enum_0__ =>
                                    Self::Available,
                                    Seek__num_enum_0__ => Self::Seek,
                                    Truncate__num_enum_0__ => Self::Truncate,
                                    Tell__num_enum_0__ => Self::Tell,
                                    FutureCreate__num_enum_0__ =>
                                    Self::FutureCreate,
                                    FutureComplete__num_enum_0__ =>
                                    Self::FutureComplete,
                                    FutureIsDone__num_enum_0__ =>
                                    Self::FutureIsDone,
                                    FutureAwait__num_enum_0__ =>
                                    Self::FutureAwait,
                                    FutureClone__num_enum_0__ =>
                                    Self::FutureClone,
                                    FutureOr__num_enum_0__ => Self::FutureOr,
                                    Unknown__num_enum_0__ =>
                                    Self::Unknown,
                                                  #[allow(unreachable_patterns)]
                                                  _ => Self::Unknown,
                                }
                            }
                        }
                        impl ::core::convert::From<usize> for SyscallNumbers {
                            #[inline]
                            fn from(number: usize) -> Self {
                                ::num_enum::FromPrimitive::from_primitive(number)
                            }
                        }
                        impl ::num_enum::TryFromPrimitive for SyscallNumbers {
                            type Primitive = usize;
                            const NAME: &'static str = "SyscallNumbers";
                            #[inline]
                            fn try_from_primitive(number: Self::Primitive)
                             ->
                                 ::core::result::Result<Self,
                                                        ::num_enum::TryFromPrimitiveError<Self>> {
                                Ok(::num_enum::FromPrimitive::from_primitive(number))
                            }
                        }
                        pub fn do_syscall(frame: *mut TrapFrame) {
                            let frame_raw = frame;
                            let frame =
                                unsafe {
                                    frame_raw.as_mut().unwrap_unchecked()
                                };
                            let number =
                                SyscallNumbers::from(frame.general_registers[Registers::A7.idx()]);
                            use SyscallNumbers::*;
                            match number {
                                Exit => { syscall_exit(frame, 0); }
                                Yield => { syscall_yield(frame); }
                                Unknown => {
                                    {
                                        let lvl = ::log::Level::Warn;
                                        if lvl <= ::log::STATIC_MAX_LEVEL &&
                                               lvl <= ::log::max_level() {
                                            ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["Unknown syscall "],
                                                                                                    &match (&frame.general_registers[Registers::A7.idx()],)
                                                                                                         {
                                                                                                         (arg0,)
                                                                                                         =>
                                                                                                         [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                       ::core::fmt::Debug::fmt)],
                                                                                                     }),
                                                                     lvl,
                                                                     &("rust_0bsd_riscv_kernel::syscall",
                                                                       "rust_0bsd_riscv_kernel::syscall",
                                                                       "src/syscall.rs",
                                                                       60u32));
                                        }
                                    };
                                }
                                _ => {
                                    {
                                        let lvl = ::log::Level::Warn;
                                        if lvl <= ::log::STATIC_MAX_LEVEL &&
                                               lvl <= ::log::max_level() {
                                            ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["Unimplemented syscall "],
                                                                                                    &match (&number,)
                                                                                                         {
                                                                                                         (arg0,)
                                                                                                         =>
                                                                                                         [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                       ::core::fmt::Debug::fmt)],
                                                                                                     }),
                                                                     lvl,
                                                                     &("rust_0bsd_riscv_kernel::syscall",
                                                                       "rust_0bsd_riscv_kernel::syscall",
                                                                       "src/syscall.rs",
                                                                       63u32));
                                        }
                                    };
                                }
                            }
                        }
                        pub fn syscall_exit(frame: &mut TrapFrame,
                                            return_code: usize) {
                            crate::process::delete_process(frame.pid);
                            context_switch::schedule_and_switch();
                        }
                        pub fn syscall_yield(frame: &mut TrapFrame) {
                            frame.pc += 4;
                            let mut p = process::try_get_process(&frame.pid);
                            let mut guard = p.write();
                            if guard.try_yield_maybe() {
                                crate::trap::use_boot_frame_if_necessary(&*guard.trap_frame
                                                                             as
                                                                             _);
                            }
                            if guard.yield_maybe() {
                                drop(guard);
                                drop(p);
                                context_switch::schedule_and_switch();
                            }
                        }
                        #[no_mangle]
                        pub extern "C" fn syscall_on_interrupt_disabled() {
                            {
                                let lvl = ::log::Level::Error;
                                if lvl <= ::log::STATIC_MAX_LEVEL &&
                                       lvl <= ::log::max_level() {
                                    ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["Can\'t make a syscall while interrupts are disabled! (Maybe you\'re holding a lock while making a syscall?)"],
                                                                                            &match ()
                                                                                                 {
                                                                                                 ()
                                                                                                 =>
                                                                                                 [],
                                                                                             }),
                                                             lvl,
                                                             &("rust_0bsd_riscv_kernel::syscall",
                                                               "rust_0bsd_riscv_kernel::syscall",
                                                               "src/syscall.rs",
                                                               94u32));
                                }
                            };
                            loop  { };
                        }
                    }
                    pub mod drivers {
                        pub mod uart {
                            pub mod ns16550a {
                                #![allow(non_camel_case_types)]
                                use volatile_register::RW;
                                #[repr(u8)]
                                enum Ns16550aInterruptEnableRegister {
                                    ReadAvailable = 1 << 0,
                                    WriteAvailable = 1 << 1,
                                    LSRChange = 1 << 2,
                                    MSRChange = 1 << 3,
                                    Sleep = 1 << 4,
                                    LowPower = 1 << 5,
                                }
                                #[repr(u8)]
                                enum Ns16550aInterruptIdentificationRegister {
                                    InterruptPending = 1 << 0,
                                    XORMask_ReadAvailable = 0b0000,
                                    XORMask_WriteAvailable = 0b0010,
                                    XORMask_LSRChange = 0b0100,
                                    XORMask_MSRChange = 0b0110,
                                }
                                #[repr(C)]
                                struct Ns16550aRegisters {
                                    byte_io: RW<u8>,
                                    interrupt_enable: RW<u8>,
                                    fifo_interrupt: RW<u8>,
                                    line_control: RW<u8>,
                                    modem_control: RW<u8>,
                                    line_status: RW<u8>,
                                    modem_status: RW<u8>,
                                    scratch: RW<u8>,
                                }
                                pub struct Ns16550a {
                                    registers: *mut Ns16550aRegisters,
                                }
                                use core::fmt;
                                use crate::trap::in_interrupt_context;
                                impl fmt::Write for Ns16550a {
                                    fn write_str(&mut self, s: &str)
                                     -> fmt::Result {
                                        for byte in s.as_bytes() {
                                            self.put(*byte)
                                        }
                                        Ok(())
                                    }
                                }
                                impl Ns16550a {
                                    pub unsafe fn new(address: usize)
                                     -> Self {
                                        Self{registers:
                                                 address as
                                                     *mut Ns16550aRegisters,}
                                    }
                                    pub fn setup(&mut self) {
                                        unsafe {
                                            (*self.registers).interrupt_enable.write((*self.registers).interrupt_enable.read()
                                                                                         |
                                                                                         Ns16550aInterruptEnableRegister::ReadAvailable
                                                                                             as
                                                                                             u8)
                                        }
                                    }
                                    #[inline(always)]
                                    pub fn put(&mut self, value: u8) {
                                        unsafe {
                                            (*self.registers).byte_io.write(value)
                                        };
                                    }
                                    pub fn get(&mut self) -> Option<u8> {
                                        let dr_bit =
                                            unsafe {
                                                (*self.registers).line_status.read()
                                            } & 1;
                                        if dr_bit == 0 {
                                            None
                                        } else {
                                            Some(unsafe {
                                                     (*self.registers).byte_io.read()
                                                 })
                                        }
                                    }
                                }
                            }
                            pub use ns16550a::Ns16550a as Uart;
                        }
                        pub mod virtio {
                            pub mod block {
                                use core::pin::Pin;
                                use core::task::{Context, Waker};
                                use core::future::Future;
                                use crate::lock::shared::Mutex;
                                use crate::{interrupt_context_waker::InterruptContextWaker};
                                use alloc::{sync::{Arc, Weak}, task::Wake,
                                            boxed::Box, vec::Vec,
                                            collections::{BTreeMap}};
                                use super::{SplitVirtqueue, VirtioDevice,
                                            VirtioDeviceType};
                                #[repr(C)]
                                pub struct RequestHeader {
                                    r#type: u32,
                                    reserved: u32,
                                    sector: u64,
                                }
                                #[automatically_derived]
                                #[allow(unused_qualifications)]
                                impl ::core::marker::Copy for RequestHeader {
                                }
                                #[automatically_derived]
                                #[allow(unused_qualifications)]
                                impl ::core::clone::Clone for RequestHeader {
                                    #[inline]
                                    fn clone(&self) -> RequestHeader {
                                        {
                                            let _:
                                                    ::core::clone::AssertParamIsClone<u32>;
                                            let _:
                                                    ::core::clone::AssertParamIsClone<u32>;
                                            let _:
                                                    ::core::clone::AssertParamIsClone<u64>;
                                            *self
                                        }
                                    }
                                }
                                #[automatically_derived]
                                #[allow(unused_qualifications)]
                                impl ::core::fmt::Debug for RequestHeader {
                                    fn fmt(&self,
                                           f: &mut ::core::fmt::Formatter)
                                     -> ::core::fmt::Result {
                                        match *self {
                                            RequestHeader {
                                            r#type: ref __self_0_0,
                                            reserved: ref __self_0_1,
                                            sector: ref __self_0_2 } => {
                                                let debug_trait_builder =
                                                    &mut ::core::fmt::Formatter::debug_struct(f,
                                                                                              "RequestHeader");
                                                let _ =
                                                    ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                                    "type",
                                                                                    &&(*__self_0_0));
                                                let _ =
                                                    ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                                    "reserved",
                                                                                    &&(*__self_0_1));
                                                let _ =
                                                    ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                                    "sector",
                                                                                    &&(*__self_0_2));
                                                ::core::fmt::DebugStruct::finish(debug_trait_builder)
                                            }
                                        }
                                    }
                                }
                                impl ::core::marker::StructuralPartialEq for
                                 RequestHeader {
                                }
                                #[automatically_derived]
                                #[allow(unused_qualifications)]
                                impl ::core::cmp::PartialEq for RequestHeader
                                 {
                                    #[inline]
                                    fn eq(&self, other: &RequestHeader)
                                     -> bool {
                                        match *other {
                                            RequestHeader {
                                            r#type: ref __self_1_0,
                                            reserved: ref __self_1_1,
                                            sector: ref __self_1_2 } =>
                                            match *self {
                                                RequestHeader {
                                                r#type: ref __self_0_0,
                                                reserved: ref __self_0_1,
                                                sector: ref __self_0_2 } =>
                                                (*__self_0_0) == (*__self_1_0)
                                                    &&
                                                    (*__self_0_1) ==
                                                        (*__self_1_1) &&
                                                    (*__self_0_2) ==
                                                        (*__self_1_2),
                                            },
                                        }
                                    }
                                    #[inline]
                                    fn ne(&self, other: &RequestHeader)
                                     -> bool {
                                        match *other {
                                            RequestHeader {
                                            r#type: ref __self_1_0,
                                            reserved: ref __self_1_1,
                                            sector: ref __self_1_2 } =>
                                            match *self {
                                                RequestHeader {
                                                r#type: ref __self_0_0,
                                                reserved: ref __self_0_1,
                                                sector: ref __self_0_2 } =>
                                                (*__self_0_0) != (*__self_1_0)
                                                    ||
                                                    (*__self_0_1) !=
                                                        (*__self_1_1) ||
                                                    (*__self_0_2) !=
                                                        (*__self_1_2),
                                            },
                                        }
                                    }
                                }
                                impl ::core::marker::StructuralEq for
                                 RequestHeader {
                                }
                                #[automatically_derived]
                                #[allow(unused_qualifications)]
                                impl ::core::cmp::Eq for RequestHeader {
                                    #[inline]
                                    #[doc(hidden)]
                                    #[no_coverage]
                                    fn assert_receiver_is_total_eq(&self)
                                     -> () {
                                        {
                                            let _:
                                                    ::core::cmp::AssertParamIsEq<u32>;
                                            let _:
                                                    ::core::cmp::AssertParamIsEq<u32>;
                                            let _:
                                                    ::core::cmp::AssertParamIsEq<u64>;
                                        }
                                    }
                                }
                                #[automatically_derived]
                                #[allow(unused_qualifications)]
                                impl ::core::cmp::PartialOrd for RequestHeader
                                 {
                                    #[inline]
                                    fn partial_cmp(&self,
                                                   other: &RequestHeader)
                                     ->
                                         ::core::option::Option<::core::cmp::Ordering> {
                                        match *other {
                                            RequestHeader {
                                            r#type: ref __self_1_0,
                                            reserved: ref __self_1_1,
                                            sector: ref __self_1_2 } =>
                                            match *self {
                                                RequestHeader {
                                                r#type: ref __self_0_0,
                                                reserved: ref __self_0_1,
                                                sector: ref __self_0_2 } =>
                                                match ::core::cmp::PartialOrd::partial_cmp(&(*__self_0_0),
                                                                                           &(*__self_1_0))
                                                    {
                                                    ::core::option::Option::Some(::core::cmp::Ordering::Equal)
                                                    =>
                                                    match ::core::cmp::PartialOrd::partial_cmp(&(*__self_0_1),
                                                                                               &(*__self_1_1))
                                                        {
                                                        ::core::option::Option::Some(::core::cmp::Ordering::Equal)
                                                        =>
                                                        match ::core::cmp::PartialOrd::partial_cmp(&(*__self_0_2),
                                                                                                   &(*__self_1_2))
                                                            {
                                                            ::core::option::Option::Some(::core::cmp::Ordering::Equal)
                                                            =>
                                                            ::core::option::Option::Some(::core::cmp::Ordering::Equal),
                                                            cmp => cmp,
                                                        },
                                                        cmp => cmp,
                                                    },
                                                    cmp => cmp,
                                                },
                                            },
                                        }
                                    }
                                }
                                #[automatically_derived]
                                #[allow(unused_qualifications)]
                                impl ::core::cmp::Ord for RequestHeader {
                                    #[inline]
                                    fn cmp(&self, other: &RequestHeader)
                                     -> ::core::cmp::Ordering {
                                        match *other {
                                            RequestHeader {
                                            r#type: ref __self_1_0,
                                            reserved: ref __self_1_1,
                                            sector: ref __self_1_2 } =>
                                            match *self {
                                                RequestHeader {
                                                r#type: ref __self_0_0,
                                                reserved: ref __self_0_1,
                                                sector: ref __self_0_2 } =>
                                                match ::core::cmp::Ord::cmp(&(*__self_0_0),
                                                                            &(*__self_1_0))
                                                    {
                                                    ::core::cmp::Ordering::Equal
                                                    =>
                                                    match ::core::cmp::Ord::cmp(&(*__self_0_1),
                                                                                &(*__self_1_1))
                                                        {
                                                        ::core::cmp::Ordering::Equal
                                                        =>
                                                        match ::core::cmp::Ord::cmp(&(*__self_0_2),
                                                                                    &(*__self_1_2))
                                                            {
                                                            ::core::cmp::Ordering::Equal
                                                            =>
                                                            ::core::cmp::Ordering::Equal,
                                                            cmp => cmp,
                                                        },
                                                        cmp => cmp,
                                                    },
                                                    cmp => cmp,
                                                },
                                            },
                                        }
                                    }
                                }
                                pub struct VirtioBlockDevice {
                                    request_virtqueue: Mutex<SplitVirtqueue>,
                                    device: Arc<Mutex<VirtioDevice>>,
                                    /// A weak pointer to itself. This has to be used when callbacks need to use self later on (when the &mut self has expired) 
                                    pub this: Weak<Mutex<Self>>,
                                    waiting_requests: BTreeMap<u16,
                                                               Vec<Waker>>,
                                    header_buffers: BTreeMap<u16, Box<[u8]>>,
                                }
                                #[automatically_derived]
                                #[allow(unused_qualifications)]
                                impl ::core::fmt::Debug for VirtioBlockDevice
                                 {
                                    fn fmt(&self,
                                           f: &mut ::core::fmt::Formatter)
                                     -> ::core::fmt::Result {
                                        match *self {
                                            VirtioBlockDevice {
                                            request_virtqueue: ref __self_0_0,
                                            device: ref __self_0_1,
                                            this: ref __self_0_2,
                                            waiting_requests: ref __self_0_3,
                                            header_buffers: ref __self_0_4 }
                                            => {
                                                let debug_trait_builder =
                                                    &mut ::core::fmt::Formatter::debug_struct(f,
                                                                                              "VirtioBlockDevice");
                                                let _ =
                                                    ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                                    "request_virtqueue",
                                                                                    &&(*__self_0_0));
                                                let _ =
                                                    ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                                    "device",
                                                                                    &&(*__self_0_1));
                                                let _ =
                                                    ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                                    "this",
                                                                                    &&(*__self_0_2));
                                                let _ =
                                                    ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                                    "waiting_requests",
                                                                                    &&(*__self_0_3));
                                                let _ =
                                                    ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                                    "header_buffers",
                                                                                    &&(*__self_0_4));
                                                ::core::fmt::DebugStruct::finish(debug_trait_builder)
                                            }
                                        }
                                    }
                                }
                                pub struct BlockRequestFuture {
                                    device: Weak<Mutex<VirtioBlockDevice>>,
                                    header: RequestHeader,
                                    pub buffer: Option<Box<[u8]>>,
                                    pub descriptor_id: Option<u16>,
                                    pub was_queued: bool,
                                }
                                use core::task::Poll;
                                impl Future for BlockRequestFuture {
                                    type Output = Option<Box<[u8]>>;
                                    fn poll(mut self: Pin<&mut Self>,
                                            cx: &mut Context<'_>)
                                     -> Poll<<Self as Future>::Output> {
                                        let mut device =
                                            self.device.upgrade().unwrap();
                                        let mut device = device.lock();
                                        if self.buffer.is_none() {
                                            if let Some(buffer) =
                                                   device.take_buffer(&self.descriptor_id.unwrap())
                                               {
                                                self.buffer = Some(buffer);
                                                Poll::Ready(self.buffer.take())
                                            } else {
                                                device.register_waker(&self.descriptor_id.unwrap(),
                                                                      cx.waker().clone());
                                                Poll::Pending
                                            }
                                        } else if self.was_queued {
                                            Poll::Ready(self.buffer.take())
                                        } else {
                                            self.was_queued = true;
                                            self.descriptor_id =
                                                Some(device.do_request(&mut self));
                                            device.register_waker(&self.descriptor_id.unwrap(),
                                                                  cx.waker().clone());
                                            drop(device);
                                            self.device.upgrade().unwrap().lock().begin_request(&self.descriptor_id.unwrap());
                                            {

                                                #[allow(unused_unsafe)]
                                                {
                                                    use core::fmt::Write;
                                                    let l =
                                                        crate::std_macros::OUTPUT_LOCK.lock();
                                                    let _ =
                                                        unsafe {
                                                            crate::drivers::uart::Uart::new(0x1000_0000)
                                                        }.write_fmt(::core::fmt::Arguments::new_v1(&["",
                                                                                                     "\r\n"],
                                                                                                   &match (&self.buffer,)
                                                                                                        {
                                                                                                        (arg0,)
                                                                                                        =>
                                                                                                        [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                      ::core::fmt::Debug::fmt)],
                                                                                                    }));
                                                }
                                            };
                                            Poll::Pending
                                        }
                                    }
                                }
                                use crate::drivers::traits::block::{BlockDevice,
                                                                    AnyRequestFuture};
                                impl BlockDevice for VirtioBlockDevice {
                                    fn _create_request(&self, sector: u64,
                                                       buffer: Box<[u8]>,
                                                       write: bool)
                                     ->
                                         Box<dyn AnyRequestFuture + Unpin +
                                             Send + Sync> {
                                        Box::new(BlockRequestFuture{device:
                                                                        self.this.clone(),
                                                                    header:
                                                                        RequestHeader{r#type:
                                                                                          if write
                                                                                             {
                                                                                              1
                                                                                          } else {
                                                                                              0
                                                                                          },
                                                                                      reserved:
                                                                                          0,
                                                                                      sector,},
                                                                    buffer:
                                                                        Some(buffer),
                                                                    descriptor_id:
                                                                        None,
                                                                    was_queued:
                                                                        false,})
                                    }
                                }
                                impl VirtioBlockDevice {
                                    fn instance_configure(&self) {
                                        self.device.lock().driver_ok();
                                    }
                                    pub fn do_request(&mut self,
                                                      future:
                                                          &mut BlockRequestFuture)
                                     -> u16 {
                                        let mut vq_lock =
                                            self.request_virtqueue.lock();
                                        let status =
                                            ::alloc::vec::from_elem(0xFFu8,
                                                                    1).into_boxed_slice();
                                        let mut last =
                                            vq_lock.new_descriptor_from_boxed_slice(status,
                                                                                    true,
                                                                                    None);
                                        if future.header.r#type == 1 {
                                            last =
                                                vq_lock.new_descriptor_from_boxed_slice(future.buffer.take().unwrap(),
                                                                                        false,
                                                                                        Some(last));
                                        } else {
                                            last =
                                                vq_lock.new_descriptor_from_boxed_slice(future.buffer.take().unwrap(),
                                                                                        true,
                                                                                        Some(last));
                                        }
                                        last =
                                            vq_lock.new_descriptor_from_sized(&future.header,
                                                                              false,
                                                                              Some(last));
                                        last
                                    }
                                    pub fn begin_request(&mut self,
                                                         descriptor_id:
                                                             &u16) {
                                        let mut vq_lock =
                                            self.request_virtqueue.lock();
                                        vq_lock.make_available(*descriptor_id);
                                        self.device.lock().queue_ready(0);
                                    }
                                    /// Sets up a callback future for when the device has finished processing a request we made
                                    fn poll_device(&mut self) {
                                        let mut device_ref =
                                            self.device.lock();
                                        let this_weak = self.this.clone();
                                        let result =
                                            Pin::new(&mut *device_ref).poll(&mut Context::from_waker(&Arc::new(InterruptContextWaker(Box::new(move
                                                                                                                                                  ||
                                                                                                                                                  {
                                                                                                                                                      let this =
                                                                                                                                                          this_weak.upgrade().unwrap().lock().poll_device();
                                                                                                                                                  }))).into()));
                                        drop(device_ref);
                                        if let Poll::Ready(queue_idx) = result
                                           {
                                            if !(queue_idx == 0) {
                                                ::core::panicking::panic("assertion failed: queue_idx == 0")
                                            };
                                            let mut vq_lock =
                                                self.request_virtqueue.lock();
                                            let mut descriptor_chain_data_iterator =
                                                vq_lock.pop_used_element_to_iterator();
                                            let descriptor_id =
                                                descriptor_chain_data_iterator.pointed_chain.unwrap();
                                            let data: Vec<u8> =
                                                descriptor_chain_data_iterator.flatten().copied().collect();
                                            let request_body =
                                                &data[core::mem::size_of::<RequestHeader>()..data.len()
                                                                                                 -
                                                                                                 1];
                                            let buffer_start_ptr =
                                                descriptor_chain_data_iterator.nth(1).unwrap().as_ptr()
                                                    as *mut u8;
                                            let buffer_len =
                                                request_body.len();
                                            let buffer_box =
                                                unsafe {
                                                    Box::from_raw(core::slice::from_raw_parts_mut(buffer_start_ptr,
                                                                                                  buffer_len))
                                                };
                                            self.header_buffers.insert(descriptor_id,
                                                                       buffer_box);
                                            let items =
                                                self.waiting_requests.get_mut(&descriptor_id).map(|vec|
                                                                                                      vec.iter_mut());
                                            if items.is_none() {
                                                {
                                                    let lvl =
                                                        ::log::Level::Info;
                                                    if lvl <=
                                                           ::log::STATIC_MAX_LEVEL
                                                           &&
                                                           lvl <=
                                                               ::log::max_level()
                                                       {
                                                        ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["No one was waiting for this!"],
                                                                                                                &match ()
                                                                                                                     {
                                                                                                                     ()
                                                                                                                     =>
                                                                                                                     [],
                                                                                                                 }),
                                                                                 lvl,
                                                                                 &("rust_0bsd_riscv_kernel::drivers::virtio::block",
                                                                                   "rust_0bsd_riscv_kernel::drivers::virtio::block",
                                                                                   "src/drivers/virtio/block.rs",
                                                                                   195u32));
                                                    }
                                                };
                                                return;
                                            }
                                            let items = items.unwrap();
                                            for i in items.into_iter() {
                                                i.wake_by_ref();
                                            }
                                            self.waiting_requests.remove(&descriptor_id);
                                        } else { }
                                    }
                                    /// Returns None if buffer doesn't exist (which meanst that the request was never done OR that it has completed)
                                    pub fn take_buffer(&mut self,
                                                       descriptor_id: &u16)
                                     -> Option<Box<[u8]>> {
                                        self.header_buffers.remove(descriptor_id)
                                    }
                                    pub fn register_waker(&mut self,
                                                          descriptor_id: &u16,
                                                          waker: Waker) {
                                        if let Some(v) =
                                               self.waiting_requests.get_mut(descriptor_id)
                                           {
                                            v.push(waker)
                                        } else {
                                            self.waiting_requests.insert(*descriptor_id,
                                                                         <[_]>::into_vec(box
                                                                                             [waker]));
                                        }
                                    }
                                }
                                impl VirtioDeviceType for VirtioBlockDevice {
                                    fn configure(device:
                                                     Arc<Mutex<VirtioDevice>>)
                                     -> Result<Arc<Mutex<Self>>, ()> {
                                        let q =
                                            device.lock().configure_queue(0);
                                        let dev =
                                            VirtioBlockDevice{request_virtqueue:
                                                                  Mutex::new(q),
                                                              device,
                                                              this:
                                                                  Weak::new(),
                                                              waiting_requests:
                                                                  BTreeMap::new(),
                                                              header_buffers:
                                                                  BTreeMap::new(),};
                                        let dev = Arc::new(Mutex::new(dev));
                                        dev.lock().this =
                                            Arc::downgrade(&dev);
                                        dev.lock().poll_device();
                                        let dev_clone = dev.clone();
                                        Arc::new(InterruptContextWaker(Box::new(move
                                                                                    ||
                                                                                    {
                                                                                        dev_clone.lock().instance_configure();
                                                                                    }))).wake();
                                        Ok(dev)
                                    }
                                    fn negotiate_features(device:
                                                              &mut VirtioDevice) {
                                        device.get_device_features();
                                        device.set_driver_features(0);
                                        device.accept_features().unwrap();
                                    }
                                    fn on_interrupt(&self) { }
                                }
                            }
                            use core::{slice, alloc::Layout, future::Future,
                                       convert::TryInto, task::Waker};
                            use alloc::{collections::BTreeMap, vec::Vec,
                                        boxed::Box, sync::Arc};
                            use itertools::Itertools;
                            use volatile_register::{RW, RO, WO};
                            use crate::lock::shared::Mutex;
                            use crate::paging::PAGE_ALIGN;
                            use self::block::VirtioBlockDevice;
                            pub enum StatusField {
                                Acknowledge = 1,
                                Driver = 2,
                                Failed = 128,
                                FeaturesOk = 8,
                                DriverOk = 4,
                                DeviceNeedsReset = 64,
                            }
                            #[repr(C)]
                            pub struct VirtqueueDescriptor {
                                /// Physical address
                                address: u64,
                                length: u32,
                                flags: u16,
                                next: u16,
                            }
                            #[automatically_derived]
                            #[allow(unused_qualifications)]
                            impl ::core::fmt::Debug for VirtqueueDescriptor {
                                fn fmt(&self, f: &mut ::core::fmt::Formatter)
                                 -> ::core::fmt::Result {
                                    match *self {
                                        VirtqueueDescriptor {
                                        address: ref __self_0_0,
                                        length: ref __self_0_1,
                                        flags: ref __self_0_2,
                                        next: ref __self_0_3 } => {
                                            let debug_trait_builder =
                                                &mut ::core::fmt::Formatter::debug_struct(f,
                                                                                          "VirtqueueDescriptor");
                                            let _ =
                                                ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                                "address",
                                                                                &&(*__self_0_0));
                                            let _ =
                                                ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                                "length",
                                                                                &&(*__self_0_1));
                                            let _ =
                                                ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                                "flags",
                                                                                &&(*__self_0_2));
                                            let _ =
                                                ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                                "next",
                                                                                &&(*__self_0_3));
                                            ::core::fmt::DebugStruct::finish(debug_trait_builder)
                                        }
                                    }
                                }
                            }
                            /// Legacy layout
                            #[repr(C)]
                            pub struct VirtioMmio {
                                magic_value: RO<u32>,
                                version: RO<u32>,
                                device_id: RO<u32>,
                                vendor_id: RO<u32>,
                                host_features: RO<u32>,
                                host_features_sel: WO<u32>,
                                _pad1: u32,
                                _pad2: u32,
                                guest_features: WO<u32>,
                                guest_features_sel: WO<u32>,
                                guest_page_size: WO<u32>,
                                _pad3: u32,
                                queue_sel: WO<u32>,
                                queue_num_max: RO<u32>,
                                queue_num: WO<u32>,
                                queue_align: WO<u32>,
                                queue_pfn: RW<u32>,
                                _pad4: u32,
                                _pad5: u32,
                                _pad6: u32,
                                queue_notify: WO<u32>,
                                _pad7: [u8; 12],
                                interrupt_status: RO<u32>,
                                interrupt_ack: WO<u32>,
                                _pad8: [u8; 8],
                                status: RW<u32>,
                                _pad9: [u8; (0x100 - 0x70)],
                                config: RW<u32>,
                            }
                            pub struct VirtioDevice {
                                configuration: *mut VirtioMmio,
                                queue_used_sizes_align: BTreeMap<u16,
                                                                 (u16, u16,
                                                                  u32)>,
                                waiting_wakers: Vec<Waker>,
                                changed_queue: Option<u16>,
                            }
                            #[automatically_derived]
                            #[allow(unused_qualifications)]
                            impl ::core::fmt::Debug for VirtioDevice {
                                fn fmt(&self, f: &mut ::core::fmt::Formatter)
                                 -> ::core::fmt::Result {
                                    match *self {
                                        VirtioDevice {
                                        configuration: ref __self_0_0,
                                        queue_used_sizes_align: ref __self_0_1,
                                        waiting_wakers: ref __self_0_2,
                                        changed_queue: ref __self_0_3 } => {
                                            let debug_trait_builder =
                                                &mut ::core::fmt::Formatter::debug_struct(f,
                                                                                          "VirtioDevice");
                                            let _ =
                                                ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                                "configuration",
                                                                                &&(*__self_0_0));
                                            let _ =
                                                ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                                "queue_used_sizes_align",
                                                                                &&(*__self_0_1));
                                            let _ =
                                                ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                                "waiting_wakers",
                                                                                &&(*__self_0_2));
                                            let _ =
                                                ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                                "changed_queue",
                                                                                &&(*__self_0_3));
                                            ::core::fmt::DebugStruct::finish(debug_trait_builder)
                                        }
                                    }
                                }
                            }
                            unsafe impl Send for VirtioDevice { }
                            /// A useful handle over a dynamically-sized pointer
                            pub struct SplitVirtqueue {
                                /// This pointer was allocated with Box::leak() and will then be reconstructed Box:from_raw before dropping
                                /// The layout of the data pointed to by this pointer is:
                                /// Virtqueue Part      Alignment   Size
                                /// Descriptor Table    16          16∗(Queue Size)
                                /// Available Ring      2           6 + 2∗(Queue Size)
                                /// Used Ring           4           6 + 8∗(Queue Size) 
                                pointer: *mut u8,
                                size: u16,
                                first_free_descriptor: u16,
                                guest_used_ring_idx: u16,
                            }
                            #[automatically_derived]
                            #[allow(unused_qualifications)]
                            impl ::core::fmt::Debug for SplitVirtqueue {
                                fn fmt(&self, f: &mut ::core::fmt::Formatter)
                                 -> ::core::fmt::Result {
                                    match *self {
                                        SplitVirtqueue {
                                        pointer: ref __self_0_0,
                                        size: ref __self_0_1,
                                        first_free_descriptor: ref __self_0_2,
                                        guest_used_ring_idx: ref __self_0_3 }
                                        => {
                                            let debug_trait_builder =
                                                &mut ::core::fmt::Formatter::debug_struct(f,
                                                                                          "SplitVirtqueue");
                                            let _ =
                                                ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                                "pointer",
                                                                                &&(*__self_0_0));
                                            let _ =
                                                ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                                "size",
                                                                                &&(*__self_0_1));
                                            let _ =
                                                ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                                "first_free_descriptor",
                                                                                &&(*__self_0_2));
                                            let _ =
                                                ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                                "guest_used_ring_idx",
                                                                                &&(*__self_0_3));
                                            ::core::fmt::DebugStruct::finish(debug_trait_builder)
                                        }
                                    }
                                }
                            }
                            unsafe impl Send for SplitVirtqueue { }
                            /// This struct iterates over each
                            /// of the descriptor data
                            pub struct SplitVirtqueueDescriptorChainIterator<'a> {
                                queue: &'a SplitVirtqueue,
                                pointed_chain: Option<u16>,
                            }
                            #[automatically_derived]
                            #[allow(unused_qualifications)]
                            impl <'a> ::core::marker::Copy for
                             SplitVirtqueueDescriptorChainIterator<'a> {
                            }
                            #[automatically_derived]
                            #[allow(unused_qualifications)]
                            impl <'a> ::core::clone::Clone for
                             SplitVirtqueueDescriptorChainIterator<'a> {
                                #[inline]
                                fn clone(&self)
                                 ->
                                     SplitVirtqueueDescriptorChainIterator<'a> {
                                    {
                                        let _:
                                                ::core::clone::AssertParamIsClone<&'a SplitVirtqueue>;
                                        let _:
                                                ::core::clone::AssertParamIsClone<Option<u16>>;
                                        *self
                                    }
                                }
                            }
                            impl <'a>
                             SplitVirtqueueDescriptorChainIterator<'a> {
                                fn fold_to_vec(&mut self) -> Vec<u8> {
                                    self.map(|s|
                                                 {
                                                     let mut v = Vec::new();
                                                     v.extend_from_slice(s);
                                                     v
                                                 }).concat()
                                }
                            }
                            impl <'a> Iterator for
                             SplitVirtqueueDescriptorChainIterator<'a> {
                                type Item = &'a [u8];
                                fn next(&mut self) -> Option<Self::Item> {
                                    if self.pointed_chain == None {
                                        return None
                                    }
                                    let descriptor =
                                        self.queue.get_descriptor(self.pointed_chain.unwrap());
                                    if descriptor.address == 0 { return None }
                                    if descriptor.flags & 1 == 0 {
                                        self.pointed_chain = None
                                    } else {
                                        self.pointed_chain =
                                            Some(descriptor.next)
                                    }
                                    let s =
                                        unsafe {
                                            slice::from_raw_parts_mut(descriptor.address
                                                                          as
                                                                          *mut u8,
                                                                      descriptor.length
                                                                          as
                                                                          usize)
                                        };
                                    Some(s)
                                }
                            }
                            #[repr(C)]
                            pub struct SplitVirtqueueUsedRing {
                                idx: u32,
                                len: u32,
                            }
                            impl SplitVirtqueue {
                                #[inline]
                                fn descriptor_table_size(size: &u16)
                                 -> usize {
                                    16 * *size as usize
                                }
                                #[inline]
                                fn available_ring_size(size: &u16) -> usize {
                                    6 + 2 * *size as usize
                                }
                                #[inline]
                                fn used_ring_size(size: &u16) -> usize {
                                    6 + 8 * *size as usize
                                }
                                #[inline]
                                fn align(value: usize) -> usize {
                                    if ((&value) & 0xFFF) == 0 {
                                        value
                                    } else { (value & (!0xFFF)) + 0x1000 }
                                }
                                #[inline]
                                fn descriptor_table_offset(size: &u16)
                                 -> usize {
                                    0usize
                                }
                                #[inline]
                                fn available_ring_offset(size: &u16)
                                 -> usize {
                                    Self::descriptor_table_size(size)
                                }
                                #[inline]
                                fn used_ring_offset(size: &u16) -> usize {
                                    Self::align(Self::descriptor_table_size(&size)
                                                    +
                                                    Self::available_ring_size(&size))
                                }
                                #[inline]
                                fn memory_size(size: &u16) -> usize {
                                    Self::used_ring_offset(&size) +
                                        Self::align(Self::used_ring_size(&size))
                                }
                                fn new(size: u16) -> SplitVirtqueue {
                                    use crate::allocator::ALLOCATOR;
                                    use core::alloc::GlobalAlloc;
                                    let mem_size = Self::memory_size(&size);
                                    let pointer =
                                        unsafe {
                                            ALLOCATOR.alloc(Layout::from_size_align(mem_size,
                                                                                    4096).unwrap())
                                        };
                                    unsafe {
                                        slice::from_raw_parts_mut(pointer,
                                                                  mem_size).fill(0)
                                    }
                                    SplitVirtqueue{pointer: pointer as _,
                                                   size,
                                                   first_free_descriptor: 0,
                                                   guest_used_ring_idx: 0,}
                                }
                                fn get_descriptor(&self, index: u16)
                                 -> &VirtqueueDescriptor {
                                    if index > self.size {
                                        ::core::panicking::panic("Out of range!")
                                    }
                                    unsafe {
                                        (self.pointer.add(Self::descriptor_table_offset(&self.size))
                                             as
                                             *const VirtqueueDescriptor).add(index
                                                                                 as
                                                                                 usize).as_ref().unwrap()
                                    }
                                }
                                fn get_descriptor_mut(&mut self, index: u16)
                                 -> &mut VirtqueueDescriptor {
                                    if index > self.size {
                                        ::core::panicking::panic("Out of range!")
                                    }
                                    unsafe {
                                        (self.pointer.add(Self::descriptor_table_offset(&self.size))
                                             as
                                             *mut VirtqueueDescriptor).add(index
                                                                               as
                                                                               usize).as_mut().unwrap()
                                    }
                                }
                                fn allocate_descriptor(&self) -> u16 {
                                    for i in 0..self.size {
                                        if self.get_descriptor(i).address == 0
                                           {
                                            return i
                                        }
                                    }
                                    ::core::panicking::panic("No descriptor found!");
                                }
                                pub unsafe fn new_descriptor_from_address(&mut self,
                                                                          address:
                                                                              *const (),
                                                                          size:
                                                                              usize,
                                                                          device_writable:
                                                                              bool,
                                                                          chain:
                                                                              Option<u16>)
                                 -> u16 {
                                    let descriptor_index =
                                        self.allocate_descriptor();
                                    *self.get_descriptor_mut(descriptor_index)
                                        =
                                        VirtqueueDescriptor{address:
                                                                address as
                                                                    u64,
                                                            length:
                                                                size as u32,
                                                            flags:
                                                                if chain !=
                                                                       None {
                                                                    1
                                                                } else { 0 } |
                                                                    if device_writable
                                                                       {
                                                                        2
                                                                    } else {
                                                                        0
                                                                    },
                                                            next:
                                                                chain.unwrap_or(0),};
                                    descriptor_index
                                }
                                pub fn new_descriptor_from_static_buffer(&mut self,
                                                                         buffer:
                                                                             &'static [u8],
                                                                         device_writable:
                                                                             bool,
                                                                         chain:
                                                                             Option<u16>)
                                 -> u16 {
                                    unsafe {
                                        self.new_descriptor_from_address(buffer.as_ptr()
                                                                             as
                                                                             _,
                                                                         buffer.len(),
                                                                         device_writable,
                                                                         chain)
                                    }
                                }
                                pub fn new_descriptor_from_static_buffer_mut(&mut self,
                                                                             buffer:
                                                                                 &'static mut [u8],
                                                                             chain:
                                                                                 Option<u16>)
                                 -> u16 {
                                    unsafe {
                                        self.new_descriptor_from_address(buffer.as_ptr()
                                                                             as
                                                                             _,
                                                                         buffer.len(),
                                                                         false,
                                                                         chain)
                                    }
                                }
                                pub fn new_descriptor_from_sized<T: Sized>(&mut self,
                                                                           buffer:
                                                                               &T,
                                                                           device_writable:
                                                                               bool,
                                                                           chain:
                                                                               Option<u16>)
                                 -> u16 {
                                    unsafe {
                                        self.new_descriptor_from_address(buffer
                                                                             as
                                                                             *const T
                                                                             as
                                                                             _,
                                                                         core::mem::size_of_val(buffer),
                                                                         device_writable,
                                                                         chain)
                                    }
                                }
                                /// Note that this leaks "buffer"
                                /// Whoever is using this needs to make sure to run Box::from_raw on the Buffer when needed
                                pub fn new_descriptor_from_boxed_slice(&mut self,
                                                                       buffer:
                                                                           Box<[u8]>,
                                                                       device_writable:
                                                                           bool,
                                                                       chain:
                                                                           Option<u16>)
                                 -> u16 {
                                    let len = buffer.len();
                                    unsafe {
                                        self.new_descriptor_from_address(Box::into_raw(buffer)
                                                                             as
                                                                             _,
                                                                         len,
                                                                         device_writable,
                                                                         chain)
                                    }
                                }
                                /// Increments the index field in the available ring
                                /// and returns the old value
                                pub fn add_available_ring_idx(&mut self)
                                 -> u16 {
                                    unsafe {
                                        let ring_ptr =
                                            (self.pointer.add(Self::available_ring_offset(&self.size))
                                                 as *mut u16).add(1);
                                        let old = *ring_ptr;
                                        *ring_ptr += 1;
                                        if *ring_ptr == self.size {
                                            {
                                                let lvl = ::log::Level::Warn;
                                                if lvl <=
                                                       ::log::STATIC_MAX_LEVEL
                                                       &&
                                                       lvl <=
                                                           ::log::max_level()
                                                   {
                                                    ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["Overflow in available queue"],
                                                                                                            &match ()
                                                                                                                 {
                                                                                                                 ()
                                                                                                                 =>
                                                                                                                 [],
                                                                                                             }),
                                                                             lvl,
                                                                             &("rust_0bsd_riscv_kernel::drivers::virtio",
                                                                               "rust_0bsd_riscv_kernel::drivers::virtio",
                                                                               "src/drivers/virtio/mod.rs",
                                                                               316u32));
                                                }
                                            };
                                            *ring_ptr = 0;
                                        }
                                        old
                                    }
                                }
                                /// Gets the index field in the available ring
                                pub fn get_available_ring_idx(&self) -> u16 {
                                    unsafe {
                                        let ring_ptr =
                                            (self.pointer.add(Self::available_ring_offset(&self.size))
                                                 as *mut u16).add(1);
                                        *ring_ptr
                                    }
                                }
                                pub fn get_available_ring_ptr(&mut self,
                                                              index: u16)
                                 -> *mut u16 {
                                    unsafe {
                                        use core::ops::Add;
                                        (self.pointer.add(Self::available_ring_offset(&self.size))
                                             as
                                             *mut u16).add(2).add(index as
                                                                      usize)
                                    }
                                }
                                pub fn get_device_used_ring_idx(&mut self)
                                 -> u16 {
                                    unsafe {
                                        use core::ops::Add;
                                        *((self.pointer.add(Self::used_ring_offset(&self.size))
                                               as *mut u16).add(1))
                                    }
                                }
                                pub fn get_used_ring_ptr(&mut self,
                                                         index: u16)
                                 -> *mut SplitVirtqueueUsedRing {
                                    unsafe {
                                        use core::ops::Add;
                                        ((self.pointer.add(Self::used_ring_offset(&self.size))
                                              as *mut u16).add(2) as
                                             *mut SplitVirtqueueUsedRing).add(index
                                                                                  as
                                                                                  usize)
                                    }
                                }
                                /// Adds a descriptor to the available ring
                                /// making it available to the device
                                pub fn make_available(&mut self,
                                                      descriptor: u16) {
                                    let old = self.get_available_ring_idx();
                                    unsafe {
                                        *self.get_available_ring_ptr(old) =
                                            descriptor
                                    }
                                    self.add_available_ring_idx();
                                }
                                pub fn pop_used_element(&mut self)
                                 -> Option<*mut SplitVirtqueueUsedRing> {
                                    if self.guest_used_ring_idx + 1 !=
                                           self.get_device_used_ring_idx() {
                                        return None;
                                    }
                                    let v =
                                        self.get_used_ring_ptr(self.guest_used_ring_idx);
                                    self.guest_used_ring_idx =
                                        self.guest_used_ring_idx.wrapping_add(1);
                                    Some(v)
                                }
                                pub fn pop_used_element_to_iterator<'this>(&'this mut self)
                                 ->
                                     SplitVirtqueueDescriptorChainIterator<'this> {
                                    let u = self.pop_used_element().unwrap();
                                    SplitVirtqueueDescriptorChainIterator{queue:
                                                                              self,
                                                                          pointed_chain:
                                                                              Some(unsafe
                                                                                   {
                                                                                       (*u).idx
                                                                                   }
                                                                                       as
                                                                                       u16),}
                                }
                                /// Returns the "Guest physical page number of the virtual queue"
                                /// this is pointer / PAGE_SIZE in our case
                                fn pfn(&self) -> usize {
                                    (self.pointer as usize) / (PAGE_ALIGN)
                                }
                            }
                            pub enum VirtioDriver {
                                Block(Arc<Mutex<VirtioBlockDevice>>),
                            }
                            use core::task::Poll;
                            impl Future for VirtioDevice {
                                type Output = u16;
                                fn poll(mut self: core::pin::Pin<&mut Self>,
                                        cx: &mut core::task::Context<'_>)
                                 -> Poll<Self::Output> {
                                    if let Some(id) =
                                           self.next_changed_used_ring_queue()
                                       {
                                        Poll::Ready(id)
                                    } else if let Some(id) =
                                                  self.changed_queue {
                                        self.changed_queue = None;
                                        Poll::Ready(id)
                                    } else {
                                        self.waiting_wakers.push(cx.waker().clone());
                                        Poll::Pending
                                    }
                                }
                            }
                            impl Drop for SplitVirtqueue {
                                fn drop(&mut self) {
                                    unsafe {
                                        drop(Box::from_raw(self.pointer))
                                    }
                                }
                            }
                            impl VirtioDevice {
                                pub unsafe fn new(base: *mut VirtioMmio)
                                 -> Self {
                                    Self{configuration: base,
                                         queue_used_sizes_align:
                                             BTreeMap::new(),
                                         changed_queue: None,
                                         waiting_wakers: Vec::new(),}
                                }
                                pub fn configure(&mut self) {
                                    unsafe {
                                        (*self.configuration).status.write(0);
                                        (*self.configuration).status.write((*self.configuration).status.read()
                                                                               |
                                                                               StatusField::Acknowledge
                                                                                   as
                                                                                   u32);
                                        (*self.configuration).status.write((*self.configuration).status.read()
                                                                               |
                                                                               StatusField::Driver
                                                                                   as
                                                                                   u32);
                                    }
                                }
                                pub fn get_virtqueue_address_size(&self,
                                                                  queue: u16)
                                 -> Option<(*const (), u16)> {
                                    let (_, size, align) =
                                        self.queue_used_sizes_align.get(&queue)?;
                                    let address: usize =
                                        unsafe {
                                            (*self.configuration).queue_sel.write(queue.into());
                                            ((*self.configuration).queue_pfn.read())
                                                * align
                                        }.try_into().unwrap();
                                    Some((address as _, *size))
                                }
                                pub fn configure_queue(&mut self, queue: u16)
                                 -> SplitVirtqueue {
                                    let virtq;
                                    unsafe {
                                        (*self.configuration).queue_sel.write(queue.into());
                                        if !((*self.configuration).queue_pfn.read()
                                                 == 0) {
                                            ::core::panicking::panic("assertion failed: (*self.configuration).queue_pfn.read() == 0")
                                        };
                                        let max_virtqueue_size =
                                            (*self.configuration).queue_num_max.read()
                                                as u16;
                                        if !(max_virtqueue_size != 0) {
                                            ::core::panicking::panic("assertion failed: max_virtqueue_size != 0")
                                        };
                                        let virtqueue_size =
                                            max_virtqueue_size;
                                        let align = crate::paging::PAGE_ALIGN;
                                        virtq =
                                            SplitVirtqueue::new(virtqueue_size);
                                        self.queue_used_sizes_align.insert(queue,
                                                                           (0,
                                                                            virtqueue_size,
                                                                            align.try_into().unwrap()));
                                        (*self.configuration).queue_num.write(virtqueue_size
                                                                                  as
                                                                                  u32);
                                        (*self.configuration).queue_align.write(align
                                                                                    as
                                                                                    u32);
                                        (*self.configuration).guest_page_size.write(align
                                                                                        as
                                                                                        u32);
                                        (*self.configuration).queue_pfn.write(virtq.pfn()
                                                                                  as
                                                                                  u32);
                                    }
                                    virtq
                                }
                                /// Sets the features_ok bit and then checks if it's still there
                                /// Returns Ok if it is, otherwise the device doesn't support this subset of features
                                /// and Err is returned
                                pub fn accept_features(&mut self)
                                 -> Result<(), ()> {
                                    unsafe {
                                        (*self.configuration).status.write((*self.configuration).status.read()
                                                                               |
                                                                               (StatusField::FeaturesOk
                                                                                    as
                                                                                    u32));
                                        if ((*self.configuration).status.read()
                                                &
                                                StatusField::FeaturesOk as
                                                    u32) != 0 {
                                            Ok(())
                                        } else { Err(()) }
                                    }
                                }
                                pub fn is_present(&mut self) -> bool {
                                    unsafe {
                                        (*self.configuration).device_id.read()
                                            != 0
                                    }
                                }
                                pub fn queue_ready(&mut self, queue: u16) {
                                    unsafe {
                                        (*self.configuration).queue_notify.write(queue.into());
                                    }
                                }
                                /// moves self into a driver
                                pub fn make_driver(this: Arc<Mutex<Self>>)
                                 -> Option<VirtioDriver> {
                                    let id =
                                        unsafe {
                                            (*this.lock().configuration).device_id.read()
                                        };
                                    match id {
                                        2 => {
                                            VirtioBlockDevice::negotiate_features(&mut this.lock());
                                            let dev =
                                                VirtioBlockDevice::configure(this).unwrap();
                                            Some(VirtioDriver::Block(dev))
                                        }
                                        _ => {
                                            {
                                                let lvl = ::log::Level::Warn;
                                                if lvl <=
                                                       ::log::STATIC_MAX_LEVEL
                                                       &&
                                                       lvl <=
                                                           ::log::max_level()
                                                   {
                                                    ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["Unknown/Unimplemented VirtIO device type: "],
                                                                                                            &match (&unsafe
                                                                                                                     {
                                                                                                                         (*this.lock().configuration).device_id.read()
                                                                                                                     },)
                                                                                                                 {
                                                                                                                 (arg0,)
                                                                                                                 =>
                                                                                                                 [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                               ::core::fmt::Display::fmt)],
                                                                                                             }),
                                                                             lvl,
                                                                             &("rust_0bsd_riscv_kernel::drivers::virtio",
                                                                               "rust_0bsd_riscv_kernel::drivers::virtio",
                                                                               "src/drivers/virtio/mod.rs",
                                                                               516u32));
                                                }
                                            };
                                            None
                                        }
                                    }
                                }
                                pub fn get_device_features(&mut self) -> u32 {
                                    unsafe {
                                        (*self.configuration).host_features.read()
                                    }
                                }
                                pub fn set_driver_features(&mut self,
                                                           features: u32) {
                                    unsafe {
                                        (*self.configuration).guest_features.write(features)
                                    }
                                }
                                pub fn driver_ok(&mut self) {
                                    unsafe {
                                        (*self.configuration).status.write((*self.configuration).status.read()
                                                                               |
                                                                               StatusField::DriverOk
                                                                                   as
                                                                                   u32)
                                    };
                                }
                                /// Should be called on an interrupt. This may wake up some used buffer wakers
                                /// The reason this takes a mutex to self is to allow the waker to lock the VirtioDevice
                                /// without deadlocking
                                pub fn on_interrupt(this: &Mutex<Self>) {
                                    let interrupt_cause =
                                        unsafe {
                                            (*this.lock().configuration).interrupt_status.read()
                                        };
                                    if (interrupt_cause & (1 << 0)) != 0 {
                                        while let Some(queue_id) =
                                                  {
                                                      let b =
                                                          this.lock().next_changed_used_ring_queue();
                                                      b
                                                  } {
                                            let wakers;
                                            {
                                                let mut this = this.lock();
                                                this.changed_queue =
                                                    Some(queue_id);
                                                wakers =
                                                    this.waiting_wakers.clone();
                                            }
                                            for i in wakers {
                                                i.wake_by_ref()
                                            }
                                        }
                                    }
                                    unsafe {
                                        (*this.lock().configuration).interrupt_ack.write(interrupt_cause)
                                    };
                                }
                                /// Gets a virtual queue number whose used ring has changed since the last time it was returned from this function
                                pub fn next_changed_used_ring_queue(&mut self)
                                 -> Option<u16> {
                                    if let Some((index, change_used_index_to))
                                           =
                                           'block:
                                               {
                                                   for (idx,
                                                        (driver_used_ring_index,
                                                         size, align)) in
                                                       self.queue_used_sizes_align.iter()
                                                       {
                                                       let addr =
                                                           self.get_virtqueue_address_size(*idx).unwrap().0;
                                                       let device_used_index =
                                                           unsafe {
                                                               ((addr as
                                                                     *const u8).add(SplitVirtqueue::used_ring_offset(size))
                                                                    as
                                                                    *mut u16).add(1).read()
                                                           };
                                                       if device_used_index !=
                                                              *driver_used_ring_index
                                                          {
                                                           break 'block
                                                               Some((*idx,
                                                                     device_used_index))

                                                       }
                                                   }
                                                   None
                                               } {
                                        self.queue_used_sizes_align.get_mut(&index)?.0
                                            = change_used_index_to;
                                        Some(index)
                                    } else { None }
                                }
                            }
                            pub trait VirtioDeviceType {
                                fn configure(device: Arc<Mutex<VirtioDevice>>)
                                -> Result<Arc<Mutex<Self>>, ()>
                                where
                                Self: Sized;
                                /// Negotiate the accepted features with the device
                                /// By default, this rejects all features
                                fn negotiate_features(device:
                                                          &mut VirtioDevice)
                                 where Self: Sized {
                                    device.get_device_features();
                                    device.set_driver_features(0);
                                    device.accept_features().unwrap();
                                }
                                fn on_used_queue_ready(&self, queue: u16) { }
                                fn on_interrupt(&self);
                            }
                        }
                        pub mod filesystem {
                            pub mod tar { }
                        }
                        pub mod traits {
                            pub mod block {
                                use alloc::boxed::Box;
                                use core::future::Future;
                                use core::any::Any;
                                pub trait BlockDevice {
                                    fn _create_request(&self, sector: u64,
                                                       buffer: Box<[u8]>,
                                                       write: bool)
                                    ->
                                        Box<dyn AnyRequestFuture + Send +
                                            Sync + Unpin + 'static>;
                                }
                                pub trait AnyBlockDevice: BlockDevice + Any {
                                }
                                pub trait AnyRequestFuture: Future<Output =
                                                                   Option<Box<[u8]>>> +
                                 Any {
                                }
                                impl <T> AnyBlockDevice for T where
                                 T: BlockDevice + Any {
                                }
                                impl <T> AnyRequestFuture for T where
                                 T: Future<Output = Option<Box<[u8]>>> + Any {
                                }
                                pub trait GenericBlockDevice: BlockDevice {
                                    fn create_request(&self, sector: u64,
                                                      buffer: Box<[u8]>,
                                                      write: bool)
                                     ->
                                         Box<dyn AnyRequestFuture + Unpin +
                                             Send + Sync> {
                                        Box::new(Box::pin(self._create_request(sector,
                                                                               buffer,
                                                                               write)))
                                    }
                                    #[must_use]
                                    #[allow(clippy :: let_unit_value, clippy
                                            :: type_complexity, clippy ::
                                            type_repetition_in_bounds, clippy
                                            :: used_underscore_binding)]
                                    fn read<'life0,
                                            'async_trait>(&'life0 self,
                                                          sector: u64,
                                                          length: usize)
                                     ->
                                         ::core::pin::Pin<Box<dyn ::core::future::Future<Output
                                                                                         =
                                                                                         Result<Box<[u8]>,
                                                                                                ()>> +
                                                              ::core::marker::Send +
                                                              'async_trait>>
                                     where 'life0: 'async_trait,
                                     Self: ::core::marker::Sync +
                                     'async_trait {
                                        Box::pin(async move
                                                     {
                                                         if let ::core::option::Option::Some(__ret)
                                                                =
                                                                ::core::option::Option::None::<Result<Box<[u8]>,
                                                                                                      ()>>
                                                            {
                                                             return __ret;
                                                         }
                                                         let __self = self;
                                                         let sector = sector;
                                                         let length = length;
                                                         let __ret:
                                                                 Result<Box<[u8]>,
                                                                        ()> =
                                                             {
                                                                 let buffer =
                                                                     ::alloc::vec::from_elem(0,
                                                                                             length).into_boxed_slice();
                                                                 Ok(__self.create_request(sector,
                                                                                          buffer,
                                                                                          false).await.unwrap())
                                                             };

                                                         #[allow(unreachable_code)]
                                                         __ret
                                                     })
                                    }
                                    #[must_use]
                                    #[allow(clippy :: let_unit_value, clippy
                                            :: type_complexity, clippy ::
                                            type_repetition_in_bounds, clippy
                                            :: used_underscore_binding)]
                                    fn read_buffer<'life0,
                                                   'async_trait>(&'life0 self,
                                                                 sector: u64,
                                                                 buffer:
                                                                     Box<[u8]>)
                                     ->
                                         ::core::pin::Pin<Box<dyn ::core::future::Future<Output
                                                                                         =
                                                                                         Result<Box<[u8]>,
                                                                                                ()>> +
                                                              ::core::marker::Send +
                                                              'async_trait>>
                                     where 'life0: 'async_trait,
                                     Self: ::core::marker::Sync +
                                     'async_trait {
                                        Box::pin(async move
                                                     {
                                                         if let ::core::option::Option::Some(__ret)
                                                                =
                                                                ::core::option::Option::None::<Result<Box<[u8]>,
                                                                                                      ()>>
                                                            {
                                                             return __ret;
                                                         }
                                                         let __self = self;
                                                         let sector = sector;
                                                         let buffer = buffer;
                                                         let __ret:
                                                                 Result<Box<[u8]>,
                                                                        ()> =
                                                             {
                                                                 Ok(__self.create_request(sector,
                                                                                          buffer,
                                                                                          false).await.unwrap())
                                                             };

                                                         #[allow(unreachable_code)]
                                                         __ret
                                                     })
                                    }
                                    #[must_use]
                                    #[allow(clippy :: let_unit_value, clippy
                                            :: type_complexity, clippy ::
                                            type_repetition_in_bounds, clippy
                                            :: used_underscore_binding)]
                                    fn write<'life0,
                                             'async_trait>(&'life0 self,
                                                           sector: u64,
                                                           buffer: Box<[u8]>)
                                     ->
                                         ::core::pin::Pin<Box<dyn ::core::future::Future<Output
                                                                                         =
                                                                                         (Box<[u8]>,
                                                                                          Result<(),
                                                                                                 ()>)> +
                                                              ::core::marker::Send +
                                                              'async_trait>>
                                     where 'life0: 'async_trait,
                                     Self: ::core::marker::Sync +
                                     'async_trait {
                                        Box::pin(async move
                                                     {
                                                         if let ::core::option::Option::Some(__ret)
                                                                =
                                                                ::core::option::Option::None::<(Box<[u8]>,
                                                                                                Result<(),
                                                                                                       ()>)>
                                                            {
                                                             return __ret;
                                                         }
                                                         let __self = self;
                                                         let sector = sector;
                                                         let buffer = buffer;
                                                         let __ret:
                                                                 (Box<[u8]>,
                                                                  Result<(),
                                                                         ()>) =
                                                             {
                                                                 (__self.create_request(sector,
                                                                                        buffer,
                                                                                        true).await.unwrap(),
                                                                  Ok(()))
                                                             };

                                                         #[allow(unreachable_code)]
                                                         __ret
                                                     })
                                    }
                                }
                                impl <T> GenericBlockDevice for T where
                                 T: BlockDevice {
                                }
                            }
                        }
                        pub use traits::block::BlockDevice;
                    }
                    pub mod sbi {
                        use crate::cpu;
                        #[repr(isize)]
                        pub enum SBIError {
                            Success,
                            Failed,
                            NotSupported,
                            InvalidParam,
                            Denied,
                            InvalidAddress,
                            AlreadyAvailable,
                            AlreadyStarted,
                            AlreadyStopped,
                            Unknown,
                        }
                        #[automatically_derived]
                        #[allow(unused_qualifications)]
                        impl ::core::fmt::Debug for SBIError {
                            fn fmt(&self, f: &mut ::core::fmt::Formatter)
                             -> ::core::fmt::Result {
                                match (&*self,) {
                                    (&SBIError::Success,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "Success");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&SBIError::Failed,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "Failed");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&SBIError::NotSupported,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "NotSupported");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&SBIError::InvalidParam,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "InvalidParam");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&SBIError::Denied,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "Denied");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&SBIError::InvalidAddress,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "InvalidAddress");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&SBIError::AlreadyAvailable,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "AlreadyAvailable");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&SBIError::AlreadyStarted,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "AlreadyStarted");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&SBIError::AlreadyStopped,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "AlreadyStopped");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&SBIError::Unknown,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "Unknown");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                }
                            }
                        }
                        impl SBIError {
                            fn from_isize(v: isize) -> Self {
                                use SBIError::*;
                                match v {
                                    0 => Success,
                                    -1 => Failed,
                                    -2 => NotSupported,
                                    -3 => InvalidParam,
                                    -4 => Denied,
                                    -5 => InvalidAddress,
                                    -6 => AlreadyAvailable,
                                    -7 => AlreadyStarted,
                                    -8 => AlreadyStopped,
                                    _ => Unknown,
                                }
                            }
                        }
                        pub unsafe fn call_sbi_0(extension_id: usize,
                                                 function_id: usize)
                         -> Result<usize, SBIError> {
                            let error_code: usize;
                            let return_value: usize;
                            llvm_asm!(r"
		mv a7, $2
		mv a6, $3
		ecall
		mv $0, a0
		mv $1, a1
	":
                                "=r"(error_code), "=r"(return_value) :
                                "r"(extension_id), "r"(function_id) : );
                            if error_code == 0 {
                                Ok(return_value)
                            } else {
                                Err(SBIError::from_isize(core::mem::transmute(return_value)))
                            }
                        }
                        pub unsafe fn call_sbi_1(extension_id: usize,
                                                 function_id: usize,
                                                 a0: usize)
                         -> Result<usize, SBIError> {
                            let error_code: usize;
                            let return_value: usize;
                            llvm_asm!(r"
		mv a7, $2
		mv a6, $3
		mv a0, $4
		ecall
		mv $0, a0
		mv $1, a1
	":
                                "=r"(error_code), "=r"(return_value) :
                                "r"(extension_id), "r"(function_id), "r"(a0) :
                                );
                            if error_code == 0 {
                                Ok(return_value)
                            } else {
                                Err(SBIError::from_isize(core::mem::transmute(return_value)))
                            }
                        }
                        pub unsafe fn call_sbi_2(extension_id: usize,
                                                 function_id: usize,
                                                 a0: usize, a1: usize)
                         -> Result<usize, SBIError> {
                            let error_code: usize;
                            let return_value: usize;
                            llvm_asm!(r"
		mv a7, $2
		mv a6, $3
		mv a0, $4
		mv a0, $5
		ecall
		mv $0, a0
		mv $1, a1
	":
                                "=r"(error_code), "=r"(return_value) :
                                "r"(extension_id), "r"(function_id), "r"(a0),
                                "r"(a1) : );
                            if error_code == 0 {
                                Ok(return_value)
                            } else {
                                Err(SBIError::from_isize(core::mem::transmute(return_value)))
                            }
                        }
                        pub unsafe fn call_sbi_3(extension_id: usize,
                                                 function_id: usize,
                                                 a0: usize, a1: usize,
                                                 a2: usize)
                         -> Result<usize, SBIError> {
                            let error_code: usize;
                            let return_value: usize;
                            llvm_asm!(r"
		mv a7, $2
		mv a6, $3
		mv a0, $4
		mv a1, $5
		mv a2, $6
		ecall
		mv $0, a0
		mv $1, a1
	":
                                "=r"(error_code), "=r"(return_value) :
                                "r"(extension_id), "r"(function_id), "r"(a0),
                                "r"(a1), "r"(a2) : );
                            if error_code == 0 {
                                Ok(return_value)
                            } else {
                                Err(SBIError::from_isize(core::mem::transmute(return_value)))
                            }
                        }
                        pub fn set_absolute_timer(time: u64)
                         -> Result<(), SBIError> {
                            unsafe {
                                call_sbi_1(0x54494D45, 0,
                                           time as usize).map(|_| { })
                            }
                        }
                        pub fn set_relative_timer(time: u64)
                         -> Result<(), SBIError> {
                            set_absolute_timer(cpu::get_time() + time)
                        }
                        pub fn shutdown(reason: usize) {
                            unsafe {

                                #[allow(unused_unsafe)]
                                {
                                    use core::fmt::Write;
                                    let l =
                                        crate::std_macros::OUTPUT_LOCK.lock();
                                    let _ =
                                        unsafe {
                                            crate::drivers::uart::Uart::new(0x1000_0000)
                                        }.write_fmt(::core::fmt::Arguments::new_v1(&[""],
                                                                                   &match (&call_sbi_2(0x53525354,
                                                                                                       0,
                                                                                                       0,
                                                                                                       reason),)
                                                                                        {
                                                                                        (arg0,)
                                                                                        =>
                                                                                        [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                      ::core::fmt::Debug::fmt)],
                                                                                    }));
                                };
                            }
                        }
                        /// Safety: Only if start_addr is an address capable of bootstrapping himself
                        pub unsafe fn start_hart(hartid: usize,
                                                 start_addr: usize,
                                                 opaque: usize)
                         -> Result<(), SBIError> {
                            call_sbi_3(0x48534D, 0, hartid, start_addr,
                                       opaque).map(|_| { })
                        }
                        /// Safety: Only if start_addr is an address capable of bootstrapping himself
                        pub fn hart_get_status(hartid: usize)
                         -> Result<usize, SBIError> {
                            unsafe { call_sbi_1(0x48534D, 2, hartid) }
                        }
                    }
                    pub mod scheduler {
                        use crate::{cpu,
                                    process::{PROCESS_SCHED_QUEUE,
                                              ProcessState}, timer_queue};
                        pub fn schedule() -> usize {
                            let mut process_sched_queue =
                                PROCESS_SCHED_QUEUE.write();
                            let mut pid = 0;
                            let mut removed_index_min = usize::MAX;
                            let mut removed_index_max = usize::MAX;
                            for (idx, this_process) in
                                process_sched_queue.iter().enumerate() {
                                match this_process.upgrade() {
                                    Some(strong) => {
                                        let mut lock = strong.write();
                                        if lock.can_be_scheduled() {
                                            pid = lock.trap_frame.pid;
                                            lock.state =
                                                ProcessState::Scheduled;
                                            break ;
                                        }
                                    }
                                    None => {
                                        if removed_index_min == usize::MAX {
                                            removed_index_max = idx;
                                            removed_index_min = idx;
                                        } else if removed_index_max == idx - 1
                                         {
                                            removed_index_max = idx;
                                        }
                                    }
                                }
                            }
                            if removed_index_min != usize::MAX {
                                for _ in
                                    removed_index_min..removed_index_max + 1 {
                                    process_sched_queue.remove(removed_index_min);
                                }
                            }
                            if pid == 0 { return 0; }
                            process_sched_queue.rotate_left(1);
                            pid
                        }
                        pub fn schedule_next_slice(slices: u64) {
                            use timer_queue::{schedule_at, TimerEvent,
                                              TimerEventCause::*};
                            schedule_at(TimerEvent{instant:
                                                       cpu::get_time() +
                                                           slices * 1_000_000,
                                                   cause: ContextSwitch,});
                        }
                    }
                    pub mod future {
                        /// Quoting Wikipedia:
                        /// > In computer science, future, promise, delay, and
                        /// > deferred refer to constructs used for synchronizing program
                        /// > execution in some concurrent programming languages. They
                        /// > describe an object that acts as a proxy for a result that
                        /// > is initially unknown, usually because the computation of
                        /// > its value is not yet complete.
                        /// this file doesn't contain anything yet
                        use alloc::sync::{Arc, Weak};
                        use alloc::task::Wake;
                        use crate::lock::shared::Mutex;
                        use alloc::boxed::Box;
                        use core::future::Future;
                        use core::task::Waker;
                        use core::task::Context;
                        use alloc::collections::VecDeque;
                        struct TaskWaker(Weak<Executor>, Weak<Task>);
                        #[automatically_derived]
                        #[allow(unused_qualifications)]
                        impl ::core::clone::Clone for TaskWaker {
                            #[inline]
                            fn clone(&self) -> TaskWaker {
                                match *self {
                                    TaskWaker(ref __self_0_0, ref __self_0_1)
                                    =>
                                    TaskWaker(::core::clone::Clone::clone(&(*__self_0_0)),
                                              ::core::clone::Clone::clone(&(*__self_0_1))),
                                }
                            }
                        }
                        pub struct Task {
                            future: Mutex<Box<dyn Future<Output = ()> + Send +
                                              Unpin>>,
                            waker: Mutex<Option<Waker>>,
                            process_waker: Waker,
                        }
                        impl Wake for TaskWaker {
                            fn wake(self: Arc<Self>) {
                                self.0.upgrade().unwrap().push_task(self.1.upgrade().unwrap());
                                self.1.upgrade().unwrap().process_waker.wake_by_ref();
                            }
                        }
                        pub struct Executor {
                            queue: Mutex<VecDeque<Arc<Task>>>,
                            this: Mutex<Weak<Self>>,
                        }
                        impl Executor {
                            pub fn new() -> Arc<Self> {
                                let t =
                                    Self{queue: Mutex::new(VecDeque::new()),
                                         this: Mutex::new(Weak::new()),};
                                let t = Arc::new(t);
                                *t.this.lock() = Arc::downgrade(&t);
                                t
                            }
                            fn push_task(&self, task: Arc<Task>) {
                                self.queue.lock().push_back(task)
                            }
                            pub fn push_future(&self,
                                               future:
                                                   Box<dyn Future<Output =
                                                                  ()> + Send +
                                                       Unpin>) {
                                let task =
                                    Task{future: Mutex::new(future),
                                         waker: Mutex::new(None),
                                         process_waker:
                                             crate::process::Process::this().read().construct_waker(),};
                                let task = Arc::new(task);
                                *task.waker.lock() =
                                    Some(Arc::new(TaskWaker(self.this.lock().clone(),
                                                            Arc::downgrade(&task))).into());
                                self.queue.lock().push_back(task)
                            }
                            pub fn run_one(&self)
                             -> Option<Option<Arc<Task>>> {
                                let task = self.queue.lock().pop_front();
                                let task =
                                    if let Some(task) = task {
                                        task
                                    } else { return None };
                                {
                                    let lvl = ::log::Level::Info;
                                    if lvl <= ::log::STATIC_MAX_LEVEL &&
                                           lvl <= ::log::max_level() {
                                        ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["Exec "],
                                                                                                &match (&(Arc::as_ref(&task)
                                                                                                              as
                                                                                                              *const _),)
                                                                                                     {
                                                                                                     (arg0,)
                                                                                                     =>
                                                                                                     [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                   ::core::fmt::Debug::fmt)],
                                                                                                 }),
                                                                 lvl,
                                                                 &("rust_0bsd_riscv_kernel::future",
                                                                   "rust_0bsd_riscv_kernel::future",
                                                                   "src/future.rs",
                                                                   79u32));
                                    }
                                };
                                use core::task::Poll;
                                let result =
                                    {
                                        let waker: Waker =
                                            task.waker.lock().as_ref().unwrap().clone();
                                        let mut context =
                                            Context::from_waker(&waker);
                                        let mut guard = task.future.lock();
                                        let mut future = &mut *guard;
                                        let t =
                                            core::pin::Pin::new(future).poll(&mut context);
                                        t
                                    };
                                match result {
                                    Poll::Ready(_) => {
                                        {

                                            #[allow(unused_unsafe)]
                                            {
                                                use core::fmt::Write;
                                                let l =
                                                    crate::std_macros::OUTPUT_LOCK.lock();
                                                let _ =
                                                    unsafe {
                                                        crate::drivers::uart::Uart::new(0x1000_0000)
                                                    }.write_fmt(::core::fmt::Arguments::new_v1(&["Ready ",
                                                                                                 "\r\n"],
                                                                                               &match (&self.queue.lock().len(),)
                                                                                                    {
                                                                                                    (arg0,)
                                                                                                    =>
                                                                                                    [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                  ::core::fmt::Debug::fmt)],
                                                                                                }));
                                            }
                                        };
                                        return Some(Some(task));
                                    }
                                    Poll::Pending => {
                                        self.queue.lock().push_front(task);
                                        Some(None)
                                    }
                                }
                            }
                        }
                    }
                    pub mod filesystem {
                        struct FsHandle(usize);
                        use alloc::boxed::Box;
                        trait Filesystem {
                            #[must_use]
                            #[allow(clippy :: type_complexity, clippy ::
                                    type_repetition_in_bounds)]
                            fn root<'async_trait>()
                            ->
                                ::core::pin::Pin<Box<dyn ::core::future::Future<Output
                                                                                =
                                                                                FsHandle> +
                                                     ::core::marker::Send +
                                                     'async_trait>>;
                            #[must_use]
                            #[allow(clippy :: type_complexity, clippy ::
                                    type_repetition_in_bounds)]
                            fn get_path<'life0,
                                        'async_trait>(from: FsHandle,
                                                      path: &'life0 str)
                            ->
                                ::core::pin::Pin<Box<dyn ::core::future::Future<Output
                                                                                =
                                                                                FsHandle> +
                                                     ::core::marker::Send +
                                                     'async_trait>>
                            where
                            'life0: 'async_trait;
                            #[must_use]
                            #[allow(clippy :: type_complexity, clippy ::
                                    type_repetition_in_bounds)]
                            fn open_path<'life0,
                                         'async_trait>(from: FsHandle,
                                                       path: &'life0 str)
                            ->
                                ::core::pin::Pin<Box<dyn ::core::future::Future<Output
                                                                                =
                                                                                FsHandle> +
                                                     ::core::marker::Send +
                                                     'async_trait>>
                            where
                            'life0: 'async_trait;
                        }
                        pub mod ext2 {
                            pub mod structures {
                                #[repr(C)]
                                pub struct Superblock {
                                    /// 32bit value indicating the total number of inodes, both used and free,
                                    /// in the file system.  This value must be lower or equal to
                                    /// (s_inodes_per_group * number of block groups).  It must be equal to the
                                    /// sum of the inodes defined in each block group.
                                    inodes_count: u32,
                                    /// 32bit value indicating the total number of blocks in the system including
                                    /// all used, free and reserved. This value must be lower or equal to
                                    /// (s_blocks_per_group * number of block groups). It can be lower than
                                    /// the previous calculation if the last block group has a smaller number of
                                    /// blocks than s_blocks_per_group du to volume size.  It must be equal to
                                    /// the sum of the blocks defined in each block group.
                                    blocks_count: u32,
                                    /// 32bit value indicating the total number of  blocks  reserved  for  the
                                    /// usage of the super user.  This is most useful if  for  some  reason  a
                                    /// user, maliciously or not, fill the file system to capacity; the  super
                                    /// user will have this specified amount of free blocks at his disposal so
                                    /// he can edit and save configuration files.
                                    r_blocks_count: u32,
                                    /// 32bit value indicating the total number of free blocks, including  the
                                    /// number of reserved blocks (see
                                    /// s_r_blocks_count).  This is a  sum
                                    /// of all free blocks of all the block groups.
                                    free_blocks_count: u32,
                                    /// 32bit value indicating the total number of free inodes.  This is a sum
                                    /// of all free inodes of all the block groups.
                                    free_inodes_count: u32,
                                    /// 32bit value identifying the first data block, in other word the id  of
                                    /// the block containing the superblock structure.
                                    first_data_block: u32,
                                    /// The block size is computed using this 32bit value  as  the  number  of
                                    /// bits to shift left the value 1024.  This value may only be non-negative.
                                    log_block_size: u32,
                                    /// The fragment size is computed using this 32bit value as the number  of
                                    /// bits to shift left the value 1024.  Note that a negative  value  would
                                    /// shift the bit right rather than left.
                                    log_frag_size: u32,
                                    /// 32bit value indicating the total number  of  blocks  per  group.  This
                                    /// value in combination with
                                    /// s_first_data_block can  be  used
                                    /// to determine the block groups boundaries.  Due to volume size boundaries,
                                    /// the last block group might have a smaller number of blocks than what is
                                    /// specified in this field.
                                    blocks_per_group: u32,
                                    /// 32bit value indicating the total number of fragments per group.  It is
                                    /// also used to determine the size of the block bitmap  of
                                    /// each block group.
                                    frags_per_group: u32,
                                    /// 32bit value indicating the total number of inodes per group.  This  is
                                    /// also used to determine the size of the inode bitmap  of
                                    /// each block group.  Note that you cannot have more than
                                    /// (block size in bytes * 8) inodes per group as the inode bitmap
                                    /// must fit within a single block. This value must be a perfect multiple
                                    /// of the number of inodes that can fit in a block
                                    /// ((1024<<s_log_block_size)/s_inode_size).
                                    inodes_per_group: u32,
                                    /// Unix time, as defined by POSIX, of the last time the file  system  was
                                    /// mounted.
                                    mtime: u32,
                                    /// Unix time, as defined by POSIX, of the last write access to  the  file
                                    /// system.
                                    wtime: u32,
                                    /// 16bit value indicating how many  time  the  file  system  was  mounted
                                    /// since the last time it was fully verified.
                                    mnt_count: u16,
                                    /// 16bit value indicating the maximum  number  of  times  that  the  file
                                    /// system may be mounted before a full check is performed.
                                    max_mnt_count: u16,
                                    /// 16bit value  identifying  the  file  system  as  Ext2.  The  value  is
                                    /// currently fixed to EXT2_SUPER_MAGIC of value 0xEF53.
                                    magic: u16,
                                    /// 16bit value indicating the file system state.  When the file system is
                                    /// mounted, this state is set  to  EXT2_ERROR_FS.  After the
                                    /// file system was cleanly unmounted, this value is set to EXT2_VALID_FS.
                                    state: u16,
                                    /// 16bit value indicating what the file system driver should do  when  an
                                    /// error is detected.  The following values have been defined:
                                    errors: u16,
                                    /// 16bit value identifying the minor revision level within its
                                    /// revision level.
                                    minor_rev_level: u16,
                                    /// Unix time, as defined by POSIX, of the last file system check.
                                    lastcheck: u32,
                                    /// Maximum Unix time interval, as defined by POSIX, allowed between file
                                    /// system checks.
                                    checkinterval: u32,
                                    /// 32bit identifier of the os that created the file system.  Defined
                                    /// values are:
                                    creator_os: u32,
                                    /// 32bit revision level value.
                                    rev_level: u32,
                                    /// 16bit value used as the default user id for reserved blocks.
                                    def_resuid: u16,
                                    /// 16bit value used as the default group id for reserved blocks.
                                    def_resgid: u16,
                                    /// 32bit value used as index to the  first  inode  useable  for  standard
                                    /// files. In revision 0, the first	non-reserved inode is fixed to
                                    /// 11 (EXT2_GOOD_OLD_FIRST_INO). In revision 1 and later
                                    /// this value may be set to any value.
                                    first_ino: u32,
                                    /// 16bit value indicating the size of the inode structure. In revision 0, this
                                    /// value is always 128 (EXT2_GOOD_OLD_INODE_SIZE). In revision 1
                                    /// and later, this value must be a perfect power of 2 and must be smaller or
                                    /// equal to the block size (1<<s_log_block_size).
                                    inode_size: u16,
                                    /// 16bit value used to indicate the block group number hosting this
                                    /// superblock structure.  This can be used to rebuild the file system
                                    /// from any superblock backup.
                                    block_group_nr: u16,
                                    /// 32bit bitmask of compatible features.  The file system implementation
                                    /// is free to support them or not without risk of damaging the meta-data.
                                    feature_compat: u32,
                                    /// 32bit bitmask of incompatible features.  The file system
                                    /// implementation should refuse to mount the file system if any of
                                    /// the indicated feature is unsupported.
                                    feature_incompat: u32,
                                    /// 32bit bitmask of “read-only” features.  The file system
                                    /// implementation should mount as read-only if any of the indicated
                                    /// feature is unsupported.
                                    feature_ro_compat: u32,
                                    /// 128bit value used as the volume id.  This should, as much as possible,
                                    /// be unique for each file system formatted.
                                    uuid: [u8; 16],
                                    /// 16 bytes volume name, mostly unusued.  A valid volume name would consist
                                    /// of only ISO-Latin-1 characters and be 0 terminated.
                                    volume_name: [u8; 16],
                                    /// 64 bytes directory path where the file system was last mounted.  While
                                    /// not normally used, it could serve for auto-finding the mountpoint when
                                    /// not indicated on the command line. Again the path should be zero
                                    /// terminated for compatibility reasons.  Valid path is constructed from
                                    /// ISO-Latin-1 characters.
                                    last_mounted: [u8; 64],
                                    /// 32bit value used by compression algorithms to determine the compression
                                    /// method(s) used.
                                    algo_bitmap: u32,
                                    /// 8-bit value representing the number of blocks the implementation should
                                    /// attempt to pre-allocate when creating a new regular file.
                                    prealloc_blocks: u8,
                                    /// 8-bit value representing the number of blocks the implementation should
                                    /// attempt to pre-allocate when creating a new directory.
                                    prealloc_dir_blocks: u8,
                                    _pad_1: [u8; 2],
                                    /// 16-byte value containing the uuid of the journal superblock.  See Ext3
                                    /// Journaling for more information.
                                    journal_uuid: [u8; 16],
                                    /// 32-bit inode number of the journal file.  See Ext3 Journaling for more
                                    /// information.
                                    journal_inum: u32,
                                    /// 32-bit device number of the journal file.  See Ext3 Journaling for more
                                    /// information.
                                    journal_dev: u32,
                                    /// 32-bit inode number, pointing to the first inode in the list of inodes
                                    /// to delete.  See Ext3 Journaling for more information.
                                    last_orphan: u32,
                                    /// An array of 4 32bit values containing the seeds used for the hash
                                    /// algorithm for directory indexing.
                                    hash_seed: [u32; 4],
                                    /// An 8bit value containing the default hash version used for directory indexing.
                                    def_hash_version: u8,
                                    _pad_2: [u8; 3],
                                    /// A 32bit value containing the default mount options for this file system. TODO: Add more information here!
                                    default_mount_options: u32,
                                    /// A 32bit value indicating the block group ID of the first meta block group.  TODO: Research if this is an Ext3-only extension.
                                    first_meta_bg: u32,
                                }
                                #[repr(C)]
                                pub struct BlockGroupDescriptor {
                                    /// 32bit block id of the first block of the
                                    /// “block bitmap”
                                    /// for the group represented.
                                    block_bitmap: u32,
                                    /// 32bit block id of the first block of the
                                    /// “inode bitmap”
                                    /// for the group represented.
                                    inode_bitmap: u32,
                                    /// 32bit block id of the first block of the
                                    /// “inode table”
                                    /// for the group represented.
                                    inode_table: u32,
                                    /// 16bit value indicating the total number of free blocks for
                                    /// the represented group.
                                    free_blocks_count: u16,
                                    /// 16bit value indicating the total number of free inodes for
                                    /// the represented group.
                                    free_inodes_count: u16,
                                    /// 16bit value indicating the number of inodes allocated to
                                    /// directories for the represented group.
                                    used_dirs_count: u16,
                                    /// 16bit value used for padding the structure on a 32bit boundary.
                                    pad: u16,
                                    /// 12 bytes of reserved space for future revisions.
                                    reserved: [u8; 12],
                                }
                                #[repr(C)]
                                pub struct Inode {
                                    /// 16bit value used to indicate the format of the described file and the
                                    /// access rights.  Here are the possible values, which can be combined
                                    /// in various ways:
                                    mode: u16,
                                    /// 16bit user id associated with the file.
                                    uid: u16,
                                    /// In revision 0, (signed) 32bit value indicating the size of the file in
                                    /// bytes.  In revision 1 and later revisions, and only for regular files, this
                                    /// represents the lower 32-bit of the file size; the upper 32-bit is located
                                    /// in the i_dir_acl.
                                    size: u32,
                                    /// 32bit value representing the number of seconds since january 1st 1970
                                    /// of the last time this inode was accessed.
                                    atime: u32,
                                    /// 32bit value representing the number of seconds since january 1st 1970, of
                                    /// when the inode was created.
                                    ctime: u32,
                                    /// 32bit value representing the number of seconds since january 1st 1970,
                                    /// of the last time this inode was modified.
                                    mtime: u32,
                                    /// 32bit value representing the number of seconds since january 1st 1970, of
                                    /// when the inode was deleted.
                                    dtime: u32,
                                    /// 16bit value of the POSIX group having access to this file.
                                    gid: u16,
                                    /// 16bit value indicating how many times this particular inode is linked
                                    /// (referred to). Most files will have a link count of 1.  Files with hard
                                    /// links pointing to them will have an additional count for each hard link.
                                    links_count: u16,
                                    /// 32-bit value representing the total number of 512-bytes blocks reserved to contain the
                                    /// data of this inode, regardless if these blocks are used or not.  The block
                                    /// numbers of these reserved blocks are contained in the
                                    /// i_block array.
                                    blocks: u32,
                                    /// 32bit value indicating how the ext2 implementation should behave when
                                    /// accessing the data for this inode.
                                    flags: u32,
                                    osd1: u32,
                                    /// 15 x 32bit block numbers pointing to the blocks containing the data for
                                    /// this inode. The first 12 blocks are direct blocks.  The 13th entry in this
                                    /// array is the block number of the first indirect block; which is a block
                                    /// containing an array of block ID containing the data.  Therefore, the 13th
                                    /// block of the file will be the first block ID contained in the indirect block.
                                    /// With a 1KiB block size, blocks 13 to 268 of the file data are contained
                                    /// in this indirect block.
                                    block: [u32; 15],
                                    /// 32bit value used to indicate the file version (used by NFS).
                                    generation: u32,
                                    /// 32bit value indicating the block number containing the extended
                                    /// attributes. In revision 0 this value is always 0.
                                    file_acl: u32,
                                    /// In revision 0 this 32bit value is always 0.  In revision 1, for regular
                                    /// files this 32bit value contains the high 32 bits of the 64bit file size.
                                    dir_acl: u32,
                                    /// 32bit value indicating the location of the file fragment.
                                    faddr: u32,
                                    osd2: u32,
                                    osd3: u16,
                                }
                                struct LinkedDirectoryEntry {
                                    /// 32bit inode number of the file entry.  A value of 0 indicate that the entry
                                    /// is not used.
                                    inode: u32,
                                    /// 16bit unsigned displacement to the next directory entry from the start of the
                                    /// current directory entry. This field must have a value at least equal to the
                                    /// length of the current record.
                                    rec_len: u16,
                                }
                                struct IndexedDirectoryRoot {
                                    _pad_1: [u8; 9],
                                    padding: u16,
                                    padding2: u8,
                                    _pad_2: [u8; 10],
                                    padding_2: u16,
                                    _pad_3: [u8; 4],
                                    /// 8bit value representing the hash version used in this indexed directory.
                                    hash_version: u8,
                                    /// 8bit length of the indexed directory information structure (dx_root);
                                    /// currently equal to 8.
                                    info_length: u8,
                                    /// 8bit value indicating how many indirect levels of indexing are present in
                                    /// this hash.
                                    indirect_levels: u8,
                                }
                                struct IndexedDirectoryEntryCountandLimit {
                                    /// 16bit value representing the total number of indexed directory entries that
                                    /// fit within the block, after removing the other structures, but including
                                    /// the count/limit entry.
                                    limit: u16,
                                    /// 16bit value representing the total number of indexed directory entries
                                    /// present in the block. TODO: Research if this value includes the count/limit entry.
                                    count: u16,
                                }
                            }
                            pub mod code {
                                use super::structures::Superblock;
                                use crate::drivers::traits::block::{AnyBlockDevice,
                                                                    AnyRequestFuture,
                                                                    GenericBlockDevice};
                                use alloc::boxed::Box;
                                use crate::lock::shared::RwLock;
                                struct Ext2 {
                                    device: Box<dyn GenericBlockDevice +
                                                Send + Sync + Unpin>,
                                    superblock: RwLock<Option<Box<Superblock>>>,
                                }
                                impl Ext2 {
                                    async fn read_block(&self) { }
                                    async fn load_superblock(&self)
                                     -> Result<(), ()> {
                                        self.device.read(2, 2).await?.unwrap()
                                            * self.superblock.write() = Some()
                                    }
                                }
                            }
                        }
                    }
                    pub mod handle {
                        use core::num::NonZeroUsize;
                        #[repr(usize)]
                        pub enum StandardHandleErrors { Unimplemented = 1, }
                        pub trait HandleBackend {
                            fn read(&mut self, buf: &mut [u8])
                             -> Result<usize, usize> {
                                Err(StandardHandleErrors::Unimplemented as
                                        usize)
                            }
                            fn write(&mut self, buf: &[u8])
                             -> Result<usize, usize> {
                                Err(StandardHandleErrors::Unimplemented as
                                        usize)
                            }
                            fn size_hint(&mut self)
                             -> (usize, Option<usize>) {
                                (0, None)
                            }
                            fn seek(&mut self, position: &usize)
                             -> Result<(), usize> {
                                Err(StandardHandleErrors::Unimplemented as
                                        usize)
                            }
                            fn tell(&mut self) -> Result<usize, usize> {
                                Err(StandardHandleErrors::Unimplemented as
                                        usize)
                            }
                            fn split(&mut self) -> Option<NonZeroUsize> {
                                None
                            }
                        }
                    }
                    pub mod process {
                        use core::{pin::Pin, future::Future};
                        use core::sync::atomic::AtomicUsize;
                        use alloc::{boxed::Box, collections::{BTreeMap},
                                    sync::{Arc, Weak}, vec::Vec};
                        use crate::asm::do_supervisor_syscall_0;
                        use crate::lock::shared::{RwLock};
                        use crate::syscall::syscall_exit;
                        use crate::trap::use_boot_frame_if_necessary;
                        use core::task::{Waker, RawWaker, RawWakerVTable};
                        use crate::{context_switch,
                                    cpu::{self, load_hartid, read_sscratch,
                                          write_sscratch},
                                    hart::get_this_hart_meta,
                                    scheduler::schedule_next_slice,
                                    trap::{TrapFrame}};
                        use crate::cpu::Registers;
                        use aligned::{A16, Aligned};
                        use alloc::string::String;
                        pub const TASK_STACK_SIZE: usize = 4096 * 8;
                        pub const PROCESS_WAKER_VTABLE: RawWakerVTable =
                            RawWakerVTable::new(Process::waker_clone,
                                                Process::waker_wake,
                                                Process::waker_wake_by_ref,
                                                Process::waker_drop);
                        pub static PROCESSES:
                         RwLock<BTreeMap<usize, Arc<RwLock<Process>>>> =
                            RwLock::new(BTreeMap::new());
                        pub static PROCESS_SCHED_QUEUE:
                         RwLock<Vec<Weak<RwLock<Process>>>> =
                            RwLock::new(Vec::new());
                        pub struct FileDescriptor {
                            fd_id: usize,
                            backend: usize,
                            backend_meta: usize,
                        }
                        #[automatically_derived]
                        #[allow(unused_qualifications)]
                        impl ::core::fmt::Debug for FileDescriptor {
                            fn fmt(&self, f: &mut ::core::fmt::Formatter)
                             -> ::core::fmt::Result {
                                match *self {
                                    FileDescriptor {
                                    fd_id: ref __self_0_0,
                                    backend: ref __self_0_1,
                                    backend_meta: ref __self_0_2 } => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_struct(f,
                                                                                      "FileDescriptor");
                                        let _ =
                                            ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                            "fd_id",
                                                                            &&(*__self_0_0));
                                        let _ =
                                            ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                            "backend",
                                                                            &&(*__self_0_1));
                                        let _ =
                                            ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                            "backend_meta",
                                                                            &&(*__self_0_2));
                                        ::core::fmt::DebugStruct::finish(debug_trait_builder)
                                    }
                                }
                            }
                        }
                        pub enum ProcessState {
                            Running,
                            Yielded,
                            Pending,
                            Scheduled,
                        }
                        #[automatically_derived]
                        #[allow(unused_qualifications)]
                        impl ::core::fmt::Debug for ProcessState {
                            fn fmt(&self, f: &mut ::core::fmt::Formatter)
                             -> ::core::fmt::Result {
                                match (&*self,) {
                                    (&ProcessState::Running,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "Running");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&ProcessState::Yielded,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "Yielded");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&ProcessState::Pending,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "Pending");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&ProcessState::Scheduled,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "Scheduled");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                }
                            }
                        }
                        pub struct Process {
                            /// The process ID of the process can be fetched by getting trap_frame.pid
                            pub is_supervisor: bool,
                            pub state: ProcessState,
                            pub file_descriptors: BTreeMap<usize,
                                                           FileDescriptor>,
                            pub trap_frame: Pin<Box<TrapFrame>>,
                            pub name: Option<String>,
                            no_op_yield_count: AtomicUsize,
                            /// For supervisor mode the kernel initially creates a small stack page for this process
                            /// This is where it's stored
                            pub kernel_allocated_stack: Option<Box<[u8; TASK_STACK_SIZE]>>,
                        }
                        #[automatically_derived]
                        #[allow(unused_qualifications)]
                        impl ::core::fmt::Debug for Process {
                            fn fmt(&self, f: &mut ::core::fmt::Formatter)
                             -> ::core::fmt::Result {
                                match *self {
                                    Process {
                                    is_supervisor: ref __self_0_0,
                                    state: ref __self_0_1,
                                    file_descriptors: ref __self_0_2,
                                    trap_frame: ref __self_0_3,
                                    name: ref __self_0_4,
                                    no_op_yield_count: ref __self_0_5,
                                    kernel_allocated_stack: ref __self_0_6 }
                                    => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_struct(f,
                                                                                      "Process");
                                        let _ =
                                            ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                            "is_supervisor",
                                                                            &&(*__self_0_0));
                                        let _ =
                                            ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                            "state",
                                                                            &&(*__self_0_1));
                                        let _ =
                                            ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                            "file_descriptors",
                                                                            &&(*__self_0_2));
                                        let _ =
                                            ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                            "trap_frame",
                                                                            &&(*__self_0_3));
                                        let _ =
                                            ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                            "name",
                                                                            &&(*__self_0_4));
                                        let _ =
                                            ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                            "no_op_yield_count",
                                                                            &&(*__self_0_5));
                                        let _ =
                                            ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                            "kernel_allocated_stack",
                                                                            &&(*__self_0_6));
                                        ::core::fmt::DebugStruct::finish(debug_trait_builder)
                                    }
                                }
                            }
                        }
                        extern "C" {
                            fn switch_to_supervisor_frame(trap_frame:
                                                              *mut TrapFrame)
                            -> !;
                        }
                        impl Process {
                            pub fn has_read_access(&self, address: usize,
                                                   size: usize) -> bool {
                                if self.is_supervisor { return true; }
                                false
                            }
                            pub fn has_write_access(&self, address: usize,
                                                    size: usize) -> bool {
                                if self.is_supervisor { return true; }
                                false
                            }
                            pub fn can_be_scheduled(&self) -> bool {
                                match self.state {
                                    ProcessState::Pending => true,
                                    _ => false,
                                }
                            }
                            pub fn run_once(&mut self) -> ! {
                                self.trap_frame.hartid = load_hartid();
                                self.trap_frame.interrupt_stack =
                                    unsafe {
                                        (*read_sscratch()).interrupt_stack
                                    };
                                let frame_pointer =
                                    Pin::as_ref(&self.trap_frame).get_ref() as
                                        *const TrapFrame as *mut TrapFrame;
                                {
                                    let lvl = ::log::Level::Info;
                                    if lvl <= ::log::STATIC_MAX_LEVEL &&
                                           lvl <= ::log::max_level() {
                                        ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["Switch to frame at \u{1b}[32m",
                                                                                                  "\u{1b}[0m (PC ",
                                                                                                  " NAME ",
                                                                                                  " HART ",
                                                                                                  ")"],
                                                                                                &match (&frame_pointer,
                                                                                                        &unsafe
                                                                                                         {
                                                                                                             (*frame_pointer).pc
                                                                                                         },
                                                                                                        &self.name,
                                                                                                        &self.trap_frame.hartid)
                                                                                                     {
                                                                                                     (arg0,
                                                                                                      arg1,
                                                                                                      arg2,
                                                                                                      arg3)
                                                                                                     =>
                                                                                                     [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                   ::core::fmt::Debug::fmt),
                                                                                                      ::core::fmt::ArgumentV1::new(arg1,
                                                                                                                                   ::core::fmt::LowerHex::fmt),
                                                                                                      ::core::fmt::ArgumentV1::new(arg2,
                                                                                                                                   ::core::fmt::Debug::fmt),
                                                                                                      ::core::fmt::ArgumentV1::new(arg3,
                                                                                                                                   ::core::fmt::Display::fmt)],
                                                                                                 }),
                                                                 lvl,
                                                                 &("rust_0bsd_riscv_kernel::process",
                                                                   "rust_0bsd_riscv_kernel::process",
                                                                   "src/process.rs",
                                                                   99u32));
                                    }
                                };
                                {
                                    let lvl = ::log::Level::Info;
                                    if lvl <= ::log::STATIC_MAX_LEVEL &&
                                           lvl <= ::log::max_level() {
                                        ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["count "],
                                                                                                &match (&(PROCESSES.read().iter().filter(|(k,
                                                                                                                                           v)|
                                                                                                                                             **k
                                                                                                                                                 !=
                                                                                                                                                 self.trap_frame.pid).filter(|(k,
                                                                                                                                                                               v)|
                                                                                                                                                                                 if let ProcessState::Running
                                                                                                                                                                                        =
                                                                                                                                                                                        v.read().state
                                                                                                                                                                                    {
                                                                                                                                                                                     true
                                                                                                                                                                                 } else {
                                                                                                                                                                                     false
                                                                                                                                                                                 }).count()
                                                                                                              +
                                                                                                              1),)
                                                                                                     {
                                                                                                     (arg0,)
                                                                                                     =>
                                                                                                     [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                   ::core::fmt::Debug::fmt)],
                                                                                                 }),
                                                                 lvl,
                                                                 &("rust_0bsd_riscv_kernel::process",
                                                                   "rust_0bsd_riscv_kernel::process",
                                                                   "src/process.rs",
                                                                   102u32));
                                    }
                                };
                                self.state = ProcessState::Running;
                                self.trap_frame.flags &= !1;
                                unsafe {
                                    switch_to_supervisor_frame(frame_pointer)
                                };
                            }
                            pub unsafe fn waker_clone(data: *const ())
                             -> RawWaker {
                                let obj =
                                    Box::from_raw(data as
                                                      *mut Weak<RwLock<Self>>);
                                let new_waker =
                                    RawWaker::new(Box::into_raw(obj.clone())
                                                      as _,
                                                  &PROCESS_WAKER_VTABLE);
                                Box::leak(obj);
                                new_waker
                            }
                            pub unsafe fn waker_wake(data: *const ()) {
                                Self::waker_wake_by_ref(data)
                            }
                            pub unsafe fn waker_wake_by_ref(data: *const ()) {
                                let process: Box<Weak<RwLock<Self>>> =
                                    Box::from_raw(data as _);
                                let process_internal =
                                    process.upgrade().expect("Waited process is gone!");
                                process_internal.write().make_pending_when_possible();
                                Box::leak(process);
                            }
                            pub unsafe fn waker_drop(data: *const ()) {
                                drop(Box::from_raw(data as
                                                       *mut Weak<RwLock<Self>>));
                            }
                            pub fn make_pending_when_possible(&mut self) {
                                match self.state {
                                    ProcessState::Yielded => {
                                        self.state = ProcessState::Pending;
                                    }
                                    _ => {
                                        self.no_op_yield_count.fetch_add(1,
                                                                         core::sync::atomic::Ordering::SeqCst);
                                    }
                                }
                            }
                            /// This creates a Waker that makes this process a Pending process when woken
                            /// The Pending process will be eventually scheduled
                            pub fn construct_waker(&self) -> Waker {
                                let raw_pointer =
                                    Box::into_raw(Box::new(weak_get_process(&self.trap_frame.pid)))
                                        as *const ();
                                unsafe {
                                    Waker::from_raw(RawWaker::new(raw_pointer,
                                                                  &PROCESS_WAKER_VTABLE))
                                }
                            }
                            /// Polls a future from this process. The waker is this processes' waker
                            pub fn poll_future<T: Future>(&mut self,
                                                          future: Pin<&mut T>)
                             -> core::task::Poll<<T as Future>::Output> {
                                use core::task::Poll;
                                let poll_result =
                                    future.poll(&mut core::task::Context::from_waker(&self.construct_waker()));
                                if poll_result.is_pending() {
                                    self.state = ProcessState::Yielded;
                                    schedule_next_slice(0);
                                }
                                poll_result
                            }
                            pub fn yield_maybe(&mut self) -> bool {
                                if self.no_op_yield_count.load(core::sync::atomic::Ordering::Acquire)
                                       == 0 {
                                    self.state = ProcessState::Yielded;
                                    true
                                } else {
                                    self.no_op_yield_count.fetch_sub(1,
                                                                     core::sync::atomic::Ordering::AcqRel);
                                    false
                                }
                            }
                            pub fn try_yield_maybe(&mut self) -> bool {
                                self.no_op_yield_count.load(core::sync::atomic::Ordering::Acquire)
                                    == 0
                            }
                            pub fn this_pid() -> usize {
                                unsafe {
                                    cpu::read_sscratch().as_ref().expect("Not running on a process!").pid
                                }
                            }
                            pub fn this() -> Arc<RwLock<Process>> {
                                try_get_process(&Self::this_pid())
                            }
                        }
                        pub fn init() { }
                        pub fn finish_executing_process(pid: usize) {
                            if pid == 0 || pid == 1 { return; }
                            try_get_process(&pid).write().state =
                                ProcessState::Pending;
                            {
                                let lvl = ::log::Level::Debug;
                                if lvl <= ::log::STATIC_MAX_LEVEL &&
                                       lvl <= ::log::max_level() {
                                    ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["Made process pending"],
                                                                                            &match ()
                                                                                                 {
                                                                                                 ()
                                                                                                 =>
                                                                                                 [],
                                                                                             }),
                                                             lvl,
                                                             &("rust_0bsd_riscv_kernel::process",
                                                               "rust_0bsd_riscv_kernel::process",
                                                               "src/process.rs",
                                                               213u32));
                                }
                            };
                        }
                        /// Finds an unused PID
                        pub fn allocate_pid() -> usize {
                            let mut pid = 2;
                            for this_pid in pid.. {
                                if !PROCESSES.read().contains_key(&this_pid) {
                                    pid = this_pid;
                                    break ;
                                }
                            }
                            pid
                        }
                        /// Creates a supervisor process and returns PID
                        /// SAFETY: Only when function is a valid function pointer (with)
                        pub fn new_supervisor_process_int(function: usize,
                                                          a0: usize)
                         -> usize {
                            let pid = allocate_pid();
                            let trapframe_box = Box::new(TrapFrame::zeroed());
                            let trapframe_box = Pin::new(trapframe_box);
                            let mut process =
                                Process{is_supervisor: true,
                                        file_descriptors: BTreeMap::new(),
                                        trap_frame: trapframe_box,
                                        state: ProcessState::Pending,
                                        kernel_allocated_stack: None,
                                        name: None,
                                        no_op_yield_count:
                                            AtomicUsize::new(0),};
                            process.trap_frame.general_registers[Registers::A0.idx()]
                                = a0;
                            process.trap_frame.general_registers[Registers::Ra.idx()]
                                = process_return_address_supervisor as usize;
                            process.trap_frame.pc = function;
                            process.trap_frame.pid = pid;
                            process.trap_frame.hartid = 0xBADC0DE;
                            let process_stack =
                                ::alloc::vec::from_elem(0,
                                                        TASK_STACK_SIZE).into_boxed_slice();
                            process.trap_frame.general_registers[Registers::Sp.idx()]
                                =
                                process_stack.as_ptr() as usize +
                                    TASK_STACK_SIZE - 0x10;
                            use core::convert::TryInto;
                            process.kernel_allocated_stack =
                                Some(process_stack.try_into().expect("Process stack has incorrect length!"));
                            let process = RwLock::new(process);
                            let process = Arc::new(process);
                            PROCESS_SCHED_QUEUE.write().push(Arc::downgrade(&process));
                            PROCESSES.write().insert(pid, process);
                            pid
                        }
                        #[no_mangle]
                        pub extern "C" fn process_return_address_supervisor() {
                            unsafe { crate::asm::do_supervisor_syscall_0(1) };
                            {
                                let lvl = ::log::Level::Debug;
                                if lvl <= ::log::STATIC_MAX_LEVEL &&
                                       lvl <= ::log::max_level() {
                                    ::log::__private_api_log(::core::fmt::Arguments::new_v1(&[""],
                                                                                            &match (&"Process return address",)
                                                                                                 {
                                                                                                 (arg0,)
                                                                                                 =>
                                                                                                 [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                               ::core::fmt::Debug::fmt)],
                                                                                             }),
                                                             lvl,
                                                             &("rust_0bsd_riscv_kernel::process",
                                                               "rust_0bsd_riscv_kernel::process",
                                                               "src/process.rs",
                                                               284u32));
                                }
                            };
                            unsafe {
                                llvm_asm!(r"
			li a7, 1
			# Trigger a software interrupt
			csrr t0, sip
			# Set SSIP
			ori t0, t0, 2
			csrw sip, t0
		":
                                     :  : "a7", "t0" : "volatile")
                            }
                        }
                        pub fn new_supervisor_process(function: fn())
                         -> usize {
                            new_supervisor_process_int(function as usize, 0)
                        }
                        pub fn new_supervisor_process_argument(function:
                                                                   fn(usize),
                                                               a0: usize)
                         -> usize {
                            new_supervisor_process_int(function as usize, a0)
                        }
                        pub fn new_supervisor_process_with_name(function:
                                                                    fn(),
                                                                name: String)
                         -> usize {
                            let pid = new_supervisor_process(function);
                            try_get_process(&pid).write().name = Some(name);
                            pid
                        }
                        pub fn delete_process(pid: usize) {
                            use_boot_frame_if_necessary(&*try_get_process(&pid).read().trap_frame
                                                            as _);
                            PROCESSES.write().remove(&pid);
                        }
                        pub fn weak_get_process(pid: &usize)
                         -> Weak<RwLock<Process>> {
                            PROCESSES.read().get(pid).map(|arc|
                                                              Arc::downgrade(arc)).unwrap_or_default()
                        }
                        pub fn try_get_process(pid: &usize)
                         -> Arc<RwLock<Process>> {
                            PROCESSES.read().get(pid).unwrap().clone()
                        }
                        /// Gets the amount of processes that aren't idle processes and are still alive
                        /// Right now the way that it checks for idle processes is that it checks if their name starts with "Idle"
                        /// TODO use a better method
                        pub fn useful_process_count() -> usize {
                            PROCESSES.read().iter().filter(|(k, v)|
                                                               v.read().name.as_ref().map(|s|
                                                                                              !s.starts_with("Idle ")).unwrap_or(false)).count()
                        }
                        pub fn idle_entry_point() {
                            cpu::wfi();
                            unsafe { do_supervisor_syscall_0(1) };
                        }
                        pub fn idle_forever_entry_point() {
                            loop  { cpu::wfi(); }
                        }
                        /// Starts a process that wfi()s once, immediately switches to the process, then exits. 
                        /// Must be called from an interrupt context.
                        pub fn idle() -> ! {
                            use alloc::format;
                            let this_process =
                                weak_get_process(&Process::this_pid()).upgrade();
                            if let Some(process) = this_process {
                                let mut process = process.write();
                                {
                                    let lvl = ::log::Level::Info;
                                    if lvl <= ::log::STATIC_MAX_LEVEL &&
                                           lvl <= ::log::max_level() {
                                        ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["F "],
                                                                                                &match (&read_sscratch(),)
                                                                                                     {
                                                                                                     (arg0,)
                                                                                                     =>
                                                                                                     [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                   ::core::fmt::Debug::fmt)],
                                                                                                 }),
                                                                 lvl,
                                                                 &("rust_0bsd_riscv_kernel::process",
                                                                   "rust_0bsd_riscv_kernel::process",
                                                                   "src/process.rs",
                                                                   363u32));
                                    }
                                };
                                crate::trap::use_boot_frame_if_necessary(&*process.trap_frame
                                                                             as
                                                                             _);
                                {
                                    let lvl = ::log::Level::Info;
                                    if lvl <= ::log::STATIC_MAX_LEVEL &&
                                           lvl <= ::log::max_level() {
                                        ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["F "],
                                                                                                &match (&read_sscratch(),)
                                                                                                     {
                                                                                                     (arg0,)
                                                                                                     =>
                                                                                                     [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                   ::core::fmt::Debug::fmt)],
                                                                                                 }),
                                                                 lvl,
                                                                 &("rust_0bsd_riscv_kernel::process",
                                                                   "rust_0bsd_riscv_kernel::process",
                                                                   "src/process.rs",
                                                                   365u32));
                                    }
                                };
                                process.state = ProcessState::Pending;
                            } else {
                                {
                                    let lvl = ::log::Level::Info;
                                    if lvl <= ::log::STATIC_MAX_LEVEL &&
                                           lvl <= ::log::max_level() {
                                        ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["No process"],
                                                                                                &match ()
                                                                                                     {
                                                                                                     ()
                                                                                                     =>
                                                                                                     [],
                                                                                                 }),
                                                                 lvl,
                                                                 &("rust_0bsd_riscv_kernel::process",
                                                                   "rust_0bsd_riscv_kernel::process",
                                                                   "src/process.rs",
                                                                   368u32));
                                    }
                                };
                            }
                            let pid =
                                new_supervisor_process_with_name(idle_entry_point,
                                                                 {
                                                                     let res =
                                                                         ::alloc::fmt::format(::core::fmt::Arguments::new_v1(&["Idle process for hart "],
                                                                                                                             &match (&load_hartid(),)
                                                                                                                                  {
                                                                                                                                  (arg0,)
                                                                                                                                  =>
                                                                                                                                  [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                ::core::fmt::Display::fmt)],
                                                                                                                              }));
                                                                     res
                                                                 });
                            schedule_next_slice(1);
                            context_switch::context_switch(&pid)
                        }
                    }
                    pub mod test_task {
                        //! The functions here are tasks that can be run to make sure that complex kernel tasks
                        //! won't crash 
                        use core::ops::{BitAnd, BitXor};
                        use core::task::Context;
                        use core::pin::Pin;
                        use alloc::{collections::BTreeSet, vec::Vec};
                        use crate::asm::do_supervisor_syscall_0;
                        use crate::cpu::read_sie;
                        use crate::drivers::traits::block::GenericBlockDevice;
                        use crate::drivers::virtio::VirtioDriver;
                        use crate::drivers::virtio::block::VirtioBlockDevice;
                        use crate::external_interrupt::ExternalInterruptHandler;
                        use crate::{cpu, fdt, process};
                        fn twist(value: &mut usize) -> usize {
                            *value =
                                value.wrapping_add(#[cfg(target_arch =
                                                         "riscv64")] {
                                                                         0x902392093222
                                                                     }).bitxor(0b10101110101).bitand(0xFF);
                            *value
                        }
                        pub fn test_task() {
                            let mut sieve = Vec::new();
                            let mut not_removed = BTreeSet::new();
                            for i in 0..500 {
                                sieve.push(false);
                                if i > 1 { not_removed.insert(i); }
                            }
                            for idx in 2..sieve.len() {
                                if sieve[idx] { continue ; }
                                let mut jdx = idx * 2;
                                while jdx < 500 {
                                    sieve[jdx] = true;
                                    jdx += idx;
                                }
                                for maybe_prime_idx in 2..idx {
                                    if !sieve[maybe_prime_idx] &&
                                           not_removed.contains(&maybe_prime_idx)
                                       {
                                        {

                                            #[allow(unused_unsafe)]
                                            {
                                                use core::fmt::Write;
                                                let l =
                                                    crate::std_macros::OUTPUT_LOCK.lock();
                                                let _ =
                                                    unsafe {
                                                        crate::drivers::uart::Uart::new(0x1000_0000)
                                                    }.write_fmt(::core::fmt::Arguments::new_v1(&["Prime: ",
                                                                                                 "\r\n"],
                                                                                               &match (&maybe_prime_idx,)
                                                                                                    {
                                                                                                    (arg0,)
                                                                                                    =>
                                                                                                    [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                  ::core::fmt::Display::fmt)],
                                                                                                }));
                                            }
                                        };
                                        not_removed.remove(&maybe_prime_idx);
                                    }
                                }
                            }
                        }
                        pub fn test_task_2() {
                            let twisted_value = 0;
                            let mut vector_vec = Vec::with_capacity(10);
                            for i in 0..70 {
                                let mut v: Vec<usize> =
                                    Vec::with_capacity(twisted_value);
                                v.resize(twisted_value, 0);
                                for i in v.iter_mut() {
                                    *i = i as *mut usize as usize;
                                }
                                vector_vec.push(v);
                            };
                            for v in vector_vec.iter() {
                                for i in v.iter() {
                                    if !(*i == i as *const usize as usize) {
                                        ::core::panicking::panic("assertion failed: *i == i as *const usize as usize")
                                    };
                                }
                            }
                            drop(vector_vec);
                            use crate::timeout::TimeoutFuture;
                            let mut future =
                                TimeoutFuture{for_time:
                                                  cpu::get_time() +
                                                      10_000_000,};
                            let waker =
                                process::Process::this().write().construct_waker();
                            use core::future::Future;
                            {
                                let lvl = ::log::Level::Info;
                                if lvl <= ::log::STATIC_MAX_LEVEL &&
                                       lvl <= ::log::max_level() {
                                    ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["Scheduling timeout.."],
                                                                                            &match ()
                                                                                                 {
                                                                                                 ()
                                                                                                 =>
                                                                                                 [],
                                                                                             }),
                                                             lvl,
                                                             &("rust_0bsd_riscv_kernel::test_task",
                                                               "rust_0bsd_riscv_kernel::test_task",
                                                               "src/test_task.rs",
                                                               79u32));
                                }
                            };
                            while TimeoutFuture::poll(Pin::new(&mut future),
                                                      &mut Context::from_waker(&waker))
                                      == core::task::Poll::Pending {
                                trigger_yield_syscall();
                            }
                            {
                                let lvl = ::log::Level::Info;
                                if lvl <= ::log::STATIC_MAX_LEVEL &&
                                       lvl <= ::log::max_level() {
                                    ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["Timeout finished"],
                                                                                            &match ()
                                                                                                 {
                                                                                                 ()
                                                                                                 =>
                                                                                                 [],
                                                                                             }),
                                                             lvl,
                                                             &("rust_0bsd_riscv_kernel::test_task",
                                                               "rust_0bsd_riscv_kernel::test_task",
                                                               "src/test_task.rs",
                                                               87u32));
                                }
                            };
                        }
                        pub fn test_task_3() {
                            {
                                {
                                    let lvl = ::log::Level::Info;
                                    if lvl <= ::log::STATIC_MAX_LEVEL &&
                                           lvl <= ::log::max_level() {
                                        ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["Waiting"],
                                                                                                &match ()
                                                                                                     {
                                                                                                     ()
                                                                                                     =>
                                                                                                     [],
                                                                                                 }),
                                                                 lvl,
                                                                 &("rust_0bsd_riscv_kernel::test_task",
                                                                   "rust_0bsd_riscv_kernel::test_task",
                                                                   "src/test_task.rs",
                                                                   93u32));
                                    }
                                };
                                use crate::lock::shared::Mutex;
                                let m = Mutex::new(0);
                                let m1 = m.lock();
                                drop(m1);
                                let m2 = m.lock();
                            }
                            use alloc::sync::Arc;
                            use crate::lock::shared::Mutex;
                            use core::any::Any;
                            let exec = crate::future::Executor::new();
                            let block =
                                async 
                                    {
                                        {
                                            let lvl = ::log::Level::Info;
                                            if lvl <= ::log::STATIC_MAX_LEVEL
                                                   &&
                                                   lvl <= ::log::max_level() {
                                                ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["Waiting"],
                                                                                                        &match ()
                                                                                                             {
                                                                                                             ()
                                                                                                             =>
                                                                                                             [],
                                                                                                         }),
                                                                         lvl,
                                                                         &("rust_0bsd_riscv_kernel::test_task",
                                                                           "rust_0bsd_riscv_kernel::test_task",
                                                                           "src/test_task.rs",
                                                                           109u32));
                                            }
                                        };
                                        crate::device_setup::is_done_future().await;
                                        {
                                            let lvl = ::log::Level::Info;
                                            if lvl <= ::log::STATIC_MAX_LEVEL
                                                   &&
                                                   lvl <= ::log::max_level() {
                                                ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["Waited"],
                                                                                                        &match ()
                                                                                                             {
                                                                                                             ()
                                                                                                             =>
                                                                                                             [],
                                                                                                         }),
                                                                         lvl,
                                                                         &("rust_0bsd_riscv_kernel::test_task",
                                                                           "rust_0bsd_riscv_kernel::test_task",
                                                                           "src/test_task.rs",
                                                                           111u32));
                                            }
                                        };
                                        use crate::drivers::traits::block::BlockDevice;
                                        let block_device:
                                                Arc<Mutex<VirtioBlockDevice>>;
                                        {
                                            let guard = fdt::root().read();
                                            let block_device_node =
                                                guard.get("soc/virtio_mmio@10008000").unwrap();
                                            let lock =
                                                block_device_node.kernel_struct.read();
                                            let bd =
                                                lock.as_ref().unwrap().downcast_ref::<(VirtioDriver,
                                                                                       Option<ExternalInterruptHandler>)>();
                                            let bd =
                                                if let VirtioDriver::Block(bd)
                                                       =
                                                       &bd.as_ref().unwrap().0
                                                   {
                                                    bd
                                                } else {
                                                    ::core::panicking::panic("Block device not found!");
                                                };
                                            let bd = bd.lock();
                                            block_device =
                                                bd.this.upgrade().unwrap();
                                        }
                                        let mut v: Vec<u8> = Vec::new();
                                        v.resize(512, 0);
                                        v[0] = 65;
                                        v[1] = 67;
                                        let request =
                                            block_device.lock().create_request(0,
                                                                               v.into_boxed_slice(),
                                                                               true);
                                        let buf = request.await;
                                        {
                                            let lvl = ::log::Level::Info;
                                            if lvl <= ::log::STATIC_MAX_LEVEL
                                                   &&
                                                   lvl <= ::log::max_level() {
                                                ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["Read "],
                                                                                                        &match (&buf,)
                                                                                                             {
                                                                                                             (arg0,)
                                                                                                             =>
                                                                                                             [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                           ::core::fmt::Debug::fmt)],
                                                                                                         }),
                                                                         lvl,
                                                                         &("rust_0bsd_riscv_kernel::test_task",
                                                                           "rust_0bsd_riscv_kernel::test_task",
                                                                           "src/test_task.rs",
                                                                           146u32));
                                            }
                                        };
                                        let mut v: Vec<u8> = Vec::new();
                                        v.resize(512, 0);
                                        let request =
                                            block_device.lock().create_request(0,
                                                                               v.into_boxed_slice(),
                                                                               false);
                                        let buf = request.await;
                                    };
                            let block = Box::pin(block);
                            let mut block = Box::new(block);
                            use alloc::boxed::Box;
                            let waker =
                                crate::process::Process::this().read().construct_waker();
                            let mut context = Context::from_waker(&waker);
                            use core::future::Future;
                            while core::task::Poll::Pending ==
                                      Pin::new(&mut block).poll(&mut context)
                                  {
                                unsafe { do_supervisor_syscall_0(2) };
                            }
                            {
                                let lvl = ::log::Level::Info;
                                if lvl <= ::log::STATIC_MAX_LEVEL &&
                                       lvl <= ::log::max_level() {
                                    ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["Ending"],
                                                                                            &match ()
                                                                                                 {
                                                                                                 ()
                                                                                                 =>
                                                                                                 [],
                                                                                             }),
                                                             lvl,
                                                             &("rust_0bsd_riscv_kernel::test_task",
                                                               "rust_0bsd_riscv_kernel::test_task",
                                                               "src/test_task.rs",
                                                               173u32));
                                }
                            };
                        }
                        #[inline]
                        fn trigger_yield_syscall() {
                            unsafe {
                                llvm_asm!(r"
			li a7, 2
			# Trigger a timer interrupt
			csrr t0, sip
			# Set SSIP
			ori t0, t0, 2
			csrw sip, t0
		":
                                     :  : "a7", "t0" : "volatile")
                            }
                        }
                    }
                    pub mod logger {
                        use log::{Record, Level, Metadata};
                        use crate::{lock::shared::Mutex,
                                    trap::in_interrupt_context};
                        pub struct ColorfulLogger {
                            lock: Mutex<()>,
                        }
                        impl log::Log for ColorfulLogger {
                            fn enabled(&self, metadata: &Metadata) -> bool {
                                metadata.level() <= Level::Info
                            }
                            fn log(&self, record: &Record) {
                                if self.enabled(record.metadata()) {
                                    let prefix =
                                        match record.level() {
                                            Level::Error => {
                                                "\x1b[91;1mERROR\x1b[0m"
                                            }
                                            Level::Warn => {
                                                "\x1b[93;1mWARN \x1b[0m"
                                            }
                                            Level::Info => {
                                                "\x1b[1mINFO \x1b[0m"
                                            }
                                            Level::Debug => {
                                                "\x1b[1;96mDEBUG\x1b[0m"
                                            }
                                            Level::Trace => {
                                                "\x1b[96mTRACE\x1b[0m"
                                            }
                                        };
                                    let guard =
                                        if !in_interrupt_context() {
                                            Some(self.lock.lock())
                                        } else { None };
                                    {

                                        #[allow(unused_unsafe)]
                                        {
                                            use core::fmt::Write;
                                            let l =
                                                crate::std_macros::OUTPUT_LOCK.lock();
                                            let _ =
                                                unsafe {
                                                    crate::drivers::uart::Uart::new(0x1000_0000)
                                                }.write_fmt(::core::fmt::Arguments::new_v1(&["",
                                                                                             " [",
                                                                                             "] ",
                                                                                             "\r\n"],
                                                                                           &match (&&record.module_path().unwrap_or("")["rust_0bsd_riscv_kernel".len()..],
                                                                                                   &prefix,
                                                                                                   &record.args())
                                                                                                {
                                                                                                (arg0,
                                                                                                 arg1,
                                                                                                 arg2)
                                                                                                =>
                                                                                                [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                              ::core::fmt::Display::fmt),
                                                                                                 ::core::fmt::ArgumentV1::new(arg1,
                                                                                                                              ::core::fmt::Display::fmt),
                                                                                                 ::core::fmt::ArgumentV1::new(arg2,
                                                                                                                              ::core::fmt::Display::fmt)],
                                                                                            }));
                                        }
                                    };
                                }
                            }
                            fn flush(&self) { }
                        }
                        pub static KERNEL_LOGGER: ColorfulLogger =
                            ColorfulLogger{lock: Mutex::new(()),};
                    }
                    pub mod fdt {
                        use core::mem::MaybeUninit;
                        use core::any::Any;
                        use itertools::Itertools;
                        use alloc::{collections::{BTreeMap, VecDeque}, format,
                                    string::String};
                        use cstr_core::CStr;
                        use alloc::borrow::ToOwned;
                        use crate::lock::shared::RwLock;
                        use crate::alloc::string::ToString;
                        use num_enum::{FromPrimitive, IntoPrimitive};
                        static mut DEVICE_TREE_BASE: *const FdtHeader =
                            core::ptr::null();
                        static mut DEVICE_TREE_ROOT: MaybeUninit<RwLock<Node>>
                         =
                            MaybeUninit::uninit();
                        pub struct Node {
                            pub name: &'static str,
                            pub unit_address: Option<usize>,
                            pub children: BTreeMap<&'static str,
                                                   BTreeMap<Option<usize>,
                                                            Node>>,
                            pub properties: BTreeMap<&'static str,
                                                     PropertyValue<'static>>,
                            /// This holds an arbitrary datatype which is the representation of this type in the kernel
                            /// This can (and is) used to own objects with functions that are called on interrupts
                            pub kernel_struct: RwLock<Option<alloc::boxed::Box<dyn Any +
                                                                               Send>>>,
                        }
                        #[automatically_derived]
                        #[allow(unused_qualifications)]
                        impl ::core::fmt::Debug for Node {
                            fn fmt(&self, f: &mut ::core::fmt::Formatter)
                             -> ::core::fmt::Result {
                                match *self {
                                    Node {
                                    name: ref __self_0_0,
                                    unit_address: ref __self_0_1,
                                    children: ref __self_0_2,
                                    properties: ref __self_0_3,
                                    kernel_struct: ref __self_0_4 } => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_struct(f,
                                                                                      "Node");
                                        let _ =
                                            ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                            "name",
                                                                            &&(*__self_0_0));
                                        let _ =
                                            ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                            "unit_address",
                                                                            &&(*__self_0_1));
                                        let _ =
                                            ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                            "children",
                                                                            &&(*__self_0_2));
                                        let _ =
                                            ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                            "properties",
                                                                            &&(*__self_0_3));
                                        let _ =
                                            ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                                            "kernel_struct",
                                                                            &&(*__self_0_4));
                                        ::core::fmt::DebugStruct::finish(debug_trait_builder)
                                    }
                                }
                            }
                        }
                        impl Node {
                            pub fn new(token_name: &'static str) -> Self {
                                let mut name_iter = token_name.splitn(2, '@');
                                Self{name: name_iter.next().unwrap(),
                                     unit_address:
                                         name_iter.next().map(|s|
                                                                  usize::from_str_radix(s,
                                                                                        16).unwrap_or(0)),
                                     children: BTreeMap::new(),
                                     properties: BTreeMap::new(),
                                     kernel_struct: RwLock::new(None),}
                            }
                            pub fn get<'this>(&'this self, path: &'this str)
                             -> Option<&'this Node> {
                                let mut path_iter = path.splitn(2, '/');
                                let first_component =
                                    path_iter.next().unwrap_or(path);
                                let child;
                                if first_component.contains('@') {
                                    let (name, address) =
                                        first_component.splitn(2,
                                                               '@').next_tuple().unwrap();
                                    if address.is_empty() {
                                        child =
                                            self.children.get(name)?.values().next()?
                                    } else {
                                        child =
                                            self.children.get(name)?.get(&Some(usize::from_str_radix(address,
                                                                                                     16).unwrap_or(0)))?;
                                    }
                                } else {
                                    child =
                                        self.children.get(first_component)?.get(&None)?;
                                }
                                if let Some(rest) = path_iter.next() {
                                    child.get(rest)
                                } else { Some(child) }
                            }
                            pub fn children_names(&self)
                             -> VecDeque<&'static str> {
                                self.children.values().flatten().map(|s|
                                                                         s.1.name).collect()
                            }
                            pub fn children_names_address(&self)
                             -> VecDeque<String> {
                                self.children.values().flatten().map(|s|
                                                                         if let Some(addr)
                                                                                =
                                                                                s.1.unit_address
                                                                            {
                                                                             {
                                                                                 let res =
                                                                                     ::alloc::fmt::format(::core::fmt::Arguments::new_v1(&["",
                                                                                                                                           "@"],
                                                                                                                                         &match (&s.1.name,
                                                                                                                                                 &addr)
                                                                                                                                              {
                                                                                                                                              (arg0,
                                                                                                                                               arg1)
                                                                                                                                              =>
                                                                                                                                              [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                            ::core::fmt::Display::fmt),
                                                                                                                                               ::core::fmt::ArgumentV1::new(arg1,
                                                                                                                                                                            ::core::fmt::LowerHex::fmt)],
                                                                                                                                          }));
                                                                                 res
                                                                             }
                                                                         } else {
                                                                             s.1.name.to_owned()
                                                                         }).collect()
                            }
                            pub fn children(&self) -> VecDeque<&Node> {
                                self.children.values().flatten().map(|s|
                                                                         s.1).collect()
                            }
                            pub fn children_mut(&mut self)
                             -> VecDeque<&mut Node> {
                                self.children.values_mut().flatten().map(|s|
                                                                             s.1).collect()
                            }
                            fn insert_child(&mut self, other: Self) {
                                match self.children.get_mut(other.name) {
                                    Some(d) => {
                                        d.insert(other.unit_address, other);
                                    }
                                    None => {
                                        let mut map = BTreeMap::new();
                                        let name = other.name;
                                        map.insert(other.unit_address, other);
                                        self.children.insert(name, map);
                                    }
                                }
                            }
                            pub fn pretty(&self, indent: usize) {
                                match self.unit_address {
                                    Some(e) => {

                                        #[allow(unused_unsafe)]
                                        {
                                            use core::fmt::Write;
                                            let l =
                                                crate::std_macros::OUTPUT_LOCK.lock();
                                            let _ =
                                                unsafe {
                                                    crate::drivers::uart::Uart::new(0x1000_0000)
                                                }.write_fmt(::core::fmt::Arguments::new_v1(&["",
                                                                                             "",
                                                                                             "@",
                                                                                             " {\r\n"],
                                                                                           &match (&"    ".repeat(indent),
                                                                                                   &self.name,
                                                                                                   &e)
                                                                                                {
                                                                                                (arg0,
                                                                                                 arg1,
                                                                                                 arg2)
                                                                                                =>
                                                                                                [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                              ::core::fmt::Display::fmt),
                                                                                                 ::core::fmt::ArgumentV1::new(arg1,
                                                                                                                              ::core::fmt::Display::fmt),
                                                                                                 ::core::fmt::ArgumentV1::new(arg2,
                                                                                                                              ::core::fmt::LowerHex::fmt)],
                                                                                            }));
                                        }
                                    }
                                    None => {

                                        #[allow(unused_unsafe)]
                                        {
                                            use core::fmt::Write;
                                            let l =
                                                crate::std_macros::OUTPUT_LOCK.lock();
                                            let _ =
                                                unsafe {
                                                    crate::drivers::uart::Uart::new(0x1000_0000)
                                                }.write_fmt(::core::fmt::Arguments::new_v1(&["",
                                                                                             "",
                                                                                             " {\r\n"],
                                                                                           &match (&"    ".repeat(indent),
                                                                                                   &self.name)
                                                                                                {
                                                                                                (arg0,
                                                                                                 arg1)
                                                                                                =>
                                                                                                [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                              ::core::fmt::Display::fmt),
                                                                                                 ::core::fmt::ArgumentV1::new(arg1,
                                                                                                                              ::core::fmt::Display::fmt)],
                                                                                            }));
                                        }
                                    }
                                };
                                for (k, v) in self.properties.iter() {
                                    {

                                        #[allow(unused_unsafe)]
                                        {
                                            use core::fmt::Write;
                                            let l =
                                                crate::std_macros::OUTPUT_LOCK.lock();
                                            let _ =
                                                unsafe {
                                                    crate::drivers::uart::Uart::new(0x1000_0000)
                                                }.write_fmt(::core::fmt::Arguments::new_v1(&["",
                                                                                             "",
                                                                                             " = ",
                                                                                             "\r\n"],
                                                                                           &match (&"    ".repeat(indent
                                                                                                                      +
                                                                                                                      1),
                                                                                                   &k,
                                                                                                   &v.as_str())
                                                                                                {
                                                                                                (arg0,
                                                                                                 arg1,
                                                                                                 arg2)
                                                                                                =>
                                                                                                [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                              ::core::fmt::Display::fmt),
                                                                                                 ::core::fmt::ArgumentV1::new(arg1,
                                                                                                                              ::core::fmt::Display::fmt),
                                                                                                 ::core::fmt::ArgumentV1::new(arg2,
                                                                                                                              ::core::fmt::Display::fmt)],
                                                                                            }));
                                        }
                                    }
                                }
                                for i in self.children().iter() {
                                    i.pretty(indent + 1);
                                }
                                {

                                    #[allow(unused_unsafe)]
                                    {
                                        use core::fmt::Write;
                                        let l =
                                            crate::std_macros::OUTPUT_LOCK.lock();
                                        let _ =
                                            unsafe {
                                                crate::drivers::uart::Uart::new(0x1000_0000)
                                            }.write_fmt(::core::fmt::Arguments::new_v1(&["",
                                                                                         "}\r\n"],
                                                                                       &match (&"    ".repeat(indent),)
                                                                                            {
                                                                                            (arg0,)
                                                                                            =>
                                                                                            [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                          ::core::fmt::Display::fmt)],
                                                                                        }));
                                    }
                                };
                            }
                            pub fn walk<F: FnMut(&'static Node)>(&'static self,
                                                                 closure:
                                                                     &mut F) {
                                closure(self);
                                for i in self.children() { i.walk(closure); };
                            }
                            pub fn walk_nonstatic<F: FnMut(&Node)>(&self,
                                                                   closure:
                                                                       &mut F) {
                                closure(self);
                                for i in self.children() {
                                    i.walk_nonstatic(closure);
                                };
                            }
                            /// The lifetimes for this function aren't <'static> because that would be an aliasing rule violation
                            /// (closure mutably borrows Node forever so no one else can mut borrow it again )
                            pub fn walk_mut<F: FnMut(&mut Node)>(&mut self,
                                                                 closure:
                                                                     &mut F) {
                                closure(self);
                                for i in self.children_mut() {
                                    i.walk_mut(closure);
                                };
                            }
                        }
                        impl Drop for Node {
                            fn drop(&mut self) {
                                {
                                    let lvl = ::log::Level::Warn;
                                    if lvl <= ::log::STATIC_MAX_LEVEL &&
                                           lvl <= ::log::max_level() {
                                        ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["Node dropped (this doesn\'t happen with the current implementation)"],
                                                                                                &match ()
                                                                                                     {
                                                                                                     ()
                                                                                                     =>
                                                                                                     [],
                                                                                                 }),
                                                                 lvl,
                                                                 &("rust_0bsd_riscv_kernel::fdt",
                                                                   "rust_0bsd_riscv_kernel::fdt",
                                                                   "src/fdt.rs",
                                                                   142u32));
                                    }
                                };
                            }
                        }
                        #[allow(non_camel_case_types)]
                        pub enum PropertyValue<'data> {
                            Empty,
                            PHandleRaw(u32),
                            PHandle(&'static Node),
                            u32(u32),
                            u64(u64),
                            PropSpecific(&'data [u8]),
                            String(&'data str),
                            StringList(VecDeque<&'data str>),
                        }
                        #[automatically_derived]
                        #[allow(unused_qualifications)]
                        #[allow(non_camel_case_types)]
                        impl <'data> ::core::fmt::Debug for
                         PropertyValue<'data> {
                            fn fmt(&self, f: &mut ::core::fmt::Formatter)
                             -> ::core::fmt::Result {
                                match (&*self,) {
                                    (&PropertyValue::Empty,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "Empty");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&PropertyValue::PHandleRaw(ref __self_0),)
                                    => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "PHandleRaw");
                                        let _ =
                                            ::core::fmt::DebugTuple::field(debug_trait_builder,
                                                                           &&(*__self_0));
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&PropertyValue::PHandle(ref __self_0),)
                                    => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "PHandle");
                                        let _ =
                                            ::core::fmt::DebugTuple::field(debug_trait_builder,
                                                                           &&(*__self_0));
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&PropertyValue::u32(ref __self_0),) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "u32");
                                        let _ =
                                            ::core::fmt::DebugTuple::field(debug_trait_builder,
                                                                           &&(*__self_0));
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&PropertyValue::u64(ref __self_0),) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "u64");
                                        let _ =
                                            ::core::fmt::DebugTuple::field(debug_trait_builder,
                                                                           &&(*__self_0));
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&PropertyValue::PropSpecific(ref __self_0),)
                                    => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "PropSpecific");
                                        let _ =
                                            ::core::fmt::DebugTuple::field(debug_trait_builder,
                                                                           &&(*__self_0));
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&PropertyValue::String(ref __self_0),) =>
                                    {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "String");
                                        let _ =
                                            ::core::fmt::DebugTuple::field(debug_trait_builder,
                                                                           &&(*__self_0));
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&PropertyValue::StringList(ref __self_0),)
                                    => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "StringList");
                                        let _ =
                                            ::core::fmt::DebugTuple::field(debug_trait_builder,
                                                                           &&(*__self_0));
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                }
                            }
                        }
                        impl <'data> PropertyValue<'data> {
                            fn guess(value: &'data [u8], name: Option<&str>)
                             -> PropertyValue<'data> {
                                if value.is_empty() {
                                    return PropertyValue::Empty;
                                }
                                let mut is_string = true;
                                let mut is_string_list = false;
                                let mut prev_is_zero = true;
                                for &i in &value[..value.len() - 1] {
                                    if i == 0 && !prev_is_zero {
                                        is_string_list = true;
                                        prev_is_zero = true;
                                    } else if i < 0x20 {
                                        is_string = false;
                                        is_string_list = false;
                                        break
                                    } else { prev_is_zero = false; }
                                }
                                if is_string_list {
                                    let mut list = VecDeque::new();
                                    let mut last_index = 0;
                                    for (index, &i) in
                                        value.iter().enumerate() {
                                        if i == 0 {
                                            list.push_back(core::str::from_utf8(&value[last_index..index]).unwrap());
                                            last_index = index + 1;
                                        }
                                    }
                                    return PropertyValue::StringList(list)
                                } else if is_string {
                                    return PropertyValue::String(core::str::from_utf8(&value[..value.len()
                                                                                                   -
                                                                                                   1]).unwrap());
                                }
                                if value.len() == 8 {
                                    return PropertyValue::u64(unsafe {
                                                                  *(value.as_ptr()
                                                                        as
                                                                        *const u64)
                                                              }.swap_bytes());
                                };
                                if value.len() == 4 {
                                    if name.unwrap_or("") ==
                                           "interrupt-parent" {
                                        return PropertyValue::PHandleRaw(unsafe
                                                                         {
                                                                             *(value.as_ptr()
                                                                                   as
                                                                                   *const u32)
                                                                         }.swap_bytes());
                                    }
                                    return PropertyValue::u32(unsafe {
                                                                  *(value.as_ptr()
                                                                        as
                                                                        *const u32)
                                                              }.swap_bytes());
                                };
                                PropertyValue::PropSpecific(value)
                            }
                            fn as_str(&self) -> String {
                                match self {
                                    Self::Empty => "true".to_string(),
                                    Self::PropSpecific(val) => {
                                        let res =
                                            ::alloc::fmt::format(::core::fmt::Arguments::new_v1(&[""],
                                                                                                &match (&val,)
                                                                                                     {
                                                                                                     (arg0,)
                                                                                                     =>
                                                                                                     [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                   ::core::fmt::Debug::fmt)],
                                                                                                 }));
                                        res
                                    }
                                    Self::StringList(val) => {
                                        let res =
                                            ::alloc::fmt::format(::core::fmt::Arguments::new_v1(&[""],
                                                                                                &match (&val,)
                                                                                                     {
                                                                                                     (arg0,)
                                                                                                     =>
                                                                                                     [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                   ::core::fmt::Debug::fmt)],
                                                                                                 }));
                                        res
                                    }
                                    Self::String(val) => val.to_string(),
                                    Self::u32(val) | Self::PHandleRaw(val) =>
                                    {
                                        let res =
                                            ::alloc::fmt::format(::core::fmt::Arguments::new_v1(&[""],
                                                                                                &match (&val,)
                                                                                                     {
                                                                                                     (arg0,)
                                                                                                     =>
                                                                                                     [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                   ::core::fmt::LowerHex::fmt)],
                                                                                                 }));
                                        res
                                    }
                                    Self::PHandle(val) => {
                                        let res =
                                            ::alloc::fmt::format(::core::fmt::Arguments::new_v1(&["<",
                                                                                                  ">"],
                                                                                                &match (&val.name,)
                                                                                                     {
                                                                                                     (arg0,)
                                                                                                     =>
                                                                                                     [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                   ::core::fmt::Display::fmt)],
                                                                                                 }));
                                        res
                                    }
                                    Self::u64(val) => {
                                        let res =
                                            ::alloc::fmt::format(::core::fmt::Arguments::new_v1(&[""],
                                                                                                &match (&val,)
                                                                                                     {
                                                                                                     (arg0,)
                                                                                                     =>
                                                                                                     [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                   ::core::fmt::LowerHex::fmt)],
                                                                                                 }));
                                        res
                                    }
                                }
                            }
                        }
                        #[repr(C)]
                        pub struct FdtHeader {
                            magic: u32,
                            total_size: u32,
                            offset_dt_struct: u32,
                            offset_dt_strings: u32,
                            offset_memory_reservemap: u32,
                            version: u32,
                            last_compatible_version: u32,
                            boot_cpuid: u32,
                            size_dt_strings: u32,
                            size_dt_struct: u32,
                        }
                        #[repr(u32)]
                        pub enum StructureToken {
                            BeginNode = 1,
                            EndNode = 2,
                            Prop = 3,
                            Nop = 4,
                            End = 9,

                            #[num_enum(default)]
                            Unknown,
                        }
                        #[automatically_derived]
                        #[allow(unused_qualifications)]
                        impl ::core::fmt::Debug for StructureToken {
                            fn fmt(&self, f: &mut ::core::fmt::Formatter)
                             -> ::core::fmt::Result {
                                match (&*self,) {
                                    (&StructureToken::BeginNode,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "BeginNode");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&StructureToken::EndNode,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "EndNode");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&StructureToken::Prop,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "Prop");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&StructureToken::Nop,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "Nop");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&StructureToken::End,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "End");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                    (&StructureToken::Unknown,) => {
                                        let debug_trait_builder =
                                            &mut ::core::fmt::Formatter::debug_tuple(f,
                                                                                     "Unknown");
                                        ::core::fmt::DebugTuple::finish(debug_trait_builder)
                                    }
                                }
                            }
                        }
                        #[automatically_derived]
                        #[allow(unused_qualifications)]
                        impl ::core::marker::Copy for StructureToken { }
                        #[automatically_derived]
                        #[allow(unused_qualifications)]
                        impl ::core::clone::Clone for StructureToken {
                            #[inline]
                            fn clone(&self) -> StructureToken { { *self } }
                        }
                        impl From<StructureToken> for u32 {
                            #[inline]
                            fn from(enum_value: StructureToken) -> Self {
                                enum_value as Self
                            }
                        }
                        impl ::num_enum::FromPrimitive for StructureToken {
                            type Primitive = u32;
                            fn from_primitive(number: Self::Primitive)
                             -> Self {
                                #![allow(non_upper_case_globals)]
                                const BeginNode__num_enum_0__: u32 = 1;
                                const EndNode__num_enum_0__: u32 = 2;
                                const Prop__num_enum_0__: u32 = 3;
                                const Nop__num_enum_0__: u32 = 4;
                                const End__num_enum_0__: u32 = 9;
                                const Unknown__num_enum_0__: u32 =
                                    u32::wrapping_add(9, 1);

                                #[deny(unreachable_patterns)]
                                match number {
                                    BeginNode__num_enum_0__ =>
                                    Self::BeginNode,
                                    EndNode__num_enum_0__ => Self::EndNode,
                                    Prop__num_enum_0__ => Self::Prop,
                                    Nop__num_enum_0__ => Self::Nop,
                                    End__num_enum_0__ => Self::End,
                                    Unknown__num_enum_0__ =>
                                    Self::Unknown,
                                                  #[allow(unreachable_patterns)]
                                                  _ => Self::Unknown,
                                }
                            }
                        }
                        impl ::core::convert::From<u32> for StructureToken {
                            #[inline]
                            fn from(number: u32) -> Self {
                                ::num_enum::FromPrimitive::from_primitive(number)
                            }
                        }
                        impl ::num_enum::TryFromPrimitive for StructureToken {
                            type Primitive = u32;
                            const NAME: &'static str = "StructureToken";
                            #[inline]
                            fn try_from_primitive(number: Self::Primitive)
                             ->
                                 ::core::result::Result<Self,
                                                        ::num_enum::TryFromPrimitiveError<Self>> {
                                Ok(::num_enum::FromPrimitive::from_primitive(number))
                            }
                        }
                        #[repr(C)]
                        pub struct PropToken {
                            token: u32,
                            len: u32,
                            name_offset: u32,
                        }
                        pub unsafe fn get_string(offset: usize)
                         -> &'static str {
                            CStr::from_ptr((DEVICE_TREE_BASE as
                                                *const u8).add((*DEVICE_TREE_BASE).offset_dt_strings.swap_bytes()
                                                                   as
                                                                   usize).add(offset
                                                                                  as
                                                                                  usize)).to_str().unwrap()
                        }
                        fn build(mut token: *const u32) -> Node {
                            let mut node_stack: VecDeque<Node> =
                                VecDeque::new();
                            let mut current_node = None;
                            loop  {
                                match StructureToken::from(unsafe {
                                                               (*token).swap_bytes()
                                                           }) {
                                    StructureToken::BeginNode => {
                                        if let Some(t) = current_node.take() {
                                            node_stack.push_back(t);
                                        }
                                        let name =
                                            unsafe {
                                                CStr::from_ptr(token.add(1) as
                                                                   *const u8)
                                            };
                                        unsafe {
                                            token =
                                                (token as
                                                     *const u8).add(name.to_bytes_with_nul().len()
                                                                        + 1)
                                                    as *const u32
                                        };
                                        let remain = token as usize % 4;
                                        let mut div: usize =
                                            token as usize / 4;
                                        if remain != 0 { div += 1; }
                                        token = (div * 4) as *const u32;
                                        current_node =
                                            Some(Node::new(name.to_str().unwrap()));
                                    }
                                    StructureToken::EndNode => {
                                        if let Some(t) = current_node.take() {
                                            if let Some(mut last) =
                                                   node_stack.pop_back() {
                                                last.insert_child(t);
                                                current_node.insert(last);
                                            } else { current_node = Some(t) }
                                        }
                                        token = unsafe { token.add(1) };
                                    }
                                    StructureToken::Prop => {
                                        let struc = token as *mut PropToken;
                                        let name =
                                            unsafe {
                                                get_string((*struc).name_offset.swap_bytes()
                                                               as usize)
                                            };
                                        let len =
                                            unsafe {
                                                (*struc).len.swap_bytes()
                                            } as usize;
                                        token = unsafe { token.add(3) };
                                        if let Some(ref mut t) = current_node
                                           {
                                            t.properties.insert(name,
                                                                PropertyValue::guess(unsafe
                                                                                     {
                                                                                         core::slice::from_raw_parts(token
                                                                                                                         as
                                                                                                                         *const u8,
                                                                                                                     len)
                                                                                     },
                                                                                     Some(name)));
                                        }
                                        token =
                                            unsafe {
                                                (token as *const u8).add(len)
                                                    as *const u32
                                            };
                                        let remain = token as usize % 4;
                                        let mut div: usize =
                                            token as usize / 4;
                                        if remain != 0 { div += 1; }
                                        token = (div * 4) as *const u32;
                                    }
                                    StructureToken::Nop => {
                                        token = unsafe { token.add(1) };
                                    }
                                    StructureToken::End => { break  }
                                    StructureToken::Unknown => {
                                        token = unsafe { token.add(1) };
                                    }
                                };
                            }
                            current_node.unwrap()
                        }
                        pub fn root() -> &'static RwLock<Node> {
                            unsafe { DEVICE_TREE_ROOT.assume_init_ref() }
                        }
                        /// Replace PHandleRaw attributes with PHandle attributes and put references to the nodes inside of them
                        pub fn link_phandles() {
                            let mut phandles: BTreeMap<u32, &'static Node> =
                                BTreeMap::new();
                            let borrow =
                                unsafe {
                                    DEVICE_TREE_ROOT.assume_init_mut()
                                }.get_mut();
                            borrow.walk(&mut (|node: &'static Node|
                                                  {
                                                      if let Some(PropertyValue::u32(phandle))
                                                             =
                                                             node.properties.get("phandle")
                                                         {
                                                          phandles.insert(*phandle,
                                                                          node);
                                                      }
                                                  }));
                            unsafe {
                                DEVICE_TREE_ROOT.assume_init_mut()
                            }.write().walk_mut(&mut (|node: &mut Node|
                                                         {
                                                             for value in
                                                                 node.properties.values_mut()
                                                                 {
                                                                 if let PropertyValue::PHandleRaw(handle)
                                                                        =
                                                                        value
                                                                    {
                                                                     if let Some(target_node)
                                                                            =
                                                                            phandles.get(handle)
                                                                        {
                                                                         *value
                                                                             =
                                                                             PropertyValue::PHandle(target_node)
                                                                     } else {
                                                                         {
                                                                             let lvl =
                                                                                 ::log::Level::Warn;
                                                                             if lvl
                                                                                    <=
                                                                                    ::log::STATIC_MAX_LEVEL
                                                                                    &&
                                                                                    lvl
                                                                                        <=
                                                                                        ::log::max_level()
                                                                                {
                                                                                 ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["fdt: unknown phandle "],
                                                                                                                                         &match (&handle,)
                                                                                                                                              {
                                                                                                                                              (arg0,)
                                                                                                                                              =>
                                                                                                                                              [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                                                            ::core::fmt::Display::fmt)],
                                                                                                                                          }),
                                                                                                          lvl,
                                                                                                          &("rust_0bsd_riscv_kernel::fdt",
                                                                                                            "rust_0bsd_riscv_kernel::fdt",
                                                                                                            "src/fdt.rs",
                                                                                                            376u32));
                                                                             }
                                                                         }
                                                                     }
                                                                 }
                                                             }
                                                         }));
                        }
                        pub fn init(header_addr: *const FdtHeader)
                         -> &'static RwLock<Node> {
                            unsafe { DEVICE_TREE_BASE = header_addr };
                            let token_addr =
                                unsafe {
                                    (DEVICE_TREE_BASE as
                                         *const u8).add((*DEVICE_TREE_BASE).offset_dt_struct.swap_bytes()
                                                            as usize)
                                } as *const u32;
                            let root_tree = build(token_addr);
                            unsafe {
                                DEVICE_TREE_ROOT =
                                    MaybeUninit::new(RwLock::new(root_tree))
                            };
                            link_phandles();
                            root()
                        }
                    }
                    pub mod plic {
                        use crate::cpu::load_hartid;
                        #[inline(always)]
                        fn get_context_number() -> usize {
                            1 + load_hartid() * 2
                        }
                        pub struct Plic0 {
                            base_addr: usize,
                            context_number: usize,
                        }
                        impl Plic0 {
                            pub fn new_with_fdt() -> Self {
                                Self{base_addr:
                                         crate::fdt::root().read().get("soc/plic@").unwrap().unit_address.unwrap(),
                                     context_number: get_context_number(),}
                            }
                            pub fn new_with_addr(base_addr: usize) -> Self {
                                Self{base_addr,
                                     context_number: get_context_number(),}
                            }
                            pub fn set_priority(&self, interrupt: u32,
                                                priority: u32) {
                                unsafe {
                                    (self.base_addr as
                                         *mut u32).add(interrupt as
                                                           usize).write_volatile(priority)
                                };
                            }
                            pub fn set_enabled(&self, interrupt: u32,
                                               enable: bool) {
                                let enables_base = self.base_addr + 0x2000;
                                let target_base =
                                    enables_base + self.context_number * 0x80;
                                let target_base = target_base as *mut u32;
                                let this_register =
                                    unsafe {
                                        target_base.add((interrupt / 32) as
                                                            usize)
                                    };
                                if enable {
                                    let flag =
                                        (enable as u32) << (interrupt % 32);
                                    unsafe {
                                        this_register.write_volatile(this_register.read_volatile()
                                                                         |
                                                                         flag);
                                    };
                                } else {
                                    let flag =
                                        !((enable as u32) <<
                                              (interrupt % 32));
                                    unsafe {
                                        this_register.write_volatile(this_register.read_volatile()
                                                                         &
                                                                         flag);
                                    };
                                }
                            }
                            pub fn set_threshold(&self, threshold: u32) {
                                let threshold_base =
                                    self.base_addr + 0x20_0000;
                                let target_base =
                                    threshold_base +
                                        self.context_number * 0x1000;
                                let target_base = target_base as *mut u32;
                                unsafe {
                                    target_base.write_volatile(threshold)
                                };
                            }
                            pub fn claim_highest_priority(&self) -> u32 {
                                let cc_base = self.base_addr + 0x20_0004;
                                let target_base =
                                    cc_base + self.context_number * 0x1000;
                                let target_base = target_base as *mut u32;
                                unsafe { target_base.read_volatile() }
                            }
                            pub fn complete(&self, interrupt: u32) {
                                let cc_base = self.base_addr + 0x20_0004;
                                let target_base =
                                    cc_base + self.context_number * 0x1000;
                                let target_base = target_base as *mut u32;
                                unsafe {
                                    target_base.write_volatile(interrupt)
                                }
                            }
                        }
                    }
                    pub mod hart {
                        use alloc::{collections::BTreeMap, sync::Arc,
                                    boxed::{Box}};
                        use core::{pin::Pin, sync::atomic::AtomicBool};
                        use crate::lock::shared::{RwLock};
                        use aligned::{A16, Aligned};
                        use crate::{cpu::{self, load_hartid}, plic::Plic0,
                                    process::{self, TASK_STACK_SIZE},
                                    s_trap_vector, sbi,
                                    scheduler::schedule_next_slice,
                                    timer_queue, trap::TrapFrame};
                        pub struct HartMeta {
                            pub plic: Plic0,
                            pub boot_stack: Option<Box<Aligned<A16,
                                                               [u8; TASK_STACK_SIZE]>>>,
                            pub boot_frame: RwLock<Pin<Box<TrapFrame>>>,
                            pub is_panicking: AtomicBool,
                        }
                        pub static HART_META:
                         RwLock<BTreeMap<usize, Arc<HartMeta>>> =
                            RwLock::new(BTreeMap::new());
                        pub fn get_hart_meta(hartid: usize)
                         -> Option<Arc<HartMeta>> {
                            HART_META.read().get(&hartid).cloned()
                        }
                        /// # Safety 
                        /// When sscratch contains a valid trap frame
                        pub unsafe fn add_boot_hart(trap_frame: TrapFrame) {
                            let meta =
                                HartMeta{plic: Plic0::new_with_fdt(),
                                         boot_stack: None,
                                         boot_frame:
                                             RwLock::new(Pin::new(Box::new(trap_frame))),
                                         is_panicking:
                                             AtomicBool::new(false),};
                            HART_META.write().insert(load_hartid(),
                                                     Arc::new(meta));
                        }
                        /// Must be run from a recently created hart
                        pub fn add_this_secondary_hart(hartid: usize,
                                                       interrupt_sp: usize) {
                            let mut trap_frame =
                                Pin::new(Box::new(TrapFrame::zeroed_interrupt_context()));
                            trap_frame.pid = 0;
                            trap_frame.hartid = hartid;
                            trap_frame.interrupt_stack = interrupt_sp;
                            unsafe {
                                cpu::write_sscratch(Pin::as_ref(&trap_frame).get_ref()
                                                        as *const TrapFrame as
                                                        usize)
                            };
                            trap_frame.pid = process::allocate_pid();
                            HART_META.write().insert(load_hartid(),
                                                     Arc::new(HartMeta{plic:
                                                                           Plic0::new_with_fdt(),
                                                                       boot_stack:
                                                                           None,
                                                                       boot_frame:
                                                                           RwLock::new(trap_frame),
                                                                       is_panicking:
                                                                           AtomicBool::new(false),}));
                        }
                        pub fn get_this_hart_meta() -> Option<Arc<HartMeta>> {
                            get_hart_meta(load_hartid())
                        }
                        /// # Safety
                        /// start_addr must be a function that is sound and sets up harts correctly
                        pub unsafe fn start_all_harts(start_addr: usize) {
                            for hartid in 0.. {
                                match sbi::hart_get_status(hartid) {
                                    Err(e) => { break ; }
                                    Ok(status) => {
                                        if status == 1 {
                                            let process_stack =
                                                ::alloc::vec::from_elem(0,
                                                                        4096 *
                                                                            2).into_boxed_slice();
                                            sbi::start_hart(hartid,
                                                            start_addr,
                                                            process_stack.as_ptr()
                                                                as usize +
                                                                (4096 * 2) -
                                                                0x10).expect("Starting hart failed!");
                                            Box::leak(process_stack);
                                        }
                                    }
                                }
                            }
                        }
                        #[no_mangle]
                        fn hart_entry(hartid: usize, interrupt_stack: usize)
                         -> ! {
                            add_this_secondary_hart(hartid, interrupt_stack);
                            timer_queue::init_hart();
                            unsafe {
                                cpu::write_stvec(s_trap_vector as usize)
                            };
                            unsafe {
                                use cpu::csr::*;
                                cpu::write_sie(SSIE | STIE | SEIE);
                                let mut sstatus: usize;
                                llvm_asm!("csrr $0, sstatus": "=r"(sstatus) :
                                    : );
                                sstatus |= 1 << 1;
                                llvm_asm!("csrw sstatus, $0":  : "r"(sstatus)
                                    :  : "volatile");
                            }
                            schedule_next_slice(0);
                            timer_queue::schedule_next();
                            loop  { cpu::wfi() };
                        }
                    }
                    pub mod timeout {
                        /// This module uses time interrupts to create a "timeout" future
                        use alloc::vec::Vec;
                        use core::task::{Waker, Poll};
                        use core::future::Future;
                        use crate::lock::shared::RwLock;
                        use crate::{cpu, timer_queue};
                        /// This Vec is sorted by item.0.for_time
                        /// TODO use binary heap?
                        pub static WAITING_TIMEOUTS:
                         RwLock<Vec<(TimeoutFuture, Waker)>> =
                            RwLock::new(Vec::new());
                        pub struct TimeoutFuture {
                            pub for_time: u64,
                        }
                        #[automatically_derived]
                        #[allow(unused_qualifications)]
                        impl ::core::marker::Copy for TimeoutFuture { }
                        #[automatically_derived]
                        #[allow(unused_qualifications)]
                        impl ::core::clone::Clone for TimeoutFuture {
                            #[inline]
                            fn clone(&self) -> TimeoutFuture {
                                {
                                    let _:
                                            ::core::clone::AssertParamIsClone<u64>;
                                    *self
                                }
                            }
                        }
                        impl Future for TimeoutFuture {
                            type Output = u64;
                            fn poll(self: core::pin::Pin<&mut Self>,
                                    cx: &mut core::task::Context<'_>)
                             -> Poll<Self::Output> {
                                if cpu::get_time() >= self.for_time {
                                    Poll::Ready(cpu::get_time())
                                } else {
                                    let index =
                                        WAITING_TIMEOUTS.write().binary_search_by(|s|
                                                                                      {
                                                                                          self.for_time.cmp(&s.0.for_time)
                                                                                      });
                                    let insert_position;
                                    match index {
                                        Ok(index) => { return Poll::Pending; }
                                        Err(index) => {
                                            insert_position = index
                                        }
                                    }
                                    WAITING_TIMEOUTS.write().insert(insert_position,
                                                                    (*self,
                                                                     cx.waker().clone()));
                                    use crate::timer_queue::{TimerEvent,
                                                             TimerEventCause};
                                    timer_queue::schedule_at(TimerEvent{instant:
                                                                            self.for_time,
                                                                        cause:
                                                                            TimerEventCause::TimeoutFuture,});
                                    Poll::Pending
                                }
                            }
                        }
                        pub fn on_timer_event(instant: u64) {
                            {
                                let lvl = ::log::Level::Info;
                                if lvl <= ::log::STATIC_MAX_LEVEL &&
                                       lvl <= ::log::max_level() {
                                    ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["Timer event for us "],
                                                                                            &match (&instant,)
                                                                                                 {
                                                                                                 (arg0,)
                                                                                                 =>
                                                                                                 [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                               ::core::fmt::Display::fmt)],
                                                                                             }),
                                                             lvl,
                                                             &("rust_0bsd_riscv_kernel::timeout",
                                                               "rust_0bsd_riscv_kernel::timeout",
                                                               "src/timeout.rs",
                                                               63u32));
                                }
                            };
                            let mut lock = WAITING_TIMEOUTS.write();
                            let mut max_remove_index = 0;
                            for (idx, (future, waker)) in
                                lock.iter().enumerate() {
                                if future.for_time <= instant {
                                    waker.wake_by_ref();
                                    max_remove_index = idx + 1;
                                }
                            }
                            for i in 0..max_remove_index {

                                #[allow(unused_must_use)]
                                lock.remove(i);
                            }
                        }
                    }
                    pub mod lock {
                        #![allow(clippy :: declare_interior_mutable_const)]
                        pub mod spin {
                            pub mod mutex {
                                use lock_api::{RawMutex, GuardSend};
                                use core::sync::atomic::{AtomicBool, Ordering,
                                                         AtomicUsize};
                                use crate::{cpu::load_hartid,
                                            trap::in_interrupt_context};
                                pub const NO_HART: usize = usize::MAX;
                                pub struct RawSpinlock {
                                    locked: AtomicBool,
                                    #[cfg(debug_assertions)]
                                    locker_hartid: AtomicUsize,
                                }
                                unsafe impl RawMutex for RawSpinlock {
                                    #[cfg(debug_assertions)]
                                    const INIT: RawSpinlock =
                                        RawSpinlock{locked:
                                                        AtomicBool::new(false),
                                                    locker_hartid:
                                                        AtomicUsize::new(NO_HART),};
                                    type GuardMarker = GuardSend;
                                    fn lock(&self) {

                                        #[cfg(debug_assertions)]
                                        if self.locked.load(Ordering::Acquire)
                                           {
                                            if self.locker_hartid.load(Ordering::Acquire)
                                                   == load_hartid() {
                                            }
                                        }
                                        while self.try_lock() == false {
                                            core::hint::spin_loop()
                                        }
                                    }
                                    fn try_lock(&self) -> bool {
                                        self.locked.compare_exchange(false,
                                                                     true,
                                                                     Ordering::SeqCst,
                                                                     Ordering::SeqCst).is_ok()
                                    }
                                    unsafe fn unlock(&self) {
                                        self.locked.store(false,
                                                          Ordering::Release);
                                    }
                                }
                                pub type Mutex<T> =
                                 lock_api::Mutex<RawSpinlock, T>;
                                pub type MutexGuard<'a, T> =
                                 lock_api::MutexGuard<'a, RawSpinlock, T>;
                            }
                            pub mod rwlock {
                                use lock_api::{RawRwLock, GuardSend};
                                use core::sync::atomic::{AtomicUsize,
                                                         Ordering};
                                use crate::{cpu::load_hartid,
                                            trap::in_interrupt_context};
                                const SHARED: usize = 1 << 1;
                                const WRITER: usize = 1 << 0;
                                pub struct RawSpinRwLock {
                                    value: AtomicUsize,
                                }
                                unsafe impl RawRwLock for RawSpinRwLock {
                                    const INIT: RawSpinRwLock =
                                        Self{value: AtomicUsize::new(0),};
                                    type GuardMarker = GuardSend;
                                    fn lock_shared(&self) {
                                        while self.try_lock_shared() == false
                                              {
                                        }
                                    }
                                    fn try_lock_shared(&self) -> bool {
                                        let mut outdated_value =
                                            self.value.load(Ordering::SeqCst);
                                        if outdated_value & WRITER != 0 {
                                            return false;
                                        }
                                        while let Err(e) =
                                                  self.value.compare_exchange(outdated_value,
                                                                              outdated_value
                                                                                  +
                                                                                  SHARED,
                                                                              Ordering::SeqCst,
                                                                              Ordering::SeqCst)
                                              {
                                            outdated_value =
                                                self.value.load(Ordering::SeqCst);
                                            if outdated_value & WRITER != 0 {
                                                return false;
                                            }
                                        };
                                        return true;
                                    }
                                    unsafe fn unlock_shared(&self) {
                                        if self.value.load(Ordering::SeqCst)
                                               == 0 {
                                            loop  { };
                                        };
                                        self.value.fetch_sub(SHARED,
                                                             Ordering::SeqCst);
                                    }
                                    fn lock_exclusive(&self) {
                                        while self.try_lock_exclusive() ==
                                                  false {
                                        }
                                    }
                                    fn try_lock_exclusive(&self) -> bool {
                                        let mut outdated_value =
                                            self.value.load(Ordering::SeqCst);
                                        if outdated_value != 0 {
                                            return false;
                                        }
                                        while let Err(e) =
                                                  self.value.compare_exchange(outdated_value,
                                                                              outdated_value
                                                                                  +
                                                                                  WRITER,
                                                                              Ordering::SeqCst,
                                                                              Ordering::SeqCst)
                                              {
                                            outdated_value =
                                                self.value.load(Ordering::SeqCst);
                                            if outdated_value != 0 {
                                                return false;
                                            }
                                        };
                                        return true;
                                    }
                                    unsafe fn unlock_exclusive(&self) {
                                        if self.value.load(Ordering::SeqCst)
                                               == 0 {
                                            loop  { };
                                        };
                                        self.value.fetch_sub(WRITER,
                                                             Ordering::SeqCst);
                                    }
                                    fn is_locked(&self) -> bool {
                                        return self.value.load(Ordering::SeqCst)
                                                   != 0
                                    }
                                }
                                pub type RwLock<T> =
                                 lock_api::RwLock<RawSpinRwLock, T>;
                                pub type RwLockReadGuard<'a, T> =
                                 lock_api::RwLockReadGuard<'a, RawSpinRwLock,
                                                           T>;
                                pub type RwLockWriteGuard<'a, T> =
                                 lock_api::RwLockWriteGuard<'a, RawSpinRwLock,
                                                            T>;
                            }
                            pub use mutex::{Mutex, MutexGuard};
                            pub use rwlock::{RwLock, RwLockReadGuard,
                                             RwLockWriteGuard};
                            pub use mutex::RawSpinlock as RawMutex;
                            pub use rwlock::RawSpinRwLock as RawRwLock;
                        }
                        pub mod shared {
                            pub mod mutex {
                                use lock_api::{RawMutex, GuardSend};
                                use core::sync::atomic::{AtomicBool, Ordering,
                                                         AtomicUsize};
                                use crate::{cpu::load_hartid,
                                            trap::in_interrupt_context};
                                pub use super::super::spin::RawMutex as
                                        RawSpinlock;
                                use super::{lock_and_disable_interrupts,
                                            unlock_and_enable_interrupts_if_necessary};
                                pub const NO_HART: usize = usize::MAX;
                                pub struct RawSharedLock {
                                    internal: RawSpinlock,
                                    old_sie: AtomicUsize,
                                }
                                unsafe impl RawMutex for RawSharedLock {
                                    const INIT: RawSharedLock =
                                        RawSharedLock{internal:
                                                          RawSpinlock::INIT,
                                                      old_sie:
                                                          AtomicUsize::new(0),};
                                    type GuardMarker = GuardSend;
                                    fn lock(&self) {
                                        lock_and_disable_interrupts();
                                        self.internal.lock()
                                    }
                                    fn try_lock(&self) -> bool {
                                        if self.internal.try_lock() {
                                            lock_and_disable_interrupts();
                                            true
                                        } else { false }
                                    }
                                    unsafe fn unlock(&self) {
                                        self.internal.unlock();
                                        unlock_and_enable_interrupts_if_necessary();
                                    }
                                }
                                pub type Mutex<T> =
                                 lock_api::Mutex<RawSharedLock, T>;
                                pub type MutexGuard<'a, T> =
                                 lock_api::MutexGuard<'a, RawSharedLock, T>;
                            }
                            pub mod rwlock {
                                use lock_api::{RawRwLock, GuardSend};
                                use core::sync::atomic::{AtomicUsize,
                                                         Ordering};
                                use alloc::vec::Vec;
                                use crate::{cpu::load_hartid, hart::HART_META,
                                            lock::shared::{lock_and_disable_interrupts,
                                                           unlock_and_enable_interrupts_if_necessary},
                                            trap::in_interrupt_context};
                                pub use super::super::spin::RawRwLock as
                                        RawSpinRwLock;
                                use super::super::spin::RwLock as SpinRwLock;
                                pub struct RawSharedRwLock {
                                    internal: RawSpinRwLock,
                                }
                                unsafe impl RawRwLock for RawSharedRwLock {
                                    const INIT: RawSharedRwLock =
                                        Self{internal: RawSpinRwLock::INIT,};
                                    type GuardMarker = GuardSend;
                                    fn lock_shared(&self) {
                                        {
                                            let lvl = ::log::Level::Debug;
                                            if lvl <= ::log::STATIC_MAX_LEVEL
                                                   &&
                                                   lvl <= ::log::max_level() {
                                                ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["",
                                                                                                          " ",
                                                                                                          " Lock shared"],
                                                                                                        &match (&load_hartid(),
                                                                                                                &((self
                                                                                                                       as
                                                                                                                       *const Self
                                                                                                                       as
                                                                                                                       usize)
                                                                                                                      &
                                                                                                                      0xffffffff))
                                                                                                             {
                                                                                                             (arg0,
                                                                                                              arg1)
                                                                                                             =>
                                                                                                             [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                           ::core::fmt::Display::fmt),
                                                                                                              ::core::fmt::ArgumentV1::new(arg1,
                                                                                                                                           ::core::fmt::LowerHex::fmt)],
                                                                                                         }),
                                                                         lvl,
                                                                         &("rust_0bsd_riscv_kernel::lock::shared::rwlock",
                                                                           "rust_0bsd_riscv_kernel::lock::shared::rwlock",
                                                                           "src/lock/shared/rwlock.rs",
                                                                           19u32));
                                            }
                                        };
                                        lock_and_disable_interrupts();
                                        self.internal.lock_shared()
                                    }
                                    fn try_lock_shared(&self) -> bool {
                                        if self.internal.try_lock_shared() {
                                            lock_and_disable_interrupts();
                                            true
                                        } else { false }
                                    }
                                    unsafe fn unlock_shared(&self) {
                                        {
                                            let lvl = ::log::Level::Debug;
                                            if lvl <= ::log::STATIC_MAX_LEVEL
                                                   &&
                                                   lvl <= ::log::max_level() {
                                                ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["",
                                                                                                          " ",
                                                                                                          " Unlock shared"],
                                                                                                        &match (&load_hartid(),
                                                                                                                &((self
                                                                                                                       as
                                                                                                                       *const Self
                                                                                                                       as
                                                                                                                       usize)
                                                                                                                      &
                                                                                                                      0xffffffff))
                                                                                                             {
                                                                                                             (arg0,
                                                                                                              arg1)
                                                                                                             =>
                                                                                                             [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                           ::core::fmt::Display::fmt),
                                                                                                              ::core::fmt::ArgumentV1::new(arg1,
                                                                                                                                           ::core::fmt::LowerHex::fmt)],
                                                                                                         }),
                                                                         lvl,
                                                                         &("rust_0bsd_riscv_kernel::lock::shared::rwlock",
                                                                           "rust_0bsd_riscv_kernel::lock::shared::rwlock",
                                                                           "src/lock/shared/rwlock.rs",
                                                                           34u32));
                                            }
                                        };
                                        self.internal.unlock_shared();
                                        unlock_and_enable_interrupts_if_necessary();
                                    }
                                    fn lock_exclusive(&self) {
                                        {
                                            let lvl = ::log::Level::Debug;
                                            if lvl <= ::log::STATIC_MAX_LEVEL
                                                   &&
                                                   lvl <= ::log::max_level() {
                                                ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["",
                                                                                                          " ",
                                                                                                          " Lock exclusive "],
                                                                                                        &match (&load_hartid(),
                                                                                                                &((self
                                                                                                                       as
                                                                                                                       *const Self
                                                                                                                       as
                                                                                                                       usize)
                                                                                                                      &
                                                                                                                      0xffffffff),
                                                                                                                &self.internal.is_locked())
                                                                                                             {
                                                                                                             (arg0,
                                                                                                              arg1,
                                                                                                              arg2)
                                                                                                             =>
                                                                                                             [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                           ::core::fmt::Display::fmt),
                                                                                                              ::core::fmt::ArgumentV1::new(arg1,
                                                                                                                                           ::core::fmt::LowerHex::fmt),
                                                                                                              ::core::fmt::ArgumentV1::new(arg2,
                                                                                                                                           ::core::fmt::Display::fmt)],
                                                                                                         }),
                                                                         lvl,
                                                                         &("rust_0bsd_riscv_kernel::lock::shared::rwlock",
                                                                           "rust_0bsd_riscv_kernel::lock::shared::rwlock",
                                                                           "src/lock/shared/rwlock.rs",
                                                                           40u32));
                                            }
                                        };
                                        lock_and_disable_interrupts();
                                        self.internal.lock_exclusive()
                                    }
                                    fn try_lock_exclusive(&self) -> bool {
                                        if self.internal.try_lock_exclusive()
                                           {
                                            lock_and_disable_interrupts();
                                            true
                                        } else { false }
                                    }
                                    unsafe fn unlock_exclusive(&self) {
                                        {
                                            let lvl = ::log::Level::Debug;
                                            if lvl <= ::log::STATIC_MAX_LEVEL
                                                   &&
                                                   lvl <= ::log::max_level() {
                                                ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["",
                                                                                                          " ",
                                                                                                          " Unlock exclusive"],
                                                                                                        &match (&load_hartid(),
                                                                                                                &((self
                                                                                                                       as
                                                                                                                       *const Self
                                                                                                                       as
                                                                                                                       usize)
                                                                                                                      &
                                                                                                                      0xffffffff))
                                                                                                             {
                                                                                                             (arg0,
                                                                                                              arg1)
                                                                                                             =>
                                                                                                             [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                           ::core::fmt::Display::fmt),
                                                                                                              ::core::fmt::ArgumentV1::new(arg1,
                                                                                                                                           ::core::fmt::LowerHex::fmt)],
                                                                                                         }),
                                                                         lvl,
                                                                         &("rust_0bsd_riscv_kernel::lock::shared::rwlock",
                                                                           "rust_0bsd_riscv_kernel::lock::shared::rwlock",
                                                                           "src/lock/shared/rwlock.rs",
                                                                           55u32));
                                            }
                                        };
                                        self.internal.unlock_exclusive();
                                        unlock_and_enable_interrupts_if_necessary();
                                    }
                                }
                                pub type RwLock<T> =
                                 lock_api::RwLock<RawSharedRwLock, T>;
                                pub type RwLockReadGuard<'a, T> =
                                 lock_api::RwLockReadGuard<'a,
                                                           RawSharedRwLock,
                                                           T>;
                                pub type RwLockWriteGuard<'a, T> =
                                 lock_api::RwLockWriteGuard<'a,
                                                            RawSharedRwLock,
                                                            T>;
                            }
                            use core::sync::atomic::{AtomicUsize, Ordering};
                            use alloc::vec::Vec;
                            pub use mutex::{Mutex, MutexGuard};
                            pub use rwlock::{RwLock, RwLockReadGuard,
                                             RwLockWriteGuard};
                            pub use mutex::RawSharedLock as RawMutex;
                            pub use rwlock::RawSharedRwLock as RawRwLock;
                            use super::spin::RwLock as SpinRwLock;
                            use crate::{trap::in_interrupt_context,
                                        cpu::load_hartid};
                            static HART_LOCK_COUNT:
                             SpinRwLock<Vec<AtomicUsize>> =
                                SpinRwLock::new(Vec::new());
                            pub fn create_hart_lock_count_entry_if_necessary(idx:
                                                                                 &usize)
                             -> bool {
                                if idx < &HART_LOCK_COUNT.read().len() {
                                    false
                                } else {
                                    HART_LOCK_COUNT.write().resize_with(idx +
                                                                            1,
                                                                        ||
                                                                            {
                                                                                AtomicUsize::new(0)
                                                                            });
                                    true
                                }
                            }
                            #[inline]
                            pub fn lock_and_disable_interrupts() {
                                if !in_interrupt_context() {
                                    unsafe { crate::cpu::write_sie(0) };
                                    create_hart_lock_count_entry_if_necessary(&load_hartid());
                                    HART_LOCK_COUNT.read()[load_hartid()].fetch_add(1,
                                                                                    Ordering::AcqRel);
                                }
                            }
                            #[inline]
                            pub fn unlock_and_enable_interrupts_if_necessary() {
                                if !in_interrupt_context() {
                                    if HART_LOCK_COUNT.read()[load_hartid()].fetch_sub(1,
                                                                                       Ordering::AcqRel)
                                           == 1 {
                                        unsafe {
                                            crate::cpu::write_sie(0x222)
                                        };
                                    }
                                }
                            }
                            #[no_mangle]
                            pub extern "C" fn this_hart_lock_count()
                             -> usize {
                                HART_LOCK_COUNT.read()[load_hartid()].load(Ordering::Acquire)
                            }
                        }
                        pub mod interrupt {
                            pub mod mutex {
                                /// Locks that are used exclusively in interrupt contexts
                                /// Essentially a spinlock
                                use lock_api::{RawMutex, GuardSend};
                                use core::sync::atomic::{AtomicBool, Ordering,
                                                         AtomicUsize};
                                use crate::{cpu::load_hartid,
                                            trap::in_interrupt_context};
                                pub use super::super::spin::RawMutex as
                                        RawSpinlock;
                                pub const NO_HART: usize = usize::MAX;
                                pub struct RawInterruptLock {
                                    internal: RawSpinlock,
                                }
                                unsafe impl RawMutex for RawInterruptLock {
                                    const INIT: RawInterruptLock =
                                        RawInterruptLock{internal:
                                                             RawSpinlock::INIT,};
                                    type GuardMarker = GuardSend;
                                    fn lock(&self) {
                                        if !in_interrupt_context() {
                                            ::core::panicking::panic("assertion failed: in_interrupt_context()")
                                        };
                                        self.internal.lock()
                                    }
                                    fn try_lock(&self) -> bool {
                                        self.internal.try_lock()
                                    }
                                    unsafe fn unlock(&self) {
                                        if !in_interrupt_context() {
                                            ::core::panicking::panic("assertion failed: in_interrupt_context()")
                                        };
                                        self.internal.unlock()
                                    }
                                }
                                pub type Mutex<T> =
                                 lock_api::Mutex<RawInterruptLock, T>;
                                pub type MutexGuard<'a, T> =
                                 lock_api::MutexGuard<'a, RawInterruptLock,
                                                      T>;
                            }
                            pub mod rwlock {
                                use lock_api::{RawRwLock, GuardSend};
                                use core::sync::atomic::{AtomicUsize,
                                                         Ordering};
                                use crate::trap::in_interrupt_context;
                                pub use super::super::spin::RawRwLock as
                                        RawSpinRwLock;
                                pub struct RawInterruptRwLock {
                                    internal: RawSpinRwLock,
                                }
                                unsafe impl RawRwLock for RawInterruptRwLock {
                                    const INIT: RawInterruptRwLock =
                                        Self{internal: RawSpinRwLock::INIT,};
                                    type GuardMarker = GuardSend;
                                    fn lock_shared(&self) {
                                        if !in_interrupt_context() {
                                            ::core::panicking::panic("assertion failed: in_interrupt_context()")
                                        };
                                        self.internal.lock_shared()
                                    }
                                    fn try_lock_shared(&self) -> bool {
                                        self.internal.try_lock_shared()
                                    }
                                    unsafe fn unlock_shared(&self) {
                                        self.internal.unlock_shared()
                                    }
                                    fn lock_exclusive(&self) {
                                        if !in_interrupt_context() {
                                            ::core::panicking::panic("assertion failed: in_interrupt_context()")
                                        };
                                        self.internal.lock_shared()
                                    }
                                    fn try_lock_exclusive(&self) -> bool {
                                        self.internal.try_lock_exclusive()
                                    }
                                    unsafe fn unlock_exclusive(&self) {
                                        if !in_interrupt_context() {
                                            ::core::panicking::panic("assertion failed: in_interrupt_context()")
                                        };
                                        self.internal.unlock_exclusive()
                                    }
                                }
                                pub type RwLock<T> =
                                 lock_api::RwLock<RawInterruptRwLock, T>;
                                pub type RwLockReadGuard<'a, T> =
                                 lock_api::RwLockReadGuard<'a,
                                                           RawInterruptRwLock,
                                                           T>;
                                pub type RwLockWriteGuard<'a, T> =
                                 lock_api::RwLockWriteGuard<'a,
                                                            RawInterruptRwLock,
                                                            T>;
                            }
                            pub use mutex::{Mutex, MutexGuard};
                            pub use rwlock::{RwLock, RwLockReadGuard,
                                             RwLockWriteGuard};
                        }
                    }
