use std::fmt;

use crate::error::CPUError;
use crate::instruction::Instruction;
use crate::sprites;

pub type Result<T> = std::result::Result<T, CPUError>;

const MEM_SIZE: usize = 4096;
const MEM_START: usize = 0x200;
const V_REGISTERS_SIZE: usize = 16;
const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const STACK_SIZE: usize = 16;

#[allow(dead_code)]
pub struct CPU {
    memory: [u8; MEM_SIZE],
    pc: u16,
    sp: usize,
    v_registers: [u8; V_REGISTERS_SIZE],
    i_register: u16,
    v_buffer: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    stack: [u16; STACK_SIZE],
}

impl CPU {
    pub fn new() -> Self {
        Self {
            memory: [0; MEM_SIZE],
            pc: 0x200,
            sp: 0,
            v_registers: [0; V_REGISTERS_SIZE],
            i_register: 0,
            v_buffer: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            stack: [0; STACK_SIZE],
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
            Instruction::ClearScreen => self.exec_clear_screen()?,
            Instruction::Jump(addr) => self.exec_jump(addr)?,
            Instruction::Call(addr) => self.exec_call(addr)?,
            Instruction::LoadVx(x, value) => self.exec_load_vx(x, value)?,
            Instruction::AddVx(x, value) => self.exec_add_vx(x, value)?,
            Instruction::Set(x, y) => self.exec_set(x, y)?,
            Instruction::Or(x, y) => self.exec_or(x, y)?,
            Instruction::And(x, y) => self.exec_and(x, y)?,
            Instruction::Xor(x, y) => self.exec_xor(x, y)?,
            Instruction::Add(x, y) => self.exec_add(x, y)?,
            Instruction::Sub(x, y) => self.exec_sub(x, y)?,
            Instruction::ShiftRightVx(x) => self.exec_shiftr_vx(x)?,
            Instruction::SubN(x, y) => self.exec_subn(x, y)?,
            Instruction::ShiftLeftVx(x) => self.exec_shiftl_vx(x)?,
            Instruction::LoadI(x) => self.exec_load_i(x)?,
            Instruction::DrawSprite(x, y, n) => self.exec_draw_sprite(x, y, n)?,
        }

        Ok(())
    }

    pub fn visual_buffer(&self) -> &[bool; SCREEN_WIDTH * SCREEN_HEIGHT] {
        &self.v_buffer
    }

    fn read_byte(&mut self) -> Result<u8> {
        let value = self
            .memory
            .get(self.pc as usize)
            .ok_or(CPUError::InvalidAddress(self.pc))?;
        self.pc += 1;
        Ok(*value)
    }

    fn read_register(&self, x: u8) -> Result<u8> {
        self.v_registers
            .get(x as usize)
            .ok_or(CPUError::InvalidVRegister(x))
            .copied()
    }

    fn set_register(&mut self, x: u8, value: u8) -> Result<()> {
        let i = self
            .v_registers
            .get_mut(x as usize)
            .ok_or(CPUError::InvalidVRegister(x))?;
        *i = value;
        Ok(())
    }

    fn push_stack(&mut self, value: u16) -> Result<()> {
        let i = self.stack.get_mut(self.sp).ok_or(CPUError::StackOverflow)?;
        *i = value;

        self.sp += 1;

        Ok(())
    }

    fn exec_clear_screen(&mut self) -> Result<()> {
        self.v_buffer.fill(false);
        Ok(())
    }

    fn exec_jump(&mut self, to: u16) -> Result<()> {
        self.pc = to;
        Ok(())
    }

    fn exec_call(&mut self, to: u16) -> Result<()> {
        self.push_stack(self.pc)?;
        self.pc = to;
        Ok(())
    }

    fn exec_load_vx(&mut self, x: u8, value: u8) -> Result<()> {
        self.set_register(x, value)
    }

    fn exec_add_vx(&mut self, x: u8, value: u8) -> Result<()> {
        let i = self
            .v_registers
            .get_mut(x as usize)
            .ok_or(CPUError::InvalidVRegister(x))?;
        *i += value;

        Ok(())
    }

    fn exec_set(&mut self, x: u8, y: u8) -> Result<()> {
        let value = self.read_register(y)?;
        self.set_register(x, value)?;
        Ok(())
    }

