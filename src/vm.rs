use std::fs;
use std::path::PathBuf;

use crate::cpu::CPU;
use crate::error::Error;

pub type Result<T> = std::result::Result<T, Error>;

pub struct VM {
    cpu: CPU,
}

impl VM {
    pub fn new() -> Self {
        Self { cpu: CPU::new() }
    }

    pub fn load_rom(&mut self, filename: PathBuf) -> Result<()> {
        self.reset();

        let rom = fs::read(filename)?;
        self.cpu.load_rom(&rom)?;

        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            self.cpu.tick()?;
        }
    }

    fn reset(&mut self) {
        self.cpu = CPU::new()
    }
}
