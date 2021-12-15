use alloc::collections::{BTreeMap, BinaryHeap};

use crate::{
    lock::shared::RwLock,
    paging::EntryBits::{RWX, VALID},
};

#[derive(Eq, PartialEq, Ord)]
pub struct VirtualBuffer {
    virt_addr: usize,
    phys_addr: usize,
    size: usize,
}

impl PartialOrd for VirtualBuffer {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        debug_assert!(self.virt_start() != other.virt_start());
        self.virt_start().partial_cmp(&other.virt_start())
    }
}

impl VirtualBuffer {
    pub fn virt_start(&self) -> usize {
        self.virt_addr
    }
    pub fn virt_end(&self) -> usize {
        self.virt_addr + self.size
    }
    pub fn phys_start(&self) -> usize {
        self.phys_addr
    }
    pub fn phys_end(&self) -> usize {
        self.phys_addr + self.size
    }
    pub unsafe fn slice_ref(&self) -> &[u8] {
        core::slice::from_raw_parts(self.virt_start() as *const u8, self.size)
    }
    pub unsafe fn slice_mut(&self) -> &[u8] {
        core::slice::from_raw_parts_mut(self.virt_start() as *mut u8, self.size)
    }
}

pub struct VirtualBufferRegistry {
    buffers: alloc::collections::BinaryHeap<VirtualBuffer>,
    buffer_handles: alloc::collections::BTreeMap<usize, crate::lock::shared::RwLock<usize>>,
    free_space_start: usize,
}

extern "C" {
    static _free_space_start: usize;
}

impl VirtualBufferRegistry {
    pub fn new() -> Self {
        Self {
            buffers: alloc::collections::BinaryHeap::new(),
            buffer_handles: alloc::collections::BTreeMap::new(),
            free_space_start: unsafe { &_free_space_start as *const usize as usize },
        }
    }
    pub fn new_buffer(&mut self, phys_addr: usize, size: usize) -> usize {
        let mut root_table = crate::paging::get_active_root_table(crate::cpu::read_satp()).unwrap();

        let buf_virtual_address = {
            let iter1 = self.buffers.iter();
            let mut iter2 = self.buffers.iter();
            iter2.next();
            let mut insert_into = None;

            for (idx, (current, next)) in iter1.zip(iter2).enumerate() {
                if (next.virt_start() - current.virt_end()) < size {
                    // This buffer fits in here
                    insert_into = Some(current.virt_end())
                }
            }
            if let Some(insert_into) = insert_into {
                insert_into
            } else if self.buffers.len() > 0 {
                self.buffers
                    .iter()
                    .nth(self.buffers.len() - 1)
                    .unwrap()
                    .virt_end()
            } else {
                self.free_space_start
            }
        };
        let buffer = VirtualBuffer {
            virt_addr: buf_virtual_address,
            phys_addr,
            size,
        };
        use core::ops::Mul;
        root_table.map(
            phys_addr.unstable_div_floor(4096).mul(4096),
            buf_virtual_address,
            size.unstable_div_ceil(4096).mul(4096),
            RWX | VALID,
        );

        self.buffers.push(buffer);
        return buf_virtual_address;
    }
}

static MAIN_REGISTRY: RwLock<Option<VirtualBufferRegistry>> = RwLock::new(None);

pub fn new_virtual_buffer(phys_addr: usize, size: usize) -> usize {
    let mut mreg_lock = MAIN_REGISTRY.write();
    let mut reg = match &mut *mreg_lock {
        Some(reg) => reg,
        None => {
            let buf = VirtualBufferRegistry::new();
            *mreg_lock = Some(buf);
            mreg_lock.as_mut().unwrap()
        }
    };
    reg.new_buffer(phys_addr, size)
}

// &_free_space_start as *const _ as usize
