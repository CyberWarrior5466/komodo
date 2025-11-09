use arch::arm::ArmOperandType;
use arch::arm::ArmOperandType::Imm;
use arch::arm::ArmOperandType::Reg;
use capstone::Capstone;
use capstone::Insn;
use capstone::arch::ArchOperand;
use capstone::arch::BuildsCapstone;
use capstone::arch::arm::ArmOperand;
use capstone::arch::arm::ArmReg::ARM_REG_APSR;
use capstone::arch::arm::ArmReg::ARM_REG_SPSR;
use capstone::arch::arm::ArmShift::Asr;
use capstone::arch::arm::ArmShift::Invalid;
use capstone::arch::arm::ArmShift::Lsl;
use capstone::arch::arm::ArmShift::Lsr;
use capstone::arch::arm::ArmShift::Ror;
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
    let valid_mnemonics = [
        "add", "sub", "mul", "and", "bic", "clz", "eor", "or", "mla", "mov", "lsl", "lsr", "asr",
        "ror", "rrx", "mvn", "cmp", "mrs",
    ];
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
            let condition = &mnemonic[m.len()..];
            if !valid_conditions.contains(&condition) {
                panic!("Unkown condition {}", condition);
            }

            return [mnemonic.to_string(), condition.to_string()];
        }
    }

    // panic!("Unrecognised mnemonic {}", mnemonic);
    return [String::from("cmp"), String::new()];
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
        "add" | "sub" | "and" | "bic" | "eor" | "orr" => match op_types.as_slice() {
            [Reg(rd_id), Reg(rn_id), Imm(n)] => regs[rd_id] = binary_op(mneomonic)(regs[rn_id], *n),
            [Reg(rd_id), Reg(rn_id), Reg(rm_id)] => {
                regs[rd_id] = binary_op(mneomonic)(regs[rn_id], regs[rm_id])
            }
            _ => panic!(),
        },

        "mul" => match op_types.as_slice() {
            [Reg(rd_id), Reg(rm_id), Reg(rs_id)] => regs[rd_id] = regs[rm_id] * regs[rs_id],
            _ => panic!(),
        },

        "clz" => match op_types.as_slice() {
            [Reg(rd_id), Reg(rm_id)] => regs[rd_id] = regs[rm_id].leading_zeros() as i32,
            _ => panic!(),
        },

        "mla" => match op_types.as_slice() {
            [Reg(rd_id), Reg(rm_id), Reg(rs_id), Reg(rn_id)] => {
                regs[rd_id] = regs[rm_id] * regs[rs_id] + regs[rn_id]
            }
            _ => panic!(),
        },

        "mov" => match op_types.as_slice() {
            [Reg(rd_id), Imm(n)] => regs[rd_id] = *n,
            [Reg(rd_id), Reg(rn_id)] => regs[rd_id] = regs[rn_id],
            _ => panic!(),
        },

        "lsl" | "lsr" | "asr" | "ror" | "rrx" => match ops.as_slice() {
            [
                ArchOperand::ArmOperand(ArmOperand {
                    vector_index: _,
                    subtracted: _,
                    shift: _,
                    op_type: Reg(rd_id),
                    access: _,
                }),
                ArchOperand::ArmOperand(ArmOperand {
                    vector_index: _,
                    subtracted: _,
                    shift,
                    op_type: Reg(rn_id),
                    access: _,
                }),
            ] => match shift {
                Lsl(n) => regs[rd_id] = regs[rn_id] << n,
                Lsr(n) => regs[rd_id] = regs[rn_id] >> n,
                // TODO: this is probably wrong
                Asr(n) => regs[rd_id] = (regs[rn_id] as u32 >> *n as u32) as i32,
                Ror(n) => regs[rd_id] = regs[rn_id].rotate_right(*n),
                Invalid => {
                    if mneomonic == "rrx" {
                        regs[rd_id] = regs[rn_id].rotate_right(1);
                        // set the 31st bit (MSB) to 0
                        regs[rn_id] &= !(1 << 31);
                    } else {
                        panic!("Unrecognised operands for {} instruction", mneomonic)
                    }
                }
                _ => panic!(
                    "Unrecognised shift {:?} in {} instruction",
                    shift, mneomonic
                ),
            },
            _ => match mneomonic.as_str() {
                "lsl" => match op_types.as_slice() {
                    [Reg(rd_id), Reg(rn_id), Reg(rm_id)] => {
                        regs[rd_id] = regs[rn_id] << regs[rm_id]
                    }
                    _ => panic!("Unrecognised operands for lsl instruction"),
                },
                "lsr" => match op_types.as_slice() {
                    [Reg(rd_id), Reg(rn_id), Reg(rm_id)] => {
                        regs[rd_id] = regs[rn_id] >> regs[rm_id]
                    }
                    _ => panic!("Unrecognised operands for lsr instruction"),
                },
                "asr" => match op_types.as_slice() {
                    // TODO: this is probably wrong
                    [Reg(rd_id), Reg(rn_id), Reg(rm_id)] => {
                        regs[rd_id] = (regs[rn_id] as u32 >> regs[rm_id] as u32) as i32
                    }
                    _ => panic!("Unrecognised operands for asr instruction"),
                },
                "ror" => match op_types.as_slice() {
                    [Reg(rd_id), Reg(rn_id), Reg(rm_id)] => {
                        regs[rd_id] = regs[rn_id].rotate_right(regs[rm_id] as u32);
                    }

                    _ => panic!("Unrecognised operands for ror instruction"),
                },
                _ => panic!("Unrecognised shift instruction"),
            },
        },

        "mvn" => match op_types.as_slice() {
            [Reg(rd_id), Imm(n)] => regs[rd_id] = !*n,
            [Reg(rd_id), Reg(rn_id)] => regs[rd_id] = !regs[rn_id],
            _ => panic!(),
        },

        "cmp" => match op_types.as_slice() {
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
