//! This module provides a function to create Wakers that
//! run a specific function on interrupt-mode when woken
//! Note that the function will only be run when an interrupt happens
//! (or if it's woken from an interrupt context, then it's run at the end of the interrupt handler)
//! Functions in interrupt contexts can wake up other InterruptContextWakers with no risk of stack overflow or deadlocks
//! This is useful for giving functions the same context whether they're ran from a kernel thread or from an interrupt context
//! *Interrupt tasks should NOT have blocking operations!*

use alloc::{boxed::Box, collections::VecDeque, sync::Arc, task::Wake};

use crate::{lock::shared::Mutex, trap::in_interrupt_context};

static WAITING_WAKERS: Mutex<Option<VecDeque<Arc<InterruptContextWaker>>>> = Mutex::new(None);

pub struct InterruptContextWaker(pub Box<dyn Fn() + Send + Sync>);

impl Wake for InterruptContextWaker {
    fn wake(self: Arc<Self>) {
        WAITING_WAKERS.lock().as_mut().unwrap().push_back(self)
    }
}

/// When running in an interrupt context, wake up all the interrupt context wakers
/// that are waiting to be woken up
pub(crate) fn wake_all() {
    assert!(in_interrupt_context());
    while let Some(i) = {
        // this is done to prevent WAITING_WAKERS from being locked while i.0 is called
        let l = WAITING_WAKERS.lock().as_mut().unwrap().pop_front();
        l
    } {
        i.0();
    }
}

pub fn init() {
    *WAITING_WAKERS.lock() = Some(VecDeque::new());
}
