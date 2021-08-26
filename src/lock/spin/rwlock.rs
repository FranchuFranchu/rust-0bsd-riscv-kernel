use lock_api::{RawRwLock, GuardSend};
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::trap::in_interrupt_context;

const SHARED: usize = 1 << 1;
const WRITER: usize = 1 << 0;


pub struct RawSpinRwLock {
	value: AtomicUsize,
}

unsafe impl RawRwLock for RawSpinRwLock {
    const INIT: RawSpinRwLock = Self { value: AtomicUsize::new(0) };

    type GuardMarker = GuardSend;

    fn lock_shared(&self) {
		while self.value.load(Ordering::Acquire) & WRITER != 0 {
            while self.value.load(Ordering::Relaxed) & WRITER != 0 {
                core::hint::spin_loop();
            }
        }
        self.value.fetch_add(SHARED, Ordering::Release);
    }

    fn try_lock_shared(&self) -> bool {
        self.value
            .compare_exchange(0, 0, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
    }

    unsafe fn unlock_shared(&self) {
        self.value.fetch_sub(SHARED, Ordering::Release);
    }

    fn lock_exclusive(&self) {
		while self.value.load(Ordering::Acquire) != 0 {
            while self.value.load(Ordering::Relaxed) != 0 {
                core::hint::spin_loop();
            }
        }
        self.value.fetch_add(WRITER, Ordering::Release);
    }

    fn try_lock_exclusive(&self) -> bool {
        self.value.load(Ordering::Relaxed) == 0
    }

    unsafe fn unlock_exclusive(&self) {
        self.value.fetch_sub(WRITER, Ordering::Release);
    }
}

pub type RwLock<T> = lock_api::RwLock<RawSpinRwLock, T>;
pub type RwLockReadGuard<'a, T> = lock_api::RwLockReadGuard<'a, RawSpinRwLock, T>;
pub type RwLockWriteGuard<'a, T> = lock_api::RwLockWriteGuard<'a, RawSpinRwLock, T>;