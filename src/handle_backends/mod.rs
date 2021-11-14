use alloc::collections::BTreeMap;

use crate::{handle::HandleBackend, lock::shared::RwLock};

use self::log_output::LogOutputHandleBackend;
use alloc::sync::Arc;

pub mod log_output;

pub static BACKEND_CONSTRUCTORS: RwLock<BTreeMap<usize, fn(&[usize]) -> Arc<dyn HandleBackend + Send + Sync>>> = RwLock::new(BTreeMap::new());

pub fn initialize_constructors() {
	BACKEND_CONSTRUCTORS.write().insert(1, LogOutputHandleBackend::open);
}

pub fn open(backend_id: &usize, options: &[usize]) -> Arc<dyn HandleBackend + Send + Sync> {
	BACKEND_CONSTRUCTORS.read().get(backend_id).unwrap()(options)
}