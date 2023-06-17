use crate::error::CPUError;

pub type Result<T> = std::result::Result<T, CPUError>;

const MEM_SIZE: usize = 4096;
const MEM_START: usize = 0x200;

pub struct CPU {
    memory: [u8; MEM_SIZE],
    #[allow(dead_code)]
    pc: u16,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            memory: [0; MEM_SIZE],
            pc: 0x200,
        }
    }

    pub fn load_rom(&mut self, rom: &[u8]) -> Result<()> {
        if rom.len() > (MEM_SIZE - MEM_START) {
            return Err(CPUError::MemoryOverflow);
        }

        self.memory[MEM_START..(MEM_START + rom.len())].copy_from_slice(rom);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let cpu = CPU::new();
        assert_eq!(cpu.memory, [0; 4096]);
        assert_eq!(cpu.pc, 0x200);
    }

    #[test]
    fn test_load_rom_ok() {
        let mut cpu = CPU::new();
        let rom: [u8; 2] = [0x00, 0xE0];

        let res = cpu.load_rom(&rom);

        assert!(res.is_ok());
        assert_eq!(cpu.memory[0x200], 0x00);
        assert_eq!(cpu.memory[0x201], 0xE0);
        assert_eq!(cpu.pc, 0x200);
    }

    #[test]
    fn test_load_rom_returns_error_on_memory_overflow() {
        let mut cpu = CPU::new();
        let rom: [u8; 4096 - 199] = [0; 4096 - 199];

        let res = cpu.load_rom(&rom);

        assert_eq!(res.unwrap_err(), CPUError::MemoryOverflow);
    }
}
