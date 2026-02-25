use capstone::{
    RegId,
    arch::arm::ArmReg::{
        ARM_REG_APSR, ARM_REG_R0, ARM_REG_R12, ARM_REG_R13, ARM_REG_R14, ARM_REG_R15,
    },
};
use std::ops::{Index, IndexMut};

#[derive(Default, Debug)]
pub struct Registers {
    pub r0: i32,
    pub r1: i32,
    pub r2: i32,
    pub r3: i32,
    pub r4: i32,
    pub r5: i32,
    pub r6: i32,
    pub r7: i32,
    pub r8: i32,
    pub r9: i32,
    pub r10: i32,
    pub r11: i32,
    pub r12: i32,
    pub r13_sp: i32,
    pub r14_lr: i32,
    pub r15_pc: i32,
    pub apsr: i32,
}

impl Registers {
    pub fn new() -> Registers {
        Registers::default()
    }
}

impl Index<u16> for Registers {
    type Output = i32;

    fn index(&self, index: u16) -> &Self::Output {
        match index {
            0 => &self.r0,
            1 => &self.r1,
            2 => &self.r2,
            3 => &self.r3,
            4 => &self.r4,
            5 => &self.r5,
            6 => &self.r6,
            7 => &self.r7,
            8 => &self.r8,
            9 => &self.r9,
            10 => &self.r10,
            11 => &self.r11,
            12 => &self.r12,
            13 => &self.r13_sp,
            14 => &self.r14_lr,
            15 => &self.r15_pc,
            16 => &self.apsr,
            _ => panic!(
                "index out of bounds: the len is 16 but the index is {}",
                index
            ),
        }
    }
}

impl IndexMut<u16> for Registers {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        match index {
            0 => &mut self.r0,
            1 => &mut self.r1,
            2 => &mut self.r2,
            3 => &mut self.r3,
            4 => &mut self.r4,
            5 => &mut self.r5,
            6 => &mut self.r6,
            7 => &mut self.r7,
            8 => &mut self.r8,
            9 => &mut self.r9,
            10 => &mut self.r10,
            11 => &mut self.r11,
            12 => &mut self.r12,
            13 => &mut self.r13_sp,
            14 => &mut self.r14_lr,
            15 => &mut self.r15_pc,
            16 => &mut self.apsr,
            _ => panic!(
                "index out of bounds: the len is 16 but the index is {}",
                index
            ),
        }
    }
}

impl Index<&RegId> for Registers {
    type Output = i32;

    fn index(&self, reg_id: &RegId) -> &Self::Output {
        let reg = reg_id.0 as u32;

        match reg as u32 {
            reg if ARM_REG_R0 <= reg && reg <= ARM_REG_R12 => &self[reg as u16 - ARM_REG_R0 as u16],
            ARM_REG_R13 => &self.r13_sp,
            ARM_REG_R14 => &self.r14_lr,
            ARM_REG_R15 => &self.r15_pc,
            ARM_REG_APSR => &self.apsr,
            _ => panic!(
                "index out of bounds: the len is 16 but the index is {}",
                reg
            ),
        }
    }
}

impl IndexMut<&RegId> for Registers {
    fn index_mut(&mut self, reg_id: &RegId) -> &mut Self::Output {
        let reg = reg_id.0 as u32;

        match reg as u32 {
            reg if ARM_REG_R0 <= reg && reg <= ARM_REG_R12 => {
                &mut self[reg as u16 - ARM_REG_R0 as u16]
            }
            ARM_REG_R13 => &mut self.r13_sp,
            ARM_REG_R14 => &mut self.r14_lr,
            ARM_REG_R15 => &mut self.r15_pc,
            ARM_REG_APSR => &mut self.apsr,
            _ => panic!(
                "index out of bounds: the len is 16 but the index is {}",
                reg
            ),
        }
    }
}
