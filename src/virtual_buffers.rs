




#[derive(Eq, PartialEq, Ord)]
struct VirtualBuffer {
	virt_addr: usize,
	phys_addr: usize,
	size: usize,
}

impl PartialOrd for VirtualBuffer {
	fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
		debug_assert!(self.virt_start() != other.virt_start());
	    self.virt_start().partial_cmp(&other.virt_start())
	}
}

impl VirtualBuffer {
	fn virt_start(&self) -> usize {
		self.virt_addr
	}
	fn virt_end(&self) -> usize {
		self.virt_addr + self.size
	}
	fn phys_start(&self) -> usize {
		self.phys_addr
	}
	fn phys_end(&self) -> usize {
		self.phys_addr + self.size
	}
}

struct VirtualBufferRegistry {
	buffers: alloc::collections::BinaryHeap<VirtualBuffer>,
	buffer_handles: alloc::collections::BTreeMap<usize, crate::lock::shared::RwLock<usize>>,
	free_space_start: usize,
}

extern "C" {
	static _free_space_start: usize;
}

impl VirtualBufferRegistry {
	fn new_buffer(&mut self, phys_addr: usize, size: usize) {
		let buf_virtual_address =  {
			let iter1 = self.buffers.iter();
			let mut iter2 = self.buffers.iter();
			iter2.next();
			let mut insert_into = None;
				
			for (idx, (current, next)) in iter1.zip(iter2).enumerate() {
				if (next.virt_start() - current.virt_end()) < size {
					// This buffer fits in here
					insert_into = Some(current.virt_end())
				}
			}
			if let Some(insert_into) = insert_into {
				insert_into
			} else {
				self.buffers.iter().nth(self.buffers.len()-1).unwrap().virt_end()
			}
		};
		let buffer = VirtualBuffer {
			virt_addr: buf_virtual_address,
			phys_addr,
			size,
		};
		self.buffers.push(buffer);
	}
}

// &_free_space_start as *const _ as usize	