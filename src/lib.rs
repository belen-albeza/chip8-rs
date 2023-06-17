mod cpu;
mod error;
pub mod vm;

use std::path::PathBuf;

pub fn run(filename: PathBuf) -> vm::Result<()> {
    let mut vm = vm::VM::new();
    vm.load_rom(filename)?;
    vm.run()
}
