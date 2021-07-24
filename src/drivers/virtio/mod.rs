// For the legacy interface

use volatile_register::{RW, RO, WO};

// from xv6

pub enum StatusField {
	Acknowledge = 1,
	Driver = 2,
	Failed = 128,
	FeaturesOk = 8,
	DriverOk = 4,
	DeviceNeedsReset = 64,
}

struct VirtioMmio {
	magic_value: RO<u32>, // R. Magic value
	// 
	version: RO<u32>, // R. Device version number
	// Legacy device returns value 0x1.
	device_id: RO<u32>, // R. Virtio Subsystem Device ID
	// 
	vendor_id: RO<u32>, // R. Virtio Subsystem Vendor ID
	// 
	host_features: RO<u32>, // R. Flags representing features the device supports
	// 
	host_features_sel: WO<u32>, // W. Device (host) features word selection.
	
	_pad1: u32,
	_pad2: u32,
	
	// 0x20
	guest_features: WO<u32>, // W. Flags representing device features understood and activated by the driver
	// 
	guest_features_sel: WO<u32>, // W. Activated (guest) features word selection
	// 
	guest_page_size: WO<u32>, // W. Guest page size
	// The driver writes the guest page size in bytes to the register during initialization, before any queues are used. This value should be a power of 2 and is used by the device to calculate the Guest address of the first queue page (see QueuePFN).
	
	_pad3: u32,
	// 0x30
	
	queue_sel: WO<u32>, // W. Virtual queue index
	// Writing to this register selects the virtual queue that the following operations on the QueueNumMax, QueueNum, QueueAlign and QueuePFN registers apply to. The index number of the first queue is zero (0x0). .
	queue_num_max: RO<u32>, // R. Maximum virtual queue size
	// Reading from the register returns the maximum size of the queue the device is ready to process or zero (0x0) if the queue is not available. This applies to the queue selected by writing to QueueSel and is allowed only when QueuePFN is set to zero (0x0), so when the queue is not actively used.
	queue_num:  WO<u32>, // W. Virtual queue size
	// Queue size is the number of elements in the queue. Writing to this register notifies the device what size of the queue the driver will use. This applies to the queue selected by writing to QueueSel.
	queue_align: WO<u32>, // W. Used Ring alignment in the virtual queue
	// Writing to this register notifies the device about alignment boundary of the Used Ring in bytes. This value should be a power of 2 and applies to the queue selected by writing to QueueSel.
	queue_pfn: RW<u32>, // RW. Guest physical page number of the virtual queue
	// Writing to this register notifies the device about location of the virtual queue in the Guest’s physical address space. This value is the index number of a page starting with the queue Descriptor Table. Value zero (0x0) means physical address zero (0x00000000) and is illegal. When the driver stops using the queue it writes zero (0x0) to this register. Reading from this register returns the currently used page number of the queue, therefore a value other than zero (0x0) means that the queue is in use. Both read and write accesses apply to the queue selected by writing to QueueSel.
	_pad4: u32,
	_pad5: u32,
	_pad6: u32,
	
	// 0x50
	queue_notify: WO<u32>, // W. Queue notifier
	_pad7: [u8; 12],
	// 
	interrupt_status: RO<u32>, // R. Interrupt status
	
	interrupt_ack: WO<u32>, // W. Interrupt acknowledge
	_pad8: [u8; 8],
	status: RO<u32>, // RW. Device status
	// Reading from this register returns the current device status flags. Writing non-zero values to this register sets the status flags, indicating the OS/driver progress. Writing zero (0x0) to this register triggers a device reset. The device sets QueuePFN to zero (0x0) for all queues in the device. Also see 3.1 Device Initialization.
	_pad9: [u8; (0x100 - 0x70)],
	config: RW<u32>, // (and further) // RW. Configuration space
	// 
}

struct VirtioDevice {
	// Pointer to the configuration
	configuration: *mut VirtioMmio,
}

impl VirtioDevice {
	pub unsafe fn new(base: *mut VirtioMmio) -> Self {
		Self {
			configuration: base,
		}
	}
	pub fn configure(&mut self) {
		unsafe { 
			// 1. Reset the device
			(*self.configuration).config.write(0);
			// 2. Set the ACKNOWLEDGE status bit: the guest OS has noticed the device. 
			(*self.configuration).config.write((*self.configuration).config.read() | (StatusField::Acknowledge as u32))
		}
	}
	pub fn configure_queue(&mut self, queue: u16) {
		unsafe { 
			(*self.configuration).queue_sel.write(queue.into());
		}
	}
}