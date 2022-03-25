use alloc::sync::Arc;

use kernel_lock::shared::{Mutex, RwLock};

use super::{VirtioDevice, VirtioDeviceType};
use crate::drivers::traits::net::GenericNetworkDevice;

#[repr(u8)]
pub enum GsoTypes {
    None = 0,
    TcpV4 = 1,
    Udp = 3,
    TcpV6 = 4,
    Ecn = 0x80,
}

struct NetworkControlPacketHeader {
    class: u8,
    command: u8,
}

struct NetworkDataPacketHeader {
    flags: u8,
    gso_type: GsoTypes,
    header_length: u16,
    checksum_start: u16,
    checksum_offset: u16,
    num_buffers: u16,
}

pub struct VirtioNetworkDevice {}

impl VirtioDeviceType for VirtioNetworkDevice {
    fn configure(
        device: Arc<Mutex<VirtioDevice>>,
    ) -> Result<Arc<RwLock<dyn to_trait::ToTraitAny + Send + Sync + Unpin>>, ()>
    where
        Self: Sized,
    {
        let r = Arc::new(RwLock::new(VirtioNetworkDevice {}));
        todo!()
    }
}
