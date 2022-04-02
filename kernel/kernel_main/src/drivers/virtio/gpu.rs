use alloc::{
    borrow::ToOwned,
    boxed::Box,
    sync::{Arc, Weak},
};
use core::mem::{size_of, MaybeUninit};

use kernel_lock::shared::{Mutex, RwLock};
use kernel_util::{boxed_slice_with_alignment_uninit, struct_to_bytes};

use super::{
    future_util::{FutureVirtioDeviceType, WrappedVirtioDeviceType},
    SplitVirtqueue, VirtioDevice,
};
use crate::drivers::traits::block::BlockRequestFutureBuffer;

#[derive(Default, Copy, Clone, Debug)]
#[repr(C)]
pub struct ColorR8G8B8A8Unorm {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl ColorR8G8B8A8Unorm {
    fn rgba_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
    fn rgb_u8(r: u8, g: u8, b: u8) -> Self {
        Self {
            r,
            g,
            b,
            a: u8::MAX,
        }
    }
}

type Pixel = ColorR8G8B8A8Unorm;

#[repr(C)]
pub struct Screen {
    width: u32,
    height: u32,
    buffer: Box<[MaybeUninit<Pixel>]>,
}

impl Screen {
    fn new(width: u32, height: u32) -> Self {
        let mut buffer: Box<[MaybeUninit<Pixel>]> =
            boxed_slice_with_alignment_uninit((width * height) as usize * size_of::<Pixel>(), 4096);
        Self {
            width,
            height,
            buffer,
        }
    }

    fn addr(&self) -> usize {
        self.buffer.as_ptr() as usize
    }

    fn mem_entry(&self) -> MemEntry {
        MemEntry {
            addr: self.addr() as u64,
            length: (self.buffer.len() * size_of::<Pixel>()) as u32,
            padding: 0,
        }
    }

    fn draw_pixel(&mut self, pixel: Pixel, x: u32, y: u32) {
        assert!(x < self.width);
        assert!(y < self.height);
        self.buffer[(x + y * self.width) as usize].write(pixel);
    }

    fn draw_rect(&mut self, pixel: Pixel, rect: Rect) {
        for x in rect.x..rect.x + rect.width {
            for y in rect.y..rect.y + rect.height {
                self.draw_pixel(pixel, x, y);
            }
        }
    }
}
pub enum RequestWithData {
    GetDisplayInfo(),
    ResourceCreate2D(ResourceCreate2D),
    ResourceUnref(ResourceUnref),
    SetScanout(SetScanout),
    ResourceFlush(ResourceFlush),
    TransferToHost2D(ResourceTransferToHost2D),
    ResourceAttachBacking(ResourceAttachBacking),
    ResourceDetachBacking(ResourceDetachBacking),
    GetCapsetInfo(),
    GetCapset(),
    GetEdid(),
}

impl RequestWithData {
    fn control_type(&self) -> ControlType {
        use ControlType::*;
        use RequestWithData::*;
        match self {
            GetDisplayInfo(..) => CommandGetDisplayInfo,
            ResourceCreate2D(..) => CommandResourcesCreate2D,
            ResourceUnref(..) => CommandResourceUnref,
            SetScanout(..) => CommandSetScanout,
            ResourceFlush(..) => CommandResourceFlush,
            TransferToHost2D(..) => CommandTransferToHost2D,
            ResourceAttachBacking(..) => CommandResourceAttachBacking,
            ResourceDetachBacking(..) => CommandResourceDetachBacking,
            GetCapsetInfo(..) => CommandGetCapsetInfo,
            GetCapset(..) => CommandGetCapset,
            GetEdid(..) => CommandGetEdid,
        }
    }
    fn data_as_slice(&self) -> &[MaybeUninit<u8>] {
        match self {
            RequestWithData::GetDisplayInfo() => todo!(),
            RequestWithData::ResourceCreate2D(d) => struct_to_bytes(d),
            RequestWithData::ResourceUnref(d) => struct_to_bytes(d),
            RequestWithData::SetScanout(d) => struct_to_bytes(d),
            RequestWithData::ResourceFlush(d) => struct_to_bytes(d),
            RequestWithData::TransferToHost2D(d) => struct_to_bytes(d),
            RequestWithData::ResourceAttachBacking(d) => struct_to_bytes(d),
            RequestWithData::ResourceDetachBacking(d) => struct_to_bytes(d),
            RequestWithData::GetCapsetInfo() => todo!(),
            RequestWithData::GetCapset() => todo!(),
            RequestWithData::GetEdid() => todo!(),
        }
    }
}

#[repr(u32)]
enum ControlType {
    None = 0,
    /* 2d commands */
    CommandGetDisplayInfo = 0x0100,
    CommandResourcesCreate2D,
    CommandResourceUnref,
    CommandSetScanout,
    CommandResourceFlush,
    CommandTransferToHost2D,
    CommandResourceAttachBacking,
    CommandResourceDetachBacking,
    CommandGetCapsetInfo,
    CommandGetCapset,
    CommandGetEdid,

