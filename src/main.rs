use goblin;
use std::env;
use std::ffi::OsString;
use std::io::{self, Write};
use std::io::{BufRead, Read};
use std::process::{self, exit};
use tempfile;

fn main() {
    do_thing();
}

fn do_thing() {
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

    let mut buf: Vec<u8> = Vec::new();
    output_file.read_to_end(&mut buf).unwrap();

    let parsed = goblin::Object::parse(&buf).unwrap();
    if let goblin::Object::Elf(elf) = parsed {
        for header in elf.section_headers.iter() {}
    } else {
        panic!("Unexpected object format");
    }
}
