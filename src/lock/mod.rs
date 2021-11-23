//! Locks that work in different contexts in the kernel

// Needed because of the way lock-api works
#![allow(clippy::declare_interior_mutable_const)]

pub mod interrupt;
pub mod shared;
pub mod simple_shared;
pub mod spin;
pub mod future;