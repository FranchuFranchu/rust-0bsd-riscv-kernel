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
	
	// These can get XORed to check the interrupt
	XORMask_ReadAvailable = 0b0000,
	XORMask_WriteAvailable = 0b0010,
	XORMask_LSRChange = 0b0100,
	XORMask_MSRChange = 0b0110,
	
}

#[repr(C)]
struct Ns16550aRegisters {
	// Either receiver buffer or transmitter holding
	byte_io: RW<u8>,
	interrupt_enable: RW<u8>,
	// Either interrupt identification or FIFO control
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
    fn write_str(&mut self, s: &str) -> fmt::Result {
    	for byte in s.as_bytes() {
    		self.put(*byte)
    	}
    	Ok(())
    }
}

impl Ns16550a {
	// SAFETY: address should be a valid MMIO 16550 address
	pub unsafe fn new(address: usize) -> Self {
		Self {
			registers: address as *mut Ns16550aRegisters
		}
	}
	
	pub fn setup(&mut self) {
		unsafe { (*self.registers).interrupt_enable.write((*self.registers).interrupt_enable.read() | Ns16550aInterruptEnableRegister::ReadAvailable as u8) }
	}
	
	#[inline(always)]
	pub fn put(&mut self, value: u8) {
		// SAFETY:
		// The unsafety was done when creating this instance
		// Writing to a proved-existing register shouldn't cause anything unsafe
		unsafe { (*self.registers).byte_io.write(value) };
	}
	
	pub fn get(&mut self) -> Option<u8> {
		let dr_bit = unsafe { (*self.registers).line_status.read() } & 1;
		
		if dr_bit == 0 {
			// If the DR bit isn't set, then there's no new data
			None
		} else {
			Some(unsafe { (*self.registers).byte_io.read() })
		}
		
	}
}