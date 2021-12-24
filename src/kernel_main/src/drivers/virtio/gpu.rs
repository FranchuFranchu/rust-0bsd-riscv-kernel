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
	type: VirtioGpuControlType,
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
        format: VirtioGpuFormats 
        le32 format; 
        le32 width; 
        le32 height; 
};