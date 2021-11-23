//! An executor for kernel futures
//!
//! Quoting Wikipedia:
//! > In computer science, future, promise, delay, and
//! > deferred refer to constructs used for synchronizing program
//! > execution in some concurrent programming languages. They
//! > describe an object that acts as a proxy for a result that
//! > is initially unknown, usually because the computation of
//! > its value is not yet complete.

use alloc::sync::{Arc, Weak};
use alloc::{boxed::Box, collections::VecDeque, task::Wake};
use core::{
    future::Future,
    task::{Context, Waker},
};

use crate::lock::shared::Mutex;

#[derive(Clone)]
struct TaskWaker(Weak<Executor>, Weak<Task>);

pub struct Task {
    future: Mutex<Box<dyn Future<Output = ()> + Send + Unpin>>,
    waker: Mutex<Option<Waker>>,
    process_waker: Waker,
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.0
            .upgrade()
            .unwrap()
            .push_task(self.1.upgrade().unwrap());
        self.1.upgrade().unwrap().process_waker.wake_by_ref();
    }
}

pub struct Executor {
    queue: Mutex<VecDeque<Arc<Task>>>,
    this: Mutex<Weak<Self>>,
}

impl Executor {
    pub fn new() -> Arc<Self> {
        let t = Self {
            queue: Mutex::new(VecDeque::new()),
            this: Mutex::new(Weak::new()),
        };
        let t = Arc::new(t);
        *t.this.lock() = Arc::downgrade(&t);
        t
    }

    fn push_task(&self, task: Arc<Task>) {
        self.queue.lock().push_back(task)
    }
    pub fn push_future(&self, future: Box<dyn Future<Output = ()> + Send + Unpin>) {
        let task = Task {
            future: Mutex::new(future),
            waker: Mutex::new(None),
            process_waker: crate::process::Process::this().read().construct_waker(),
        };
        let task = Arc::new(task);
        *task.waker.lock() =
            Some(Arc::new(TaskWaker(self.this.lock().clone(), Arc::downgrade(&task))).into());
        self.queue.lock().push_back(task)
    }
    pub fn run_one(&self) -> Option<Option<Arc<Task>>> {
        let task = self.queue.lock().pop_front();

        let task = task?;

        info!("Exec {:?}", Arc::as_ref(&task) as *const _);

        use core::task::Poll;

        let result = {
            let waker: Waker = task.waker.lock().as_ref().unwrap().clone();

            let mut context = Context::from_waker(&waker);

            let mut guard = task.future.lock();

            let future = &mut *guard;

            core::pin::Pin::new(future).poll(&mut context)
        };

        match result {
            Poll::Ready(_) => Some(Some(task)),
            Poll::Pending => {
                self.queue.lock().push_front(task);
                Some(None)
            }
        }
    }
}
