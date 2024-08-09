⚠️ This is **deprecated** in favor of [chip8-wasm](https://github.com/belen-albeza/chip8-wasm/)

# chip8-rs

CHIP-8 emulator in Rust

## Usage

From binary:

```zsh
./chip8-rs <FILE>
```

From source with [Cargo](https://doc.rust-lang.org/cargo/):

```zsh
cargo run <FILE>
```

You can quit by closing the window or with the `Esc` key.

### Included ROMs

- `invalid.ch8`: this one contains a single, invalid instruction. The emulator should yield an error if you try to run it.
- `jump.ch8`: this one contains a single, valid instruction that makes the code run indefinitely. You can quite
- `poker.ch8`: displays the four poker suits. This ROM uses the same instruction set as the famous `IBM Logo.ch8` and it's [a good starter ROM](https://tobiasvl.github.io/blog/write-a-chip-8-emulator/#instructions) if you are implementing your own CHIP-8 emulator.
- `wait_for_key.ch8`: displays a sprite and waits for a key press. Then displays a different sprite.
- `buzzer.ch8`: plays the audio buzzer for 1 second (assuming sound timer ticking at 60Hz).
- `fabada.ch8`: displays the hex digits `"fabada"` on the screen.

## Implemented opcodes

Standard CHIP-8 instructions:

- [x] 0nnn - SYS addr
- [x] 00E0 - CLS
- [x] 00EE - RET
- [x] 1nnn - JP addr
- [x] 2nnn - CALL addr
- [x] 3xkk - SE Vx, byte
- [x] 4xkk - SNE Vx, byte
- [x] 5xy0 - SE Vx, Vy
- [x] 6xkk - LD Vx, byte
- [x] 7xkk - ADD Vx, byte
- [x] 8xy0 - LD Vx, Vy
- [x] 8xy1 - OR Vx, Vy
- [x] 8xy2 - AND Vx, Vy
- [x] 8xy3 - XOR Vx, Vy
- [x] 8xy4 - ADD Vx, Vy
- [x] 8xy5 - SUB Vx, Vy
- [x] 8xy6 - SHR Vx {, Vy}
- [x] 8xy7 - SUBN Vx, Vy
- [x] 8xyE - SHL Vx {, Vy}
- [x] 9xy0 - SNE Vx, Vy
- [x] Annn - LD I, addr
- [x] Bnnn - JP V0, addr
- [x] Cxkk - RND Vx, byte
- [x] Dxyn - DRW Vx, Vy, nibble
- [x] Ex9E - SKP Vx
- [x] ExA1 - SKNP Vx
- [x] Fx07 - LD Vx, DT
- [x] Fx0A - LD Vx, K
- [x] Fx15 - LD DT, Vx
- [x] Fx18 - LD ST, Vx
- [x] Fx1E - ADD I, Vx
<<<<<<< HEAD
- [x] Fx29 - LD F, Vx
=======
- [ ] Fx29 - LD F, Vx
>>>>>>> main
- [x] Fx33 - LD B, Vx
- [x] Fx55 - LD [I], Vx
- [x] Fx65 - LD Vx, [I]

> ⚠️ Note: for ambiguous opcodes (`8xy6`, `8xyE`, `Bnnn`), the Super-CHIP behaviour has been implemented.

> ⚠️ Note:`Fx1E` opcode has been implemented with setting the `VF` register on carry.
