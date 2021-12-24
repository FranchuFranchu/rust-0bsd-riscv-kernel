pub use kernel_trap_frame::TrapFrame;

use crate::{
    cpu,
    paging::{sv39::RootTable, Table},
};

pub trait TrapFrameExt {
    fn use_current_satp_as_kernel_satp(&mut self);
    fn print(&self);
    /// You need to be the only owner of the trap frame to make it the current one
    unsafe fn make_current(&mut self);
    unsafe fn satp_as_sv39_root_table(&mut self) -> RootTable;
    #[cfg(feature = "backtrace")]
    fn set_gimli_register(&mut self, register: &gimli::Register, value: usize);
    #[cfg(feature = "backtrace")]
    fn get_gimli_register(&self, register: &gimli::Register) -> usize;
    /// The rule must not be SameValue or Undefined
    #[cfg(feature = "backtrace")]
    unsafe fn apply_gimli_rule<R: gimli::Reader>(
        &self,
        cfa: &u64,
        rule: &gimli::RegisterRule<R>,
    ) -> usize;
}

impl TrapFrameExt for TrapFrame {
    fn use_current_satp_as_kernel_satp(&mut self) {
        self.kernel_satp = cpu::read_satp();
    }

    fn print(&self) {
        println!("{:?}", "trap");
        for (idx, i) in self.general_registers[1..].iter().enumerate() {
            print!("0x{:0<8x} ", i);
            if idx % 4 == 0 {
                println!();
            }
        }
    }

    /// You need to be the only owner of the trap frame to make it the current one
    unsafe fn make_current(&mut self) {
        self.flags = (*cpu::read_sscratch()).flags;
        self.flags |= 8;
        (*cpu::read_sscratch()).flags &= !8;
        cpu::write_sscratch(self as *const TrapFrame as usize)
    }

    unsafe fn satp_as_sv39_root_table(&mut self) -> RootTable {
        use crate::paging::sv39::RootTable;
        RootTable(((self.satp << 12) as *mut Table).as_mut().unwrap())
    }

    #[cfg(feature = "backtrace")]
    fn set_gimli_register(&mut self, register: &gimli::Register, value: usize) {
        match register.0 {
            0..32 => self.general_registers[register.0 as usize] = value,
            _ => unimplemented!(),
        }
    }

    #[cfg(feature = "backtrace")]
    fn get_gimli_register(&self, register: &gimli::Register) -> usize {
        match register.0 {
            0..32 => self.general_registers[register.0 as usize],
            _ => unimplemented!(),
        }
    }

    /// The rule must not be SameValue or Undefined
    #[cfg(feature = "backtrace")]
    unsafe fn apply_gimli_rule<R: gimli::Reader>(
        &self,
        cfa: &u64,
        rule: &gimli::RegisterRule<R>,
    ) -> usize {
        use gimli::RegisterRule::*;
        match rule {
            Undefined => todo!(),
            SameValue => todo!(),
            Offset(offset) => *((cfa.wrapping_add_signed(*offset)) as *const usize),
            ValOffset(offset) => cfa.wrapping_add_signed(*offset) as usize,
            Register(register) => self.get_gimli_register(register),
            Expression(_expression) => todo!(),
            ValExpression(_val_expression) => todo!(),
            Architectural => todo!(),
        }
    }
}
