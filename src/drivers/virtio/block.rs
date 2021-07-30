use super::{SplitVirtqueue, VirtioDevice, VirtioDeviceType};

// See https://docs.oasis-open.org/virtio/virtio/v1.1/cs01/virtio-v1.1-cs01.html#x1-2440004
// section 5.2.6

// This is followed by data and a status bit
#[repr(C)]
struct RequestHeader {
	r#type: u32,
	reserved: u32,
	sector: u64,
}

pub struct VirtioBlockDevice {
	request_virtqueue: SplitVirtqueue,
	device: VirtioDevice,
}

impl VirtioBlockDevice {
	fn instance_configure(&mut self) {
		self.device.driver_ok();
		let header = RequestHeader { r#type: 1, reserved: 0, sector: 0 };
		let mut data = alloc::vec![0u8; 512].into_boxed_slice();
		let mut status = alloc::vec![0xFFu8; 1].into_boxed_slice();
		
		// Write "ABCDE"
		data[0] = 65;
		data[1] = 66;
		data[2] = 67;
		data[3] = 68;
		data[4] = 69;
		
		// We create the descriptors backwards
		// so that we can get their descriptor indexes and chain each one to the following one
		let mut last = self.request_virtqueue.new_descriptor_from_boxed_slice(status, true, None);
		last = self.request_virtqueue.new_descriptor_from_boxed_slice(data, false, Some(last));
		last = self.request_virtqueue.new_descriptor_from_sized(&header, false, Some(last));
		// Make the descriptor chain available 
		self.request_virtqueue.make_available(last);
		
		self.device.queue_ready(0);
		
		info!("Writing block data to the device...");
	}
}

impl VirtioDeviceType for VirtioBlockDevice {
	fn configure(mut device: VirtioDevice) -> Result<(), ()> {
		VirtioBlockDevice { request_virtqueue: device.configure_queue(0), device: device }.instance_configure();
		Ok(())
	}
	fn negotiate_features(device: &mut VirtioDevice) {
		device.get_device_features(); // ignore their features
		device.set_driver_features(0);
		device.accept_features().unwrap();
	}
}