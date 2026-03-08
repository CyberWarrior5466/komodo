mod registers;
mod status_flags;

use capstone::{
    Capstone, Insn,
    arch::{
        ArchOperand, BuildsCapstone,
        arm::{
            ArchMode, ArmOperand,
            ArmOperandType::{self, Imm, Reg},
            ArmReg::{ARM_REG_APSR, ARM_REG_SPSR},
            ArmShift,
        },
    },
    prelude::*,
};
use core::ops;
use goblin;
use os_info;
pub use registers::{RegTuple, Registers};
use status_flags::StatusFlags;
use std::{
    env,
    ffi::OsString,
    io::{self, BufRead, Read, Write},
    process::{self},
};
use tempfile::{self, NamedTempFile};

use crate::status_flags::update_from_flags;

#[derive(Default, Debug)]
struct Instr {
    mnemonic: String,
    update_status_flags: Option<bool>,
    condition: Condition,
}

#[derive(Default, Debug)]
enum Condition {
    /// Equal
    Eq,
    /// Not equal
    Ne,
    /// Carry set / Unsigned higher or same
    CsHs,
    /// Carry clear / Unsigned lower
    CcLo,
    /// Minus
    Mi,
    /// Plus
    Pl,
    /// Overflow
    Vs,
    /// No overflow
    Vc,
    /// Unsigned higher
    Hi,
    /// Unsigned lower or same
    Ls,
    /// Signed greater than or equal
    Ge,
    /// Signed less than
    Lt,
    /// Signed greataer than
    Gt,
    /// Signed less than or equal
    Le,
    /// Always
    #[default]
    Al,
}

impl From<&str> for Condition {
    fn from(value: &str) -> Self {
        use Condition::*;
        match value.to_ascii_lowercase().as_str() {
            "eq" => Eq,
            "ne" => Ne,
            "cs" | "hs" => CsHs,
            "cc" | "lo" => CcLo,
            "mi" => Mi,
            "pl" => Pl,
            "vs" => Vs,
            "vc" => Vc,
            "hi" => Hi,
            "ls" => Ls,
            "ge" => Ge,
            "lt" => Lt,
            "gt" => Gt,
            "le" => Le,
            "" | "al" => Al,
            _ => panic!("Unrecognised condition {}", value),
        }
    }
}

pub fn run_program(input_file: &mut NamedTempFile, regs: &mut registers::Registers, mock: bool) {
    let input_path = if mock {
        input_file.path().as_os_str().to_owned()
    } else {
        read_input_path_from_user(input_file)
    };

    let mut output_file = tempfile::NamedTempFile::new().unwrap();
    let output_path = output_file.path().as_os_str().to_os_string();

    run_gnu_gas(input_path, output_path).expect("could not run gnu gas");

    let text_section =
        extract_text_section(&mut output_file).expect("could not find .text section");

    let cs = Capstone::new()
        .arm()
        .mode(ArchMode::Arm)
        .detail(true)
        .build()
        .expect("Failed to run capstone");

    let instrs = cs
        .disasm_all(text_section.as_slice(), 0)
        .expect("Failed to diassemble");

    for i in instrs.as_ref() {
        let instr = extract_instr(i);
        execute(&cs, i, regs, &instr);
    }
}

fn read_input_path_from_user(input_file: &mut NamedTempFile) -> OsString {
    let args: Vec<OsString> = env::args_os().collect();

    let input_path = if args.len() > 1 {
        // get file path from cli args
        args[1].clone()
    } else {
        // get temporary file path with content from stdin
        for line in io::stdin().lock().lines() {
            input_file.write(line.unwrap().as_bytes()).unwrap();
            input_file.write(b"\n").unwrap();
        }
        input_file.path().as_os_str().to_os_string()
    };
    return input_path;
}

