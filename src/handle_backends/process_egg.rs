//! Handle backend that allows processes to create "egg" child processes and set them up before executing them 


use alloc::{boxed::Box, collections::BTreeMap, sync::Arc};
use crate::lock::future::rwlock::RwLock;
use crate::handle::HandleBackend;
use crate::paging::sv39::RootTable;
use alloc::string::String;

struct ProcessEgg {
    name: String,
    root_table: Box<crate::paging::Table>,
}

struct ProcessEggBackend {
    handle_eggs: RwLock<BTreeMap<usize, ProcessEgg>>
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

    async fn open(&self, fd_id: &usize, options: &[usize]) {
        let egg = ProcessEgg {
            root_table: Box::new(crate::paging::Table::zeroed()),
            name: String::new(),
        };
        self.handle_eggs.write().await.insert(*fd_id, egg);
    }

    fn name(&self) -> &'static str {
        "ProcessEggBackend"
    }
    
    async fn write(&self, fd_id: &usize, buf: &[u8], options: &[usize]) -> Result<usize, usize> {
        Ok(0)
    }
    async fn read(&self, fd_id: &usize, buf: &mut [u8], options: &[usize]) -> Result<usize, usize> {
        Ok(0)
    }
}
