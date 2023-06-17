use std::fs;
use std::path::PathBuf;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::EventPump;

use crate::cpu::CPU;
use crate::error::Error;

pub type Result<T> = std::result::Result<T, Error>;

const SCALE: usize = 10;
const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;

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
        let video_system = sdl_context.video().map_err(to_sdl_err)?;
        let window = video_system
            .window(
                "CHIP-8 by ladybenko",
                (SCREEN_WIDTH * SCALE) as u32,
                (SCREEN_HEIGHT * SCALE) as u32,
            )
            .position_centered()
            .build()?;
        let mut canvas = window.into_canvas().present_vsync().build()?;
        canvas
            .set_scale(SCALE as f32, SCALE as f32)
            .map_err(to_sdl_err)?;
        let mut event_pump = sdl_context.event_pump().map_err(to_sdl_err)?;

        let creator = canvas.texture_creator();
        let mut texture = creator.create_texture_target(
            PixelFormatEnum::RGB24,
            SCREEN_WIDTH as u32,
            SCREEN_HEIGHT as u32,
        )?;

        let mut screen_buffer = [0 as u8; 3 * SCREEN_WIDTH * SCREEN_HEIGHT];

        loop {
            let shall_halt = self.handle_user_input(&mut event_pump);
            if shall_halt {
                break;
            }

            self.cpu.tick()?;

            if self.update_screen_buffer(&mut screen_buffer) {
                texture.update(None, &screen_buffer, SCREEN_WIDTH * 3)?;
                canvas.copy(&texture, None, None).map_err(to_sdl_err)?;
                canvas.present();
            }

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

    fn update_screen_buffer(
        &mut self,
        buffer: &mut [u8; 3 * SCREEN_WIDTH * SCREEN_HEIGHT],
    ) -> bool {
        for pixel in buffer {
            *pixel = !*pixel;
        }

        true
    }

    // fn init_sdl(
    //     &mut self,
    // ) -> Result<(
    //     sdl2::render::Canvas<sdl2::video::Window>,
    //     sdl2::render::Texture,
    //     sdl2::EventPump,
    // )> {

    //     Ok((canvas, texture, event_pump))
    // }
}

fn to_sdl_err(err: String) -> Error {
    Error::SystemError(err)
}
