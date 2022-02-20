use core::{
    alloc::{AllocError, Allocator, GlobalAlloc},
    ptr::NonNull,
};

use slab_allocator_rs::Heap;
use spin::Mutex;

use crate::memory::{self, alloc_pages};

pub struct UserspaceAllocator(Mutex<Option<Heap>>);

unsafe impl GlobalAlloc for UserspaceAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let mut heap_lock = self.0.lock();
        let mut heap = heap_lock.as_mut().expect("Heap not initialized!");
        let mut pointer = heap.allocate(layout);
        while pointer.is_err() {
            // If we don't have enough memory, grow the heap
            let vaddr = alloc_pages(None, None, layout.size(), 0xfff).unwrap_or(0);
            crate::println_crate!("{:x}", vaddr);
            if vaddr == 0 {
                // Allocation failure
                return 0 as *mut u8;
            }
            crate::println_crate!("{:?}", "hello!");
            unsafe { heap.grow(vaddr, layout.size(), Heap::layout_to_allocator(&layout)) }
            pointer = heap.allocate(layout);
        }
        pointer.unwrap().as_ptr() as *mut u8
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        self.0
            .lock()
            .as_mut()
            .expect("Heap not initialized!")
            .deallocate(NonNull::new(ptr).unwrap(), layout)
    }
}

impl UserspaceAllocator {
    pub const fn new() -> Self {
        unsafe { UserspaceAllocator(Mutex::new(None)) }
    }
    pub fn initialize_min_size(&self) -> memory::Result<()> {
        let size = slab_allocator_rs::MIN_HEAP_SIZE * 8;
        let vaddr = alloc_pages(None, None, size, 7)?;
        *self.0.lock() = Some(unsafe { Heap::new(vaddr, size) });
        Ok(())
    }
    pub fn is_initialized(&self) -> bool {
        self.0.lock().is_some()
    }
}
