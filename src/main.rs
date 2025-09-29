use capstone::Capstone;
use capstone::arch::BuildsCapstone;
use capstone::prelude::*;
use goblin;
use std::env;
use std::ffi::OsString;
use std::io::{self, Write};
use std::io::{BufRead, Read};
use std::process::{self, exit};
use tempfile::{self, NamedTempFile};

fn main() {
    let args: Vec<OsString> = env::args_os().collect();

    let mut input_file = tempfile::NamedTempFile::new().unwrap();
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
    // for byte in text_section.iter() {
    //     print!("{:02x} ", byte);
    // }
    // println!();

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
        println!("{} {}", i.mnemonic().unwrap(), i.op_str().unwrap());

        let detail: InsnDetail = cs.insn_detail(&i).expect("Failed to get insn detail");
        let arch_detail: ArchDetail = detail.arch_detail();
        let ops = arch_detail.operands();
        for op in ops {
            println!("{:8}{:?}", "", op);
        }
    }
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
