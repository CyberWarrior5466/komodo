use std::{
    fmt::Debug,
    ops::{Index, IndexMut},
};

use capstone::{
    RegId,
    arch::arm::ArmReg::{ARM_REG_APSR, ARM_REG_R0, ARM_REG_SPSR},
};

pub struct Registers {
    regs: [i32; 18],
}

impl Registers {
    pub fn new() -> Registers {
        Registers { regs: [0; 18] }
    }
}

impl Index<&RegId> for Registers {
    type Output = i32;

    fn index(&self, rd_id: &RegId) -> &Self::Output {
        // capstone::arch::arm64::Arm64Sysreg

        if rd_id.0 == ARM_REG_APSR as u16 {
            return &self.regs[16];
        } else if rd_id.0 == ARM_REG_SPSR as u16 {
            return &self.regs[17];
        }
        &self.regs[rd_id.0 as usize - ARM_REG_R0 as usize]
    }
}

impl IndexMut<&RegId> for Registers {
    fn index_mut(&mut self, rd_id: &RegId) -> &mut Self::Output {
        if rd_id.0 == ARM_REG_APSR as u16 {
            return &mut self.regs[16];
        } else if rd_id.0 == ARM_REG_SPSR as u16 {
            return &mut self.regs[17];
        }
        &mut self.regs[rd_id.0 as usize - ARM_REG_R0 as usize]
    }
}

impl Index<u16> for Registers {
    type Output = i32;

    fn index(&self, index: u16) -> &Self::Output {
        if index == ARM_REG_APSR as u16 {
            return &self.regs[16];
        } else if index == ARM_REG_SPSR as u16 {
            return &self.regs[17];
        }
        &self.regs[index as usize - ARM_REG_R0 as usize]
    }
}

impl IndexMut<u16> for Registers {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        if index == ARM_REG_APSR as u16 {
            return &mut self.regs[16];
        } else if index == ARM_REG_SPSR as u16 {
            return &mut self.regs[17];
        }
        &mut self.regs[index as usize - ARM_REG_R0 as usize]
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
