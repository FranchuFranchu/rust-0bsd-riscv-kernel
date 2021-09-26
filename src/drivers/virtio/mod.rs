// For the legacy interface

pub mod block;

use alloc::{boxed::Box, collections::BTreeMap, sync::Arc, vec::Vec};
use core::{alloc::Layout, convert::TryInto, future::Future, slice, task::Waker};

use itertools::Itertools;
use volatile_register::{RO, RW, WO};

use self::block::VirtioBlockDevice;
use crate::{lock::shared::Mutex, paging::PAGE_ALIGN};

// from xv6
pub enum StatusField {
    Acknowledge = 1,
    Driver = 2,
    Failed = 128,
    FeaturesOk = 8,
    DriverOk = 4,
    DeviceNeedsReset = 64,
}

#[derive(Debug)]
#[repr(C)]
pub struct VirtqueueDescriptor {
    /// Physical address
    address: u64,
    length: u32,
    flags: u16,
    next: u16,
}

/// Legacy layout
#[repr(C)]
pub struct VirtioMmio {
    // 0x0
    magic_value: RO<u32>, // R. Magic value
    //
    version: RO<u32>, // R. Device version number
    // Legacy device returns value 0x1.
    device_id: RO<u32>, // R. Virtio Subsystem Device ID
    //
    vendor_id: RO<u32>, // R. Virtio Subsystem Vendor ID
    // 0x10
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
    queue_num: WO<u32>, // W. Virtual queue size
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
    status: RW<u32>, // RW. Device status
    // Reading from this register returns the current device status flags. Writing non-zero values to this register sets the status flags, indicating the OS/driver progress. Writing zero (0x0) to this register triggers a device reset. The device sets QueuePFN to zero (0x0) for all queues in the device. Also see 3.1 Device Initialization.
    _pad9: [u8; (0x100 - 0x70)],
    config: RW<u32>, // (and further) // RW. Configuration space
                     //
}

#[derive(Debug)]
pub struct VirtioDevice {
    // Pointer to the configuration
    configuration: *mut VirtioMmio,
    queue_used_sizes_align: BTreeMap<u16, (u16, u16, u32)>,
    waiting_wakers: Vec<Waker>,
    changed_queue: Option<u16>,
}

// SAFETY: the only reason raw pointers arent thread safe is backwards compatibility
unsafe impl Send for VirtioDevice {}

/// A useful handle over a dynamically-sized pointer
#[derive(Debug)]
pub struct SplitVirtqueue {
    // See section 2.6 of https://docs.oasis-open.org/virtio/virtio/v1.1/cs01/virtio-v1.1-cs01.html#x1-230005
    /// This pointer was allocated with Box::leak() and will then be reconstructed Box:from_raw before dropping
    /// The layout of the data pointed to by this pointer is:
    /// Virtqueue Part      Alignment   Size
    /// Descriptor Table    16          16∗(Queue Size)
    /// Available Ring      2           6 + 2∗(Queue Size)
    /// Used Ring           4           6 + 8∗(Queue Size)
    pointer: *mut u8,
    size: u16,
    first_free_descriptor: u16,
    guest_used_ring_idx: u16,
}
unsafe impl Send for SplitVirtqueue {}

/// This struct iterates over each
/// of the descriptor data
#[derive(Copy, Clone)]
pub struct SplitVirtqueueDescriptorChainIterator<'a> {
    queue: &'a SplitVirtqueue,
    pointed_chain: Option<u16>,
}

impl<'a> SplitVirtqueueDescriptorChainIterator<'a> {
    fn fold_to_vec(&mut self) -> Vec<u8> {
        self.map(|s| {
            let mut v = Vec::new();
            v.extend_from_slice(s);
            v
        })
        .concat()
    }
}

impl<'a> Iterator for SplitVirtqueueDescriptorChainIterator<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.pointed_chain == None {
            return None;
        }

        let descriptor = self.queue.get_descriptor(self.pointed_chain.unwrap());
        if descriptor.address == 0 {
            return None;
        }
        if descriptor.flags & 1 == 0 {
            self.pointed_chain = None
        } else {
            self.pointed_chain = Some(descriptor.next)
        }

        // TODO fix the other parts of this module so that this is sound?
        let s = unsafe {
            slice::from_raw_parts_mut(descriptor.address as *mut u8, descriptor.length as usize)
        };
        Some(s)
    }
}

