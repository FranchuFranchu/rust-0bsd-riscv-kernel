pub mod mutex;
pub mod rwlock;

use core::sync::atomic::{AtomicUsize, Ordering};
use alloc::vec::Vec;

pub use mutex::{Mutex, MutexGuard};
pub use rwlock::{RwLock, RwLockReadGuard, RwLockWriteGuard};
pub use mutex::RawSharedLock as RawMutex;
pub use rwlock::RawSharedRwLock as RawRwLock;

use super::spin::RwLock as SpinRwLock;

use crate::{
	trap::in_interrupt_context,
	cpu::load_hartid,
};


static HART_LOCK_COUNT: SpinRwLock<Vec<AtomicUsize>> = SpinRwLock::new(Vec::new());

// Resizes HART_LOCK_COUNT up to idx + 1, but does not exclusively lock if not necessary
pub fn create_hart_lock_count_entry_if_necessary(idx: &usize) -> bool {
    if idx < &HART_LOCK_COUNT.read().len() {
        false
    } else {
        HART_LOCK_COUNT.write().resize_with(idx+1, || { AtomicUsize::new(0) });
        true
    }
}

#[inline]
pub fn lock_and_disable_interrupts() {
    if !in_interrupt_context() {
        unsafe { crate::cpu::write_sie(0) };
        create_hart_lock_count_entry_if_necessary(&load_hartid());
        HART_LOCK_COUNT.read()[load_hartid()].fetch_add(1, Ordering::AcqRel);
    }
}

#[inline]
pub fn unlock_and_enable_interrupts_if_necessary() {
    if !in_interrupt_context() {
        if HART_LOCK_COUNT.read()[load_hartid()].fetch_sub(1, Ordering::AcqRel) == 1 {
            // This was the last lock remaining for this hart
            unsafe { crate::cpu::write_sie(0x222) };
        }
    }
}

#[no_mangle]
pub extern "C"  fn this_hart_lock_count() -> usize {
    HART_LOCK_COUNT.read()[load_hartid()].load(Ordering::Acquire)
}