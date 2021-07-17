use alloc::collections::BinaryHeap;
use core::{mem::MaybeUninit};

use core::cmp::Reverse;

use spin::RwLock;

use crate::{sbi, timer_queue};

/// SBI only allows us to have 1 timer set at a time
/// So instead we have to keep track of all points in time we want to get interrupted on
/// and only set the lowest

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TimerEventCause {
	ContextSwitch,
	TimeoutFuture,
}

#[derive(Debug, Eq, PartialEq)]
pub struct TimerEvent {
	pub instant: u64,
	pub cause: TimerEventCause,
}

impl PartialOrd for TimerEvent {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
	// add code here
}

impl Ord for TimerEvent {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        if self.instant == other.instant {
        	return self.cause.cmp(&other.cause)
        } else {
        	return self.instant.cmp(&other.instant).reverse()
        }
    }
	// add code here
}

// For some reason, BinaryHeap::new() is not a const fn
// so we have to use unsafe to minimize runtime overhead
// MaybeUninit is essentially an unsafe Option with no runtime information
static mut TIMER_QUEUE: MaybeUninit<RwLock<BinaryHeap<TimerEvent>>> = MaybeUninit::uninit();

pub fn init() {
	unsafe { TIMER_QUEUE = MaybeUninit::new(RwLock::new(BinaryHeap::new())) };
}

// All functions below invoke UB if init() is not called
pub fn get_timer_queue() -> &'static RwLock<BinaryHeap<TimerEvent>> {
	unsafe { TIMER_QUEUE.assume_init_ref() }
}


// These two functions get called on a timer interrupt
/// Removes the earliest time event and returns the time (with maybe some metadata in the future?)
pub fn last_cause() -> TimerEvent {
	// Remove the lowest element in the heap (which we will assume is the time this call happened)
	get_timer_queue().write().pop().unwrap()
}

pub fn schedule_next() {
	// Call SBI to schedule the next timer interrupt
	let next_time = get_timer_queue().read().peek().expect("Deadlock: Timer queue was drained to zero This should never happen!").instant;
	// Note that the get_timer_queue().read() must be unlocked here
	// because the timer interrupt might trigger immeditately
	sbi::set_absolute_timer(next_time).unwrap();
}

pub fn schedule_at(event: TimerEvent) {
	get_timer_queue().write().push(event)
}