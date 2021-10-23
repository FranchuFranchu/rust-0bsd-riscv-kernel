//! Wraps around an allocator, intended to detect memory leaks

use core::alloc::{GlobalAlloc, Layout};

use core::sync::atomic::{AtomicBool, Ordering, AtomicUsize};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

static RECURSION_PREVENTION_BIT: AtomicBool = AtomicBool::new(false);
static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static OLDEST_PTRS: crate::lock::spin::Mutex<Vec<usize>> = crate::lock::spin::Mutex::new(Vec::new());
static PTR_ALLOC_COUNT: crate::lock::spin::Mutex<BTreeMap<usize, usize>> = crate::lock::spin::Mutex::new(BTreeMap::new());

pub struct ProxyAllocator<T: GlobalAlloc>(pub T);

struct InterruptGuard(usize);

impl InterruptGuard {
	fn lock() -> Self {
		Self(crate::cpu::read_sie())
	}
}

impl Drop for InterruptGuard {
	fn drop(&mut self) {
		unsafe { crate::cpu::write_sie(self.0) }
	}
}

unsafe impl<T: GlobalAlloc> GlobalAlloc for ProxyAllocator<T> {
	#[inline]
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		let g = InterruptGuard::lock();
		let ptr = self.0.alloc(layout);
		// The store and the load will be done in the same hart so the ordering isn't important
		if RECURSION_PREVENTION_BIT.swap(true, Ordering::Relaxed) == false {
			ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
			OLDEST_PTRS.lock().push(ptr as usize);
			PTR_ALLOC_COUNT.lock().insert(ptr as usize, ALLOC_COUNT.load(Ordering::Relaxed));
			println!("{} {:?}", ALLOC_COUNT.load(Ordering::Relaxed), OLDEST_PTRS.lock().iter().skip(200).next().map(|s| PTR_ALLOC_COUNT.lock()[s]));
			
			RECURSION_PREVENTION_BIT.store(false, Ordering::Relaxed);
		};
		ptr
	}

	#[inline]
	unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
		let g = InterruptGuard::lock();
		if RECURSION_PREVENTION_BIT.swap(true, Ordering::Relaxed) == false {
			ALLOC_COUNT.fetch_sub(1, Ordering::Relaxed);
			let mut l = OLDEST_PTRS.lock();
			let pos = l.iter().position(|r| *r == ptr as usize).unwrap_or(0);
			l.remove(pos);
			RECURSION_PREVENTION_BIT.store(false, Ordering::Relaxed);
		}
		self.0.dealloc(ptr, layout)
	}
}
