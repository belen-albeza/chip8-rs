use crate::error::CPUError;
use crate::instruction::Instruction;

pub type Result<T> = std::result::Result<T, CPUError>;

const MEM_SIZE: usize = 4096;
const MEM_START: usize = 0x200;
const V_REGISTERS_SIZE: usize = 16;
const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;

#[allow(dead_code)]
pub struct CPU {
    memory: [u8; MEM_SIZE],
    pc: u16,
    v_registers: [u8; V_REGISTERS_SIZE],
    i_register: u16,
    v_buffer: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
}

impl CPU {
    pub fn new() -> Self {
        Self {
            memory: [0; MEM_SIZE],
            pc: 0x200,
            v_registers: [0; V_REGISTERS_SIZE],
            i_register: 0,
            v_buffer: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
        }
    }

    pub fn load_rom(&mut self, rom: &[u8]) -> Result<()> {
        if rom.len() > (MEM_SIZE - MEM_START) {
            return Err(CPUError::MemoryOverflow);
        }

        self.memory[MEM_START..(MEM_START + rom.len())].copy_from_slice(rom);
        Ok(())
    }

    pub fn tick(&mut self) -> Result<()> {
        let opcode = (self.read_byte()? as u16) << 8 | self.read_byte()? as u16;
        let instruction = Instruction::try_from(opcode)?;

        match instruction {
            Instruction::Jump(addr) => self.exec_jump(addr)?,
            Instruction::LoadVx(x, value) => self.exec_load_vx(x, value)?,
            Instruction::AddVx(x, value) => self.exec_add_vx(x, value)?,
            Instruction::LoadI(x) => self.exec_load_i(x)?,
        }

        Ok(())
    }

    fn read_byte(&mut self) -> Result<u8> {
        let value = self
            .memory
            .get(self.pc as usize)
            .ok_or(CPUError::InvalidAddress(self.pc))?;
        self.pc += 1;
        Ok(*value)
    }

    fn exec_jump(&mut self, to: u16) -> Result<()> {
        self.pc = to;
        Ok(())
    }

    fn exec_load_vx(&mut self, x: u8, value: u8) -> Result<()> {
        let i = self
            .v_registers
            .get_mut(x as usize)
            .ok_or(CPUError::InvalidVRegister(x))?;
        *i = value;

        Ok(())
    }

    fn exec_add_vx(&mut self, x: u8, value: u8) -> Result<()> {
        let i = self
            .v_registers
            .get_mut(x as usize)
            .ok_or(CPUError::InvalidVRegister(x))?;
        *i += value;

        Ok(())
    }

    fn exec_load_i(&mut self, value: u16) -> Result<()> {
        self.i_register = value;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn any_cpu_with_rom(rom: &[u8]) -> CPU {
        let mut cpu = CPU::new();
        cpu.load_rom(rom).expect("Couldn't load ROM");
        cpu
    }

    #[test]
    fn test_new() {
        let cpu = CPU::new();
        assert_eq!(cpu.memory, [0; 4096]);
        assert_eq!(cpu.pc, 0x200);
        assert_eq!(cpu.v_registers, [0; 16]);
        assert_eq!(cpu.i_register, 0);
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

    #[test]
    fn test_tick_returns_err_on_invalid_opcode() {
        let mut cpu = any_cpu_with_rom(&[0xFF, 0xFF]);
        let res = cpu.tick();

        assert_eq!(res.unwrap_err(), CPUError::InvalidOpcode(0xFFFF));
    }

    #[test]
    fn test_tick_returns_err_if_invalid_pc() {
        let mut cpu = any_cpu_with_rom(&[]);
        cpu.pc = 0x1000;

        let res = cpu.tick();

        assert_eq!(res.unwrap_err(), CPUError::InvalidAddress(0x1000));
    }

    #[test]
    fn test_exec_jump() {
        let mut cpu = any_cpu_with_rom(&[0x13, 0x21]);

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x0321);
    }

    #[test]
    fn test_load_vx() {
        let mut cpu = any_cpu_with_rom(&[0x6A, 0x8F]);

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x0202);
        assert_eq!(cpu.v_registers[0xA], 0x8F);
    }

    #[test]
    fn test_add_vx() {
        let mut cpu = any_cpu_with_rom(&[0x7A, 0x8F]);
        cpu.v_registers[0xA] = 0x1;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x0202);
        assert_eq!(cpu.v_registers[0xA], 0x90);
    }

    #[test]
    fn test_load_i() {
        let mut cpu = any_cpu_with_rom(&[0xA1, 0x23]);

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x0202);
        assert_eq!(cpu.i_register, 0x123);
    }
}
