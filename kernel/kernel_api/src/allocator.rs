use core::{alloc::{Allocator, GlobalAlloc, AllocError}, ptr::NonNull};
use spin::Mutex;

use slab_allocator_rs::Heap;

use crate::memory::alloc_pages;

pub struct UserspaceAllocator(Mutex<Option<Heap>>);

unsafe impl GlobalAlloc for UserspaceAllocator {
	unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
		let mut heap_lock = self.0.lock();
		let mut heap = heap_lock.as_mut().expect("Heap not initialized!");
		let mut pointer = heap.allocate(layout);
		while pointer.is_err() {
			// If we don't have enough memory, grow the heap
			let (vaddr, _) = alloc_pages(None, None, layout.size(), 0xfff);
			unsafe{ heap.grow(vaddr, layout.size(), Heap::layout_to_allocator(&layout)) } 
			pointer = heap.allocate(layout);
		}
		pointer.unwrap().as_ptr() as *mut u8
	}
	unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
	    self.0.lock().as_mut().expect("Heap not initialized!").deallocate(NonNull::new(ptr).unwrap(), layout)
	}
}

impl UserspaceAllocator {
	pub const fn new() -> Self {
		unsafe { UserspaceAllocator(Mutex::new(None)) }
	}
	pub fn initialize_empty(&self) {
		let (vaddr, _) = alloc_pages(None, None, 0, 0xfff);
	}
	pub fn initialize_min_size(&self) {
		let size = slab_allocator_rs::MIN_HEAP_SIZE * 8;
		let (vaddr, _) = alloc_pages(None, None, size, 7);
		*self.0.lock() = Some(unsafe { Heap::new(vaddr, size) });
	}
}