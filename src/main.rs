use arch::arm::ArmOperandType;
use capstone::Capstone;
use capstone::Insn;
use capstone::arch::ArchOperand;
use capstone::arch::BuildsCapstone;
use capstone::prelude::*;
use goblin;
use std::env;
use std::ffi::OsString;
use std::io::{self, Write};
use std::io::{BufRead, Read};
use std::process::{self, exit};
use tempfile::{self, NamedTempFile};

mod registers;
use registers::Registers;

fn main() {
    let mut input_file = tempfile::NamedTempFile::new().unwrap();
    let input_path = get_input_path(&mut input_file);

    let mut output_file = tempfile::NamedTempFile::new().unwrap();
    let output_path = output_file.path().as_os_str().to_os_string();

    let output = process::Command::new("arm-linux-gnueabi-as")
        .arg(input_path)
        .arg("-o")
        .arg(output_path.clone())
        .output()
        .unwrap();

    if !output.status.success() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        exit(output.status.code().unwrap_or(1));
    }

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

    let mut regs = Registers::new();

    for i in instrs.as_ref() {
        let [mnemonic, condition] = extract_condition(i);
        execute(&cs, i, mnemonic, condition, &mut regs);
    }

    println!("{:?}", regs);
}

fn get_input_path(input_file: &mut NamedTempFile) -> OsString {
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

fn extract_condition(i: &Insn) -> [String; 2] {
    // A4.2, p436 from DDI01001 spec
    let valid_mnemonics = [
        "add", "sub", "mul", "and", "bic", "clz", "eor", "mla", "mov", "mrs",
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
                panic!("Uknown condition {}", condition);
            }

            return [mnemonic.to_string(), condition.to_string()];
        }
    }

    // panic!("Unrecognised mnemonic {}", mnemonic);
    return [String::new(), String::new()];
}

fn execute(cs: &Capstone, i: &Insn, mneomonic: String, _condition: String, regs: &mut Registers) {
    let detail: InsnDetail = cs.insn_detail(&i).expect("Failed to get insn detail");
    let arch_detail: ArchDetail = detail.arch_detail();
    let ops = arch_detail.operands();

    println!("{:?}", ops);
    println!("{}", i.op_str().unwrap());

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

    if mneomonic == "add" {
        match op_types.as_slice() {
            [
                ArmOperandType::Reg(rd_id),
                ArmOperandType::Reg(rn_id),
                ArmOperandType::Imm(n),
            ] => regs[rd_id] = regs[rn_id] + n,
            [
                ArmOperandType::Reg(rd_id),
                ArmOperandType::Reg(rn_id),
                ArmOperandType::Reg(rm_id),
            ] => regs[rd_id] = regs[rn_id] + regs[rm_id],
            _ => panic!("Unrecognised operands for add instruction"),
        }
    } else if mneomonic == "sub" {
        match op_types.as_slice() {
            [
                ArmOperandType::Reg(rd_id),
                ArmOperandType::Reg(rn_id),
                ArmOperandType::Imm(n),
            ] => regs[rd_id] = regs[rn_id] - n,
            [
                ArmOperandType::Reg(rd_id),
                ArmOperandType::Reg(rn_id),
                ArmOperandType::Reg(rm_id),
            ] => regs[rd_id] = regs[rn_id] - regs[rm_id],
            _ => panic!("Unrecognised operands for sub instruction"),
        }
    } else if mneomonic == "mul" {
        match op_types.as_slice() {
            [
                ArmOperandType::Reg(rd_id),
                ArmOperandType::Reg(rm_id),
                ArmOperandType::Reg(rs_id),
            ] => regs[rd_id] = regs[rm_id] * regs[rs_id],
            _ => panic!("Unrecognised operands for mul instruction"),
        }
    } else if mneomonic == "and" {
        match op_types.as_slice() {
            [
                ArmOperandType::Reg(rd_id),
                ArmOperandType::Reg(rn_id),
                ArmOperandType::Imm(n),
            ] => regs[rd_id] = regs[rn_id] & n,
            [
                ArmOperandType::Reg(rd_id),
                ArmOperandType::Reg(rn_id),
                ArmOperandType::Reg(rm_id),
            ] => regs[rd_id] = regs[rn_id] & regs[rm_id],
            _ => panic!("Unrecognised operands for and instruction"),
        }
    } else if mneomonic == "bic" {
        match op_types.as_slice() {
            [
                ArmOperandType::Reg(rd_id),
                ArmOperandType::Reg(rn_id),
                ArmOperandType::Imm(n),
            ] => regs[rd_id] = regs[rn_id] & n,
            [
                ArmOperandType::Reg(rd_id),
                ArmOperandType::Reg(rn_id),
                ArmOperandType::Reg(rm_id),
            ] => regs[rd_id] = regs[rn_id] & regs[rm_id],
            _ => panic!("Unrecognised operands for bic instruction"),
        }
    } else if mneomonic == "clz" {
        match op_types.as_slice() {
            [ArmOperandType::Reg(rd_id), ArmOperandType::Reg(rm_id)] => {
                regs[rd_id] = regs[rm_id].leading_zeros() as i32
            }
            _ => panic!(),
        }
    } else if mneomonic == "eor" {
        match op_types.as_slice() {
            [
                ArmOperandType::Reg(rd_id),
                ArmOperandType::Reg(rn_id),
                ArmOperandType::Imm(n),
            ] => regs[rd_id] = regs[rn_id] ^ n,
            [
                ArmOperandType::Reg(rd_id),
                ArmOperandType::Reg(rn_id),
                ArmOperandType::Reg(rm_id),
            ] => regs[rd_id] = regs[rn_id] ^ regs[rm_id],
            _ => panic!("Unrecognised operands for bic instruction"),
        }
    } else if mneomonic == "mla" {
        match op_types.as_slice() {
            [
                ArmOperandType::Reg(rd_id),
                ArmOperandType::Reg(rm_id),
                ArmOperandType::Reg(rs_id),
                ArmOperandType::Reg(rn_id),
            ] => regs[rd_id] = regs[rm_id] * regs[rs_id] + regs[rn_id],
            _ => panic!("Unrecognised operands for mla instruction"),
        }
    } else if mneomonic == "mov" {
        match op_types.as_slice() {
            [ArmOperandType::Reg(rd_id), ArmOperandType::Imm(n)] => regs[rd_id] = *n,
            [ArmOperandType::Reg(rd_id), ArmOperandType::Reg(rn_id)] => regs[rd_id] = regs[rn_id],
            _ => panic!("Unrecognised operands for mov instruction"),
        }
    } else {
        panic!("Unrecognised instruction");
    }
}
