use crate::lock::simple_shared::Mutex;
use core::ptr::NonNull;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;
use slab_allocator_rs::Heap as SlabAllocator;
use core::alloc::{Layout, GlobalAlloc};

/// We're using this because we can't impl for foreign types
pub struct MutexWrapper<T>(pub Mutex<T>);

impl<T> MutexWrapper<T> {
    pub const fn new(t: T) -> Self {
        Self(Mutex::new(t))
    }
	pub fn lock(&self) -> crate::lock::simple_shared::MutexGuard<T> {
		self.0.lock()
	}
}

static a: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for MutexWrapper<Option<SlabAllocator>> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        //println!("{:?} {}", layout.size(), a.load(Ordering::SeqCst));
        a.fetch_add(layout.size(), Ordering::SeqCst);
        self.0.lock().as_mut().unwrap().allocate(layout).expect("Out of memory!").as_ptr()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
    	self.0.lock().as_mut().unwrap().deallocate(NonNull::new(ptr).unwrap(), layout)
    }
}

impl MutexWrapper<Option<SlabAllocator>> {
	pub const fn empty() -> Self {
		Self(Mutex::new(None))
	}
	
    pub unsafe fn init(&self, heap_start_addr: usize, size: usize) {
        info!("initialized");
        *self.0.lock() = Some(SlabAllocator::new(heap_start_addr, size));
        println!("{:?}", size);
    }

}
