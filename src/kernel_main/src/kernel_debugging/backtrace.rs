//! WIP. Will include code to backtrace

use alloc::collections::BinaryHeap;

use gimli::{
    BaseAddresses, CallFrameInstruction, CfaRule, EhFrame, FrameDescriptionEntry, LittleEndian,
    NativeEndian, UnwindContext, UnwindSection,
};

use crate::{
    cpu::Registers,
    trap_frame::{TrapFrame, TrapFrameExt},
};

extern "C" {
    fn store_to_trap_frame(frame: *const TrapFrame);
    static __eh_frame_end: u32;
    static __eh_frame_start: u32;
    static __text_start: u32;
    static __text_end: u32;
}

#[derive(Eq, PartialEq)]
enum FdeSortedByAddress<'a> {
    Entry(FrameDescriptionEntry<gimli::EndianSlice<'a, NativeEndian>>),
    Dummy(u64),
}

impl<'a> PartialOrd for FdeSortedByAddress<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for FdeSortedByAddress<'a> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.addr().cmp(&other.addr())
    }
}

impl<'a> FdeSortedByAddress<'a> {
    fn unwrap(&self) -> &FrameDescriptionEntry<gimli::EndianSlice<'a, NativeEndian>> {
        match self {
            FdeSortedByAddress::Entry(e) => e,
            FdeSortedByAddress::Dummy(_) => unreachable!(),
        }
    }
    fn addr(&self) -> u64 {
        match self {
            FdeSortedByAddress::Entry(e) => e.initial_address(),
            FdeSortedByAddress::Dummy(e) => *e,
        }
    }
}

impl<'a> From<FrameDescriptionEntry<gimli::EndianSlice<'a, NativeEndian>>>
    for FdeSortedByAddress<'a>
{
    fn from(e: FrameDescriptionEntry<gimli::EndianSlice<'a, NativeEndian>>) -> Self {
        Self::Entry(e)
    }
}
impl<'a> From<u64> for FdeSortedByAddress<'a> {
    fn from(e: u64) -> Self {
        Self::Dummy(e)
    }
}

pub fn row_finder(_address: usize) {}

pub fn backtrace() {
    let mut fde_list: BinaryHeap<FdeSortedByAddress> = BinaryHeap::new();

    let mut frame = crate::trap::TrapFrame::zeroed();
    unsafe { store_to_trap_frame(&frame as *const TrapFrame) };
    let eh_start = unsafe { &__eh_frame_start as *const _ as usize };
    let eh_end = unsafe { &__eh_frame_end as *const _ as usize };
    let eh_size = eh_end - eh_start;

    let eh_slice = unsafe { core::slice::from_raw_parts(eh_start as *const u8, eh_size) };
    use gimli::read::EndianSlice;

    let endian_slice = EndianSlice::new(eh_slice, NativeEndian);
    let eh_section = EhFrame::new(eh_slice, NativeEndian);
    let ba = BaseAddresses::default();
    let ba = ba.set_text(unsafe { &__text_start as *const _ as usize } as u64);
    let ba = ba.set_eh_frame(eh_start as u64);
    let mut iter = eh_section.entries(&ba);
    let mut g_cie = None;

    while let Some(entry) = iter.next().unwrap() {
        match entry {
            gimli::CieOrFde::Cie(cie) => {
                //println!("{:?}", cie);
                let mut iter = cie.instructions(&eh_section, &ba);
                while let Some(instruction) = iter.next().unwrap() {
                    let instruction: CallFrameInstruction<EndianSlice<LittleEndian>> = instruction;
                    println!("{:?}", instruction);
                }
                if g_cie.is_some() {
                    panic!("{:?}", ">1 CIE!");
                };
                g_cie = Some(cie);
            }
            gimli::CieOrFde::Fde(fde) => {
                let fde = fde
                    .parse(|_section, _ba, _offset| {
                        return Ok(g_cie.as_ref().unwrap().clone());
                    })
                    .unwrap();

                /*while let Some(row) = table.next_row().unwrap() {
                    println!("{:x} {:x}", frame.general_registers[Registers::Sp as usize], row.start_address());
                }*/
                fde_list.push(fde.clone().into());
            }
        }
    }

    // This mess seems necessary to specify the relation between the closure's lifetimes and endian_slice's lifetime.
    use gimli::UnwindTableRow;
    fn constrain<'c, F>(_a: &EndianSlice<'c, NativeEndian>, f: F) -> F
    where
        F: for<'a, 'b> Fn(
            u64,
            &'a mut UnwindContext<EndianSlice<'c, NativeEndian>>,
        ) -> core::result::Result<
            &'a UnwindTableRow<EndianSlice<'c, gimli::LittleEndian>>,
            gimli::Error,
        >,
    {
        f
    }

    let fde_list = fde_list.into_sorted_vec();
    let find_row_for_address = constrain(
        &endian_slice,
        |addr: u64, ctx: &mut UnwindContext<EndianSlice<NativeEndian>>| {
            let index = fde_list
                .binary_search(&addr.into())
                .unwrap_or_else(|a| a - 1);
            let fde = fde_list[index].unwrap();
            assert!(fde.contains(addr));
            return fde.unwind_info_for_address(&eh_section, &ba, ctx, addr);
        },
    );

    //let mut next_addr = backtrace as *const fn() as usize as u64 + 0x10;
    let mut ctx = alloc::boxed::Box::new(UnwindContext::new());
    loop {
        let row = find_row_for_address(
            frame.general_registers[Registers::Ra as usize] as u64,
            &mut ctx,
        )
        .unwrap();
        for &(register, ref rule) in row.registers() {
            println!("{:?} {:?} {:?}", row.cfa(), register.0, rule);

            let new_cfa = if let CfaRule::RegisterAndOffset {
                register: cfa_register,
                offset,
            } = row.cfa()
            {
                frame.general_registers[cfa_register.0 as usize]
                    .wrapping_add(*offset as isize as usize)
            } else {
                unimplemented!();
            };

            println!("{:x}", new_cfa);

            // If the debug data is bad or incorrect (points to invalid memory) then this might cause UB!
            let old_register_value = unsafe { frame.apply_gimli_rule(&(new_cfa as u64), &rule) };

            frame.set_gimli_register(&register, old_register_value);
        }
        println!(
            "{:?} {:x}",
            "Addr",
            frame.general_registers[Registers::Ra as usize]
        );
        if frame.general_registers[Registers::Ra as usize] == 0 {
            break;
        }
        drop(row);
    }

    panic!();

    //let ra = frame.general_registers[Registers::Ra.idx()];
    loop {}
}
