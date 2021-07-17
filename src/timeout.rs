/// This module uses time interrupts to create a "timeout" future

use alloc::vec::Vec;
use core::task::{Waker, Context, Poll};
use core::future::Future;
use core::cmp::Ordering;

use spin::RwLock;

use crate::{cpu, timer_queue};

/// This Vec is sorted by item.0.for_time
/// TODO use binary heap?
pub static WAITING_TIMEOUTS: RwLock<Vec<(TimeoutFuture, Waker)>> = RwLock::new(Vec::new());

#[must_use = "Futures do nothing unless polled"]
#[derive(Copy, Clone)]
pub struct TimeoutFuture {
	pub for_time: u64,
}

impl Future for TimeoutFuture {
    type Output = u64;

    fn poll(self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> Poll<Self::Output> {
        if cpu::get_time() >= self.for_time {
        	return Poll::Ready(cpu::get_time());
        } else {
        	// Register the current task for wakeup
        	// WAITING_TIMEOUTS gets checked during a time interrupt
        
            // binary_search_by returns Err when the item is not found
            // This is what we expect, because there shouldn't be two equal TimeoutFuture::for_time values in the Vec
        	let index = WAITING_TIMEOUTS.write().binary_search_by(|s| {
        		return self.for_time.cmp(&s.0.for_time);
        	});
            
            let insert_position;
            
            match index {
                Ok(index) => {
                    // If this waker was already present, then we don't need to schedule it again
                    return Poll::Pending;
                },
                Err(index) => {insert_position = index},
            }
        	
            WAITING_TIMEOUTS.write().insert(insert_position, (self.clone(), cx.waker().clone()));
            
            use crate::timer_queue::{TimerEvent, TimerEventCause};
            timer_queue::schedule_at(TimerEvent { instant: self.for_time, cause: TimerEventCause::TimeoutFuture,  });
            
            // Trigger a timer interrupt in the target time
            
            
        	return Poll::Pending;
        }
    }
	// add code here
}

// This gets called by trap.rs on a timer interrupt scheduled by us
pub fn on_timer_event(instant: u64) {
    info!("Timer event for us {}", instant);
    let mut lock = WAITING_TIMEOUTS.write();
    let mut max_remove_index = 0;
    for (idx, (future, waker)) in lock.iter().enumerate() {
        if future.for_time <= instant {
            // This future has already timed out. Wake it.
            waker.wake_by_ref();
            max_remove_index = idx + 1;
        }
    }
    for i in 0..max_remove_index {
        lock.remove(i);
    }
}