    /* cursor commands */
    CommandUpdateCursor = 0x0300,
    CommandMoveCursor,

    /* success responses */
    ResponseOkNoData = 0x1100,
    ResponseOkDisplayInfo,
    ResponseOkCapsetInfo,
    ResponseOkCapset,
    ResponseOkEdid,

    /* error responses */
    ResponseErrorUnspec = 0x1200,
    ResponseErrorOutOfMemory,
    ResponseErrorInvalidScanoutId,
    ResponseErrorInvalidResourceId,
    ResponseErrorInvalidContextId,
    ResponseErrorInvalidParameter,
}

impl Default for ControlType {
    fn default() -> Self {
        Self::None
    }
}

#[repr(u32)]
enum Formats {
    B8G8R8A8Unorm = 1,
    B8G8R8X8Unorm = 2,
    A8R8G8B8Unorm = 3,
    X8R8G8B8Unorm = 4,

    R8G8B8A8Unorm = 67,
    X8B8G8R8Unorm = 68,

    A8B8G8R8Unorm = 121,
    R8G8B8X8Unorm = 134,
}

#[derive(Default)]
#[repr(C)]
pub struct ControlHeader {
    r#type: ControlType,
    flags: u32,
    fence_id: u64,
    ctx_id: u32,
    padding: u32,
}

#[repr(C)]
/// > For any coordinates given 0,0 is top left, larger x moves right, larger y moves down.
pub struct Rect {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl Rect {
    fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[repr(C)]
pub struct ResourceCreate2D {
    resource_id: u32,
    format: Formats,
    width: u32,
    height: u32,
}
#[repr(C)]
pub struct ResourceFlush {
    rect: Rect,
    resource_id: u32,
    padding: u32,
}
#[repr(C)]
pub struct ResourceUnref {
    resource_id: u32,
    padding: u32,
}

#[repr(C)]
pub struct ResourceDetachBacking {
    resource_id: u32,
    padding: u32,
}
#[repr(C)]
pub struct ResourceAttachBacking {
    resource_id: u32,
    nr_entries: u32,
}

#[repr(C)]
pub struct ResourceTransferToHost2D {
    rect: Rect,
    offset: u64,
    resource_id: u32,
    padding: u32,
}

#[repr(C)]
pub struct MemEntry {
    addr: u64,
    length: u32,
    padding: u32,
}

#[repr(C)]
pub struct DisplayOne {
    rect: Rect,
    enabled: u32,
    flags: u32,
}

#[repr(C)]
pub struct SetScanout {
    rect: Rect,
    scanout_id: u32,
    resource_id: u32,
}

#[repr(C)]
pub struct ResponseDisplayInfo {
    pmodes: [DisplayOne; 16],
}

fn init() {}

pub struct VirtioGpuDriver {
    request_virtqueue: Mutex<SplitVirtqueue>,
    cursor_virtqueue: Mutex<SplitVirtqueue>,

    device: Arc<Mutex<VirtioDevice>>,
    this: Weak<RwLock<FutureVirtioDeviceType<Self>>>,
}

impl WrappedVirtioDeviceType for VirtioGpuDriver {
    type RequestMeta = RequestWithData;

    type RequestBuildingData = (RequestWithData, Option<Box<[u8]>>);

    type Trait = ();

    fn create_request(
        &mut self,
        data: Self::RequestBuildingData,
    ) -> super::future_util::RequestFuture<Self>
    where
        Self: Sized,
    {
        super::future_util::RequestFuture {
            driver: self.this.clone(),
            header: (),
            buffer: data.1.map(BlockRequestFutureBuffer::Owned),
            meta: data.0,
            descriptor_id: None,
            was_queued: false,
        }
    }

    fn device(&self) -> &kernel_lock::shared::Mutex<super::VirtioDevice> {
        &self.device
    }