#[repr(C)]
pub struct SplitVirtqueueUsedRing {
    idx: u32,
    len: u32,
}

impl SplitVirtqueue {
    #[inline]
    fn descriptor_table_size(size: &u16) -> usize {
        16 * *size as usize
    }
    #[inline]
    fn available_ring_size(size: &u16) -> usize {
        6 + 2 * *size as usize
    }
    #[inline]
    fn used_ring_size(size: &u16) -> usize {
        6 + 8 * *size as usize
    }
    #[inline]
    fn align(value: usize) -> usize {
        if ((&value) & 0xFFF) == 0 {
            value
        } else {
            (value & (!0xFFF)) + 0x1000
        }
    }
    #[inline]
    fn descriptor_table_offset(size: &u16) -> usize {
        0usize
    }
    #[inline]
    fn available_ring_offset(size: &u16) -> usize {
        Self::descriptor_table_size(size)
    }
    #[inline]
    fn used_ring_offset(size: &u16) -> usize {
        Self::align(Self::descriptor_table_size(&size) + Self::available_ring_size(&size))
    }
    #[inline]
    fn memory_size(size: &u16) -> usize {
        Self::used_ring_offset(&size) + Self::align(Self::used_ring_size(&size))
    }
    fn new(size: u16) -> SplitVirtqueue {
        use core::alloc::GlobalAlloc;

        use crate::allocator::ALLOCATOR;

        // Allocate page-aligned memory
        let mem_size = Self::memory_size(&size);
        let pointer = unsafe { ALLOCATOR.alloc(Layout::from_size_align(mem_size, 4096).unwrap()) };

        // Zero out the memroy
        unsafe { slice::from_raw_parts_mut(pointer, mem_size).fill(0) }

        SplitVirtqueue {
            pointer: pointer as _,
            size,
            first_free_descriptor: 0,
            guest_used_ring_idx: 0,
        }
    }

    fn get_descriptor(&self, index: u16) -> &VirtqueueDescriptor {
        if index > self.size {
            panic!("Out of range!")
        }
        // TODO This breaks aliasing rules if it's called twice!
        unsafe {
            (self.pointer.add(Self::descriptor_table_offset(&self.size))
                as *const VirtqueueDescriptor)
                .add(index as usize)
                .as_ref()
                .unwrap()
        }
    }

    fn get_descriptor_mut(&mut self, index: u16) -> &mut VirtqueueDescriptor {
        if index > self.size {
            panic!("Out of range!")
        }
        // TODO This breaks aliasing rules if it's called twice!
        unsafe {
            (self.pointer.add(Self::descriptor_table_offset(&self.size))
                as *mut VirtqueueDescriptor)
                .add(index as usize)
                .as_mut()
                .unwrap()
        }
    }

    fn allocate_descriptor(&self) -> u16 {
        // Finds an unused descriptor to use
        for i in 0..self.size {
            if self.get_descriptor(i).address == 0 {
                return i;
            }
        }
        panic!("No descriptor found!");
    }

    pub unsafe fn new_descriptor_from_address(
        &mut self,
        address: *const (),
        size: usize,
        device_writable: bool,
        chain: Option<u16>,
    ) -> u16 {
        let descriptor_index = self.allocate_descriptor();

        *self.get_descriptor_mut(descriptor_index) = VirtqueueDescriptor {
            address: address as u64,
            length: size as u32,
            flags: if chain != None { 1 } else { 0 } | if device_writable { 2 } else { 0 },
            next: chain.unwrap_or(0),
        };

        descriptor_index
    }

    pub fn new_descriptor_from_static_buffer(
        &mut self,
        buffer: &'static [u8],
        device_writable: bool,
        chain: Option<u16>,
    ) -> u16 {
        unsafe {
            self.new_descriptor_from_address(
                buffer.as_ptr() as _,
                buffer.len(),
                device_writable,
                chain,
            )
        }
    }

