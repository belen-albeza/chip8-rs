use crate::error::CPUError;

pub const DIGIT_SIZE: usize = 5;

type Result<T> = std::result::Result<T, CPUError>;

pub fn draw(
    sprite: &[u8],
    x: usize,
    y: usize,
    bounds: (usize, usize),
    buffer: &mut [bool],
) -> bool {
    let x = x % bounds.0;
    let mut did_collide = false;

    for row in 0..sprite.len() {
        let y = (y + row) % bounds.1;
        for col in 0..8 {
            let x = (x + col) % bounds.0;
            let raw_pixel = sprite[row] >> (8 - col - 1) & 0b_0000_0001;
            let pixel = raw_pixel == 0x1;

            let index = y * bounds.0 + x;

            did_collide |= buffer[index] & pixel;
            // XOR existing screen pixel with sprite pixel to draw
            buffer[index] ^= pixel;
        }
    }

    did_collide
}

pub fn read_sprite(addr: usize, size: usize, memory: &[u8]) -> Result<&[u8]> {
    if (addr + size - 1) >= memory.len() {
        return Err(CPUError::InvalidAddress((addr + size - 1) as u16));
    }

    let sprite = &memory[addr..addr + size];

    Ok(sprite)
}

pub fn digit_sprite_data(x: u8) -> Result<[u8; DIGIT_SIZE]> {
    match x {
        0x0 => Ok([0xF0, 0x90, 0x90, 0x90, 0xF0]),
        0x1 => Ok([0x20, 0x60, 0x20, 0x20, 0x70]),
        0x2 => Ok([0xF0, 0x10, 0xF0, 0x80, 0xF0]),
        0x3 => Ok([0xF0, 0x10, 0xF0, 0x10, 0xF0]),
        0x4 => Ok([0x90, 0x90, 0xF0, 0x10, 0x10]),
        0x5 => Ok([0xF0, 0x80, 0xF0, 0x10, 0xF0]),
        0x6 => Ok([0xF0, 0x80, 0xF0, 0x90, 0xF0]),
        0x7 => Ok([0xF0, 0x10, 0x20, 0x40, 0x40]),
        0x8 => Ok([0xF0, 0x90, 0xF0, 0x90, 0xF0]),
        0x9 => Ok([0xF0, 0x90, 0xF0, 0x10, 0xF0]),
        0xA => Ok([0xF0, 0x90, 0xF0, 0x90, 0x90]),
        0xB => Ok([0xE0, 0x90, 0xE0, 0x90, 0xE0]),
        0xC => Ok([0xF0, 0x80, 0x80, 0x80, 0xF0]),
        0xD => Ok([0xE0, 0x90, 0x90, 0x90, 0xE0]),
        0xE => Ok([0xF0, 0x80, 0xF0, 0x80, 0xF0]),
        0xF => Ok([0xF0, 0x80, 0xF0, 0x80, 0x80]),
        _ => Err(CPUError::InvalidDigit(x)),
    }
}