fn extract_text_section(output_file: &mut NamedTempFile) -> Option<Vec<u8>> {
    let mut buf: Vec<u8> = Vec::new();
    output_file.read_to_end(&mut buf).unwrap();

    let parsed = goblin::Object::parse(&buf).unwrap();
    if let goblin::Object::Elf(elf) = parsed {
        for header in elf.section_headers.iter() {
            let name = elf.shdr_strtab.get_at(header.sh_name).unwrap();
            if name == ".text" {
                let start = header.sh_offset as usize;
                let end = header.sh_offset as usize + header.sh_size as usize;
                let text_bytes = buf[start..end].to_owned();
                return Some(text_bytes);
            }
        }
    }

    return None;
}

fn run_gnu_gas(input_path: OsString, output_path: OsString) -> Result<(), ()> {
    let command = match os_info::get().os_type() {
        os_info::Type::Ubuntu | os_info::Type::Debian => "arm-linux-gnueabi-as",
        os_info::Type::Fedora => "arm-linux-gnu-as",
        os_info::Type::Linux => panic!("Unsupported linux distro"),
        os => panic!("Unsupported OS {}", os),
    };

    let output = process::Command::new(command)
        .arg("-march=armv4")
        .arg(input_path)
        .arg("-o")
        .arg(output_path.clone())
        .output();

    match output {
        Err(e) => {
            if let io::ErrorKind::NotFound = e.kind() {
                match os_info::get().os_type() {
                    os_info::Type::Ubuntu | os_info::Type::Debian => panic!(
                        "Cannot find `arm-linux-gnueabi-as`, try running:\n\tapt install binutils-arm-linux-gnueabi"
                    ),
                    os_info::Type::Fedora => panic!(
                        "Cannot find `arm-linux-gnu-as`, try running:\n\tdnf install binutils-arm-linux-gnu"
                    ),
                    os_info::Type::Linux => panic!("Unsupported linux distro"),
                    _ => panic!("Unsupported OS"),
                }
            } else {
                panic!("{}", e);
            }
        }
        Ok(output) => {
            if output.status.success() {
                return Ok(());
            }
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            return Err(());
        }
    }
}

fn extract_instr(insn: &Insn) -> Instr {
    // A4.2, p436 from DDI01001 spec

    let instr_s_cond = [
        "add", "sub", "adc", "and", "bic", "eor", "mla", "mov", "mul", "mvn", "orr", "rsb", "rsc",
        "smlal", "smull", "umlal", "umull", "lsl", "lsr", "asr", "ror", "rrx",
    ];

    let instr_cond_instr = [
        ("ldr", "b"),
        ("ldr", "bt"),
        ("ldr", "h"),
        ("ldr", "sb"),
        ("ldr", "sh"),
        ("ldr", "t"),
        ("str", "b"),
        ("str", "bt"),
        ("str", "h"),
        ("str", "t"),
        ("swp", "b"),
    ];

    let instr_cond = [
        "bl", "b", "cmn", "cmp", "ldr", "mrs", "msr", "str", "svc", "swp", "teq", "tst",
    ];

    let target = insn.mnemonic().unwrap();
    {
        let instr = match_instr(&instr_s_cond, &target);

        if !instr.is_empty() {
            let rest = &target[instr.len()..];
            let is_s = rest.starts_with("s");
            let condition_str = if is_s { &rest[1..] } else { rest };
            return Instr {
                mnemonic: instr,
                update_status_flags: Some(is_s),
                condition: Condition::from(condition_str),
            };
        }
    }

    for (instr_start, instr_end) in instr_cond_instr {
        if target.starts_with(&instr_start) && target.ends_with(&instr_end) {
            let instr = String::from(instr_start) + instr_end;
            let rest = &target[instr_start.len()..(target.len() - instr_end.len())];
            return Instr {
                mnemonic: instr,
                update_status_flags: None,
                condition: Condition::from(rest),
            };
        }
    }

    {
        let instr = match_instr(&instr_cond, &target);

        if !instr.is_empty() {
            let rest = &target[instr.len()..];
            return Instr {
                mnemonic: instr,
                update_status_flags: None,
                condition: Condition::from(rest),
            };
        }
    }

    panic!("Unrecognised mnemonic {}", target);
}

