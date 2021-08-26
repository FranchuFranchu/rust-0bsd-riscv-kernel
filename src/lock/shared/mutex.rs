use lock_api::{RawMutex, GuardSend};
use core::sync::atomic::{AtomicBool, Ordering, AtomicUsize};

use crate::{cpu::load_hartid, trap::in_interrupt_context};

pub use super::super::spin::RawMutex as RawSpinlock;

pub const NO_HART: usize = usize::MAX;

// 1. Define our raw lock type
pub struct RawSharedLock { 
    internal: RawSpinlock,
}

// 2. Implement RawMutex for this type
unsafe impl RawMutex for RawSharedLock {
    const INIT: RawSharedLock = RawSharedLock { internal: RawSpinlock::INIT };

    // A spinlock guard can be sent to another thread and unlocked there
    type GuardMarker = GuardSend;

    fn lock(&self) {
        if !in_interrupt_context() {
            unsafe { crate::cpu::write_sie(0) };
        }
        self.internal.lock()
    }

    fn try_lock(&self) -> bool {
        self.internal.try_lock()
    }

    unsafe fn unlock(&self) {
        self.internal.unlock();
        if !in_interrupt_context() {
            unsafe { crate::cpu::write_sie(0x222) };
        }
    }
}

// 3. Export the wrappers. This are the types that your users will actually use.
pub type Mutex<T> = lock_api::Mutex<RawSharedLock, T>;
pub type MutexGuard<'a, T> = lock_api::MutexGuard<'a, RawSharedLock, T>;