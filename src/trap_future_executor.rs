use core::future::Future;
use core::pin::Pin;
use core::task::Context;
use core::hint::unreachable_unchecked;
use core::task::Poll;
use alloc::{sync::Arc, task::Wake};
use alloc::boxed::Box;

use crate::context_switch::context_switch;
use crate::{context_switch::schedule_and_switch, trap::TrapFrame};

struct TrapFutureWaker {}

impl Wake for TrapFutureWaker {
	fn wake(self: Arc<Self>) {
		
	}
}

/// Executor for futures in a trap context.
///
/// This never returns and the future should return to userspace by itself when it is completed.
//
/// You should include all code that passes whatever return value the future has to userspace in the future itself
fn block_until_never<F>(future: Pin<&mut F>) -> ! where F: Future<Output=(!)> {
	let w = Arc::new(TrapFutureWaker{});
	let ctx = Context::from_waker(&w.into());
	if let Poll::Ready(never) = future.poll(&mut ctx) {
		// Safety: The ! type is never constructed so we can't reach this
		unsafe { unreachable_unchecked() }
	};
	// Otherwise, it means that the future is still Pending
	// The waker will be called eventually when it's needed
	// Meanwhile, switch to another process that has something to do
	schedule_and_switch()
}

pub fn block_and_return_to_userspace<F>(process: usize, future: Pin<&mut F>) -> ! where F: Future<Output=()> {
	let block = async {
		future.await;
		// Whatever task we had to do is done, return to userspace now
		context_switch(&process);
	};
	let f = Box::pin(block);
	let f = Pin::new(&mut f);
	block_until_never(f)
}