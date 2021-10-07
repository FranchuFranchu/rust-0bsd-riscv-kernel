pub mod mutex;
pub mod rwlock;

use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};

pub use mutex::{Mutex, MutexGuard, RawSharedLock as RawMutex};
pub use rwlock::{RawSharedRwLock as RawRwLock, RwLock, RwLockReadGuard, RwLockWriteGuard};

use super::spin::RwLock as SpinRwLock;
use crate::{cpu::load_hartid, trap::in_interrupt_context};

