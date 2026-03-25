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
        let mut bytes: Vec<u8> = Vec::new();
        for &b in i.bytes().iter().rev() {
            bytes.push(b);
        }
        let encoding = i32::from_be_bytes(bytes.clone().try_into().unwrap());

        let str = format!(
            "{:x}:\t{}\t\t{}\t{}",
            i.address(),
            encoding,
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
            input_file.write_all(line.unwrap().as_bytes()).unwrap();
            input_file.write(b"\n").unwrap();
        }
        input_file.path().as_os_str().to_os_string()
    };
    input_path
}