    fn exec_or(&mut self, x: u8, y: u8) -> Result<()> {
        let value = self.read_register(x)? | self.read_register(y)?;
        self.set_register(x, value)?;
        Ok(())
    }

    fn exec_and(&mut self, x: u8, y: u8) -> Result<()> {
        let value = self.read_register(x)? & self.read_register(y)?;
        self.set_register(x, value)?;
        Ok(())
    }

    fn exec_xor(&mut self, x: u8, y: u8) -> Result<()> {
        let value = self.read_register(x)? ^ self.read_register(y)?;
        self.set_register(x, value)?;
        Ok(())
    }

    fn exec_add(&mut self, x: u8, y: u8) -> Result<()> {
        let (value, carry) = self
            .read_register(x)?
            .overflowing_add(self.read_register(y)?);
        self.set_register(x, value)?;
        self.set_register(0xF, carry as u8)?;
        Ok(())
    }

    fn exec_sub(&mut self, x: u8, y: u8) -> Result<()> {
        let (value, carry) = self
            .read_register(x)?
            .overflowing_sub(self.read_register(y)?);
        self.set_register(x, value)?;
        self.set_register(0xF, !carry as u8)?;
        Ok(())
    }

    fn exec_shiftr_vx(&mut self, x: u8) -> Result<()> {
        let value = self.read_register(x)?;
        let shifted_out = value & 0b_0000_0001;
        self.set_register(x, value >> 1)?;
        self.set_register(0xF, shifted_out)?;
        Ok(())
    }

    fn exec_subn(&mut self, x: u8, y: u8) -> Result<()> {
        let (value, carry) = self
            .read_register(y)?
            .overflowing_sub(self.read_register(x)?);
        self.set_register(x, value)?;
        self.set_register(0xF, !carry as u8)?;
        Ok(())
    }

    fn exec_shiftl_vx(&mut self, x: u8) -> Result<()> {
        let value = self.read_register(x)?;
        let shifted_out = (value & 0b_1000_0000) >> 7;
        self.set_register(x, value << 1)?;
        self.set_register(0xF, shifted_out)?;
        Ok(())
    }

    fn exec_load_i(&mut self, value: u16) -> Result<()> {
        self.i_register = value;
        Ok(())
    }

    fn exec_draw_sprite(&mut self, vx: u8, vy: u8, n: u8) -> Result<()> {
        let sprite = sprites::read_sprite(self.i_register as usize, n as usize, &self.memory)?;

        let x = self.read_register(vx)?;
        let y = self.read_register(vy)?;

        let did_collide = sprites::draw(
            sprite,
            x as usize,
            y as usize,
            (SCREEN_WIDTH, SCREEN_HEIGHT),
            &mut self.v_buffer,
        );

        self.v_registers[0xF] = did_collide as u8;

        Ok(())
    }
}

