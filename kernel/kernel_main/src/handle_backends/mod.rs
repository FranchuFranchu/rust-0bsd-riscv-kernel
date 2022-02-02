use alloc::{collections::BTreeMap, sync::Arc};

use self::{
    filesystem::FilesystemHandleBackend, log_output::LogOutputHandleBackend,
    process_egg::ProcessEggBackend,
};
use crate::{handle::HandleBackend, lock::shared::RwLock};

pub mod filesystem;
pub mod log_output;
pub mod process_egg;

pub static BACKEND_CONSTRUCTORS: RwLock<
    BTreeMap<usize, fn() -> Arc<dyn HandleBackend + Send + Sync + 'static>>,
> = RwLock::new(BTreeMap::new());
pub static BACKEND_SINGLETONS: RwLock<
    BTreeMap<usize, Arc<dyn HandleBackend + Send + Sync + 'static>>,
> = RwLock::new(BTreeMap::new());

pub fn initialize_constructors() {
    BACKEND_CONSTRUCTORS
        .write()
        .insert(1, LogOutputHandleBackend::create_singleton);
    BACKEND_CONSTRUCTORS
        .write()
        .insert(2, FilesystemHandleBackend::create_singleton);
    BACKEND_CONSTRUCTORS
        .write()
        .insert(3, ProcessEggBackend::create_singleton);
}

pub async fn open(
    backend_id: &usize,
    fd_id: &usize,
    options: &[usize],
) -> Arc<dyn HandleBackend + Send + Sync + 'static> {
    let backend = {
        let lock = BACKEND_SINGLETONS.read();
        match lock.get(backend_id) {
            Some(backend) => {
                let b = backend.clone();
                drop(lock);
                b
            }
            None => {
                let backend = BACKEND_CONSTRUCTORS.read().get(backend_id).unwrap()();
                drop(lock);
                BACKEND_SINGLETONS
                    .write()
                    .insert(*backend_id, backend.clone());
                backend
            }
        }
    };
    backend.open(fd_id, options).await;
    backend
}
