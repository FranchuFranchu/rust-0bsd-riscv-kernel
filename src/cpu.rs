// this enum is from osblog
#[derive(Copy, Clone)]
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
	A0, /* 10 */
	A1,
	A2,
	A3,
	A4,
	A5,
	A6,
	A7,
	S2,
	S3,
	S4, /* 20 */
	S5,
	S6,
	S7,
	S8,
	S9,
	S10,
	S11,
	T3,
	T4,
	T5, /* 30 */
	T6
}

impl Registers {
	pub const fn idx(&self) -> usize {
		*self as usize
	}
}

#[inline(always)]
pub unsafe fn write_sie(value: usize) {
	llvm_asm!("csrw sie, $0" :: "r"(value) :: "volatile")
}

#[inline(always)]
pub unsafe fn write_sip(value: usize) {
	llvm_asm!("csrw sip, $0" :: "r"(value) :: "volatile")
}

#[inline(always)]
pub unsafe fn write_stvec(value: usize) {
	llvm_asm!("csrw stvec, $0" :: "r"(value) :: "volatile")
}

#[inline(always)]
pub unsafe fn write_satp(value: usize) {
	llvm_asm!("csrw satp, $0" :: "r"(value) :: "volatile")
}


#[inline(always)]
pub unsafe fn write_sstatus(value: usize) {
	llvm_asm!("csrw sstatus, $0" :: "r"(value) :: "volatile")
}

// This is unsafe because other parts of the kernel rely on sscratch being a valid pointer
#[inline(always)]
pub unsafe fn write_sscratch(value: usize) {
	llvm_asm!("csrw sscratch, $0" :: "r"(value) :: "volatile")
}


#[inline(always)]
pub fn read_sscratch() -> *mut crate::trap::TrapFrame {
	let value: usize;
	unsafe { llvm_asm!("csrr $0, sscratch" : "=r"(value) ::: "volatile") };
	value as _
}


#[inline(always)]
pub fn read_sp() -> usize {
	let value: usize;
	unsafe { llvm_asm!("mv $0, sp" : "=r"(value) ::: "volatile") };
	value
}

#[inline(always)]
pub fn read_sip() -> usize {
	let value: usize;
	unsafe { llvm_asm!("csrr $0, sip" : "=r"(value) ::: "volatile") };
	value
}

#[inline(always)]
pub fn read_sstatus() -> usize {
	let value: usize;
	unsafe { llvm_asm!("csrr $0, sstatus" : "=r"(value) ::: "volatile") };
	value
}

#[inline(always)]
pub fn read_time() -> usize {
	let value: usize;
	unsafe { llvm_asm!("csrr $0, time" : "=r"(value) ::: "volatile") };
	value
}

#[inline(always)]
pub fn read_cycle() -> usize {
	let value: usize;
	unsafe { llvm_asm!("csrr $0, cycle" : "=r"(value) ::: "volatile") };
	value
}

/// Gets hartid from sscratch
/// This assumes that sscratch holds a valid value
pub fn load_hartid() -> usize {
	unsafe { (*read_sscratch()).hartid }
}


#[inline(always)]
pub fn wfi() {
	// SAFETY:
	// wfi never changes any register state and is always safe
	// it's essentially a processor hint and can act as a NOP
	unsafe {
		llvm_asm!("wfi");
	}	
}

pub fn fence_vma() {
	unsafe { llvm_asm!("sfence.vma zero, zero") };
}

const MMIO_MTIME: *const u64 = 0x0200_BFF8 as *const u64;

pub fn get_time() -> u64 {
	unsafe { *MMIO_MTIME }
}

// This module describes CSR bits and layouts
pub mod csr {
	// First are the xip and xep CSRs
	// In the first characture, U means user, S means supervisor, and M means machine
	// In the second one, S means software, T means timer, and E means external
	
	
	// For the xip CSRS (interrupt pending)
	// Software
	pub const USIP: usize = 1 << 0;
	pub const SSIP: usize = 1 << 1;
	pub const MSIP: usize = 1 << 3;
	
	// Timer
	pub const UTIP: usize = 1 << 4;
	pub const STIP: usize = 1 << 5;
	pub const MTIP: usize = 1 << 7;
	
	// External (PLIC)
	pub const UEIP: usize = 1 << 8;
	pub const SEIP: usize = 1 << 9;
	pub const MEIP: usize = 1 << 11;
	
	// For the xie CSRS (interrupt enable)
	// Software
	pub const USIE: usize = 1 << 0;
	pub const SSIE: usize = 1 << 1;
	pub const MSIE: usize = 1 << 3;
	
	// Timer
	pub const UTIE: usize = 1 << 4;
	pub const STIE: usize = 1 << 5;
	pub const MTIE: usize = 1 << 7;
	
	// External
	pub const UEIE: usize = 1 << 8;
	pub const SEIE: usize = 1 << 9;
	pub const MEIE: usize = 1 << 11;
	
	// SATP flags
	pub const SATP_BARE: usize = 0;
	pub const SATP_SV32: usize = 1 << 30;
	pub const SATP_SV39: usize = 8 << 60;
	pub const SATP_SV48: usize = 9 << 60;
}