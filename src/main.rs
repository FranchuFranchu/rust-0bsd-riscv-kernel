#![feature(llvm_asm,
	asm,
	naked_functions,
	const_trait_impl,
	default_alloc_error_handler,
	const_mut_refs,
	panic_info_message,
	maybe_uninit_ref,
	option_result_unwrap_unchecked,
	unchecked_math,
	global_asm)]
#![no_std]
#![no_main]
#![allow(incomplete_features)]
#![allow(dead_code)]
#![allow(unused_variables)]

use core::panic::PanicInfo;

#[macro_use]
extern crate log;




extern crate alloc;
use core::ffi::c_void;

// Linker symbols
extern "C" {
	// Linker symbols
	static _heap_start: c_void;
	static _heap_end: c_void;
	
	fn s_trap_vector();
}


use allocator::{LinkedListAllocator, MutexWrapper};



#[global_allocator]
static ALLOCATOR: MutexWrapper<LinkedListAllocator> =
    MutexWrapper::new(LinkedListAllocator::new());


#[macro_use]
pub mod std_macros;
pub mod process;
pub mod test_task;
pub mod logger;

// The boot frame is the frame that is active in the boot thread
// It needs to be statically allocated because it has to be there before
// memory allocation is up and running
static mut BOOT_FRAME: trap::TrapFrame = trap::TrapFrame::zeroed();



#[no_mangle]
pub fn main(hartid: usize, opaque: usize) -> ! {
	if unsafe { BOOT_FRAME.pid } != 0 {
		panic!("main() called more than once!");
	}
	
	// SAFETY: We're the only hart, there's no way the data gets changed by someone else meanwhile
	unsafe { BOOT_FRAME.hartid = hartid }
	unsafe { BOOT_FRAME.pid = 1 }
	
	// SAFETY: BOOT_FRAME has a valid trap frame value so this doesn't break the rest of the kernel
	unsafe { crate::cpu::write_sscratch(&BOOT_FRAME as *const trap::TrapFrame as usize) }
	
	// Now, set up the logger
	log::set_logger(&logger::KERNEL_LOGGER).unwrap();
	log::set_max_level(log::LevelFilter::Info);
	
	
	
	// SAFETY: identity_map is valid when the root page is valid, which in this case is true
	// and paging is disabled now
	unsafe { paging::sv39::identity_map(&mut paging::ROOT_PAGE as *mut paging::Table) }
	
	// Initialize memory allocation
	
	
	
	let heap_end = unsafe {&_heap_end as *const c_void as usize};
	let heap_start = unsafe {&_heap_start as *const c_void as usize};
	// SAFETY: This relies on the assumption that heap_end and heap_start are valid addresses (which are provided by the linker script)
	// If they aren't, then it's pointless
	unsafe { ALLOCATOR.lock().init(heap_start, heap_end - heap_start) }; 
	
	// SAFETY: s_trap_vector is a valid trap vector
	unsafe { cpu::write_stvec(s_trap_vector as usize) };
	
	/*
	// Setup paging
	// SAFETY: If identity mapping did its thing right, then nothing should change
	unsafe { cpu::write_satp((&mut paging::ROOT_PAGE as *mut paging::Table as usize) >> 12 | cpu::csr::SATP_SV39) }
	
	
	cpu::fence_vma();
	*/
	
	
	
	// SAFETY: We're enabling interrupts, since we've set stvec already that's not dangerous
	unsafe { 
		use cpu::csr::*;
		// Enable software, external, and timer interrupts
		cpu::write_sie(SSIE | SEIE | STIE);
		
		let mut sstatus: usize;
		llvm_asm!("csrr $0, sstatus" : "=r"(sstatus));
		sstatus |= 3;
		llvm_asm!("csrw sstatus, $0" :: "r"(sstatus));
	}
	
	
	process::init();
	process::new_supervisor_process(test_task::test_task);
	process::new_supervisor_process(test_task::test_task_2);
	sbi::set_relative_timer(1);
	
	loop {
		cpu::wfi();
	}
}



#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    //let mut host_stderr = HStderr::new();
    
    // logs "panicked at '$reason', src/main.rs:27:4" to the host stderr
    //writeln!(host_stderr, "{}", info).ok();
    
    let fnomsg = format_args!("<no message>");
    let message = info.message().unwrap_or(&fnomsg);
    
    let trap_frame = cpu::read_sscratch();
    
    debug!("{:?}", trap_frame);
    
    // Check if trap frame is out of bounds (which means we can't read data from it)
    if (trap_frame as usize) > 0x80200000 && (trap_frame as usize) < (unsafe { &_heap_end } as *const c_void as usize) {
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
    
    // Disable ALL interrupts
    unsafe { 
		
		cpu::write_sie(0);
	}
    

    loop {
    	// Now, poll the UART until we get a Ctrl+C
    	// and then shutdown
    	match unsafe {crate::drivers::uart::Uart::new(0x1000_0000).get()} {
    		Some(3) => crate::sbi::shutdown(0),
    		_ => {},
    	}
    }
}

pub mod asm;
pub mod allocator;
#[macro_use]
pub mod cpu;
pub mod trap;
pub mod paging;
pub mod syscall;
pub mod drivers;
pub mod sbi;
pub mod scheduler;