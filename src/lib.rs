mod audio;
mod cpu;
mod error;
mod instruction;
mod screen;
mod sprites;
pub mod vm;

use std::path::PathBuf;

pub fn run(filename: PathBuf) -> vm::Result<()> {
    let mut rng = rand::thread_rng();
    let mut vm = vm::VM::new(&mut rng);
    vm.load_rom(filename)?;
    vm.run()
}
