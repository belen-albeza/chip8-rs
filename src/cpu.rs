use rand::{Rng, RngCore};

use crate::error::CPUError;
use crate::instruction::Instruction;
use crate::sprites;

pub type Result<T> = std::result::Result<T, CPUError>;

const MEM_SIZE: usize = 4096;
const MEM_END: usize = 0xFFF;
const MEM_START: usize = 0x200;
const V_REGISTERS_SIZE: usize = 16;
const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const STACK_SIZE: usize = 16;
const KEYMAP_SIZE: usize = 16;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TickStatus {
    pub is_waiting_for_key: bool,
    pub is_buzzing: bool,
}

impl Default for TickStatus {
    fn default() -> Self {
        Self {
            is_waiting_for_key: false,
            is_buzzing: false,
        }
    }
}

#[allow(dead_code)]
pub struct CPU<'a> {
    memory: [u8; MEM_SIZE],
    pc: u16,
    sp: usize,
    v_registers: [u8; V_REGISTERS_SIZE],
    i_register: u16,
    v_buffer: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    stack: [u16; STACK_SIZE],
    rng: &'a mut dyn RngCore,
    keypad: [bool; KEYMAP_SIZE],
    delay_timer: u8,
    sound_timer: u8,
    is_waiting_for_key: (bool, usize),
}

impl<'a> CPU<'a> {
    pub fn new(rng: &'a mut impl RngCore) -> Self {
        Self {
            memory: [0; MEM_SIZE],
            pc: 0x200,
            sp: 0,
            v_registers: [0; V_REGISTERS_SIZE],
            i_register: 0,
            v_buffer: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            stack: [0; STACK_SIZE],
            rng: rng,
            keypad: [false; KEYMAP_SIZE],
            delay_timer: 0,
            sound_timer: 0,
            is_waiting_for_key: (false, 0x0),
        }
    }

    pub fn load_rom(&mut self, rom: &[u8]) -> Result<()> {
        if rom.len() > (MEM_SIZE - MEM_START) {
            return Err(CPUError::MemoryOverflow);
        }

        self.memory[MEM_START..(MEM_START + rom.len())].copy_from_slice(rom);
        Ok(())
    }

    pub fn reset(&mut self) {
        self.memory = [0; MEM_SIZE];
        self.pc = 0x200;
        self.sp = 0;
        self.v_registers = [0; V_REGISTERS_SIZE];
        self.i_register = 0;
        self.v_buffer = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.stack = [0; STACK_SIZE];
        self.keypad = [false; KEYMAP_SIZE];
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.is_waiting_for_key = (false, 0x0);
    }

    pub fn set_key_status(&mut self, i: usize, status: bool) -> Result<()> {
        let key = self
            .keypad
            .get_mut(i as usize)
            .ok_or(CPUError::InvalidKey(i))?;
        *key = status;

        let (is_waiting, vx) = self.is_waiting_for_key;

        if is_waiting && status {
            self.set_register(vx as u8, i as u8)?;
            self.is_waiting_for_key = (false, 0x00);
        }

        Ok(())
    }

