use arch::arm::ArmOperandType;
use arch::arm::ArmOperandType::Imm;
use arch::arm::ArmOperandType::Reg;
use capstone::Capstone;
use capstone::Insn;
use capstone::arch::ArchOperand;
use capstone::arch::ArchOperand::ArmOperand;
use capstone::arch::BuildsCapstone;
use capstone::arch::arm;
use capstone::arch::arm::ArmReg::ARM_REG_APSR;
use capstone::arch::arm::ArmReg::ARM_REG_SPSR;
use capstone::arch::arm::ArmShift;
use capstone::prelude::*;
use core::ops;
use goblin;
use os_info;
use std::env;
use std::ffi::OsString;
use std::io::{self, Write};
use std::io::{BufRead, Read};
use std::process::{self};
use tempfile::{self, NamedTempFile};

pub mod registers;

struct StatusFlags {
    negative: bool,
    zero: bool,
    carry: bool,
    overflow: bool,
    processor_mode: ProcessorMode,
}

impl StatusFlags {
    fn new() -> StatusFlags {
        StatusFlags {
            negative: false,
            zero: false,
            carry: false,
            overflow: false,
            processor_mode: ProcessorMode::User,
        }
    }
}

impl From<i32> for StatusFlags {
    fn from(n: i32) -> Self {
        StatusFlags {
            negative: n & (1 << 31) != 0,
            zero: n & (1 << 30) != 0,
            carry: n & (1 << 29) != 0,
            overflow: n & (1 << 28) != 0,
            processor_mode: ProcessorMode::User,
        }
    }
}

impl From<StatusFlags> for i32 {
    fn from(flags: StatusFlags) -> Self {
        let mut ans = 0i32;
        if flags.negative {
            ans |= 1 << 31;
        }
        if flags.zero {
            ans |= 1 << 30;
        }
        if flags.carry {
            ans |= 1 << 29;
        }
        if flags.overflow {
            ans |= 1 << 28;
        }
        ans |= flags.processor_mode as i32;

        return ans;
    }
}

enum ProcessorMode {
    User = 0b10000,
    // Fiq = 0b10001,
    // Irq = 0b10010,
    // Supervisor = 0b10011,
    // Abort = 0b10111,
    // Undefined = 0b11011,
    // System = 0b11111,
}

pub fn run_program(input_file: &mut NamedTempFile, regs: &mut registers::Registers, mock: bool) {
    let input_path = if mock {
        input_file.path().as_os_str().to_owned()
    } else {
        read_input_path(input_file)
    };

    let mut output_file = tempfile::NamedTempFile::new().unwrap();
    let output_path = output_file.path().as_os_str().to_os_string();

    run_gnu_gas(input_path, output_path).expect("could not run gnu gas");

    let text_section =
        extract_text_section(&mut output_file).expect("could not find .text section");

    let cs = Capstone::new()
        .arm()
        .mode(arch::arm::ArchMode::Arm)
        .detail(true)
        .build()
        .expect("Failed to run capstone");

    let instrs = cs
        .disasm_all(text_section.as_slice(), 0)
        .expect("Failed to diassemble");

    for i in instrs.as_ref() {
        let [mnemonic, condition] = extract_condition(i);
        execute(&cs, i, mnemonic, condition, regs);
    }
}

