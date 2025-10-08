use std::{
    fmt::Debug,
    ops::{Index, IndexMut},
};

use capstone::{RegId, arch::arm::ArmReg::ARM_REG_R0};

pub struct Registers {
    regs: [i32; 16],
}

impl Registers {
    pub fn new() -> Registers {
        Registers { regs: [0; 16] }
    }
}

impl Index<&RegId> for Registers {
    type Output = i32;

    fn index(&self, rd_id: &RegId) -> &Self::Output {
        &self.regs[rd_id.0 as usize - ARM_REG_R0 as usize]
    }
}

impl IndexMut<&RegId> for Registers {
    fn index_mut(&mut self, rd_id: &RegId) -> &mut Self::Output {
        &mut self.regs[rd_id.0 as usize - ARM_REG_R0 as usize]
    }
}

impl Debug for Registers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{R0: {}, R1: {}, R2: {}, R3: {}, R4: {}, R5: {}, R6: {}, R7: {}, R8: {}, R9: {}, R10: {}, R11: {}, R12: {}, R13: {}, R14: {}, R15: {}}}",
            self.regs[0],
            self.regs[1],
            self.regs[2],
            self.regs[3],
            self.regs[4],
            self.regs[5],
            self.regs[6],
            self.regs[7],
            self.regs[8],
            self.regs[9],
            self.regs[10],
            self.regs[11],
            self.regs[12],
            self.regs[13],
            self.regs[14],
            self.regs[15],
        )
    }
}
