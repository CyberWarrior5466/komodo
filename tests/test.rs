use capstone::RegId;
use capstone::arch::arm::ArmReg::ARM_REG_R0;
use capstone::arch::arm::ArmReg::ARM_REG_R1;
use capstone::arch::arm::ArmReg::ARM_REG_R2;
use capstone::arch::arm::ArmReg::ARM_REG_R3;
use capstone::arch::arm::ArmReg::ARM_REG_R4;
use capstone::arch::arm::ArmReg::ARM_REG_R5;
use capstone::arch::arm::ArmReg::ARM_REG_R6;
use capstone::arch::arm::ArmReg::ARM_REG_R7;
use capstone::arch::arm::ArmReg::ARM_REG_R8;
use capstone::arch::arm::ArmReg::ARM_REG_R9;
use capstone::arch::arm::ArmReg::ARM_REG_R10;
use komodo::registers::Registers;
use std::io::Write;
use tempfile::{self, NamedTempFile};

fn mock_program(buf: &[u8]) -> Registers {
    let mut regs = Registers::new();
    let mut input_file = NamedTempFile::new().unwrap();
    input_file.write(buf).unwrap();
    komodo::run_program(&mut input_file, &mut regs, true);
    return regs;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mov() {
        let regs = mock_program(
            b"
            mov r0, #0
            mov r1, #1
            mov r2, #'a'
            mov r3, #0b1010
            // an immediate is formed by rotating an 8 bit constant in a 32 bit word
            mov r4, #0xff
            mov r5, #0x104
            mov r6, #0xff0
            mov r7, #0xff00
            mov r8, #0xff000
            mov r9, #0xff000000
            mov r10, #0xf000000f
            ",
        );
        assert_eq!(regs[&RegId(ARM_REG_R0 as u16)], 0);
        assert_eq!(regs[&RegId(ARM_REG_R1 as u16)], 1);
        assert_eq!(regs[&RegId(ARM_REG_R2 as u16)], 'a' as i32);
        assert_eq!(regs[&RegId(ARM_REG_R3 as u16)], 0b1010);
        assert_eq!(regs[&RegId(ARM_REG_R4 as u16)], 0xff);
        assert_eq!(regs[&RegId(ARM_REG_R5 as u16)], 0x104);
        assert_eq!(regs[&RegId(ARM_REG_R6 as u16)], 0xff0);
        assert_eq!(regs[&RegId(ARM_REG_R7 as u16)], 0xff00);
        assert_eq!(regs[&RegId(ARM_REG_R8 as u16)], 0xff000);
        assert_eq!(regs[&RegId(ARM_REG_R9 as u16)], 0xff000000_u32 as i32);
        assert_eq!(regs[&RegId(ARM_REG_R10 as u16)], 0xf000000f_u32 as i32);
    }

    #[test]
    #[should_panic]
    fn test_mov_panic_1() {
        mock_program(b"mov r0, #0x101");
    }

    #[test]
    #[should_panic]
    fn test_mov_panic_2() {
        mock_program(b"mov r0, #0x102");
    }

    #[test]
    #[should_panic]
    fn test_mov_panic_3() {
        mock_program(b"mov r0, #0xff1");
    }

    #[test]
    #[should_panic]
    fn test_mov_panic_4() {
        mock_program(b"mov r0, #0xf04");
    }

    #[test]
    #[should_panic]
    fn test_mov_panic_5() {
        mock_program(b"mov r0, #0xff003");
    }

    #[test]
    #[should_panic]
    fn test_mov_panic_6() {
        mock_program(b"mov r0, #0xF000001F");
    }

    #[test]
    fn test_mvn() {
        let regs = mock_program(b"");
    }
}
