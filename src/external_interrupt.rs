//! Includes code for delegating different interrupts to different handlers
//!

use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};

use crate::lock::shared::RwLock;

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
            Arc::ptr_eq(&r, &function)
        })
        .unwrap();
    v.remove(index);
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