    fn from_device(
        device: alloc::sync::Arc<kernel_lock::shared::Mutex<super::VirtioDevice>>,
    ) -> Self {
        let mut device_lock = device.lock();
        let request_virtqueue = Mutex::new(device_lock.configure_queue(0));
        let cursor_virtqueue = Mutex::new(device_lock.configure_queue(1));
        drop(device_lock);
        Self {
            request_virtqueue,
            cursor_virtqueue,
            device: device,
            this: Weak::new(),
        }
    }

    fn set_this(&mut self, this: Weak<RwLock<FutureVirtioDeviceType<Self>>>)
    where
        Self: Sized,
    {
        self.this = this;
    }

    fn begin_request(&self, descriptor_id: u16) {
        let mut vq_lock = self.request_virtqueue.lock();
        // Make the descriptor chain available and notify the device that the virtqueue is ready
        vq_lock.make_available(descriptor_id);
        self.device.lock().queue_ready(0);
    }

    fn do_request(&self, request: &mut super::future_util::RequestFuture<Self>) -> u16
    where
        Self: Sized,
    {
        let control_header = ControlHeader {
            r#type: request.meta.control_type(),
            ..Default::default()
        };
        let mut vq_lock = self.get_virtqueue(0).lock();

        let slice = struct_to_bytes(&control_header);
        let mut last = vq_lock.new_descriptor_from_boxed_slice(
            slice.to_owned().into_boxed_slice(),
            true,
            None,
        );

        let mut last = if let Some(buffer) = request.buffer.take() {
            Some(vq_lock.new_descriptor_from_boxed_slice(
                match buffer {
                    BlockRequestFutureBuffer::Owned(e) => e,
                    _ => unimplemented!(),
                },
                false,
                Some(last),
            ))
        } else {
            Some(last)
        };

        let slice = request.meta.data_as_slice();
        let mut last = vq_lock.new_descriptor_from_boxed_slice(
            slice.to_owned().into_boxed_slice(),
            false,
            last,
        );

        let slice = struct_to_bytes(&control_header);
        let mut last = vq_lock.new_descriptor_from_boxed_slice(
            slice.to_owned().into_boxed_slice(),
            false,
            Some(last),
        );

        last
    }

    fn get_virtqueue(
        &self,
        virtqueue_id: u16,
    ) -> &kernel_lock::shared::Mutex<super::SplitVirtqueue> {
        match virtqueue_id {
            0 => &self.request_virtqueue,
            1 => &self.cursor_virtqueue,
            _ => todo!(),
        }
    }
}

pub type VirtioGpuDriverE = FutureVirtioDeviceType<VirtioGpuDriver>;

pub fn init_dev(dev: &mut VirtioGpuDriver) {
    let mut create_and_send_buf = |req, buf: Option<Box<[u8]>>| {
        let mut req = dev.create_request((req, buf));
        let req = dev.do_request(&mut req);
        dev.begin_request(req);
    };
    let mut create_and_send = |req| create_and_send_buf(req, None);
    let width = 100;
    let height = 100;
    let mut screen = Screen::new(width, height);

    create_and_send(RequestWithData::ResourceCreate2D(ResourceCreate2D {
        width: screen.width,
        height: screen.height,
        resource_id: 1,
        format: Formats::R8G8B8A8Unorm,
    }));
    drop(create_and_send);
    create_and_send_buf(
        RequestWithData::ResourceAttachBacking(ResourceAttachBacking {
            resource_id: 1,
            nr_entries: 1,
        }),
        Some(
            struct_to_bytes(&screen.mem_entry())
                .iter()
                .map(|s| unsafe { s.assume_init() })
                .collect(),
        ),
    );

    let mut create_and_send = |req| create_and_send_buf(req, None);

    // Draw a cool curve!
    for x in 0..screen.width {
        screen.draw_pixel(
            Pixel::rgb_u8(255, 255, 255),
            x,
            (x * x) / screen.width % screen.height,
        );
    }
    create_and_send(RequestWithData::SetScanout(SetScanout {
        scanout_id: 0,
        resource_id: 1,
        rect: Rect::new(0, 0, screen.width, screen.height),
    }));
    create_and_send(RequestWithData::TransferToHost2D(
        ResourceTransferToHost2D {
            rect: Rect::new(0, 0, screen.width, screen.height),
            resource_id: 1,
            offset: 0,
            padding: 0,
        },
    ));
    create_and_send(RequestWithData::ResourceFlush(ResourceFlush {
        rect: Rect::new(0, 0, screen.width, screen.height),
        resource_id: 1,
        padding: 0,
    }));
}
