use lock_api::{RawRwLock, GuardSend};
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::trap::in_interrupt_context;

pub use super::super::spin::RawRwLock as RawSpinRwLock;


pub struct RawSharedRwLock {
	internal: RawSpinRwLock,
}

unsafe impl RawRwLock for RawSharedRwLock {
    const INIT: RawSharedRwLock = Self { internal: RawSpinRwLock::INIT };

    type GuardMarker = GuardSend;

    fn lock_shared(&self) {
        if !in_interrupt_context() {
            unsafe { crate::cpu::write_sie(0) };
        }
		self.internal.lock_shared()
    }

    fn try_lock_shared(&self) -> bool {
        self.internal.try_lock_shared()
    }

    unsafe fn unlock_shared(&self) {
        self.internal.unlock_shared();
        if !in_interrupt_context() {
            unsafe { crate::cpu::write_sie(0x222) };
        }
    }

    fn lock_exclusive(&self) {
        if !in_interrupt_context() {
            unsafe { crate::cpu::write_sie(0) };
        }
        self.internal.lock_exclusive()
    }

    fn try_lock_exclusive(&self) -> bool {
        self.internal.try_lock_exclusive()
    }

    unsafe fn unlock_exclusive(&self) {
        self.internal.unlock_exclusive();
        if !in_interrupt_context() {
            unsafe { crate::cpu::write_sie(0x222) };
        }
    }
}

pub type RwLock<T> = lock_api::RwLock<RawSharedRwLock, T>;
pub type RwLockReadGuard<'a, T> = lock_api::RwLockReadGuard<'a, RawSharedRwLock, T>;
pub type RwLockWriteGuard<'a, T> = lock_api::RwLockWriteGuard<'a, RawSharedRwLock, T>;