    pub fn tick(&mut self) -> Result<TickStatus> {
        // update internal timers
        // TODO: decouple 1 cpu tick = 1 decrement
        self.delay_timer = self.delay_timer.saturating_sub(1);
        self.sound_timer = self.sound_timer.saturating_sub(1);

        // skip execution of instructions if we are waiting for a key press
        let (is_waiting, _) = self.is_waiting_for_key;
        if is_waiting {
            return Ok(TickStatus {
                is_waiting_for_key: true,
                is_buzzing: self.sound_timer > 0,
            });
        }

        let opcode = (self.read_byte()? as u16) << 8 | self.read_byte()? as u16;
        let instruction = Instruction::try_from(opcode)?;

        let mut status = match instruction {
            Instruction::NoOp => Ok(TickStatus::default()),
            Instruction::ClearScreen => self.exec_clear_screen(),
            Instruction::Return => self.exec_return(),
            Instruction::Jump(addr) => self.exec_jump(addr),
            Instruction::Call(addr) => self.exec_call(addr),
            Instruction::SkipVxEqual(x, value) => self.exec_skip_vx_if_equal(x, value),
            Instruction::SkipVxNotEqual(x, value) => self.exec_skip_vx_if_not_equal(x, value),
            Instruction::SkipEqual(x, y) => self.exec_skip_if_equal(x, y),
            Instruction::LoadVx(x, value) => self.exec_load_vx(x, value),
            Instruction::AddVx(x, value) => self.exec_add_vx(x, value),
            Instruction::Set(x, y) => self.exec_set(x, y),
            Instruction::Or(x, y) => self.exec_or(x, y),
            Instruction::And(x, y) => self.exec_and(x, y),
            Instruction::Xor(x, y) => self.exec_xor(x, y),
            Instruction::Add(x, y) => self.exec_add(x, y),
            Instruction::Sub(x, y) => self.exec_sub(x, y),
            Instruction::ShiftRightVx(x) => self.exec_shiftr_vx(x),
            Instruction::SubN(x, y) => self.exec_subn(x, y),
            Instruction::ShiftLeftVx(x) => self.exec_shiftl_vx(x),
            Instruction::SkipNotEqual(x, y) => self.exec_skip_if_not_equal(x, y),
            Instruction::LoadI(x) => self.exec_load_i(x),
            Instruction::JumpOffset(x, addr) => self.exec_jump_offset(x, addr),
            Instruction::Rand(x, value) => self.exec_rand(x, value),
            Instruction::DrawSprite(x, y, n) => self.exec_draw_sprite(x, y, n),
            Instruction::SkipIfKey(vx) => self.exec_skip_if_key(vx),
            Instruction::SkipIfNotKey(vx) => self.exec_skip_if_not_key(vx),
            Instruction::LoadDelay(vx) => self.exec_load_delay(vx),
            Instruction::WaitForKey(vx) => self.exec_wait_for_key(vx),
            Instruction::SetDelay(vx) => self.exec_set_delay(vx),
            Instruction::SetSound(vx) => self.exec_set_sound(vx),
            Instruction::AddToIndex(vx) => self.exec_add_to_index(vx),
            Instruction::LoadBCD(vx) => self.exec_load_bcd(vx),
            Instruction::LoadMem(vx) => self.exec_load_mem(vx),
        }?;

        status.is_buzzing = self.sound_timer > 0;
        Ok(status)
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

    fn set_memory(&mut self, addr: u16, value: u8) -> Result<()> {
        let mem_range = MEM_START..=MEM_END;
        if !mem_range.contains(&(addr as usize)) {
            return Err(CPUError::InvalidAddress(addr));
        }

        self.memory[addr as usize] = value;
        Ok(())
    }

    fn get_memory(&mut self, addr: u16) -> Result<u8> {
        let mem_range = MEM_START..=MEM_END;
        if !mem_range.contains(&(addr as usize)) {
            return Err(CPUError::InvalidAddress(addr));
        }

        Ok(self.memory[addr as usize])
    }

    fn set_i_register(&mut self, value: u16) -> u8 {
        let mut carry = 0u8;
        let mut x = value;

        if x as usize > MEM_END {
            x = x & (MEM_END as u16);
            carry = 0x01;
        }

        self.i_register = x;
        carry
    }

    fn read_key(&self, i: u8) -> Result<bool> {
        self.keypad
            .get(i as usize)
            .ok_or(CPUError::InvalidKey(i as usize))
            .copied()
    }

    fn push_stack(&mut self, value: u16) -> Result<()> {
        let i = self.stack.get_mut(self.sp).ok_or(CPUError::StackOverflow)?;
        *i = value;

        self.sp += 1;

        Ok(())
    }

    fn pop_stack(&mut self) -> Result<u16> {
        let value = self
            .stack
            .get(self.sp - 1)
            .ok_or(CPUError::StackOverflow)
            .copied()?;

        self.sp -= 1;
        Ok(value)
    }

    fn exec_clear_screen(&mut self) -> Result<TickStatus> {
        self.v_buffer.fill(false);
        Ok(TickStatus::default())
    }

    fn exec_return(&mut self) -> Result<TickStatus> {
        let to = self.pop_stack()?;
        self.pc = to;
        Ok(TickStatus::default())
    }

    fn exec_jump(&mut self, to: u16) -> Result<TickStatus> {
        self.pc = to;
        Ok(TickStatus::default())
    }

    fn exec_call(&mut self, to: u16) -> Result<TickStatus> {
        self.push_stack(self.pc)?;
        self.pc = to;
        Ok(TickStatus::default())
    }

    fn exec_skip_vx_if_equal(&mut self, x: u8, value: u8) -> Result<TickStatus> {
        if self.read_register(x)? == value {
            self.pc += 2;
        }
        Ok(TickStatus::default())
    }

    fn exec_skip_vx_if_not_equal(&mut self, x: u8, value: u8) -> Result<TickStatus> {
        if self.read_register(x)? != value {
            self.pc += 2;
        }
        Ok(TickStatus::default())
    }

    fn exec_skip_if_equal(&mut self, x: u8, y: u8) -> Result<TickStatus> {
        if self.read_register(x)? == self.read_register(y)? {
            self.pc += 2;
        }
        Ok(TickStatus::default())
    }

    fn exec_load_vx(&mut self, x: u8, value: u8) -> Result<TickStatus> {
        self.set_register(x, value)?;
        Ok(TickStatus::default())
    }

    fn exec_add_vx(&mut self, x: u8, value: u8) -> Result<TickStatus> {
        let i = self
            .v_registers
            .get_mut(x as usize)
            .ok_or(CPUError::InvalidVRegister(x))?;
        *i += value;

        Ok(TickStatus::default())
    }

    fn exec_set(&mut self, x: u8, y: u8) -> Result<TickStatus> {
        let value = self.read_register(y)?;
        self.set_register(x, value)?;
        Ok(TickStatus::default())
    }

    fn exec_or(&mut self, x: u8, y: u8) -> Result<TickStatus> {
        let value = self.read_register(x)? | self.read_register(y)?;
        self.set_register(x, value)?;
        Ok(TickStatus::default())
    }

    fn exec_and(&mut self, x: u8, y: u8) -> Result<TickStatus> {
        let value = self.read_register(x)? & self.read_register(y)?;
        self.set_register(x, value)?;
        Ok(TickStatus::default())
    }

    fn exec_xor(&mut self, x: u8, y: u8) -> Result<TickStatus> {
        let value = self.read_register(x)? ^ self.read_register(y)?;
        self.set_register(x, value)?;
        Ok(TickStatus::default())
    }

    fn exec_add(&mut self, x: u8, y: u8) -> Result<TickStatus> {
        let (value, carry) = self
            .read_register(x)?
            .overflowing_add(self.read_register(y)?);
        self.set_register(x, value)?;
        self.set_register(0xF, carry as u8)?;
        Ok(TickStatus::default())
    }

    fn exec_sub(&mut self, x: u8, y: u8) -> Result<TickStatus> {
        let (value, carry) = self
            .read_register(x)?
            .overflowing_sub(self.read_register(y)?);
        self.set_register(x, value)?;
        self.set_register(0xF, !carry as u8)?;
        Ok(TickStatus::default())
    }

    fn exec_shiftr_vx(&mut self, x: u8) -> Result<TickStatus> {
        let value = self.read_register(x)?;
        let shifted_out = value & 0b_0000_0001;
        self.set_register(x, value >> 1)?;
        self.set_register(0xF, shifted_out)?;
        Ok(TickStatus::default())
    }

    fn exec_subn(&mut self, x: u8, y: u8) -> Result<TickStatus> {
        let (value, carry) = self
            .read_register(y)?
            .overflowing_sub(self.read_register(x)?);
        self.set_register(x, value)?;
        self.set_register(0xF, !carry as u8)?;
        Ok(TickStatus::default())
    }

    fn exec_shiftl_vx(&mut self, x: u8) -> Result<TickStatus> {
        let value = self.read_register(x)?;
        let shifted_out = (value & 0b_1000_0000) >> 7;
        self.set_register(x, value << 1)?;
        self.set_register(0xF, shifted_out)?;
        Ok(TickStatus::default())
    }

    fn exec_skip_if_not_equal(&mut self, x: u8, y: u8) -> Result<TickStatus> {
        if self.read_register(x)? != self.read_register(y)? {
            self.pc += 2;
        }
        Ok(TickStatus::default())
    }

    fn exec_load_i(&mut self, value: u16) -> Result<TickStatus> {
        self.i_register = value;
        Ok(TickStatus::default())
    }

    fn exec_jump_offset(&mut self, x: u8, addr: u16) -> Result<TickStatus> {
        let offset = self.read_register(x)?;
        self.pc = addr + offset as u16;
        Ok(TickStatus::default())
    }

    fn exec_rand(&mut self, x: u8, value: u8) -> Result<TickStatus> {
        let randomized: u8 = self.rng.gen();
        self.set_register(x, randomized & value)?;

        Ok(TickStatus::default())
    }

    fn exec_draw_sprite(&mut self, vx: u8, vy: u8, n: u8) -> Result<TickStatus> {
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

        Ok(TickStatus::default())
    }

    fn exec_skip_if_key(&mut self, vx: u8) -> Result<TickStatus> {
        let key_idx = self.read_register(vx)?;
        let is_key_pressed = self.read_key(key_idx)?;

        if is_key_pressed {
            self.pc += 2;
        }

        Ok(TickStatus::default())
    }

    fn exec_skip_if_not_key(&mut self, vx: u8) -> Result<TickStatus> {
        let key_idx = self.read_register(vx)?;
        let is_key_pressed = self.read_key(key_idx)?;

        if !is_key_pressed {
            self.pc += 2;
        }

        Ok(TickStatus::default())
    }

    fn exec_load_delay(&mut self, vx: u8) -> Result<TickStatus> {
        self.set_register(vx, self.delay_timer)?;
        Ok(TickStatus::default())
    }

    fn exec_wait_for_key(&mut self, vx: u8) -> Result<TickStatus> {
        let _ = self.read_register(vx)?; // ensure vx is valid
        self.is_waiting_for_key = (true, vx as usize);

        let mut status = TickStatus::default();
        status.is_waiting_for_key = true;

        Ok(status)
    }

    fn exec_set_delay(&mut self, vx: u8) -> Result<TickStatus> {
        self.delay_timer = self.read_register(vx)?;
        Ok(TickStatus::default())
    }

    fn exec_set_sound(&mut self, vx: u8) -> Result<TickStatus> {
        self.sound_timer = self.read_register(vx)?;
        Ok(TickStatus::default())
    }

    fn exec_add_to_index(&mut self, vx: u8) -> Result<TickStatus> {
        let value = self.i_register + self.read_register(vx)? as u16;
        let carry = self.set_i_register(value);
        self.set_register(0xF, carry)?;

        Ok(TickStatus::default())
    }

    fn exec_load_bcd(&mut self, vx: u8) -> Result<TickStatus> {
        let (hundreds, tens, ones) = self.read_register(vx)?.to_bcd();
        self.set_memory(self.i_register, hundreds)?;
        self.set_memory(self.i_register + 1, tens)?;
        self.set_memory(self.i_register + 2, ones)?;
        println!(
            "BCD: {}{}{} -> {:#03X}",
            hundreds, tens, ones, self.i_register
        );

        Ok(TickStatus::default())
    }

    fn exec_load_mem(&mut self, vx: u8) -> Result<TickStatus> {
        for i in 0..=vx {
            let value = self.get_memory(self.i_register + i as u16)?;
            self.set_register(i, value)?;
        }

        Ok(TickStatus::default())
    }
}

trait BCD {
    fn to_bcd(&self) -> (u8, u8, u8);
}

impl BCD for u8 {
    fn to_bcd(&self) -> (u8, u8, u8) {
        let hundreds = self / 100;
        let tens = (self / 10) % 10;
        let ones = self % 10;

        (hundreds, tens, ones)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn any_mocked_rng() -> impl RngCore {
        rand::rngs::mock::StepRng::new(1, 1)
    }

    fn any_cpu_with_rom<'a>(rom: &[u8], rng: &'a mut impl RngCore) -> CPU<'a> {
        let mut cpu = CPU::new(rng);
        cpu.load_rom(rom).expect("Couldn't load ROM");
        cpu
    }

