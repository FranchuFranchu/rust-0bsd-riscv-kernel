//! Includes code for delegating different interrupts to different handlers
//!

use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};
use core::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicUsize, Ordering},
    task::{Context, Poll, Waker},
};

use crate::{hart::get_this_hart_meta, lock::shared::RwLock};

static EXTERNAL_INTERRUPT_HANDLERS: RwLock<BTreeMap<u32, Vec<Arc<dyn Fn(u32) + Send + Sync>>>> =
    RwLock::new(BTreeMap::new());

pub fn external_interrupt(id: u32) {
    if let Some(fns) = EXTERNAL_INTERRUPT_HANDLERS.read().get(&id) {
        for function in fns.iter() {
            function(id);
        }
    }
}

fn add_handler(id: u32, function: Arc<dyn Fn(u32) + Send + Sync>) {
    let mut lock = EXTERNAL_INTERRUPT_HANDLERS.write();
    match lock.get_mut(&id) {
        Some(expr) => expr.push(function),
        None => {
            let plic = &get_this_hart_meta().unwrap().plic;
            plic.set_enabled(id, true);
            plic.set_priority(id, 3);
            lock.insert(id, alloc::vec![function]);
        }
    };
}

fn remove_handler(id: u32, function: &Arc<dyn Fn(u32) + Send + Sync>) -> Result<(), ()> {
    let mut guard = EXTERNAL_INTERRUPT_HANDLERS.write();
    let v = guard.get_mut(&id).unwrap();
    let index = v
        .iter()
        .position(|r| {
            // If both Arcs were created from the same object, then this should always be correct
            // The dyn Fn vtable is only crated once, when the External Interrupt Handler is registered
            // (but i'm not actually sure though)
            #[allow(clippy::vtable_address_comparisons)]
            Arc::ptr_eq(r, function)
        })
        .unwrap();
    v.remove(index);
    if v.is_empty() {
        drop(v);
        let plic = &get_this_hart_meta().unwrap().plic;
        plic.set_enabled(id, false);
        plic.set_priority(id, 6);
        guard.remove(&id);
    }
    Ok(())
}

/// This acts as a guard; the handler is removed when this object is removed
pub struct ExternalInterruptHandler {
    id: u32,
    function: Arc<dyn Fn(u32) + Send + Sync>,
}

impl ExternalInterruptHandler {
    pub fn new(id: u32, function: Arc<dyn Fn(u32) + Send + Sync>) -> Self {
        add_handler(id, function.clone());
        Self { id, function }
    }
}

impl Drop for ExternalInterruptHandler {
    fn drop(&mut self) {
        remove_handler(self.id, &self.function).unwrap();
    }
}

use spin::Lazy;

#[derive(Default)]
pub struct ExternalInterruptFuture {
    wakers: Vec<Waker>,
    count: AtomicUsize,
    handler: Option<ExternalInterruptHandler>,
}

impl ExternalInterruptFuture {
    pub fn new(id: u32) -> NType {
        crate::context_switch::make_this_process_pending();
        let future = Arc::new(RwLock::new(ExternalInterruptFuture::default()));
        {
            let mut lock = future.write();
            let future = future.clone();
            lock.handler = Some(ExternalInterruptHandler::new(
                id,
                Arc::new(move |_| {
                    let mut lock = future.write();
                    for i in core::mem::take(&mut lock.wakers).into_iter() {
                        i.wake()
                    }
                    lock.count.fetch_add(1, Ordering::SeqCst);
                }),
            ));
        }
        NType(future)
    }
}

impl Future for ExternalInterruptFuture {
    type Output = usize;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if *self.count.get_mut() == 0 {
            self.wakers.push(cx.waker().clone());
            Poll::Pending
        } else {
            self.count.fetch_sub(1, Ordering::SeqCst);
            self.handler.take();
            Poll::Ready(*self.count.get_mut())
        }
    }
}

pub struct NType(Arc<RwLock<ExternalInterruptFuture>>);

impl Future for NType {
    type Output = usize;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut *self.0.write()).poll(cx)
    }
}
