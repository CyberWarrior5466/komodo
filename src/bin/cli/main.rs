use capstone::Instructions;
use komodo::Registers;
use std::{
    env,
    ffi::OsString,
    io::{self, BufRead, Write},
};
use tempfile::{self, NamedTempFile};

fn main() {
    let cs = komodo::new_capstone();
    let mut input_file = NamedTempFile::new().unwrap();
    let mut print = |str| {
        print!("{}", str);
        io::stdout().flush().unwrap();
    };
    let input_path = read_input_path(&mut input_file);
    let (data_section, text_section, instrs) = komodo::disassemble(&cs, input_path, print);

    print_disasm(&instrs);

    let mut regs = Registers::new();
    komodo::run_program(
        &cs,
        data_section,
        text_section,
        &mut regs,
        instrs,
        &mut print,
    );

    eprintln!("{:?}", regs);
}

fn print_disasm<'a>(instrs: &Instructions<'a>) {
    for i in instrs.iter() {
        let mut hex = String::new();
        for b in i.bytes().iter().rev() {
            hex.push_str(format!("{:x}", b).as_str());
        }

        let str = format!(
            "{:x}:\t{}\t\t{}\t{}",
            i.address(),
            hex,
            i.mnemonic().unwrap(),
            i.op_str().unwrap()
        );
        eprintln!("{str}");
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
