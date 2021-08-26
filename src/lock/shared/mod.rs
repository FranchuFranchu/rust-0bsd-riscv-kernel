pub mod mutex;
pub mod rwlock;

pub use mutex::{Mutex, MutexGuard};
pub use rwlock::{RwLock, RwLockReadGuard, RwLockWriteGuard};
pub use mutex::RawSharedLock as RawMutex;
pub use rwlock::RawSharedRwLock as RawRwLock;