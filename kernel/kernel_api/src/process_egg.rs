use crate::Handle;
use flat_bytes::Flat;
use kernel_syscall_abi::process_egg::ProcessEggPacketHeader;

pub struct ProcessEgg {
	handle: Handle,
}


impl ProcessEgg {
	pub fn new() -> Self {
		Self {
			handle: Handle::open(3, &[]),
		}
	}
	pub fn set_memory(&mut self, address: usize, buffer: &[u8]) {
		let mut packet = ProcessEggPacketHeader::Memory(address).serialize();
		packet.extend_from_slice(buffer);
		self.handle.write(&mut packet, &[]);
	}
	pub fn get_memory(&mut self, address: usize, buffer: &mut [u8]) {
		let mut packet = ProcessEggPacketHeader::Memory(address).serialize();
		packet.extend_from_slice(buffer);
		self.handle.read(&mut packet, &[]);
	}
	pub fn set_start_address(&mut self, address: usize) {
		let mut packet = ProcessEggPacketHeader::Entry(address).serialize();
		self.handle.write(&mut packet, &[]);
	}
	pub fn get_start_address(&mut self) -> usize {
		todo!()
	}
	pub fn hatch(&mut self) {
		let mut packet = ProcessEggPacketHeader::Hatch.serialize();
		self.handle.write(&mut packet, &[]);
	}
}