pub mod sv32;
pub mod sv39;
pub mod sv48;

#[repr(usize)]
enum EntryBits {
	// The V bit indicates whether the PTE is valid; if it is 0, all other bits in the PTE are don’t-cares and may be used freely by software.
	Valid = 1 << 0,
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
	Read = 1 << 1,
	Write = 1 << 2,
	Execute = 1 << 3,
	// The U bit indicates whether the page is accessible to user mode.  U-mode software may only accessthe page when U=1.  If the SUM bit in thesstatusregister is set, supervisor mode software mayalso access pages with U=1.
	User = 1 << 4,
	// The G bit designates aglobalmapping.  Global mappings are those that exist in all address spaces.For non-leaf PTEs, the global setting implies that all mappings in the subsequent levels of the pagetable are global.  Note that failing to mark a global mapping as global merely reduces performance,whereas  marking  a  non-global  mapping  as  global  is  a  software  bug  that,  after  switching  to  anaddress space with a different non-global mapping for that address range, can unpredictably resultin either mapping being used.
	Global = 1 << 5,
	// Each leaf PTE contains an accessed (A) and dirty (D) bit.  The A bit indicates the virtual page hasbeen read, written, or fetched from since the last time the A bit was cleared.  The D bit indicatesthe virtual page has been written since the last time the D bit was cleared.
	Accessed = 1 << 6,
	Dirty = 1 << 7,
	
	AddressMask = !((1 << 8) - 1),
	
	CodeSupervisor = (1 << 1 | 1 << 3 | 1) as usize,
	DataSupervisor = (1 << 1 | 1 << 2 | 1) as usize,
}

impl const ::core::ops::BitOr for EntryBits {
	type Output = usize;
	fn bitor(self, a: EntryBits) -> Self::Output { self as usize | a as usize }
}

#[derive(Default, Copy, Clone)]
pub struct Entry {
	pub value: usize,
}

impl Entry {
	pub const fn zeroed() -> Self {
		Entry { value: 0}
	}
}


impl Entry {
	pub unsafe fn get_table(&self) -> *mut Table {
		(self.value & EntryBits::AddressMask as usize) as *mut Table
	}
}

#[repr(C)]
#[repr(align(4096))]
pub struct Table {
	pub entries: [Entry; 512],
}

impl Table {
	pub const fn zeroed() -> Self {
		Table { entries: [Entry { value: 0}; 512] }
	}
}


pub unsafe fn enable(root_table_physical: usize) {
	
}

pub static mut PAGE_TABLE_TABLE: Table = Table::zeroed();
#[link_section = ".data"]
pub static mut ROOT_PAGE: Table = Table::zeroed();


const ENTRY_COUNT: usize = 512;
const PAGE_ALIGN: usize = 4096;
const MEGAPAGE_SIZE: usize = PAGE_ALIGN * ENTRY_COUNT;
const GIGAPAGE_SIZE: usize = PAGE_ALIGN * ENTRY_COUNT * ENTRY_COUNT;
const TERAPAGE_SIZE: usize = PAGE_ALIGN * ENTRY_COUNT * ENTRY_COUNT * ENTRY_COUNT;