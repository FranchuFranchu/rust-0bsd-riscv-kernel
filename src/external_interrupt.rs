//! Includes code for delegating different interrupts to different handlers
//! 

use alloc::{collections::BTreeMap, vec::Vec, boxed::Box};
use alloc::sync::Arc;
use spin::RwLock;

static EXTERNAL_INTERRUPT_HANDLERS: RwLock<BTreeMap<u32, Vec<Arc<dyn Fn(u32) + Send + Sync>>>> = RwLock::new(BTreeMap::new());

pub fn external_interrupt(id: u32) {
	info!("External int {}", id);
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
		None => { lock.insert(id, alloc::vec![function]); },
	};
	
}

fn remove_handler(id: u32, function: &Arc<dyn Fn(u32) + Send + Sync>) -> Result<(), ()> {
	let mut guard = EXTERNAL_INTERRUPT_HANDLERS.write();
	let v = guard.get_mut(&id).unwrap();
	println!("{:?}", id);
	let index = v.iter().position(|r| Arc::ptr_eq(r, function)).unwrap();
	v.remove(index);
	Ok(())
}

/// This acts as a guard; the handler is removed when this object is removed
pub struct ExternalInterruptHandler {
	id: u32,
	function: Arc<dyn Fn(u32) + Send + Sync>,
}

impl  ExternalInterruptHandler {
	pub fn new(id: u32, function: Arc<dyn Fn(u32) + Send + Sync>) -> Self {
		println!("register {:?}", Arc::as_ptr(&function));
		add_handler(id, function.clone());
		Self { id, function: function }
	}
}

impl Drop for ExternalInterruptHandler {
	fn drop(&mut self) {
		println!("dropped {:?}", Arc::as_ptr(&self.function));
		remove_handler(self.id, &self.function).unwrap();
	}
}