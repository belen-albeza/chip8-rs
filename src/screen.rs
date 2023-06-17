use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};

use crate::error::Error;

const SCALE: usize = 10;
const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const BUFFER_SIZE: usize = 3 * SCREEN_WIDTH * SCREEN_HEIGHT;

pub type Result<T> = std::result::Result<T, Error>;

pub fn build_canvas_and_creator(
    context: &sdl2::Sdl,
) -> Result<(Canvas<Window>, TextureCreator<WindowContext>)> {
    let video_system = context.video().map_err(to_sdl_err)?;
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

    let texture_creator = canvas.texture_creator();

    Ok((canvas, texture_creator))
}

pub struct Screen<'a> {
    pub texture: Texture<'a>,
    pub buffer: [u8; BUFFER_SIZE],
}

impl<'a> Screen<'a> {
    pub fn frame(
        &mut self,
        canvas: &mut Canvas<Window>,
        vmem: &[bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    ) -> Result<()> {
        if self.update_screen_buffer(vmem) {
            self.texture.update(None, &self.buffer, SCREEN_WIDTH * 3)?;
            canvas.copy(&self.texture, None, None).map_err(to_sdl_err)?;
            canvas.present();
        }

        Ok(())
    }

    fn update_screen_buffer(&mut self, vmem: &[bool; SCREEN_WIDTH * SCREEN_HEIGHT]) -> bool {
        for i in 0..(SCREEN_WIDTH * SCREEN_HEIGHT) {
            let (red, green, blue) = ((i * 3), (i * 3 + 1), (i * 3 + 2));

            self.buffer[red] = if vmem[i] { 0xFF } else { 0x00 };
            self.buffer[green] = if vmem[i] { 0xFF } else { 0x00 };
            self.buffer[blue] = if vmem[i] { 0xFF } else { 0x00 };
        }

        true
    }
}

impl<'a> TryFrom<&'a TextureCreator<WindowContext>> for Screen<'a> {
    type Error = Error;

    fn try_from(
        texture_creator: &'a TextureCreator<WindowContext>,
    ) -> std::result::Result<Self, Self::Error> {
        // let creator = canvas.texture_creator();
        let texture = texture_creator.create_texture_target(
            PixelFormatEnum::RGB24,
            SCREEN_WIDTH as u32,
            SCREEN_HEIGHT as u32,
        )?;

        Ok(Self {
            texture,
            buffer: [0; BUFFER_SIZE],
        })
    }
}

fn to_sdl_err(err: String) -> Error {
    Error::SystemError(err)
}
