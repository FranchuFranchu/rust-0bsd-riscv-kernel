use core::ops::{Index, IndexMut};

pub mod sv32;
#[cfg(target_arch = "riscv64")]
pub mod sv39;
#[cfg(target_arch = "riscv64")]
pub mod sv48;

pub mod EntryBits {
    // The V bit indicates whether the PTE is valid; if it is 0, all other bits in the PTE are don’t-cares and may be used freely by software.
    pub const VALID: usize = 1 << 0;
    // The permission bits, R, W, and X, indicate whether the page is readable, writable, and executable, respectively.When all three are zero, the PTE is a pointer to the next level of the page table; otherwise, it isa leaf PTE. Writable pages must also be marked readable; the contrary combinations are reservedfor future use.  Table 4.4 summarizes the encoding of the permission bits.
    // XWR Meaning
    // 000 Pointer to next level of page table
    // 001 Read-only page
    // 010 Reserved for future use
    // 011 Read-write page
    // 100 Execute-only page
    // 101 Read-execute page
    // 110 Reserved for future use
    // 111 Read-write-execute page
    pub const READ: usize = 1 << 1;
    pub const WRITE: usize = 1 << 2;
    pub const EXECUTE: usize = 1 << 3;
    // The U bit indicates whether the page is accessible to user mode.  U-mode software may only accessthe page when U=1.  If the SUM bit in thesstatusregister is set, supervisor mode software mayalso access pages with U=1.
    pub const USER: usize = 1 << 4;
    // The G bit designates aglobalmapping.  Global mappings are those that exist in all address spaces.For non-leaf PTEs, the global setting implies that all mappings in the subsequent levels of the pagetable are global.  Note that failing to mark a global mapping as global merely reduces performance,whereas  marking  a  non-global  mapping  as  global  is  a  software  bug  that,  after  switching  to  anaddress space with a different non-global mapping for that address range, can unpredictably resultin either mapping being used.
    pub const GLOBAL: usize = 1 << 5;
    // Each leaf PTE contains an accessed (A) and dirty (D) bit.  The A bit indicates the virtual page hasbeen read, written, or fetched from since the last time the A bit was cleared.  The D bit indicatesthe virtual page has been written since the last time the D bit was cleared.
    pub const ACCESSED: usize = 1 << 6;
    pub const DIRTY: usize = 1 << 7;

    pub const ADDRESS_MASK: usize = usize::MAX ^ ((1 << 8) - 1);
    pub const RWX: usize = 2 | 4 | 8;

    pub const CODE_SUPERVISOR: usize = 1 << 1 | 1 << 3 | 1;
    pub const DATA_SUPERVISOR: usize = 1 << 1 | 1 << 2 | 1;
}

#[derive(Default, Copy, Clone, Debug)]
pub struct Entry {
    pub value: usize,
}

impl Entry {
    pub const fn zeroed() -> Self {
        Entry { value: 0 }
    }
}

impl Entry {
    /// # Safety
    /// The entry's value must be a valid physical address pointer
    pub unsafe fn as_table_mut(&mut self) -> &mut Table {
        (((self.value & EntryBits::ADDRESS_MASK) << 2) as *mut Table)
            .as_mut()
            .unwrap()
    }
    /// # Safety
    /// The entry's value must be a valid physical address pointer
    pub unsafe fn as_table(&self) -> &Table {
        (((self.value & EntryBits::ADDRESS_MASK) << 2) as *mut Table)
            .as_ref()
            .unwrap()
    }

    pub unsafe fn try_as_table_mut(&mut self) -> Option<&mut Table> {
        if self.is_leaf() {
            None
        } else {
            Some(self.as_table_mut())
        }
    }
    pub unsafe fn try_as_table(&self) -> Option<&Table> {
        if self.is_leaf() {
            None
        } else {
            Some(self.as_table())
        }
    }

    pub fn is_leaf(&self) -> bool {
        (self.value & EntryBits::RWX) != 0
    }
    /// This takes a leaf entry and turns it into a reference to a page table with the same effect.
    /// Increment should be one of the PAGE_SIZE, MEGAPAGE_SIZE, GIGAPAGE_SIZE, etc constants
    /// If this entry is a megapage, for example, the increment should be PAGE_SIZE

    pub unsafe fn split(&mut self, increment: usize) {
        println!("S {:p}", self);
        use alloc::boxed::Box;

        let mut table = Box::new(Table::zeroed());
        let mut current_address = self.value & EntryBits::ADDRESS_MASK;
        //info!("{:?}", unsafe { *((current_address as *const u32).add(10)) });
        let flags = self.value & !(EntryBits::ADDRESS_MASK);

        for entry in table.entries.iter_mut() {
            entry.value = flags | current_address;
            current_address += increment >> 2;
        }
        self.value = 1 | ((&*table as *const Table as usize) >> 2);
        Box::leak(table);
        println!("{:x}", self.value);

        debug_assert!(!self.is_leaf());
    }
}

#[repr(C)]
#[repr(align(4096))]
#[derive(Debug)]
pub struct Table {
    pub entries: [Entry; 512],
}

impl Table {
    pub const fn zeroed() -> Self {
        Table {
            entries: [Entry { value: 0 }; 512],
        }
    }
}

impl Index<usize> for Table {
    type Output = Entry;
    fn index(&self, idx: usize) -> &Entry {
        &self.entries[idx]
    }
}

impl IndexMut<usize> for Table {
    fn index_mut(&mut self, idx: usize) -> &mut Entry {
        &mut self.entries[idx]
    }
}

pub trait Paging {
    fn map(&mut self, physical_addr: usize, virtual_addr: usize, length: usize, flags: usize) {}
}

pub unsafe fn enable(root_table_physical: usize) {}

pub static mut PAGE_TABLE_TABLE: Table = Table::zeroed();
#[link_section = ".data"]
pub static mut ROOT_PAGE: Table = Table::zeroed();

pub const ENTRY_COUNT: usize = 512;
pub const PAGE_ALIGN: usize = 4096;
pub const PAGE_SIZE: usize = PAGE_ALIGN;
pub const MEGAPAGE_SIZE: usize = PAGE_ALIGN * ENTRY_COUNT;
#[cfg(target_arch = "riscv64")]
pub const GIGAPAGE_SIZE: usize = PAGE_ALIGN * ENTRY_COUNT * ENTRY_COUNT;
#[cfg(target_arch = "riscv64")]
pub const TERAPAGE_SIZE: usize = PAGE_ALIGN * ENTRY_COUNT * ENTRY_COUNT * ENTRY_COUNT;
