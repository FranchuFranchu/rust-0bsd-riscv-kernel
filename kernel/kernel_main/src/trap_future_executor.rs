use alloc::{boxed::Box, sync::Arc, task::Wake};
use core::{
    future::Future,
    hint::unreachable_unchecked,
    pin::Pin,
    task::{Context, Poll, Waker},
};

use kernel_cpu::read_sscratch;

use crate::{
    context_switch::{context_switch, schedule_and_switch},
    interrupt_context_waker::InterruptContextWaker,
    lock::shared::Mutex,
    process::ProcessState,
};

struct ExtraState {
    satp: usize,
    pid: usize,
}

impl ExtraState {
    fn save() -> Self {
        ExtraState {
            satp: kernel_cpu::read_satp(),
            pid: unsafe { (*read_sscratch()).pid },
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
    future: Mutex<Option<Box<dyn Future<Output = !> + Send + Unpin>>>,
    state: Mutex<ExtraState>,
}

impl Wake for TrapFutureWaker {
    fn wake(self: Arc<Self>) {
        poll_trap_future_waker(self)
    }
}

impl Drop for TrapFutureWaker {
    fn drop(&mut self) {
        info!("Dropped future! (canceled)");
    }
}

fn poll_trap_future_waker(waker: Arc<TrapFutureWaker>) -> ! {
    {
        let interrupt_waker = Arc::new({
            let waker = Arc::downgrade(&waker);
            InterruptContextWaker(Box::new(move || waker.upgrade().unwrap().wake_by_ref()))
        });
        let raw_waker: Waker = interrupt_waker.clone().into();
        let mut ctx = Context::from_waker(&raw_waker);

        waker.state.lock().restore();

        // If the future never returns, we want to avoid leaking the waker
        let mut k = waker.future.lock();
        let waker = Arc::downgrade(&waker);
        if let Poll::Ready(_never) = Pin::new(k.as_mut().unwrap()).poll(&mut ctx) {
            // Safety: The ! type can not be constructed so we can't reach this
            unsafe { unreachable_unchecked() }
        };
        let waker = waker.upgrade().unwrap();

        waker.state.lock().save_in_place();
    }
    crate::process::try_get_process(&waker.state.lock().pid)
        .write()
        .state = ProcessState::Yielded;

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
    F: FnOnce(Arc<TrapFutureWaker>) -> Pin<Box<dyn Future<Output = !> + Send>>,
{
    let waker = Arc::new(TrapFutureWaker {
        future: Mutex::new(None),
        state: Mutex::new(ExtraState::save()),
    });
    waker.future.lock().replace(Box::new(future(waker.clone())));
    poll_trap_future_waker(waker)
}

pub struct TrackDrop<G>(pub G);

impl<G: Future> Future for TrackDrop<G> {
    type Output = G::Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe { self.map_unchecked_mut(|s| &mut s.0) }.poll(cx)
    }
}

impl<G> Drop for TrackDrop<G> {
    fn drop(&mut self) {
        println!("{:?}", "droppde!");
    }
}

pub fn block_and_return_to_userspace<F: 'static>(process: usize, future: F) -> !
where
    F: Future<Output = ()> + Send + Unpin,
{
    let f = {
        crate::process::try_get_process(&process).write().state = ProcessState::Yielded;
        // Pin<Box<Fn() -> (Future<Box<Pin<Future>>)>>
        move |trap| {
            let a: Pin<Box<dyn Future<Output = !> + Send>> = Box::pin(async move {
                future.await;
                //println!("{:?}", &crate::process::try_get_process(&process).read().trap_frame.general_registers[Registers::A0.idx()..Registers::A7.idx()]);
                // Whatever task we had to do is done, return to userspace now
                unsafe {
                    if Arc::strong_count(&trap) != 2 {
                        warn!(
                            "Possible memory leak of future (See {}:{}:{})",
                            file!(),
                            line!(),
                            column!()
                        )
                    };
                    Arc::decrement_strong_count(&trap as *const _);
                    drop(trap);
                }
                context_switch(&process);
            });
            a
        }
    };
    block_until_never(f)
}
