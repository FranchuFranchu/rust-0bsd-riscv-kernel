/// This module uses time interrupts to create a "timeout" future
use alloc::vec::Vec;
use core::{
    future::Future,
    task::{Poll, Waker},
};

use cpu::{MMIO_MTIME, read_time};

use crate::{
    cpu,
    lock::shared::RwLock,
    paging::PAGE_ALIGN,
    timer_queue,
    virtual_buffers::{new_virtual_buffer, VirtualBuffer},
};

/// This Vec is sorted by item.0.for_time
/// TODO use binary heap?
pub static WAITING_TIMEOUTS: RwLock<Vec<(TimeoutFuture, Waker)>> = RwLock::new(Vec::new());

pub static MMIO_MTIME_VIRT_BUFFER_ADDR: RwLock<Option<usize>> = RwLock::new(None);

// Uses a virt buffer, so it
pub fn get_time_setup() {
    let mmio_mtime_aligned = (MMIO_MTIME as usize) & !(PAGE_ALIGN - 1);
    
    //println!("{:?}", unsafe {*MMIO_MTIME});
    let virt = new_virtual_buffer(mmio_mtime_aligned, 0x1000);
    println!("{:x}", MMIO_MTIME as usize);
    println!("{:x}", virt + ((MMIO_MTIME as usize) & (PAGE_ALIGN - 1)));
    *MMIO_MTIME_VIRT_BUFFER_ADDR.write() = Some(virt + ((MMIO_MTIME as usize) & (PAGE_ALIGN - 1)))
}
pub fn get_time() -> u64 {
    return unsafe { kernel_cpu::read_time() };
    unsafe {
        match &*MMIO_MTIME_VIRT_BUFFER_ADDR.read() {
            Some(a) => *((*a) as *const u64),
            None => *MMIO_MTIME,
        }
    }
}

#[must_use = "Futures do nothing unless polled"]
#[derive(Clone)]
pub struct TimeoutFuture {
    pub for_time: u64,
}

impl TimeoutFuture {
    pub fn absolute(for_time: u64) -> Self {
        Self { for_time }
    }
    pub fn relative(for_time: u64) -> Self {
        Self::absolute(for_time + get_time())
    }
}

impl Future for TimeoutFuture {
    type Output = u64;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> Poll<Self::Output> {
        if get_time() >= self.for_time {
            Poll::Ready(get_time())
        } else {
            // Register the current task for wakeup
            // WAITING_TIMEOUTS gets checked during a time interrupt

            // binary_search_by returns Err when the item is not found
            // This is what we expect, because there shouldn't be two equal TimeoutFuture::for_time values in the Vec
            let index = WAITING_TIMEOUTS
                .write()
                .binary_search_by(|s| self.for_time.cmp(&s.0.for_time));

            let insert_position;

            match index {
                Ok(_index) => {
                    // If this waker was already present, then we don't need to schedule it again
                    return Poll::Pending;
                }
                Err(index) => insert_position = index,
            }

            // Remember to wake up this future when necessary
            WAITING_TIMEOUTS
                .write()
                .insert(insert_position, (self.clone(), cx.waker().clone()));

            // Trigger a timer interrupt in the target time
            use crate::timer_queue::{TimerEvent, TimerEventCause};
            timer_queue::schedule_at(TimerEvent {
                instant: self.for_time,
                cause: TimerEventCause::TimeoutFuture,
            });

            Poll::Pending
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
        // This would trigger a unused-future warning otherwise
        // because rustc isn't smart enough to realize that all futures before max_remove_index would have been woken up before being removed
        #[allow(unused_must_use)]
        {
            lock.remove(i);
        }
    }
}
