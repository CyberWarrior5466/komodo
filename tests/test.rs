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
    use capstone::arch::arm::ArmReg::ARM_REG_APSR;

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
            mov r10, #0xf000000f",
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
    fn test_mov_shift() {
        let regs = mock_program(
            b"
            mov r0, #4
            mov r1, #1

            mov r2, r0, LSL #2
            mov r3, r0, LSL r1

            mov r4, r0, LSR #2
            mov r5, r0, LSR r1

            mov r6, r0, ASR #2
            mov r7, r0, ASR r1

            mov r8, r1, ROR #2 // 0b01000000...
            mov r9, r1, ROR r1 // 0b10000000...

            mov r10, r1, RRX // 0b1000000...
        ",
        );

        assert_eq!(regs[&RegId(ARM_REG_R0 as u16)], 4);
        assert_eq!(regs[&RegId(ARM_REG_R1 as u16)], 1);

        assert_eq!(regs[&RegId(ARM_REG_R2 as u16)], 16);
        assert_eq!(regs[&RegId(ARM_REG_R3 as u16)], 8);

        assert_eq!(regs[&RegId(ARM_REG_R4 as u16)], 1);
        assert_eq!(regs[&RegId(ARM_REG_R5 as u16)], 2);

        assert_eq!(regs[&RegId(ARM_REG_R6 as u16)], 1);
        assert_eq!(regs[&RegId(ARM_REG_R7 as u16)], 2);

        assert_eq!(regs[&RegId(ARM_REG_R8 as u16)], 1073741824);
        assert_eq!(regs[&RegId(ARM_REG_R9 as u16)], -2147483648);

        assert_eq!(regs[&RegId(ARM_REG_R10 as u16)], -2147483648);

        // TODO: assert_eq r8,r9,r10
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
        let regs = mock_program(
            b"
            mvn r0, #0
            mvn r1, #0xf",
        );
        assert_eq!(regs[&RegId(ARM_REG_R0 as u16)], -1);
        assert_eq!(regs[&RegId(ARM_REG_R1 as u16)], -16);
    }

    #[test]
    fn test_add() {
        let regs = mock_program(
            b"
            mov r0, #1
            add r1, r0, #2
            add r2, r0, r1

            mvn r3, #0 // -1
            add r4, r3, r3",
        );
        assert_eq!(regs[&RegId(ARM_REG_R1 as u16)], 3);
        assert_eq!(regs[&RegId(ARM_REG_R2 as u16)], 4);

        assert_eq!(regs[&RegId(ARM_REG_R4 as u16)], -2);
    }

    #[test]
    fn test_sub() {
        let regs = mock_program(
            b"
            mov r0, #3
            sub r1, r0, #1
            sub r2, r0, r1

            mov r3, #0
            sub r4, r3, #1
            // 0 - 1 = -1
            ",
        );

        assert_eq!(regs[ARM_REG_R1 as u16], 2);
        assert_eq!(regs[ARM_REG_R2 as u16], 1);

        assert_eq!(regs[ARM_REG_R4 as u16], -1);
    }

    #[test]
    fn test_cmp_1() {
        let regs = mock_program(
            b"
            mov r0, #0
            cmp r0, #0",
        );
        assert_eq!(regs[ARM_REG_APSR as u16], 0x60000010);
    }

    #[test]
    fn test_cmp_2() {
        let regs = mock_program(
            b"
            mov r0, #0
            cmp r0, #1",
        );
        assert_eq!(regs[ARM_REG_APSR as u16], 0x80000010u32 as i32);
    }

    #[test]
    fn test_cmp_3() {
        let regs = mock_program(
            b"
                mov r0, #1
                cmp r0, #0x80000000",
        );
        assert_eq!(regs[ARM_REG_APSR as u16], 0x90000010u32 as i32);
    }

    #[test]
    fn test_cmp_4() {
        let regs = mock_program(
            b"
                mov r0, #0x80000000
                cmp r0, #1",
        );
        assert_eq!(regs[ARM_REG_APSR as u16], 0x30000010);
    }

    #[test]
    fn test_cmp_5() {
        let regs = mock_program(
            b"
                mov r0, #1
                mov r1, #-2
                cmp r0, r1",
        );
        assert_eq!(regs[ARM_REG_APSR as u16], 0x10);
    }

    #[test]
    fn test_cmp_6() {
        let regs = mock_program(
            b"
                mov r0, #2
                cmp r0, #1",
        );
        assert_eq!(regs[ARM_REG_APSR as u16], 0x20000010);
    }

    #[test]
    fn test_cmn_1() {
        let regs = mock_program(
            b"
                mov r0, #0
                cmn r0, #0",
        );
        assert_eq!(regs[ARM_REG_APSR as u16], 0x40000010);
    }

    #[test]
    fn test_cmn_2() {
        let regs = mock_program(
            b"
                mov r0, #0
                cmn r0, #1",
        );
        assert_eq!(regs[ARM_REG_APSR as u16], 0x10);
    }

    #[test]
    fn test_cmn_3() {
        let regs = mock_program(
            b"
                mov r0, #0
                mov r1, #-1
                cmn r0, r1",
        );
        assert_eq!(regs[ARM_REG_APSR as u16], 0x80000010u32 as i32);
    }

    #[test]
    fn test_cmn_4() {
        let regs = mock_program(
            b"
                mov r0, #0x7fffffff
                cmn r0, #1",
        );
        assert_eq!(regs[ARM_REG_APSR as u16], 0x90000010u32 as i32);
    }

    #[test]
    fn test_cmn_5() {
        let regs = mock_program(
            b"
                mov r0, #0x80000000
                mov r1, #-1
                cmn r0, r1",
        );
        assert_eq!(regs[ARM_REG_APSR as u16], 0x30000010u32 as i32);
    }
}
