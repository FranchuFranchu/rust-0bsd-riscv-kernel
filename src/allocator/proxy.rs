//! Wraps around an allocator. Useful for debugging.

use core::alloc::{GlobalAlloc, Layout};

pub struct ProxyAllocator<T: GlobalAlloc> (pub T);


unsafe impl<T: GlobalAlloc> GlobalAlloc for ProxyAllocator<T> {
	#[inline]
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		self.0.alloc(layout)
	}

	#[inline]
	unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0.dealloc(ptr, layout)
    }
	
}