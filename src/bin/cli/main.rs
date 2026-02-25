mod emulator;
mod registers;

use crate::registers::Registers;
use tempfile::{self, NamedTempFile};

fn main() {
    let mut input_file = NamedTempFile::new().unwrap();
    let mut regs = Registers::new();
    emulator::run_program(&mut input_file, &mut regs, false);
    eprintln!("{:?}", regs);
}

#[cfg(test)]
mod test;
