use lock_api::{RawMutex, GuardSend};
use core::sync::atomic::{AtomicBool, Ordering, AtomicUsize};

use crate::{cpu::load_hartid, trap::in_interrupt_context};

pub use super::super::spin::RawMutex as RawSpinlock;
use super::{lock_and_disable_interrupts, unlock_and_enable_interrupts_if_necessary};

pub const NO_HART: usize = usize::MAX;

// 1. Define our raw lock type
pub struct RawSharedLock { 
    internal: RawSpinlock,
    old_sie: AtomicUsize,
}

// 2. Implement RawMutex for this type
unsafe impl RawMutex for RawSharedLock {
    const INIT: RawSharedLock = RawSharedLock { internal: RawSpinlock::INIT, old_sie: AtomicUsize::new(0) };

    // A spinlock guard can be sent to another thread and unlocked there
    type GuardMarker = GuardSend;

    fn lock(&self) {
        lock_and_disable_interrupts();
        self.internal.lock()
    }

    fn try_lock(&self) -> bool {
        if self.internal.try_lock() {
            lock_and_disable_interrupts();
            true
        } else {
            false
        }
    }

    unsafe fn unlock(&self) {
        self.internal.unlock();
        unlock_and_enable_interrupts_if_necessary();
    }
}

// 3. Export the wrappers. This are the types that your users will actually use.
pub type Mutex<T> = lock_api::Mutex<RawSharedLock, T>;
pub type MutexGuard<'a, T> = lock_api::MutexGuard<'a, RawSharedLock, T>;