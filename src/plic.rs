// 0BSD

// An interrupt controller controls external interrupts
// A "riscv,plic0"-compatible interrupt controller has many different "hart contexts" (or "targets")
// Each hart can have specific external interrupts enabled or disabled for it
// The context number also depends on whether it's S-mode or M-mode
// See https://static.dev.sifive.com/SiFive-U5-Coreplex-v1.0.pdf, section 4.1

use crate::cpu::load_hartid;

#[inline(always)]
fn get_context_number() -> usize {
	
	// From the PDF:
	// "SiFive cores always supports a machine-mode context for each hart.  
	//  For machine-mode hart contexts, interrupts generated by the PLIC
	//  appear on meip in the mip register.  SiFive cores can optionally support
	//  user-level interrupts with a user-mode context for each hart. If
	//  external interrupts are delegated to the user-mode hart context (by
	//  setting the appropriate bits in the machine-modemidelegregister),
	//  then the PLIC interrupts appear on ueip in the uip register. The PLIC
	//  interrupts for the user-mode hart context always appear on the ueip bit
	//  in the mipregister, regardless of delegation setting. Interrupt targets
	//  are mapped to harts sequentially,  with interrupt targets being added
	//  for each hart’s M-mode, H-mode, S-mode, and U-mode contexts
	//  sequentially in that order.  For example,if the system has one hart
	//  with M-mode and U-mode, and two harts with M-mode, S-mode, and U-mode,
	//  the mappings are as
	//  follows:""
	
	// Target 	Hart   	Mode
	// 0        0		M
	// 1 		0		U
	// 2 		1		M
	// 3 		1		S
	// 4 		1		U
	// 5 		2		M
	// 6 		2		S
	// 7 		2		U

	
	// For now we'll assume that each hart always has a M and S mode
	// We're always on S-mode, so always add 1
	1 + load_hartid() * 2
}

pub struct Plic0 {
	base_addr: usize,
	context_number: usize,
}

impl Plic0 {
	pub fn new_with_fdt() -> Self {
		Self {
			base_addr: crate::fdt::root().read().get("soc/plic@").unwrap().unit_address.unwrap(),
			context_number: get_context_number()
		}
	}
	pub fn new_with_addr(base_addr: usize) -> Self {
		Self {
			base_addr,
			context_number: get_context_number()
		}
	}
	
	pub fn set_priority(&self, interrupt: u32, priority: u32) {
		unsafe { (self.base_addr as *mut u32).add(interrupt as usize).write_volatile(priority) };
	}
	pub fn set_enabled(&self, interrupt: u32, enable: bool) {
		
		// The enable bits are at offset 0x2000 + context * 0x80
		
		// From the pdf:
		// > For each interrupt target, each device’s interrupt can be enabled by 
		// > setting the corresponding bit in that target’s enables registers. 
		// > The enables for a target are accessed as a contiguous array of 
		// > 32×32-bit words, packed the same way as the pending bits. 
		// > For each target, bit 0 of enable word 0 represents the non-existent
		// > interrupt ID 0 and is hardwired to 0.  
		// > Unused interrupt IDs are also hardwired to zero. The enables arrays
		// > for different targets are packed contiguously in the addressspace.
		
		// The pending bits are 32 words of 32 bits. The pending bit for interrupt id N
		// is a bit (N % 32) of word (N / 32)
		let enables_base = self.base_addr + 0x2000;
		let target_base = enables_base + self.context_number * 0x80;
		let target_base = target_base as *mut u32;
		let this_register = unsafe { target_base.add((interrupt / 32) as usize) };
		if enable {
			let flag = (enable as u32) << (interrupt % 32);
			unsafe { this_register.write_volatile(this_register.read_volatile() | flag); };
		} else {
			let flag = !((enable as u32) << (interrupt % 32));
			unsafe { this_register.write_volatile(this_register.read_volatile() & flag); };
		}
	}
	pub fn set_threshold(&self, threshold: u32) {
		let threshold_base = self.base_addr + 0x20_0000;
		let target_base = threshold_base + self.context_number * 0x1000;
		let target_base = target_base as *mut u32;
		unsafe { target_base.write_volatile(threshold) };
	}
	pub fn claim_highest_priority(&self) -> u32 {
		let cc_base = self.base_addr + 0x20_0004;
		let target_base = cc_base + self.context_number * 0x1000;
		let target_base = target_base as *mut u32;
		unsafe { target_base.read_volatile() }
	}
	pub fn complete(&self, interrupt: u32) {
		let cc_base = self.base_addr + 0x20_0004;
		let target_base = cc_base + self.context_number * 0x1000;
		let target_base = target_base as *mut u32;
		unsafe { target_base.write_volatile(interrupt) }
	}
}
