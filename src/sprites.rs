use crate::error::CPUError;

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