fn read_input_path(input_file: &mut NamedTempFile) -> OsString {
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

fn extract_condition(i: &Insn) -> [String; 2] {
    // A4.2, p436 from DDI01001 spec
    // list of (mneomonic, should_update_cpsr)
    let valid_mnemonics = [
        "add", "sub", "mul", "and", "eor", "orr", "mla", "mov", "lsl", "lsr", "asr", "ror", "rrx",
        "mvn", "cmp", "cmn", "mrs",
    ];

    /*
    conditions that can take in {S}, all except:
    - mrs
    - cmp
    */

    // A3.2.1, p112 from DDI01001 spec
    let valid_conditions = [
        "", "eq", "ne", "cs", "hs", "cc", "lo", "mi", "pl", "vs", "vc", "hi", "ls", "ge", "lt",
        "gt", "le", "al",
    ];

    let mnemonic = i
        .mnemonic()
        .expect("failed to get mnemonic for instruction");

    for m in valid_mnemonics {
        if mnemonic.starts_with(m) {
            let rest = &mnemonic[m.len()..];
            if rest.starts_with("s") {}
            let condition = &mnemonic[m.len()..];
            if !valid_conditions.contains(&condition) {
                panic!("Unkown condition {}", condition);
            }

            return [mnemonic.to_string(), condition.to_string()];
        }
    }

    panic!("Unrecognised mnemonic {}", mnemonic);
}

fn execute(
    cs: &Capstone,
    i: &Insn,
    mneomonic: String,
    _condition: String,
    regs: &mut registers::Registers,
) {
    let detail: InsnDetail = cs.insn_detail(&i).expect("Failed to get insn detail");
    let arch_detail: ArchDetail = detail.arch_detail();
    let ops = arch_detail.operands();

    eprintln!("{} {}", i.mnemonic().unwrap(), i.op_str().unwrap());
    eprintln!("{:?}", ops);

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

    match mneomonic.as_str() {
        "add" | "sub" | "and" | "bic" | "eor" | "orr" => match ops.as_slice() {
            [ArmOperand(op1), ArmOperand(op2), ArmOperand(op3)] => {
                if let Reg(rd) = op1.op_type {
                    if let Reg(rn) = op2.op_type {
                        regs[&rd] = binary_op(mneomonic)(regs[&rn], value_of(op3, regs))
                    }
                }
            }
            _ => panic!(),
        },

        "mul" => match op_types.as_slice() {
            [Reg(rd), Reg(rm), Reg(rn)] => regs[rd] = regs[rm] * regs[rn],
            _ => panic!(),
        },

        "mla" => match op_types.as_slice() {
            [Reg(rd), Reg(rm), Reg(rs), Reg(rn)] => regs[rd] = regs[rm] * regs[rs] + regs[rn],
            _ => panic!(),
        },

        "lsl" | "lsr" | "asr" | "ror" | "rrx" | "mov" => match ops.as_slice() {
            [ArmOperand(op1), ArmOperand(shifter_operand)] => {
                if let Reg(rd) = op1.op_type {
                    regs[&rd] = value_of(shifter_operand, regs);
                }
            }
            [ArmOperand(op1), ArmOperand(op2), ArmOperand(op3)] => {
                if let Reg(rd) = op1.op_type {
                    if let Reg(rm) = op2.op_type {
                        if let Reg(rn) = op3.op_type {
                            let shift = match mneomonic.as_str() {
                                "lsl" => ArmShift::LslReg(rn),
                                "lsr" => ArmShift::LsrReg(rn),
                                "asr" => ArmShift::AsrReg(rn),
                                "ror" => ArmShift::RorReg(rn),
                                "Rrx" => ArmShift::RrxReg(rn),
                                _ => panic!(),
                            };
                            regs[&rd] = apply_shift(&regs, regs[&rm], &shift);
                        }
                    }
                }
            }
            _ => panic!(),
        },

        "mvn" => match ops.as_slice() {
            [ArmOperand(op1), ArmOperand(shifter_operand)] => {
                if let Reg(rd) = op1.op_type {
                    regs[&rd] = !value_of(shifter_operand, regs);
                }
            }
            _ => panic!(),
        },

        "cmp" => match ops.as_slice() {
            [ArmOperand(op1), ArmOperand(shifter_operand)] => {
                if let Reg(rn) = op1.op_type {
                    let shifter_operand_value = value_of(shifter_operand, regs);
                    let mut flags = StatusFlags::new();

                    let (alu_out, should_overflow) =
                        regs[&rn].overflowing_sub(shifter_operand_value);
                    flags.negative = (alu_out as i32) < 0;
                    flags.zero = alu_out == 0;
                    flags.carry = !(regs[&rn] as u32)
                        .overflowing_sub(shifter_operand_value as u32)
                        .1;
                    flags.overflow = should_overflow;

                    regs[ARM_REG_APSR as u16] = i32::from(flags);
                }
            }
            _ => panic!(),
        },

        "cmn" => match ops.as_slice() {
            [ArmOperand(op1), ArmOperand(shifter_operand)] => {
                if let Reg(rn) = op1.op_type {
                    let shifter_operand_value = value_of(shifter_operand, regs);
                    let mut flags = StatusFlags::new();

                    let (alu_out, should_overflow) =
                        regs[&rn].overflowing_add(shifter_operand_value);
                    flags.negative = alu_out < 0;
                    flags.zero = alu_out == 0;
                    flags.carry = (regs[&rn] as u32)
                        .overflowing_add(shifter_operand_value as u32)
                        .1;
                    flags.overflow = should_overflow;

                    regs[ARM_REG_APSR as u16] = i32::from(flags);
                }
            }
            _ => panic!(),
        },

        "mrs" => match op_types.as_slice() {
            [Reg(rd_id), Reg(rn_id)] => {
                if rn_id.0 == ARM_REG_APSR as u16 || rn_id.0 == ARM_REG_SPSR as u16 {
                    regs[rd_id] = regs[rn_id];
                } else {
                    panic!();
                }
            }
            _ => panic!("mrs instr"),
        },

        _ => panic!("Unrecognised mnemonic {}", mneomonic),
    }
}

fn binary_op(mneomonic: String) -> fn(i32, i32) -> i32 {
    return match mneomonic.as_str() {
        "add" => ops::Add::add,
        "sub" => ops::Sub::sub,
        "and" => ops::BitAnd::bitand,
        "eor" => ops::BitXor::bitxor,
        "orr" => ops::BitOr::bitor,
        _ => panic!(),
    };
}

fn value_of(operand: &arm::ArmOperand, registers: &registers::Registers) -> i32 {
    return match operand.op_type {
        Reg(reg_id) => apply_shift(&registers, registers[&reg_id], &operand.shift),
        Imm(n) => n,
        ArmOperandType::Invalid | _ => panic!(),
    };
}

fn apply_shift(registers: &registers::Registers, num: i32, shift: &ArmShift) -> i32 {
    return match shift {
        ArmShift::Lsl(s) => num << s,
        ArmShift::Lsr(s) => num >> s,
        ArmShift::Asr(s) => num >> s,
        ArmShift::Ror(s) => num.rotate_right(*s),
        ArmShift::LslReg(reg_id) => num << registers[reg_id],
        ArmShift::LsrReg(reg_id) => num >> registers[reg_id],
        ArmShift::AsrReg(reg_id) => num >> registers[reg_id],
        ArmShift::RorReg(reg_id) => num.rotate_right(registers[reg_id] as u32),
        ArmShift::Rrx(_) | ArmShift::RrxReg(_) => num.rotate_right(1),
        ArmShift::Invalid => num,
    };
}
