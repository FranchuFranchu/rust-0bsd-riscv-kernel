pub mod mutex;
pub mod rwlock;

pub use mutex::{Mutex, MutexGuard};
pub use rwlock::{RwLock, RwLockReadGuard, RwLockWriteGuard};
pub use mutex::RawSpinlock as RawMutex;
pub use rwlock::RawSpinRwLock as RawRwLock;