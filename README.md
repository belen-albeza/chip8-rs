# chip8-rs

CHIP-8 emulator in Rust

## Implemented opcodes

Standard CHIP-8 instructions:

- [x] 0nnn - SYS addr
- [x] 00E0 - CLS
- [x] 00EE - RET
- [x] 1nnn - JP addr
- [x] 2nnn - CALL addr
- [ ] 3xkk - SE Vx, byte
- [ ] 4xkk - SNE Vx, byte
- [ ] 5xy0 - SE Vx, Vy
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
- [ ] 9xy0 - SNE Vx, Vy
- [x] Annn - LD I, addr
- [ ] Bnnn - JP V0, addr
- [ ] Cxkk - RND Vx, byte
- [x] Dxyn - DRW Vx, Vy, nibble
- [ ] Ex9E - SKP Vx
- [ ] ExA1 - SKNP Vx
- [ ] Fx07 - LD Vx, DT
- [ ] Fx0A - LD Vx, K
- [ ] Fx15 - LD DT, Vx
- [ ] Fx18 - LD ST, Vx
- [ ] Fx1E - ADD I, Vx
- [ ] Fx29 - LD F, Vx
- [ ] Fx33 - LD B, Vx
- [ ] Fx55 - LD [I], Vx
- [ ] Fx65 - LD Vx, [I]
