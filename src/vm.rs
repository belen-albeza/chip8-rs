use std::fs;
use std::path::PathBuf;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::EventPump;

use crate::cpu::CPU;
use crate::error::Error;
use crate::screen;

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
        let sdl_context = sdl2::init().map_err(to_sdl_err)?;
        let (mut canvas, texture_creator) = screen::build_canvas_and_creator(&sdl_context)?;
        let mut screen = screen::Screen::try_from(&texture_creator)?;
        let mut event_pump = sdl_context.event_pump().map_err(to_sdl_err)?;

        loop {
            let shall_halt = self.handle_user_input(&mut event_pump);
            if shall_halt {
                break;
            }

            self.cpu.tick()?;
            screen.frame(&mut canvas, self.cpu.visual_buffer())?;

            ::std::thread::sleep(std::time::Duration::from_millis(
                (1.0 / 30.0 * 1000.0) as u64,
            ));
        }

        Ok(())
    }

    fn reset(&mut self) {
        self.cpu = CPU::new();
    }

    fn handle_user_input(&mut self, event_pump: &mut EventPump) -> bool {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    return true;
                }
                _ => {}
            }
        }

        false
    }
}

fn to_sdl_err(err: String) -> Error {
    Error::SystemError(err)
}
