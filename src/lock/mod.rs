// Needed because of the way lock-api works
#![allow(clippy::declare_interior_mutable_const)]

pub mod interrupt;
pub mod shared;
pub mod spin;
pub mod simple_shared;
