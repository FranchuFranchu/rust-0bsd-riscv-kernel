use core::arch::asm;

use super::*;

/// SAFETY: It's safe if root is a valid pointer
/// and paging is disabled
/// Otherwise, it can remap things the wrong way and break everything
pub unsafe fn identity_map(root: *mut Table) {
    for (idx, i) in ((*root).entries).iter_mut().enumerate() {
        i.value = EntryBits::VALID | EntryBits::RWX | (GIGAPAGE_SIZE / 4 * idx);
    }
}

#[derive(Debug)]
pub struct RootTable<'a>(pub &'a mut Table);


impl<'a> Paging for RootTable<'a> {
    fn map(&mut self, physical_addr: usize, virtual_addr: usize, length: usize, flags: usize) {
        let vpn2_min = (virtual_addr >> 30) & (ENTRY_COUNT - 1);
        let vpn1_min = (virtual_addr >> 21) & (ENTRY_COUNT - 1);
        let vpn0_min = (virtual_addr >> 12) & (ENTRY_COUNT - 1);

        //println!("Len {:?}", length);
        let mut vpn2_max = ((virtual_addr + length - 1) >> 30) & (ENTRY_COUNT - 1);
        let mut vpn1_max = ((virtual_addr + length - 1) >> 21) & (ENTRY_COUNT - 1);
        let mut vpn0_max = ((virtual_addr + length) >> 12) & (ENTRY_COUNT - 1);
        //println!("vp2 {:?} {:?}", vpn2_min, vpn2_max);
        //println!("vp1 {:?} {:?}", vpn1_min, vpn1_max);
        //println!("vp0 {:?} {:?}", vpn0_min, vpn0_max);
        //println!("{:?}", flags);
        if vpn0_min > vpn0_max {
            vpn0_max = ENTRY_COUNT
        };
        if vpn1_min > vpn1_max {
            vpn1_max = ENTRY_COUNT - 1
        };
        if vpn2_min > vpn2_max {
            vpn2_max = ENTRY_COUNT - 1
        };

        let offset: usize = physical_addr.wrapping_sub(virtual_addr) >> 2;

        for vpn2 in vpn2_min..vpn2_max + 1 {
            let mut entry = &mut self.0.entries[vpn2];
            //println!("vp2 {} {:p}", vpn2, &entry);

            if (vpn2 == vpn2_max || vpn2 == vpn2_min) && entry.is_leaf() {
                unsafe { entry.split(MEGAPAGE_SIZE) };
                //info!("{}", "Split")
            };
            //println!("vp2 {} {:x} {:?}", vpn2, vpn2 << 30, entry);
            //println!("{:?}", entry.value);
            if let Some(table) = unsafe { entry.try_as_table_mut() } {
                //println!("{:?}", "Table");
                //println!("T {:p}", table);
                for vpn1 in vpn1_min..vpn1_max + 1 {
                    let mut entry = &mut table[vpn1];
                    if (vpn1 == vpn1_max || vpn1 == vpn1_min) && entry.is_leaf() {
                        //info!("{}", "Split2");
                        unsafe { entry.split(PAGE_SIZE) };
                    };
                    //println!(" vp1 {} {:x} {:?}", vpn1, vpn2 << 30 | vpn1 << 21, entry);
                    if let Some(table) = unsafe { entry.try_as_table_mut() } {
                        //println!("T {:p}", table);
                        for vpn0 in vpn0_min..vpn0_max {
                            let mut entry = &mut table[vpn0];
                            let virt = vpn2 << 30 | vpn1 << 21 | vpn0 << 12;
                            // TODO: Write down somewhere that qemu doesn't care when using info mem
                            // if the high bits are set. Ask qemu to change this (later)
                            entry.value = (virt >> 2 | flags).wrapping_add(offset) & (2usize.pow(54) - 1);
                            //println!("  vp0 {} {:x} {:?}", vpn0, virt, entry);
                            //println!("newval {:x}", entry.value);
                        }
                    } else {
                        //println!("oldval {:?}", entry);
                        //println!("virt {:x}", (vpn2 << 30 | vpn1 << 21));
                        entry.value = (vpn2 << 28 | vpn1 << 19 | flags).wrapping_add(offset) & (2usize.pow(54) - 1);
                        //println!("newval {:?}", entry);
                    }
                }
            } else {
                //println!("oldval {:x}", entry.value);
                //println!("virt {:x}", (vpn2 << 30));
                entry.value = (vpn2 << 28 | flags).wrapping_add(offset) & (2usize.pow(54) - 1);
                //println!("newval {:x}", entry.value);
            }
        }

        //println!("{:?}", "finish");

        unsafe { asm!("sfence.vma") };
        unsafe { asm!("fence rw, rw") };

        //info!("entry");
    }
    unsafe fn query(&self, virtual_addr: usize) -> Result<(Entry, usize), PageLookupError> {
        let vpn2 = (virtual_addr >> 30) & (ENTRY_COUNT - 1);
        let vpn1 = (virtual_addr >> 21) & (ENTRY_COUNT - 1);
        let vpn0 = (virtual_addr >> 12) & (ENTRY_COUNT - 1);
        let offset = virtual_addr & (ENTRY_COUNT - 1);
        let table = &self.0;
        if let Some(table) = table[vpn2].try_as_table() {
            if let Some(table) = table[vpn1].try_as_table() {
                assert!(table[vpn0].is_leaf());
                if table[vpn0].value & EntryBits::VALID == 0 {
                    Err(PageLookupError::Invalid)
                } else {
                    Ok((table[vpn0], offset))
                }
            } else if table[vpn1].value & EntryBits::VALID == 0 {
                Err(PageLookupError::Invalid)
            } else {
                Ok((table[vpn1], offset + vpn0 * ENTRY_COUNT))
            }
        } else if table[vpn2].value & EntryBits::VALID == 0 {
            Err(PageLookupError::Invalid)
        } else {
            Ok((table[vpn2], offset + vpn0 * ENTRY_COUNT + vpn1 * ENTRY_COUNT * ENTRY_COUNT))
        }
    }
    fn identity_map(&mut self) {
        for (idx, i) in (self.0.entries).iter_mut().enumerate() {
            i.value = EntryBits::VALID | EntryBits::RWX | (GIGAPAGE_SIZE / 4 * idx);
        }
    }
}

impl<'a> PagingDebug for RootTable<'a> {}
