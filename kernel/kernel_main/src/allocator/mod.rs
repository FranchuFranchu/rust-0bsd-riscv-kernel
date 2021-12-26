use core::ffi::c_void;

use proxy::ProxyAllocator;
use slab_allocator_rs::Heap as SlabAllocator;

#[global_allocator]
pub static ALLOCATOR: ProxyAllocator<shared_mutex_allocator::MutexWrapper<Option<SlabAllocator>>> =
    ProxyAllocator(shared_mutex_allocator::MutexWrapper::empty());

// Linker symbols
extern "C" {
    static _heap_start: c_void;
    static _heap_end: c_void;
}

pub fn init() {
    info!("Initialized memory allocation");
    // Initialize memory allocation
    let heap_end = unsafe { &_heap_end as *const c_void as usize };
    let heap_start = unsafe { &_heap_start as *const c_void as usize };
    let mut heap_size: usize = heap_end - heap_start;

    // Align the size to min heap size boundaries
    heap_size /= slab_allocator_rs::MIN_HEAP_SIZE;
    heap_size *= slab_allocator_rs::MIN_HEAP_SIZE;

    println!("{:?}", heap_size);

    // SAFETY: This relies on the assumption that heap_end and heap_start are valid addresses (which are provided by the linker script)
    unsafe { ALLOCATOR.0.init(heap_start, heap_size) };
}

pub mod proxy;
pub mod proxy_trace;
pub mod shared_mutex_allocator;