    pub fn new_descriptor_from_static_buffer_mut(
        &mut self,
        buffer: &'static mut [u8],
        chain: Option<u16>,
    ) -> u16 {
        unsafe {
            self.new_descriptor_from_address(buffer.as_ptr() as _, buffer.len(), false, chain)
        }
    }

    pub fn new_descriptor_from_sized<T: Sized>(
        &mut self,
        buffer: &T,
        device_writable: bool,
        chain: Option<u16>,
    ) -> u16 {
        unsafe {
            self.new_descriptor_from_address(
                buffer as *const T as _,
                core::mem::size_of_val(buffer),
                device_writable,
                chain,
            )
        }
    }

    /// Note that this leaks "buffer"
    /// Whoever is using this needs to make sure to run Box::from_raw on the Buffer when needed
    pub fn new_descriptor_from_boxed_slice(
        &mut self,
        buffer: Box<[u8]>,
        device_writable: bool,
        chain: Option<u16>,
    ) -> u16 {
        let len = buffer.len();
        unsafe {
            self.new_descriptor_from_address(
                Box::into_raw(buffer) as _,
                len,
                device_writable,
                chain,
            )
        }
    }

    /// Increments the index field in the available ring
    /// and returns the old value
    pub fn add_available_ring_idx(&mut self) -> u16 {
        unsafe {
            let ring_ptr =
                (self.pointer.add(Self::available_ring_offset(&self.size)) as *mut u16).add(1);
            let old = *ring_ptr;
            *ring_ptr += 1;
            if *ring_ptr == self.size {
                warn!("Overflow in available queue");
                *ring_ptr = 0;
            }
            old
        }
    }

    /// Gets the index field in the available ring
    pub fn get_available_ring_idx(&self) -> u16 {
        unsafe {
            let ring_ptr =
                (self.pointer.add(Self::available_ring_offset(&self.size)) as *mut u16).add(1);
            *ring_ptr
        }
    }

    pub fn get_available_ring_ptr(&mut self, index: u16) -> *mut u16 {
        unsafe {
            use core::ops::Add;
            (self.pointer.add(Self::available_ring_offset(&self.size)) as *mut u16)
                .add(2)
                .add(index as usize)
        }
    }

    pub fn get_device_used_ring_idx(&mut self) -> u16 {
        unsafe {
            use core::ops::Add;
            *((self.pointer.add(Self::used_ring_offset(&self.size)) as *mut u16).add(1))
        }
    }

    pub fn get_used_ring_ptr(&mut self, index: u16) -> *mut SplitVirtqueueUsedRing {
        unsafe {
            use core::ops::Add;
            ((self.pointer.add(Self::used_ring_offset(&self.size)) as *mut u16).add(2)
                as *mut SplitVirtqueueUsedRing)
                .add(index as usize)
        }
    }

    /// Adds a descriptor to the available ring
    /// making it available to the device
    pub fn make_available(&mut self, descriptor: u16) {
        // Get the first free index
        let old = self.get_available_ring_idx();
        unsafe { *self.get_available_ring_ptr(old) = descriptor }
        self.add_available_ring_idx();
    }

    pub fn pop_used_element(&mut self) -> Option<*mut SplitVirtqueueUsedRing> {
        if self.guest_used_ring_idx + 1 != self.get_device_used_ring_idx() {
            return None;
        }
        let v = self.get_used_ring_ptr(self.guest_used_ring_idx);
        self.guest_used_ring_idx = self.guest_used_ring_idx.wrapping_add(1);
        Some(v)
    }

    pub fn pop_used_element_to_iterator<'this>(
        &'this mut self,
    ) -> SplitVirtqueueDescriptorChainIterator<'this> {
        let u = self.pop_used_element().unwrap();
        SplitVirtqueueDescriptorChainIterator {
            queue: self,
            pointed_chain: Some(unsafe { (*u).idx } as u16),
        }
    }

    /// Returns the "Guest physical page number of the virtual queue"
    /// this is pointer / PAGE_SIZE in our case
    fn pfn(&self) -> usize {
        (self.pointer as usize) / (PAGE_ALIGN)
    }
}

pub enum VirtioDriver {
    Block(Arc<Mutex<VirtioBlockDevice>>),
}