fn execute(cs: &Capstone, i: &Insn, regs: &mut Registers, instr: &Instr) {
    let detail: InsnDetail = cs.insn_detail(&i).expect("Failed to get insn detail");
    let arch_detail: ArchDetail = detail.arch_detail();
    let ops = arch_detail.operands();

    let op_types: Vec<ArmOperandType> = ops
        .iter()
        .map(|op| {
            if let ArchOperand::ArmOperand(arm_op) = op {
                return arm_op.op_type.clone();
            } else {
                panic!();
            }
        })
        .collect();

    // A3.2.1, p112 from DDI01001 spec
    let flags = StatusFlags::from(regs.apsr);
    use Condition::*;
    let condition_matches = match instr.condition {
        Eq => flags.zero,
        Ne => !flags.zero,
        CsHs => flags.carry,
        CcLo => !flags.carry,
        Mi => flags.negative,
        Vs => flags.overflow,
        Vc => !flags.overflow,
        Hi => flags.carry && !flags.zero,
        Ls => flags.carry || flags.zero,
        Ge => flags.negative == flags.overflow,
        Lt => flags.negative != flags.overflow,
        Gt => !flags.zero || flags.negative == flags.overflow,
        Le => flags.zero || flags.negative != flags.overflow,
        Al => true,
        _ => panic!(),
    };

    if !condition_matches {
        return;
    }

    match (instr.mnemonic.as_str(), op_types.as_slice()) {
        ("add" | "sub" | "and" | "bic" | "eor" | "orr", [Reg(rd), Reg(rn), _shifter]) => {
            if let ArchOperand::ArmOperand(shifter_operand) = &ops[2] {
                let value = binary_op(&instr.mnemonic)(regs[rn], value_of(&shifter_operand, regs));
                regs[rd] = value;

                if instr.update_status_flags.unwrap() {
                    update_status_flags(&mut regs.apsr, value);
                }
            }
        }

        ("mul", [Reg(rd), Reg(rm), Reg(rn)]) => regs[rd] = regs[rm] * regs[rn],

        ("mla", [Reg(rd), Reg(rm), Reg(rs), Reg(rn)]) => regs[rd] = regs[rm] * regs[rs] + regs[rn],

        ("lsl" | "lsr" | "asr" | "ror" | "rrx" | "mov", [Reg(rd), _shifter]) => {
            if let ArchOperand::ArmOperand(shifter_operand) = &ops[1] {
                let value = value_of(shifter_operand, regs);
                regs[rd] = value;

                if instr.update_status_flags.unwrap() {
                    update_status_flags(&mut regs.apsr, value);
                }
            }
        }
        ("lsl" | "lsr" | "asr" | "ror" | "rrx" | "mov", [Reg(rd), Reg(rm), Reg(rn)]) => {
            let shift = match instr.mnemonic.as_str() {
                "lsl" => ArmShift::LslReg(*rn),
                "lsr" => ArmShift::LsrReg(*rn),
                "asr" => ArmShift::AsrReg(*rn),
                "ror" => ArmShift::RorReg(*rn),
                "rrx" => ArmShift::RrxReg(*rn),
                _ => panic!(),
            };

            let value = apply_shift(&regs, regs[rm], &shift);
            regs[rd] = value;

            if instr.update_status_flags.unwrap() {
                update_status_flags(&mut regs.apsr, value);
            }
        }

        ("mvn", [Reg(rd), _shifter]) => {
            if let ArchOperand::ArmOperand(shifter_operand) = &ops[1] {
                let value = !value_of(shifter_operand, regs);
                regs[rd] = value;
                if instr.update_status_flags.unwrap() {
                    update_status_flags(&mut regs.apsr, value);
                }
            }
        }

        ("cmp", [Reg(rn), _shifter]) => {
            if let ArchOperand::ArmOperand(shifter_operand) = &ops[1] {
                let shifter_operand_value = value_of(shifter_operand, regs);
                let mut flags = StatusFlags::from(regs.apsr);

                let (alu_out, should_overflow) = regs[rn].overflowing_sub(shifter_operand_value);
                flags.negative = (alu_out as i32) < 0;
                flags.zero = alu_out == 0;
                flags.carry = !(regs[rn] as u32)
                    .overflowing_sub(shifter_operand_value as u32)
                    .1;
                flags.overflow = should_overflow;

                regs.apsr = update_from_flags(regs.apsr, &flags);
            }
        }

        ("cmn", [Reg(rn), _shifter]) => {
            if let ArchOperand::ArmOperand(shifter_operand) = &ops[1] {
                let shifter_operand_value = value_of(shifter_operand, regs);
                let mut flags = StatusFlags::new();

                let (alu_out, should_overflow) = regs[rn].overflowing_add(shifter_operand_value);
                flags.negative = alu_out < 0;
                flags.zero = alu_out == 0;
                flags.carry = (regs[rn] as u32)
                    .overflowing_add(shifter_operand_value as u32)
                    .1;
                flags.overflow = should_overflow;

                regs.apsr = update_from_flags(regs.apsr, &flags);
            }
        }

        ("mrs", [Reg(rd), Reg(rn)]) => {
            assert!(rn.0 == ARM_REG_APSR as u16 || rn.0 == ARM_REG_SPSR as u16);
            regs[rd] = regs[rn];
        }

        (
            "adc" | "b" | "bl" | "ldr" | "ldrb" | "ldrbt" | "ldrh" | "ldrsb" | "ldrsh" | "ldrt"
            | "msr" | "rsb" | "rsc" | "smlal" | "smull" | "str" | "strb" | "strbt" | "strh"
            | "strt" | "subs" | "swp" | "swpb" | "teq" | "tst" | "umlal" | "umull",
            _,
        ) => {
            todo!()
        }

        _ => panic!("Unrecognised mnemonic {}", instr.mnemonic),
    }
}

