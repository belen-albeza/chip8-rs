use rand::RngCore;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::EventPump;

use crate::cpu::CPU;
use crate::error::Error;
use crate::screen;

pub type Result<T> = std::result::Result<T, Error>;

pub struct VM<'a> {
    cpu: CPU<'a>,
    keymap: HashMap<Scancode, u8>,
}

impl<'a> VM<'a> {
    pub fn new(rng: &'a mut impl RngCore) -> Self {
        Self {
            cpu: CPU::new(rng),
            keymap: Self::default_keymap(),
        }
    }

    pub fn load_rom(&mut self, filename: PathBuf) -> Result<()> {
        self.reset();

        let rom = fs::read(filename)?;
        self.cpu.load_rom(&rom)?;

        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        let sdl_context = sdl2::init().map_err(to_sdl_err)?;
        let (mut canvas, texture_creator) = screen::build_canvas_and_creator(&sdl_context)?;
        let mut screen = screen::Screen::try_from(&texture_creator)?;
        let mut event_pump = sdl_context.event_pump().map_err(to_sdl_err)?;

        loop {
            let shall_halt = self.handle_user_input(&mut event_pump)?;
            if shall_halt {
                break;
            }

            let _ = self.cpu.tick()?;
            screen.frame(&mut canvas, self.cpu.visual_buffer())?;

            ::std::thread::sleep(std::time::Duration::from_millis(
                (1.0 / 30.0 * 1000.0) as u64,
            ));
        }

        Ok(())
    }

    fn default_keymap() -> HashMap<Scancode, u8> {
        HashMap::from([
            (Scancode::Num1, 0x01),
            (Scancode::Num2, 0x02),
            (Scancode::Num3, 0x03),
            (Scancode::Num4, 0x0C),
            (Scancode::Q, 0x04),
            (Scancode::W, 0x05),
            (Scancode::E, 0x06),
            (Scancode::R, 0x0D),
            (Scancode::A, 0x07),
            (Scancode::S, 0x08),
            (Scancode::D, 0x09),
            (Scancode::F, 0x0E),
            (Scancode::Z, 0x0A),
            (Scancode::X, 0x00),
            (Scancode::C, 0x0B),
            (Scancode::V, 0x0F),
            (Scancode::Left, 0x07),
            (Scancode::Right, 0x09),
            (Scancode::Up, 0x05),
            (Scancode::Down, 0x08),
        ])
    }

    fn reset(&mut self) {
        self.cpu.reset();
    }

    fn handle_user_input(&mut self, event_pump: &mut EventPump) -> Result<bool> {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    return Ok(true);
                }
                Event::KeyDown {
                    scancode: Some(ref code),
                    ..
                } => {
                    if let Some(key_index) = self.keymap.get(code) {
                        self.cpu.set_key_status(*key_index as usize, true)?;
                    }
                }
                Event::KeyUp {
                    scancode: Some(ref code),
                    ..
                } => {
                    if let Some(key_index) = self.keymap.get(code) {
                        self.cpu.set_key_status(*key_index as usize, false)?;
                    }
                }
                _ => {}
            }
        }

        Ok(false)
    }
}

fn to_sdl_err(err: String) -> Error {
    Error::SystemError(err)
}