use core::task::Poll;
impl Future for VirtioDevice {
    type Output = u16;

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> Poll<Self::Output> {
        if let Some(id) = self.next_changed_used_ring_queue() {
            Poll::Ready(id)
        } else if let Some(id) = self.changed_queue {
            self.changed_queue = None;
            Poll::Ready(id)
        } else {
            self.waiting_wakers.push(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl Drop for SplitVirtqueue {
    fn drop(&mut self) {
        unsafe { drop(Box::from_raw(self.pointer)) }
    }
}

impl VirtioDevice {
    pub unsafe fn new(base: *mut VirtioMmio) -> Self {
        Self {
            configuration: base,
            queue_used_sizes_align: BTreeMap::new(),
            changed_queue: None,
            waiting_wakers: Vec::new(),
        }
    }
    pub fn configure(&mut self) {
        unsafe {
            // 1. Reset the device
            (*self.configuration).status.write(0);
            // 2. Set the ACKNOWLEDGE status bit: the guest OS has noticed the device.
            (*self.configuration)
                .status
                .write((*self.configuration).status.read() | StatusField::Acknowledge as u32);
            (*self.configuration)
                .status
                .write((*self.configuration).status.read() | StatusField::Driver as u32);
        }
    }

    pub fn get_virtqueue_address_size(&self, queue: u16) -> Option<(*const (), u16)> {
        let (_, size, align) = self.queue_used_sizes_align.get(&queue)?;
        let address: usize = unsafe {
            (*self.configuration).queue_sel.write(queue.into());
            ((*self.configuration).queue_pfn.read()) * align
        }
        .try_into()
        .unwrap();
        Some((address as _, *size))
    }

    pub fn configure_queue(&mut self, queue: u16) -> SplitVirtqueue {
        let virtq;
        unsafe {
            // 1. Select the queue writing its index (first queue is 0) to QueueSel.
            (*self.configuration).queue_sel.write(queue.into());

            // Check if the queue is not already in use: read QueuePFN, expecting a returned value of zero (0x0).

            assert!((*self.configuration).queue_pfn.read() == 0);

            // Read maximum queue size (number of elements) from QueueNumMax. If the returned value is zero (0x0) the queue is not available.
            let max_virtqueue_size = (*self.configuration).queue_num_max.read() as u16;
            assert!(max_virtqueue_size != 0);

            // The driver should choose a queue size smaller than or equal to QueueNumMax.
            let virtqueue_size = max_virtqueue_size;

            // Allocate and zero the queue pages in contiguous virtual memory, aligning the Used Ring to an optimal boundary (usually page size).
            let align = crate::paging::PAGE_ALIGN; // 4096 in RISC-V

            virtq = SplitVirtqueue::new(virtqueue_size);

            self.queue_used_sizes_align
                .insert(queue, (0, virtqueue_size, align.try_into().unwrap()));

            // Notify the device about the queue size by writing the size to QueueNum.
            (*self.configuration).queue_num.write(virtqueue_size as u32);
            // Notify the device about the used alignment by writing its value in bytes to QueueAlign
            (*self.configuration).queue_align.write(align as u32);
            (*self.configuration).guest_page_size.write(align as u32);

            // Write the physical number of the first page of the queue to the QueuePFN register.
            (*self.configuration).queue_pfn.write(virtq.pfn() as u32);
        }
        virtq
    }

    /// Sets the features_ok bit and then checks if it's still there
    /// Returns Ok if it is, otherwise the device doesn't support this subset of features
    /// and Err is returned
    pub fn accept_features(&mut self) -> Result<(), ()> {
        unsafe {
            (*self.configuration)
                .status
                .write((*self.configuration).status.read() | (StatusField::FeaturesOk as u32));

            // Check if it's still set
            if ((*self.configuration).status.read() & StatusField::FeaturesOk as u32) != 0 {
                Ok(())
            } else {
                Err(())
            }
        }
    }

    pub fn is_present(&mut self) -> bool {
        unsafe { (*self.configuration).device_id.read() != 0 }
    }

    pub fn queue_ready(&mut self, queue: u16) {
        unsafe {
            (*self.configuration).queue_notify.write(queue.into());
        }
    }

    /// moves self into a driver
    pub fn make_driver(this: Arc<Mutex<Self>>) -> Option<VirtioDriver> {
        let id = unsafe { (*this.lock().configuration).device_id.read() };
        match id {
            2 => {
                // Block device
                VirtioBlockDevice::negotiate_features(&mut this.lock());
                let dev = VirtioBlockDevice::configure(this).unwrap();
                Some(VirtioDriver::Block(dev))
            }
            _ => {
                warn!("Unknown/Unimplemented VirtIO device type: {}", unsafe {
                    (*this.lock().configuration).device_id.read()
                });
                None
            }
        }
    }

    pub fn get_device_features(&mut self) -> u32 {
        unsafe { (*self.configuration).host_features.read() }
    }
    pub fn set_driver_features(&mut self, features: u32) {
        unsafe { (*self.configuration).guest_features.write(features) }
    }
    pub fn driver_ok(&mut self) {
        unsafe {
            (*self.configuration)
                .status
                .write((*self.configuration).status.read() | StatusField::DriverOk as u32)
        };
    }

    /// Should be called on an interrupt. This may wake up some used buffer wakers
    /// The reason this takes a mutex to self is to allow the waker to lock the VirtioDevice
    /// without deadlocking
    pub fn on_interrupt(this: &Mutex<Self>) {
        let interrupt_cause = unsafe { (*this.lock().configuration).interrupt_status.read() };
        if (interrupt_cause & (1 << 0)) != 0 {
            // Used Buffer Notification
            // Wake up all relevant wakers
            while let Some(queue_id) = {
                // This is done to make sure this is unlocked at the start of each iteration of the loop
                let b = this.lock().next_changed_used_ring_queue();
                b
            } {
                // First we create a list of wakers with the VirtioDevice locked
                let wakers;
                {
                    let mut this = this.lock();
                    this.changed_queue = Some(queue_id);
                    wakers = this.waiting_wakers.clone();
                }

                // Then we wake all of them with the VirtioDevice unlocked
                for i in wakers {
                    i.wake_by_ref()
                }
            }
        }
        // TODO this won't work well if more than one waker is waiting on the device!
        // this.lock().changed_queue = None;

        unsafe {
            (*this.lock().configuration)
                .interrupt_ack
                .write(interrupt_cause)
        };
    }

    /// Gets a virtual queue number whose used ring has changed since the last time it was returned from this function
    pub fn next_changed_used_ring_queue(&mut self) -> Option<u16> {
        // Iterate over all virtqueues, then when we
        // find one where the driver used ring index we stored
        // and the device used index are different, we return a
        // tuple from the block with the label_break_value feature

        // this is to prevent aliasing violations when we modify the current
        // driver used ring index from inside the loop
        if let Some((index, change_used_index_to)) = 'block: {
            for (idx, (driver_used_ring_index, size, align)) in self.queue_used_sizes_align.iter() {
                let addr = self.get_virtqueue_address_size(*idx).unwrap().0;
                let device_used_index = unsafe {
                    ((addr as *const u8).add(SplitVirtqueue::used_ring_offset(size)) as *mut u16)
                        .add(1)
                        .read()
                };
                if device_used_index != *driver_used_ring_index {
                    // This means the device has added an entry to the used ring!
                    // TODO check for overflow?
                    break 'block Some((*idx, device_used_index));
                }
            }
            None
        } {
            self.queue_used_sizes_align.get_mut(&index)?.0 = change_used_index_to;
            Some(index)
        } else {
            // No device changed
            None
        }
    }
}

pub trait VirtioDeviceType {
    fn configure(device: Arc<Mutex<VirtioDevice>>) -> Result<Arc<Mutex<Self>>, ()>
    where
        Self: Sized;

    /// Negotiate the accepted features with the device
    /// By default, this rejects all features
    fn negotiate_features(device: &mut VirtioDevice)
    where
        Self: Sized,
    {
        device.get_device_features(); // ignore their features
        device.set_driver_features(0);
        device.accept_features().unwrap(); // We don't care if our features aren't accepted
    }

    fn on_used_queue_ready(&self, queue: u16) {}

    // Called directly by the interrupt handler outside of the VirtIO modulse
    fn on_interrupt(&self);
}
