use core::{cell::UnsafeCell, ops::Deref};

pub use arcinject::{ArcInject, WeakInject};
use lock_api::RwLockReadGuard;

pub struct WeakInjectRwLock<R: lock_api::RawRwLock, T: ?Sized, U: ?Sized> {
    pub weak: WeakInject<lock_api::RwLock<R, T>, U>,
}

impl<R: lock_api::RawRwLock, T: ?Sized, U: ?Sized> WeakInjectRwLock<R, T, U> {
    pub fn read(&self) -> WeakInjectRwLockReadGuard<R, T, U> {
        let upgraded = self.weak.upgrade().unwrap();
        let t = ArcInject::deref_inner(&upgraded);
        core::mem::forget(t.read());
        WeakInjectRwLockReadGuard(upgraded)
    }
}

pub struct WeakInjectRwLockReadGuard<R: lock_api::RawRwLock, T: ?Sized, U: ?Sized>(
    ArcInject<lock_api::RwLock<R, T>, U>,
);

impl<R: lock_api::RawRwLock, T: ?Sized, U: ?Sized> Deref for WeakInjectRwLockReadGuard<R, T, U> {
    type Target = U;
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<R: lock_api::RawRwLock, T: ?Sized, U: ?Sized> Drop for WeakInjectRwLockReadGuard<R, T, U> {
    fn drop(&mut self) {
        unsafe { ArcInject::deref_inner(&self.0).force_unlock_read() };
    }
}

unsafe impl<R: lock_api::RawRwLock, T: ?Sized, U: ?Sized> Send for WeakInjectRwLock<R, T, U> {}
unsafe impl<R: lock_api::RawRwLock, T: ?Sized, U: ?Sized> Sync for WeakInjectRwLock<R, T, U> {}
