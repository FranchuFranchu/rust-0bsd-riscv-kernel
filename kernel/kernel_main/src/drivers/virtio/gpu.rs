#[derive(Default, Copy, Clone)]
#[repr(C)]
struct ColorR8G8B8A8Unorm {
	r: u8, g: u8, b: u8, a: u8
}

impl ColorR8G8B8A8Unorm {
	fn rgba_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
		Self { r, g, b, a }
	}
	fn rgb_u8(r: u8, g: u8, b: u8) -> Self {
		Self { r, g, b, a: u8::MAX }
	}
}

#[repr(C)]
struct Screen {
	width: u32,
	height: u32,
	buffer: Box<[MaybeUninit<Pixel>]>,
}

#[repr(C, u32)]
enum VirtioGpuControlType { 
 
        /* 2d commands */ 
        CommandGetDisplayInfo = 0x0100, 
        CommandResourcesCreate2D, 
        CommandResourceUnref,
        CommandSetScanout, 
        CommandResourceFlush, 
        CommandTransferToHost2D, 
        CommandResourceAttachBacking, 
        CommandResourceDetachBacking, 
        VIRTIO_GPU_CMD_GET_CAPSET_INFO, 
        VIRTIO_GPU_CMD_GET_CAPSET, 
        VIRTIO_GPU_CMD_GET_EDID, 
 
        /* cursor commands */ 
        VIRTIO_GPU_CMD_UPDATE_CURSOR = 0x0300, 
        VIRTIO_GPU_CMD_MOVE_CURSOR, 
 
        /* success responses */ 
        VIRTIO_GPU_RESP_OK_NODATA = 0x1100, 
        VIRTIO_GPU_RESP_OK_DISPLAY_INFO, 
        VIRTIO_GPU_RESP_OK_CAPSET_INFO, 
        VIRTIO_GPU_RESP_OK_CAPSET, 
        VIRTIO_GPU_RESP_OK_EDID, 
 
        /* error responses */ 
        VIRTIO_GPU_RESP_ERR_UNSPEC = 0x1200, 
        VIRTIO_GPU_RESP_ERR_OUT_OF_MEMORY, 
        VIRTIO_GPU_RESP_ERR_INVALID_SCANOUT_ID, 
        VIRTIO_GPU_RESP_ERR_INVALID_RESOURCE_ID, 
        VIRTIO_GPU_RESP_ERR_INVALID_CONTEXT_ID, 
        VIRTIO_GPU_RESP_ERR_INVALID_PARAMETER, 
}

#[repr(C, u32)]
enum VirtioGpuFormats {
	B8G8R8A8_UNORM = 1,
	B8G8R8X8_UNORM = 2,
	A8R8G8B8_UNORM = 3,
	X8R8G8B8_UNORM = 4,
 
        VIRTIO_GPU_FORMAT_R8G8B8A8_UNORM  = 67, 
        VIRTIO_GPU_FORMAT_X8B8G8R8_UNORM  = 68, 
 
        VIRTIO_GPU_FORMAT_A8B8G8R8_UNORM  = 121, 
        VIRTIO_GPU_FORMAT_R8G8B8X8_UNORM  = 134, 
}
 

#[repr(C)]
struct VirtioGpuControlHeader { 
	r#type: VirtioGpuControlType,
	flags: u32,
	fence_id: u64,
	ctx_id: u32,
	padding: u32,
}

#[repr(C)]
/// > For any coordinates given 0,0 is top left, larger x moves right, larger y moves down. 
struct VirtioGpuRect {
	x: u32,
	y: u32,
	width: u32,
	height: u32,
}

#[repr(C)]
struct VirtioGpuResourceCreate2D { 
        resource_id: u32,
        format: VirtioGpuFormat, 
        width: u32,
        height: u32, 
};
#[repr(C)]
struct VirtioGpuResourceFlush { 
        resource_id: u32,
        rect: VirtioGpuRect, 
};
#[repr(C)]
struct VirtioGpuResourceUnref { 
        resource_id: u32,
        padding: u32, 
};

#[repr(C)]
struct VirtioGpuResourceDetachBacking { 
        resource_id: u32,
        padding: u32, 
};
#[repr(C)]
struct VirtioGpuResourceAttachBacking {
	resource_id: u32,
	nr_entries: u32,
}


#[repr(C)]
struct VirtioGpuMemEntry {
	addr: u64,
	length: u32,
	padding: u32,
}

#[repr(C)]
struct VirtioGpuDisplayOne { 
        rect: VirtioGpuRect,
	enabled: u32,
	flags: u32,
};

#[repr(C)]
struct VirtioGpuSetScanout {
	rect: VirtioGpuRect,
	scanout_id: u32,
	resource_id: u32,
}

#[repr(C)]
struct VirtioGpuResponseDisplayInfo {
	pmodes: [VirtioGpuDisplayOne; 16],
}

fn init() {
	let width = 800;
	let height = 640;
	ResourceCreate2d {
		screen.width, screen.height,
		resource_id: 1,
		format: VirtioGpuFormats::R8G8B8A8Unorm,
	};
	AttachBacking {
		resource_id: 1,
		nr_entries: 1,
	};
	MemEntry {
		addr: screen.addr()
	};
	SetScanout {
		scanout_id: 0,
		resource_id: 1,
		rect: VirtioGpuRect::new(0, 0, screen.width, screen.height),
	};
	TransferToHost2d {
		rect: VirtioGpuRect::new(0, 0, screen.width, screen.height),
		resource_id: 1,
		offset: 1,
		padding: 0,
	};
	ResourceFlush {
		rect: VirtioGpuRect::new(0, 0, screen.width, screen.height),
		resource_id: 1,
		padding: 0,
	}
}
