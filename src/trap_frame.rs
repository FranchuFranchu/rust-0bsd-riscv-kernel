use crate::{
    cpu,
    paging::{sv39::RootTable, Table},
};

/// A pointer to this struct is placed in sscratch
#[derive(Default, Debug, Clone)] // No copy because they really shouldn't be copied and used without changing the PID
#[repr(C)]
pub struct TrapFrame {
    pub general_registers: [usize; 32],
    pub pc: usize,              // 32
    pub hartid: usize,          // 33
    pub pid: usize,             // 34
    pub interrupt_stack: usize, // 35. This may be shared between different processes executing the same hart
    pub flags: usize,           // 36
    pub satp: usize,            // 37
    pub kernel_satp: usize,     // 38
}

impl TrapFrame {
    pub const fn zeroed() -> Self {
        Self {
            general_registers: [0; 32],
            hartid: 0,
            pid: 0,
            pc: 0,
            interrupt_stack: 0,
            flags: 0,
            satp: 0,
            kernel_satp: 0,
        }
    }
    pub const fn zeroed_interrupt_context() -> Self {
        Self {
            general_registers: [0; 32],
            hartid: 0,
            pid: 0,
            pc: 0,
            interrupt_stack: 0,
            flags: 1,
            satp: 0,
            kernel_satp: 0,
        }
    }
    pub fn use_current_satp_as_kernel_satp(&mut self) {
        self.kernel_satp = cpu::read_satp();
    }
    // Inherit hartid, interrupt_stack, and flags from the other trap frame
    pub fn inherit_from(&mut self, other: &TrapFrame) -> &mut TrapFrame {
        self.hartid = other.hartid;
        self.interrupt_stack = other.interrupt_stack;
        self.flags = other.flags;
        self.satp = other.satp;
        self
    }
    pub fn print(&self) {
        println!("{:?}", "trap");
        for (idx, i) in self.general_registers[1..].iter().enumerate() {
            print!("0x{:0<8x} ", i);
            if idx % 4 == 0 {
                println!();
            }
        }
    }
    pub fn is_interrupt_context(&self) -> bool {
        self.flags & 1 != 0
    }
    pub fn has_trapped_before(&self) -> bool {
        self.flags & 2 != 0
    }
    pub fn is_double_faulting(&self) -> bool {
        self.flags & 4 != 0
    }
    pub fn is_in_fault_trap(&self) -> bool {
        self.flags & 8 != 0
    }
    pub fn set_trapped_before(&mut self) {
        self.flags |= 2
    }
    pub fn set_double_faulting(&mut self) {
        self.flags |= 4
    }
    pub fn set_in_fault_trap(&mut self) {
        self.flags |= 8
    }
    pub fn clear_in_fault_trap(&mut self) {
        self.flags &= !8
    }
    /// You need to be the only owner of the trap frame to make it the current one
    pub unsafe fn make_current(&mut self) {
        self.flags = (*cpu::read_sscratch()).flags;
        self.flags |= 8;
        (*cpu::read_sscratch()).flags &= !8;
        cpu::write_sscratch(self as *const TrapFrame as usize)
    }

    pub unsafe fn satp_as_sv39_root_table(&mut self) -> RootTable {
        use crate::paging::sv39::RootTable;
        RootTable(((self.satp << 12) as *mut Table).as_mut().unwrap())
    }

    #[cfg(feature = "backtrace")]
    pub fn set_gimli_register(&mut self, register: &gimli::Register, value: usize) {
        match register.0 {
            0..32 => self.general_registers[register.0 as usize] = value,
            _ => unimplemented!(),
        }
    }

    #[cfg(feature = "backtrace")]
    pub fn get_gimli_register(&self, register: &gimli::Register) -> usize {
        match register.0 {
            0..32 => self.general_registers[register.0 as usize],
            _ => unimplemented!(),
        }
    }

    /// The rule must not be SameValue or Undefined
    #[cfg(feature = "backtrace")]
    pub unsafe fn apply_gimli_rule<R: gimli::Reader>(
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
            Expression(expression) => todo!(),
            ValExpression(val_expression) => todo!(),
            Architectural => todo!(),
        }
    }
}
