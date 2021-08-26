
use lock_api::{RawMutex, GuardSend};
use core::sync::atomic::{AtomicBool, Ordering, AtomicUsize};

use crate::{cpu::load_hartid, trap::in_interrupt_context};

pub const NO_HART: usize = usize::MAX;

// 1. Define our raw lock type
pub struct RawSpinlock { 
    locked: AtomicBool,
    #[cfg(debug_assertions)] locker_hartid: AtomicUsize,
}

// 2. Implement RawMutex for this type
unsafe impl RawMutex for RawSpinlock {
    #[cfg(not(debug_assertions))] const INIT: RawSpinlock = RawSpinlock { locked: AtomicBool::new(false) };
    #[cfg(debug_assertions)] const INIT: RawSpinlock = RawSpinlock { locked: AtomicBool::new(false), locker_hartid: AtomicUsize::new(NO_HART) };

    // A spinlock guard can be sent to another thread and unlocked there
    type GuardMarker = GuardSend;

    fn lock(&self) {
        // Can fail to lock even if the spinlock is not locked. May be more efficient than `try_lock`
        // when called in a loop.
        
        #[cfg(debug_assertions)] if self.locked.load(Ordering::Acquire) {
            if self.locker_hartid.load(Ordering::Relaxed) == load_hartid() {
                warn!("Hart number {} tried locking the same lock twice! (Maybe you're holding a lock a function you're calling needs, or you're waking up a future which uses a lock you're holding)", self.locker_hartid.load(Ordering::Relaxed));
            }
        }
        while self.locked.compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {
            // Wait until the lock looks unlocked before retrying
            while self.locked.load(Ordering::Relaxed) == true {
                core::hint::spin_loop();
            }
        }
        self.locker_hartid.store(load_hartid(), Ordering::Relaxed);
    }

    fn try_lock(&self) -> bool {
        self.locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }

    unsafe fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }
}

// 3. Export the wrappers. This are the types that your users will actually use.
pub type Mutex<T> = lock_api::Mutex<RawSpinlock, T>;
pub type MutexGuard<'a, T> = lock_api::MutexGuard<'a, RawSpinlock, T>;