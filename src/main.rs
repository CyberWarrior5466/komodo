use tempfile::{self, NamedTempFile};

use komodo::registers::Registers;

fn main() {
    let mut input_file = NamedTempFile::new().unwrap();
    let mut regs = Registers::new();
    komodo::run_program(&mut input_file, &mut regs, false);
    eprintln!("{:?}", regs);
}
