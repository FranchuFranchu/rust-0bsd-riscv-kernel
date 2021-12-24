struct FsHandle(usize);

use alloc::boxed::Box;

#[async_trait]
trait Filesystem {
    async fn root() -> FsHandle;
    async fn get_path(from: FsHandle, path: &str) -> FsHandle;
    async fn open_path(from: FsHandle, path: &str) -> FsHandle;
}

pub mod ext2;
