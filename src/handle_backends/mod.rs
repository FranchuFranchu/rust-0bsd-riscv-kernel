use alloc::collections::BTreeMap;

use crate::{handle::HandleBackend, lock::shared::RwLock};

use self::log_output::LogOutputHandleBackend;
use alloc::sync::Arc;

pub mod log_output;
pub mod filesystem;

pub static BACKEND_CONSTRUCTORS: RwLock<BTreeMap<usize, fn() -> Arc<dyn HandleBackend + Send + Sync>>> = RwLock::new(BTreeMap::new());
pub static BACKEND_SINGLETONS: RwLock<BTreeMap<usize, Arc<dyn HandleBackend + Send + Sync>>> = RwLock::new(BTreeMap::new());

pub fn initialize_constructors() {
	BACKEND_CONSTRUCTORS.write().insert(1, LogOutputHandleBackend::create_singleton);
}

pub fn open(backend_id: &usize, fd_id: &usize, options: &[usize]) -> Arc<dyn HandleBackend + Send + Sync> {
	let lock = BACKEND_SINGLETONS.read();
	let backend = match lock.get(backend_id) {
	    Some(backend) => {
	    	backend.clone()
	    },
	    None => {
	    	let backend = BACKEND_CONSTRUCTORS.read().get(backend_id).unwrap()();
	    	drop(lock);
	    	BACKEND_SINGLETONS.write().insert(*backend_id, backend.clone());
	    	backend
	    },
	};
	backend.open(fd_id, options);
	backend
}