impl fmt::Display for CPU {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = "".to_string();
        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                let pixel = if self.v_buffer[y * SCREEN_WIDTH + x] {
                    "*"
                } else {
                    " "
                };
                output += pixel;
            }
            output += "\n";
        }

        write!(f, "{}", output)
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
        assert_eq!(cpu.sp, 0);
        assert_eq!(cpu.stack, [0; 16]);
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
    fn test_clear_screen() {
        let mut cpu = any_cpu_with_rom(&[0x00, 0xe0]);
        cpu.v_buffer = [true; SCREEN_WIDTH * SCREEN_HEIGHT];

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x0202);
        assert_eq!(cpu.v_buffer, [false; SCREEN_WIDTH * SCREEN_HEIGHT]);
    }

    #[test]
    fn test_jump() {
        let mut cpu = any_cpu_with_rom(&[0x13, 0x21]);

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x0321);
    }

    #[test]
    fn test_call() {
        let mut cpu = any_cpu_with_rom(&[0x23, 0x21]);

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x0321);
        assert_eq!(cpu.stack[0], 0x202);
        assert_eq!(cpu.sp, 1);
    }

    #[test]
    fn test_call_stack_overflow() {
        let mut cpu = any_cpu_with_rom(&[0x23, 0x21]);
        cpu.sp = 16;

        let res = cpu.tick();

        assert_eq!(res.unwrap_err(), CPUError::StackOverflow);
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
    fn test_set() {
        let mut cpu = any_cpu_with_rom(&[0x80, 0x10]);
        cpu.v_registers[0x1] = 0xA;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.v_registers[0x0], 0xA);
        assert_eq!(cpu.v_registers[0x1], 0xA);
    }

    #[test]
    fn test_or() {
        let mut cpu = any_cpu_with_rom(&[0x80, 0x11]);
        cpu.v_registers[0x0] = 0b_0001_1111;
        cpu.v_registers[0x1] = 0b_0110_1111;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.v_registers[0x0], 0b_0111_1111);
        assert_eq!(cpu.v_registers[0x1], 0b_0110_1111);
    }

    #[test]
    fn test_and() {
        let mut cpu = any_cpu_with_rom(&[0x80, 0x12]);
        cpu.v_registers[0x0] = 0b_0001_1111;
        cpu.v_registers[0x1] = 0b_0110_1111;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.v_registers[0x0], 0b_0000_1111);
        assert_eq!(cpu.v_registers[0x1], 0b_0110_1111);
    }

    #[test]
    fn test_xor() {
        let mut cpu = any_cpu_with_rom(&[0x80, 0x13]);
        cpu.v_registers[0x0] = 0b_0001_1111;
        cpu.v_registers[0x1] = 0b_0110_1111;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.v_registers[0x0], 0b_0111_0000);
        assert_eq!(cpu.v_registers[0x1], 0b_0110_1111);
    }

    #[test]
    fn test_add() {
        let mut cpu = any_cpu_with_rom(&[0x80, 0x14]);
        cpu.v_registers[0x0] = 0x0F;
        cpu.v_registers[0x1] = 0x11;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.v_registers[0x0], 0x20);
        assert_eq!(cpu.v_registers[0x1], 0x11);
        assert_eq!(cpu.v_registers[0xF], 0x0);
    }

    #[test]
    fn test_add_overflow() {
        let mut cpu = any_cpu_with_rom(&[0x80, 0x14]);
        cpu.v_registers[0x0] = 0xFD;
        cpu.v_registers[0x1] = 0x04;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.v_registers[0x0], 0x01);
        assert_eq!(cpu.v_registers[0x1], 0x04);
        assert_eq!(cpu.v_registers[0xF], 0x01);
    }

    #[test]
    fn test_sub() {
        let mut cpu = any_cpu_with_rom(&[0x80, 0x15]);
        cpu.v_registers[0x0] = 0xF0;
        cpu.v_registers[0x1] = 0x11;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.v_registers[0x0], 0xDF);
        assert_eq!(cpu.v_registers[0x1], 0x11);
        assert_eq!(cpu.v_registers[0xF], 0x1);
    }

    #[test]
    fn test_sub_overflow() {
        let mut cpu = any_cpu_with_rom(&[0x80, 0x15]);
        cpu.v_registers[0x0] = 0xF0;
        cpu.v_registers[0x1] = 0xF1;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.v_registers[0x0], 0xFF);
        assert_eq!(cpu.v_registers[0x1], 0xF1);
        assert_eq!(cpu.v_registers[0xF], 0x0);
    }

    #[test]
    fn test_shift_right_vx() {
        let mut cpu = any_cpu_with_rom(&[0x80, 0x16]);
        cpu.v_registers[0x0] = 0b_0100_1110;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.v_registers[0x0], 0b_0010_0_111);
        assert_eq!(cpu.v_registers[0xF], 0x0);
    }

    #[test]
    fn test_shift_right_vx_with_shifted_out_bit() {
        let mut cpu = any_cpu_with_rom(&[0x80, 0x16]);
        cpu.v_registers[0x0] = 0b_0100_1111;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.v_registers[0x0], 0b_0010_0_111);
        assert_eq!(cpu.v_registers[0xF], 0x01);
    }

    #[test]
    fn test_subn() {
        let mut cpu = any_cpu_with_rom(&[0x80, 0x17]);
        cpu.v_registers[0x0] = 0x11;
        cpu.v_registers[0x1] = 0xF0;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.v_registers[0x0], 0xDF);
        assert_eq!(cpu.v_registers[0x1], 0xF0);
        assert_eq!(cpu.v_registers[0xF], 0x1);
    }

    #[test]
    fn test_subn_overflow() {
        let mut cpu = any_cpu_with_rom(&[0x80, 0x17]);
        cpu.v_registers[0x0] = 0xF1;
        cpu.v_registers[0x1] = 0xF0;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.v_registers[0x0], 0xFF);
        assert_eq!(cpu.v_registers[0x1], 0xF0);
        assert_eq!(cpu.v_registers[0xF], 0x0);
    }

    #[test]
    fn test_shift_left_vx() {
        let mut cpu = any_cpu_with_rom(&[0x80, 0x1E]);
        cpu.v_registers[0x0] = 0b_0100_1110;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.v_registers[0x0], 0b_1001_1100);
        assert_eq!(cpu.v_registers[0xF], 0x0);
    }

    #[test]
    fn test_shift_left_vx_with_shifted_out_bit() {
        let mut cpu = any_cpu_with_rom(&[0x80, 0x1E]);
        cpu.v_registers[0x0] = 0b_1100_1111;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.v_registers[0x0], 0b_1001_1110);
        assert_eq!(cpu.v_registers[0xF], 0x01);
    }

    #[test]
    fn test_load_i() {
        let mut cpu = any_cpu_with_rom(&[0xA1, 0x23]);

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x0202);
        assert_eq!(cpu.i_register, 0x123);
    }

    #[test]
    fn test_draw_sprite_simple() {
        let mut cpu = any_cpu_with_rom(&[0xD0, 0x13]);
        cpu.i_register = 0x300;
        cpu.v_registers[0] = 0x1;
        cpu.v_registers[1] = 0x2;
        cpu.memory[0x300] = 0xFF;
        cpu.memory[0x301] = 0x00;
        cpu.memory[0x302] = 0xFF;

        let res = cpu.tick();

        let i = (2 * SCREEN_WIDTH) + 1;
        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x0202);
        assert_eq!(cpu.v_buffer[i..(i + 8)], [true; 8]);
        assert_eq!(cpu.v_buffer[i + 64..(i + 64 + 8)], [false; 8]);
        assert_eq!(cpu.v_buffer[i + 128..(i + 128 + 8)], [true; 8]);
        assert_eq!(cpu.v_registers[0xF], 0);
    }

    #[test]
    fn test_draw_sprite_wraps() {
        let mut cpu = any_cpu_with_rom(&[0xD0, 0x13]);
        cpu.i_register = 0x300;
        cpu.v_registers[0] = 60;
        cpu.v_registers[1] = 30;
        cpu.memory[0x300] = 0xFF;
        cpu.memory[0x301] = 0x00;
        cpu.memory[0x302] = 0xFF;

        let res = cpu.tick();

        let mut i = (30 * SCREEN_WIDTH) + 60;
        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x0202);
        assert_eq!(cpu.v_buffer[i..(i + 4)], [true; 4]);
        assert_eq!(cpu.v_buffer[i - 60..(i - 60 + 4)], [true; 4]);
        i = (30 * SCREEN_WIDTH) + 64;
        assert_eq!(cpu.v_buffer[i..(i + 4)], [false; 4]);
        assert_eq!(cpu.v_buffer[i - 60..(i - 60 + 4)], [false; 4]);
        i = 60;
        assert_eq!(cpu.v_buffer[i..(i + 4)], [true; 4]);
        assert_eq!(cpu.v_buffer[i - 60..(i - 60 + 4)], [true; 4]);
        assert_eq!(cpu.v_registers[0xF], 0);
    }

    #[test]
    fn test_draw_sprite_with_collision() {
        let mut cpu = any_cpu_with_rom(&[0xD0, 0x11]);
        cpu.i_register = 0x300;
        cpu.v_registers[0] = 0;
        cpu.v_registers[1] = 0;
        cpu.memory[0x300] = 0xFF;
        cpu.v_buffer[0..8].copy_from_slice(&[false, false, false, false, true, true, true, true]);

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(
            cpu.v_buffer[0..8],
            [true, true, true, true, false, false, false, false]
        );
        assert_eq!(cpu.v_registers[0xF], 1);
    }
}