    fn any_cpu_with_noop<'a>(rng: &'a mut impl RngCore) -> CPU<'a> {
        any_cpu_with_rom(&[0x01, 0x23], rng)
    }

    #[test]
    fn test_new() {
        let mut rng = any_mocked_rng();
        let cpu = CPU::new(&mut rng);
        assert_eq!(cpu.memory, [0; 4096]);
        assert_eq!(cpu.pc, 0x200);
        assert_eq!(cpu.v_registers, [0; 16]);
        assert_eq!(cpu.i_register, 0);
        assert_eq!(cpu.sp, 0);
        assert_eq!(cpu.stack, [0; 16]);
        assert_eq!(cpu.keypad, [false; 16]);
        assert_eq!(cpu.delay_timer, 0);
    }

    #[test]
    fn test_load_rom_ok() {
        let mut rng = any_mocked_rng();
        let mut cpu = CPU::new(&mut rng);
        let rom: [u8; 2] = [0x00, 0xE0];

        let res = cpu.load_rom(&rom);

        assert!(res.is_ok());
        assert_eq!(cpu.memory[0x200], 0x00);
        assert_eq!(cpu.memory[0x201], 0xE0);
        assert_eq!(cpu.pc, 0x200);
    }

    #[test]
    fn test_load_rom_returns_error_on_memory_overflow() {
        let mut rng = any_mocked_rng();
        let mut cpu = CPU::new(&mut rng);
        let rom: [u8; 4096 - 199] = [0; 4096 - 199];

        let res = cpu.load_rom(&rom);

        assert_eq!(res.unwrap_err(), CPUError::MemoryOverflow);
    }

    #[test]
    fn test_set_key_status() {
        let mut rng = any_mocked_rng();
        let mut cpu = CPU::new(&mut rng);

        let res_down = cpu.set_key_status(0xF, true);

        assert!(res_down.is_ok());
        assert_eq!(cpu.keypad[0xF], true);

        let res_up = cpu.set_key_status(0xF, false);
        assert!(res_up.is_ok());
        assert_eq!(cpu.keypad[0xF], false);
    }

    #[test]
    fn test_set_key_status_returns_err() {
        let mut rng = any_mocked_rng();
        let mut cpu = CPU::new(&mut rng);

        let res = cpu.set_key_status(0x10, true);
        assert_eq!(res.unwrap_err(), CPUError::InvalidKey(0x10));
    }

    #[test]
    fn test_tick_returns_err_on_invalid_opcode() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0xFF, 0xFF], &mut rng);
        let res = cpu.tick();

        assert_eq!(res.unwrap_err(), CPUError::InvalidOpcode(0xFFFF));
    }

    #[test]
    fn test_tick_returns_err_if_invalid_pc() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[], &mut rng);
        cpu.pc = 0x1000;

        let res = cpu.tick();

        assert_eq!(res.unwrap_err(), CPUError::InvalidAddress(0x1000));
    }

    #[test]
    fn test_tick_does_not_advance_if_waiting_for_key() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[], &mut rng);
        cpu.is_waiting_for_key = (true, 0xF);

        let res = cpu.tick();

        assert_eq!(
            res.unwrap(),
            TickStatus {
                is_waiting_for_key: true,
                is_buzzing: false,
            }
        )
    }

    #[test]
    fn test_tick_updates_timers() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[], &mut rng);
        cpu.is_waiting_for_key = (true, 0x0);
        cpu.delay_timer = 1;
        cpu.sound_timer = 1;

        let res_1st_tick = cpu.tick();
        assert!(res_1st_tick.is_ok());
        assert_eq!(cpu.delay_timer, 0);
        assert_eq!(cpu.sound_timer, 0);

        let res_2nd_tick = cpu.tick();
        assert!(res_2nd_tick.is_ok());
        assert_eq!(cpu.delay_timer, 0); // no overflow
        assert_eq!(cpu.sound_timer, 0); // no overflow
    }

    #[test]
    fn test_tick_returns_not_buzzing_when_sound_timer_is_zero() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_noop(&mut rng); // no op
        cpu.sound_timer = 0;

        let res = cpu.tick();

        assert_eq!(res.unwrap().is_buzzing, false);
    }

    #[test]
    fn test_tick_returns_is_buzzing_when_sound_timer_is_greater_than_zero() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_noop(&mut rng); // no op
        cpu.sound_timer = 0x20;

        let res = cpu.tick();

        assert_eq!(res.unwrap().is_buzzing, true);
    }

    #[test]
    fn test_tick_returns_is_buzzing_despite_waiting_for_key() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[], &mut rng);
        cpu.sound_timer = 0x20;
        cpu.is_waiting_for_key = (true, 0x0);

        let res = cpu.tick();

        assert_eq!(res.unwrap().is_buzzing, true);
    }

    #[test]
    fn test_noop() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0x01, 0x23], &mut rng);

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x0202);
    }

    #[test]
    fn test_clear_screen() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0x00, 0xe0], &mut rng);
        cpu.v_buffer = [true; SCREEN_WIDTH * SCREEN_HEIGHT];

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x0202);
        assert_eq!(cpu.v_buffer, [false; SCREEN_WIDTH * SCREEN_HEIGHT]);
    }

    #[test]
    fn test_return() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0x00, 0xee], &mut rng);
        cpu.stack[0] = 0x300;
        cpu.sp = 1;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.sp, 0);
        assert_eq!(cpu.pc, 0x300);
    }

    #[test]
    fn test_jump() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0x13, 0x21], &mut rng);

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x0321);
    }

    #[test]
    fn test_call() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0x23, 0x21], &mut rng);

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x0321);
        assert_eq!(cpu.stack[0], 0x202);
        assert_eq!(cpu.sp, 1);
    }

    #[test]
    fn test_call_stack_overflow() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0x23, 0x21], &mut rng);
        cpu.sp = 16;

        let res = cpu.tick();

        assert_eq!(res.unwrap_err(), CPUError::StackOverflow);
    }

    #[test]
    fn test_skip_vx_if_equal_skips() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0x30, 0x42], &mut rng);
        cpu.v_registers[0] = 0x42;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x204);
    }

    #[test]
    fn test_skip_vx_if_equal_does_not_skip() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0x30, 0x42], &mut rng);
        cpu.v_registers[0] = 0x00;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_skip_vx_if_not_equal_skips() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0x40, 0x42], &mut rng);
        cpu.v_registers[0] = 0x00;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x204);
    }

    #[test]
    fn test_skip_vx_if_not_equal_does_not_skip() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0x40, 0x42], &mut rng);
        cpu.v_registers[0] = 0x42;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_skip_if_equal_skips() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0x50, 0x10], &mut rng);
        cpu.v_registers[0] = 0xFF;
        cpu.v_registers[1] = 0xFF;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x204);
    }

    #[test]
    fn test_skip_if_equal_does_not_skip() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0x50, 0x10], &mut rng);
        cpu.v_registers[0] = 0x00;
        cpu.v_registers[1] = 0xFF;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_load_vx() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0x6A, 0x8F], &mut rng);

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x0202);
        assert_eq!(cpu.v_registers[0xA], 0x8F);
    }

    #[test]
    fn test_add_vx() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0x7A, 0x8F], &mut rng);
        cpu.v_registers[0xA] = 0x1;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x0202);
        assert_eq!(cpu.v_registers[0xA], 0x90);
    }

    #[test]
    fn test_set() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0x80, 0x10], &mut rng);
        cpu.v_registers[0x1] = 0xA;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.v_registers[0x0], 0xA);
        assert_eq!(cpu.v_registers[0x1], 0xA);
    }

    #[test]
    fn test_or() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0x80, 0x11], &mut rng);
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
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0x80, 0x12], &mut rng);
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
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0x80, 0x13], &mut rng);
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
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0x80, 0x14], &mut rng);
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
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0x80, 0x14], &mut rng);
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
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0x80, 0x15], &mut rng);
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
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0x80, 0x15], &mut rng);
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
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0x80, 0x16], &mut rng);
        cpu.v_registers[0x0] = 0b_0100_1110;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.v_registers[0x0], 0b_0010_0_111);
        assert_eq!(cpu.v_registers[0xF], 0x0);
    }

    #[test]
    fn test_shift_right_vx_with_shifted_out_bit() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0x80, 0x16], &mut rng);
        cpu.v_registers[0x0] = 0b_0100_1111;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.v_registers[0x0], 0b_0010_0_111);
        assert_eq!(cpu.v_registers[0xF], 0x01);
    }

    #[test]
    fn test_subn() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0x80, 0x17], &mut rng);
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
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0x80, 0x17], &mut rng);
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
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0x80, 0x1E], &mut rng);
        cpu.v_registers[0x0] = 0b_0100_1110;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.v_registers[0x0], 0b_1001_1100);
        assert_eq!(cpu.v_registers[0xF], 0x0);
    }

    #[test]
    fn test_shift_left_vx_with_shifted_out_bit() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0x80, 0x1E], &mut rng);
        cpu.v_registers[0x0] = 0b_1100_1111;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.v_registers[0x0], 0b_1001_1110);
        assert_eq!(cpu.v_registers[0xF], 0x01);
    }

    #[test]
    fn test_skip_if_not_equal_skips() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0x90, 0x10], &mut rng);
        cpu.v_registers[0] = 0xFF;
        cpu.v_registers[1] = 0x0F;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x204);
    }

    #[test]
    fn test_skip_if_not_equal_does_not_skip() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0x90, 0x10], &mut rng);
        cpu.v_registers[0] = 0xFF;
        cpu.v_registers[1] = 0xFF;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_load_i() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0xA1, 0x23], &mut rng);

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x0202);
        assert_eq!(cpu.i_register, 0x123);
    }

    #[test]
    fn test_jump_offset() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0xB2, 0x23], &mut rng);
        cpu.v_registers[0x2] = 0x10;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x233);
    }

    #[test]
    fn test_rand() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0xC0, 0xAB], &mut rng);

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.v_registers[0], 0x01 & 0xAB);
    }

    #[test]
    fn test_draw_sprite_simple() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0xD0, 0x13], &mut rng);
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
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0xD0, 0x13], &mut rng);
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
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0xD0, 0x11], &mut rng);
        cpu.i_register = 0x300;
        cpu.v_registers[0] = 0;
        cpu.v_registers[1] = 0;
        cpu.memory[0x300] = 0xFF;
        cpu.v_buffer[0..8].copy_from_slice(&[false, false, false, false, true, true, true, true]);

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(
            cpu.v_buffer[0..8],
            [true, true, true, true, false, false, false, false]
        );
        assert_eq!(cpu.v_registers[0xF], 1);
    }

    #[test]
    fn test_skip_if_key_skips_when_key_is_pressed() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0xE0, 0x9E], &mut rng);
        cpu.v_registers[0x0] = 0x07;
        cpu.keypad[0x07] = true;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x204);
    }

    #[test]
    fn test_skip_if_key_does_not_skip_when_key_is_not_pressed() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0xE0, 0x9E], &mut rng);
        cpu.v_registers[0x0] = 0x07;
        cpu.keypad[0x07] = false;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_skip_if_key_returns_error_when_invalid_key() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0xE0, 0x9E], &mut rng);
        cpu.v_registers[0x0] = 0x10;

        let res = cpu.tick();

        assert_eq!(res.unwrap_err(), CPUError::InvalidKey(0x10));
    }

    #[test]
    fn test_skip_if_not_key_skips_when_key_is_not_pressed() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0xE0, 0xA1], &mut rng);
        cpu.v_registers[0x0] = 0x07;
        cpu.keypad[0x07] = false;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x204);
    }

    #[test]
    fn test_skip_if_not_key_does_not_skip_when_key_is_pressed() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0xE0, 0xA1], &mut rng);
        cpu.v_registers[0x0] = 0x07;
        cpu.keypad[0x07] = true;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_load_delay() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0xF0, 0x07], &mut rng);
        cpu.delay_timer = 0xCC + 0x01; // +1 because it will be decremented with tick

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.v_registers[0x0], 0xCC);
    }

    #[test]
    fn test_wait_for_key() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0xF1, 0x0A], &mut rng);

        let res = cpu.tick();
        assert_eq!(
            res.unwrap(),
            TickStatus {
                is_waiting_for_key: true,
                is_buzzing: false,
            }
        );
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.is_waiting_for_key, (true, 0x01));

        // unblocked execution with a key pressed
        cpu.set_key_status(0xF, true).unwrap();

        assert_eq!(cpu.is_waiting_for_key, (false, 0x00));
        assert_eq!(cpu.v_registers[0x01], 0x0F);
    }

    #[test]
    fn test_set_delay() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0xF0, 0x15], &mut rng);
        cpu.v_registers[0x0] = 0xFA;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.delay_timer, 0xFA);
    }

    #[test]
    fn test_set_sound() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0xF0, 0x18], &mut rng);
        cpu.v_registers[0x0] = 0xFA;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.sound_timer, 0xFA);
    }

    #[test]
    fn test_add_to_index() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0xF0, 0x1E], &mut rng);
        cpu.i_register = 0xFA;
        cpu.v_registers[0] = 0x02;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.i_register, 0xFC);
    }

    #[test]
    fn test_add_to_index_overflows() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0xF0, 0x1E], &mut rng);
        cpu.i_register = 0xFFE;
        cpu.v_registers[0x0] = 0x02;
        cpu.v_registers[0xF] = 0x0;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.i_register, 0x00);
        assert_eq!(cpu.v_registers[0xF], 0x01);
    }

    #[test]
    fn test_load_bcd() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0xF0, 0x33], &mut rng);
        cpu.v_registers[0x0] = 251;
        cpu.i_register = 0x500;

        let res = cpu.tick();

        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.memory[0x500], 0x02);
        assert_eq!(cpu.memory[0x501], 0x05);
        assert_eq!(cpu.memory[0x502], 0x01);
    }

    #[test]
    fn test_load_bcd_returns_invalid_address_error() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0xF0, 0x33], &mut rng);
        cpu.i_register = 0xFFF;

        let res = cpu.tick();
        assert_eq!(res.unwrap_err(), CPUError::InvalidAddress(0x1000));
    }

    #[test]
    fn test_load_mem() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0xF2, 0x55], &mut rng);
        cpu.i_register = 0x500;
        cpu.memory[0x500..=0x503].copy_from_slice(&[0x02, 0x04, 0x06, 0xFF]);

        let res = cpu.tick();
        assert!(res.is_ok());
        assert_eq!(cpu.pc, 0x202);
        assert_eq!(cpu.i_register, 0x500);
        assert_eq!(cpu.v_registers[0x0], 0x02);
        assert_eq!(cpu.v_registers[0x1], 0x04);
        assert_eq!(cpu.v_registers[0x2], 0x06);
        assert_eq!(cpu.v_registers[0x3], 0x00);
    }

    #[test]
    fn test_load_mem_returns_invalid_address_error() {
        let mut rng = any_mocked_rng();
        let mut cpu = any_cpu_with_rom(&[0xF1, 0x55], &mut rng);
        cpu.i_register = 0xFFF;

        let res = cpu.tick();
        assert_eq!(res.unwrap_err(), CPUError::InvalidAddress(0x1000));
    }
}
