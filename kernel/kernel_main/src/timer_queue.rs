use alloc::collections::{BTreeMap, BinaryHeap};

use crate::{cpu::load_hartid, lock::shared::RwLock, sbi};

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
            self.cause.cmp(&other.cause)
        } else {
            self.instant.cmp(&other.instant).reverse()
        }
    }
    // add code here
}

pub static TIMER_QUEUE: RwLock<BTreeMap<usize, RwLock<BinaryHeap<TimerEvent>>>> =
    RwLock::new(BTreeMap::new());

pub fn init() {}

/// Does initialization local to this hart
pub fn init_hart() {
    let mut l = TIMER_QUEUE.write();
    let hid = load_hartid();
    l.insert(hid, RwLock::new(BinaryHeap::new()));
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
    let next_time = TIMER_QUEUE
        .read()
        .get(&load_hartid())
        .expect("Hartid queue not found!")
        .read()
        .peek()
        .expect("Deadlock: Timer queue was drained to zero This should never happen!")
        .instant;
    // Note that the get_timer_queue().read() must be unlocked here
    // because the timer interrupt might trigger immeditately

    sbi::set_absolute_timer(next_time).unwrap();
}

pub fn schedule_at(event: TimerEvent) {
    let t = TIMER_QUEUE.read();
    let e = t.get(&load_hartid()).expect("Hartid queue not found! (2)");
    e.write().push(event);
    drop(t);
}

/// same as schedule_at, but if theres an earlier event for this type, dont bother scheduling
pub fn schedule_at_or_earlier(event: TimerEvent) {
    let t = TIMER_QUEUE.read();

    let e = t.get(&load_hartid()).expect("Hartid queue not found! (2)");
    let mut timer_queue = e.write();
    if timer_queue
        .iter()
        .filter(|ev| ev.cause == event.cause && ev.instant < event.instant)
        .next()
        .is_some()
    {
        return;
    } else {
        timer_queue.push(event);
    }
}
