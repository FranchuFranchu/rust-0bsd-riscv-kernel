use alloc::{boxed::Box, sync::Arc, task::Wake};
use core::{
    future::Future,
    hint::unreachable_unchecked,
    pin::Pin,
    task::{Context, Poll, Waker},
};

use crate::{
    context_switch::{context_switch, schedule_and_switch},
    interrupt_context_waker::InterruptContextWaker,
    lock::shared::Mutex,
    process::ProcessState,
};

struct ExtraState {
    satp: usize,
}

impl ExtraState {
    fn save() -> Self {
        ExtraState {
            satp: kernel_cpu::read_satp(),
        }
    }
    fn save_in_place(&mut self) {
        *self = Self::save();
    }

    fn restore(&mut self) {
        unsafe { kernel_cpu::write_satp(self.satp) }
    }
}

struct TrapFutureWaker {
    future: Mutex<Box<dyn Future<Output = !> + Send + Unpin>>,
    state: Mutex<ExtraState>,
}

impl Wake for TrapFutureWaker {
    fn wake(self: Arc<Self>) {
        poll_trap_future_waker(self)
    }
}

fn poll_trap_future_waker(waker: Arc<TrapFutureWaker>) -> ! {
    let interrupt_waker = Arc::new({
        let waker = waker.clone();
        InterruptContextWaker(Box::new(move || waker.wake_by_ref()))
    });
    let raw_waker: Waker = interrupt_waker.clone().into();
    let mut ctx = Context::from_waker(&raw_waker);

    waker.state.lock().restore();
    if let Poll::Ready(_never) = Pin::new(&mut *waker.future.lock()).poll(&mut ctx) {
        // Safety: The ! type can not be constructed so we can't reach this
        unsafe { unreachable_unchecked() }
    };
    waker.state.lock().save_in_place();

    // Otherwise, it means that the future is still Pending
    // The waker will be called eventually when it's needed
    // Meanwhile, switch to another process that has something to do
    schedule_and_switch()
}

/// Executor for futures in a trap context.
///
/// This never returns and the future should return to userspace by itself when it is completed.
//
/// You should include all code that passes whatever return value the future has to userspace in the future itself
fn block_until_never<F: 'static>(future: F) -> !
where
    F: Future<Output = !> + Send,
{
    let waker = Arc::new(TrapFutureWaker {
        future: Mutex::new(Box::new(Box::pin(future))),
        state: Mutex::new(ExtraState::save()),
    });
    poll_trap_future_waker(waker)
}

pub fn block_and_return_to_userspace<F: 'static>(process: usize, future: F) -> !
where
    F: Future<Output = ()> + Send,
{
    crate::process::try_get_process(&process).write().state = ProcessState::Yielded;
    let block = async move {
        future.await;
        // Whatever task we had to do is done, return to userspace now
        context_switch(&process);
    };
    let f = Box::pin(block);
    block_until_never(f)
}
