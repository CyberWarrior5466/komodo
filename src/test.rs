use crate::{Registers, new_capstone};
use std::io::Write;
use tempfile::{self, NamedTempFile};

fn mock_print_program(buf: &'static str, print: &mut impl FnMut(String)) -> Registers {
    let cs = new_capstone();
    let mut input_file: NamedTempFile = NamedTempFile::new().unwrap();
    write!(input_file, "{}", buf).unwrap();
    let input_path = input_file.path().as_os_str().to_owned();
    let print_dism = |str| eprint!("{str}");
    let (data_section, text_section, instrs) = crate::disassemble(&cs, input_path, print_dism);

    let mut regs = Registers::new();
    crate::run_program(&cs, data_section, text_section, &mut regs, instrs, print);

    return regs;
}

fn mock_program(buf: &'static str) -> Registers {
    mock_print_program(buf, &mut |_| {})
}

#[test]
fn test_mov() {
    let regs = mock_program(
        "
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
    assert_eq!(regs.r0, 0);
    assert_eq!(regs.r1, 1);
    assert_eq!(regs.r2, 'a' as i32);
    assert_eq!(regs.r3, 0b1010);
    assert_eq!(regs.r4, 0xff);
    assert_eq!(regs.r5, 0x104);
    assert_eq!(regs.r6, 0xff0);
    assert_eq!(regs.r7, 0xff00);
    assert_eq!(regs.r8, 0xff000);
    assert_eq!(regs.r9, 0xff000000_u32 as i32);
    assert_eq!(regs.r10, 0xf000000f_u32 as i32);
}

#[test]
fn test_mov_shift() {
    let regs = mock_program(
        "
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

    assert_eq!(regs.r0, 4);
    assert_eq!(regs.r1, 1);

    assert_eq!(regs.r2, 16);
    assert_eq!(regs.r3, 8);

    assert_eq!(regs.r4, 1);
    assert_eq!(regs.r5, 2);

    assert_eq!(regs.r6, 1);
    assert_eq!(regs.r7, 2);

    assert_eq!(regs.r8, 1073741824);
    assert_eq!(regs.r9, -2147483648);

    // assert_eq!(regs[&RegId(ARM_REG_R10 as u16)], -2147483648);
    /*
    incorrect,
    need to use carry flag as a 33rd bit
    */
}

#[test]
#[should_panic]
fn test_mov_panic_1() {
    mock_program("mov r0, #0x101");
}

#[test]
#[should_panic]
fn test_mov_panic_2() {
    mock_program("mov r0, #0x102");
}

#[test]
#[should_panic]
fn test_mov_panic_3() {
    mock_program("mov r0, #0xff1");
}

#[test]
#[should_panic]
fn test_mov_panic_4() {
    mock_program("mov r0, #0xf04");
}

#[test]
#[should_panic]
fn test_mov_panic_5() {
    mock_program("mov r0, #0xff003");
}

#[test]
#[should_panic]
fn test_mov_panic_6() {
    mock_program("mov r0, #0xF000001F");
}

#[test]
fn test_mvn() {
    let regs = mock_program(
        "
        mvn r0, #0
        mvn r1, #0xf
        ",
    );
    assert_eq!(regs.r0, -1);
    assert_eq!(regs.r1, -16);
}

#[test]
fn test_add() {
    let regs = mock_program(
        "
        mov r0, #1
        add r1, r0, #2
        add r2, r0, r1

        mvn r3, #0 // -1
        add r4, r3, r3
        ",
    );
    assert_eq!(regs.r1, 3);
    assert_eq!(regs.r2, 4);

    assert_eq!(regs.r4, -2);
}

#[test]
fn test_sub() {
    let regs = mock_program(
        "
        mov r0, #3
        sub r1, r0, #1
        sub r2, r0, r1

        mov r3, #0
        sub r4, r3, #1
        // 0 - 1 = -1
        ",
    );

    assert_eq!(regs.r1, 2);
    assert_eq!(regs.r2, 1);
    assert_eq!(regs.r4, -1);
}

#[test]
fn test_cmp_1() {
    let regs = mock_program(
        "
        mov r0, #0
        cmp r0, #0
        ",
    );
    assert_eq!(regs.apsr, 0x60000010);
}

#[test]
fn test_cmp_2() {
    let regs = mock_program(
        "
        mov r0, #0
        cmp r0, #1
        ",
    );
    assert_eq!(regs.apsr, 0x80000010u32 as i32);
}

#[test]
fn test_cmp_3() {
    let regs = mock_program(
        "
        mov r0, #1
        cmp r0, #0x80000000
        ",
    );
    assert_eq!(regs.apsr, 0x90000010u32 as i32);
}

#[test]
fn test_cmp_4() {
    let regs = mock_program(
        "
        mov r0, #0x80000000
        cmp r0, #1
        ",
    );
    assert_eq!(regs.apsr, 0x30000010);
}

#[test]
fn test_cmp_5() {
    let regs = mock_program(
        "
        mov r0, #1
        mov r1, #-2
        cmp r0, r1
        ",
    );
    assert_eq!(regs.apsr, 0x10);
}

#[test]
fn test_cmp_6() {
    let regs = mock_program(
        "
        mov r0, #2
        cmp r0, #1
        ",
    );
    assert_eq!(regs.apsr, 0x20000010);
}

#[test]
fn test_cmn_1() {
    let regs = mock_program(
        "
        mov r0, #0
        cmn r0, #0
        ",
    );
    assert_eq!(regs.apsr, 0x40000010);
}

#[test]
fn test_cmn_2() {
    let regs = mock_program(
        "
        mov r0, #0
        cmn r0, #1
        ",
    );
    assert_eq!(regs.apsr, 0x10);
}

#[test]
fn test_cmn_3() {
    let regs = mock_program(
        "
        mov r0, #0
        mov r1, #-1
        cmn r0, r1
    ",
    );
    assert_eq!(regs.apsr, 0x80000010u32 as i32);
}

#[test]
fn test_cmn_4() {
    let regs = mock_program(
        "
        mov r0, #0x7fffffff
        cmn r0, #1
        ",
    );
    assert_eq!(regs.apsr, 0x90000010u32 as i32);
}

#[test]
fn test_cmn_5() {
    let regs = mock_program(
        "
        mov r0, #0x80000000
        mov r1, #-1
        cmn r0, r1
        ",
    );
    assert_eq!(regs.apsr, 0x30000010u32 as i32);
}

#[test]
fn test_mrs() {
    let regs = mock_program(
        "
        movs r0, #0
        mrs r0, cpsr
        movs r1, #-1
        mrs r1, cpsr
        ",
    );
    assert_eq!(regs.r0, 0x40000010);
    assert_eq!(regs.r1, 0xc0000010u32 as i32);
}

#[test]
fn test_hello() {
    let mut out: Vec<String> = Vec::new();

    mock_print_program(
        "
        .section .data
        label:
            .asciz \"hello\n\"

        .section .text
        _start:
            ldr r0, =label
            swi 3 // print r0
            swi 2 // exit
        ",
        &mut |str| out.push(str),
    );

    assert_eq!(out.len(), 1);
    assert_eq!(out[0], "hello\n");
}