fn update_status_flags(apsr: &mut i32, value: i32) {
    let mut flags = StatusFlags::from(*apsr);
    flags.negative = value < 0;
    flags.zero = value == 0;
    *apsr = update_from_flags(*apsr, &flags);
}

fn match_instr(instrs: &[&'static str], target: &str) -> String {
    for i in instrs.iter() {
        if target.starts_with(i) {
            let string = i.to_string();
            return string;
        }
    }
    String::new()
}

fn binary_op(mneomonic: &String) -> fn(i32, i32) -> i32 {
    match mneomonic.as_str() {
        "add" => ops::Add::add,
        "sub" => ops::Sub::sub,
        "and" => ops::BitAnd::bitand,
        "eor" => ops::BitXor::bitxor,
        "orr" => ops::BitOr::bitor,
        _ => panic!(),
    }
}

fn value_of(operand: &ArmOperand, registers: &registers::Registers) -> i32 {
    match operand.op_type {
        Reg(reg_id) => apply_shift(&registers, registers[&reg_id], &operand.shift),
        Imm(n) => n,
        ArmOperandType::Invalid | _ => panic!(),
    }
}

fn apply_shift(registers: &registers::Registers, num: i32, shift: &ArmShift) -> i32 {
    use ArmShift::*;
    match shift {
        Lsl(s) => ((num as u32) << s) as i32,
        Lsr(s) => ((num as u32) >> s) as i32,
        Asr(s) => num >> s,
        Ror(s) => num.rotate_right(*s),
        LslReg(reg) => ((num as u32) << registers[reg]) as i32,
        LsrReg(reg) => ((num as u32) >> registers[reg]) as i32,
        AsrReg(reg) => num >> registers[reg],
        RorReg(reg) => num.rotate_right(registers[reg] as u32),
        Rrx(_) | RrxReg(_) => num.rotate_right(1),
        Invalid => num,
    }
}

#[cfg(test)]
mod test;
