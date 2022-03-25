use alloc::{boxed::Box, collections::BTreeMap};
use core::{borrow::BorrowMut, convert::TryInto};

use kernel_as_register::AsRegister;
use kernel_lock::future::RwLock;

use crate::{
    external_interrupt::ExternalInterruptFuture,
    handle::HandleBackend,
    handle_backends::{call_as_register_function, EncodedError},
    timeout::TimeoutFuture,
};

// Simply waits for an interrupt to happen
#[derive(Default)]
pub struct InterruptHandleBackend {
    // handle id -> interrupt number
    values: RwLock<BTreeMap<usize, usize>>,
}

#[derive(KError, Debug, AsRegister)]
pub enum InterruptError {
    NoPermission,
}

#[async_trait]
impl HandleBackend for InterruptHandleBackend {
    async fn open(&self, fd_id: &usize, options: &[usize]) -> Result<usize, EncodedError> {
        call_as_register_function::<InterruptError, _, _, _>(async move || {
            self.values.write().await.insert(*fd_id, options[0]);
            Ok(0)
        })
        .await
    }

    async fn read(
        &self,
        fd_id: &usize,
        buf: &mut [u8],
        options: &[usize],
    ) -> Result<usize, EncodedError> {
        call_as_register_function::<InterruptError, _, _, _>(async move || {
            let a = ExternalInterruptFuture::new(
                self.values
                    .write()
                    .await
                    .get(fd_id)
                    .unwrap()
                    .clone()
                    .try_into()
                    .unwrap(),
            );
            let _ = a.await;
            Ok(0)
        })
        .await
    }

    fn create_singleton() -> alloc::sync::Arc<dyn HandleBackend + Send + Sync + 'static>
    where
        Self: Sized,
    {
        alloc::sync::Arc::new(InterruptHandleBackend::default())
    }

    fn name(&self) -> &'static str {
        "InterruptHandleBackend"
    }
}
