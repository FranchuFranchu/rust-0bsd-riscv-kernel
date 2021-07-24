use alloc::collections::{BinaryHeap, BTreeMap};




use spin::RwLock;

use crate::cpu::load_hartid;
use crate::{sbi};

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

pub static TIMER_QUEUE: RwLock<BTreeMap<usize, RwLock<BinaryHeap<TimerEvent>>>> = RwLock::new(BTreeMap::new());

pub fn init() {
	
}

/// Does initialization local to this hart
pub fn init_hart() {
	TIMER_QUEUE.write().insert(load_hartid(), RwLock::new(BinaryHeap::new()));
}

// All functions below invoke UB if init() is not called
// (well, not with the current implementation)

// These two functions get called on a timer interrupt
/// Removes the earliest time event and returns the time it happened and its cause
/// Note that the time it happened might actually be _after_ the current time, in which case this functions shouldn't have been called
pub fn last_cause() -> TimerEvent {
	// Remove the lowest element in the heap (which we will assume is the time this call happened)
	TIMER_QUEUE.read()[&load_hartid()].write().pop().unwrap()
}

pub fn schedule_next() {
	// Call SBI to schedule the next timer interrupt
	let next_time = TIMER_QUEUE.read()[&load_hartid()].read().peek().expect("Deadlock: Timer queue was drained to zero This should never happen!").instant;
	// Note that the get_timer_queue().read() must be unlocked here
	// because the timer interrupt might trigger immeditately
	
	sbi::set_absolute_timer(next_time).unwrap();
}

pub fn schedule_at(event: TimerEvent) {
	TIMER_QUEUE.read()[&load_hartid()].write().push(event)
}