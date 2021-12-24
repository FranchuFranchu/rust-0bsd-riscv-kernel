//! Handle backend that allows processes to create "egg" child processes and set them up before executing them

use alloc::{boxed::Box, collections::BTreeMap, string::String, sync::Arc};

use crate::{handle::HandleBackend, lock::future::rwlock::RwLock};

pub struct ProcessEgg {
    name: String,
    root_table: Box<crate::paging::Table>,
}

pub struct ProcessEggBackend {
    handle_eggs: RwLock<BTreeMap<usize, ProcessEgg>>,
}

#[async_trait]
impl<'this> HandleBackend for ProcessEggBackend {
    fn create_singleton() -> alloc::sync::Arc<dyn HandleBackend + Send + Sync + 'static>
    where
        Self: Sized,
    {
        Arc::new(Self {
            handle_eggs: RwLock::new(BTreeMap::new()),
        })
    }

    async fn open(&self, fd_id: &usize, _options: &[usize]) {
        let egg = ProcessEgg {
            root_table: Box::new(crate::paging::Table::zeroed()),
            name: String::new(),
        };
        self.handle_eggs.write().await.insert(*fd_id, egg);
    }

    fn name(&self) -> &'static str {
        "ProcessEggBackend"
    }

    async fn write(&self, _fd_id: &usize, _buf: &[u8], _options: &[usize]) -> Result<usize, usize> {
        Ok(0)
    }
    async fn read(
        &self,
        _fd_id: &usize,
        _buf: &mut [u8],
        _options: &[usize],
    ) -> Result<usize, usize> {
        Ok(0)
    }
}
