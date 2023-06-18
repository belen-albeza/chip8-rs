use crate::error::CPUError;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Instruction {
    // Unsupported opcode
    NoOp,
    // 00e0 -> clear screen
    ClearScreen,
    // 00ee -> SP -=1; PC = Stack[SP];
    Return,
    // 1nnn -> PC = nnn
    Jump(u16),
    // 2nnn -> Stack[SP] = PC; SP += 1; PC = nnn
    Call(u16),
    // 3xkk -> Skip next if Vx == kk
    SkipVxEqual(u8, u8),
    // 4xkk -> Skip next if Vx != kk
    SkipVxNotEqual(u8, u8),
    // 5xy0 -> Skip next if Vx == Vy
    SkipEqual(u8, u8),
    // 6xkk -> Vx = kk
    LoadVx(u8, u8),
    // 7xkk -> Vx += kk
    AddVx(u8, u8),
    // 8xy0 -> Vx = Vy
    Set(u8, u8),
    // 8xy1 -> Vx = Vx OR Vy
    Or(u8, u8),
    // 8xy2 -> Vx = Vx AND Vy
    And(u8, u8),
    // 8xy3 -> Vx = Vx XOR Vy
    Xor(u8, u8),
    // 8xy4 -> Vx = Vx + Vy; VF = carry
    Add(u8, u8),
    // 8xy5 -> Vx = Vx - Vy; VF = NOT borrow
    Sub(u8, u8),
    // 8xy6 -> Vx >> 1; VF = shifted out bit
    ShiftRightVx(u8),
    // 8xy7 -> Vx = Vy - Vy; VF = NOT borrow
    SubN(u8, u8),
    // 8xyE -> Vx << 1; VF = shifted out bit
    ShiftLeftVx(u8),
    // Annn -> I = nnn
    LoadI(u16),
    // Dxyn -> Draw n-byte sprite starting at I at (Vx,Vy); VF = collision
    DrawSprite(u8, u8, u8),
}

impl TryFrom<u16> for Instruction {
    type Error = CPUError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        let nibbles = (
            ((value & 0xF000) >> 12) as u8,
            ((value & 0x0F00) >> 8) as u8,
            ((value & 0x00F0) >> 4) as u8,
            (value & 0x000F) as u8,
        );

        let nnn = (value & 0x0FFF) as u16;
        let kk = (value & 0x00FF) as u8;

        match nibbles {
            (0x0, 0x0, 0xe, 0x0) => Ok(Self::ClearScreen),
            (0x0, 0x0, 0xe, 0xe) => Ok(Self::Return),
            (0x0, _, _, _) => Ok(Self::NoOp),
            (0x1, _, _, _) => Ok(Self::Jump(nnn)),
            (0x2, _, _, _) => Ok(Self::Call(nnn)),
            (0x3, x, _, _) => Ok(Self::SkipVxEqual(x, kk)),
            (0x4, x, _, _) => Ok(Self::SkipVxNotEqual(x, kk)),
            (0x5, x, y, 0) => Ok(Self::SkipEqual(x, y)),
            (0x6, x, _, _) => Ok(Self::LoadVx(x, kk)),
            (0x7, x, _, _) => Ok(Self::AddVx(x, kk)),
            (0x8, x, y, 0x0) => Ok(Self::Set(x, y)),
            (0x8, x, y, 0x1) => Ok(Self::Or(x, y)),
            (0x8, x, y, 0x2) => Ok(Self::And(x, y)),
            (0x8, x, y, 0x3) => Ok(Self::Xor(x, y)),
            (0x8, x, y, 0x4) => Ok(Self::Add(x, y)),
            (0x8, x, y, 0x5) => Ok(Self::Sub(x, y)),
            (0x8, x, _, 0x6) => Ok(Self::ShiftRightVx(x)),
            (0x8, x, y, 0x7) => Ok(Self::SubN(x, y)),
            (0x8, x, _, 0xE) => Ok(Self::ShiftLeftVx(x)),
            (0xA, _, _, _) => Ok(Self::LoadI(nnn)),
            (0xD, x, y, n) => Ok(Self::DrawSprite(x, y, n)),
            _ => Err(CPUError::InvalidOpcode(value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_invalid_opcode() {
        let res = Instruction::try_from(0xFFFF as u16);
        assert_eq!(res.unwrap_err(), CPUError::InvalidOpcode(0xFFFF));
    }

    #[test]
    fn test_try_from_valid_opcodes() {
        assert_eq!(Instruction::try_from(0x0123), Ok(Instruction::NoOp));
        assert_eq!(Instruction::try_from(0x00E0), Ok(Instruction::ClearScreen));
        assert_eq!(Instruction::try_from(0x00EE), Ok(Instruction::Return));
        assert_eq!(Instruction::try_from(0x1123), Ok(Instruction::Jump(0x123)));
        assert_eq!(Instruction::try_from(0x2123), Ok(Instruction::Call(0x123)));
        assert_eq!(
            Instruction::try_from(0x3A11),
            Ok(Instruction::SkipVxEqual(0xA, 0x11))
        );
        assert_eq!(
            Instruction::try_from(0x4A11),
            Ok(Instruction::SkipVxNotEqual(0xA, 0x11))
        );
        assert_eq!(
            Instruction::try_from(0x5AB0),
            Ok(Instruction::SkipEqual(0xA, 0xB))
        );
        assert_eq!(
            Instruction::try_from(0x6122),
            Ok(Instruction::LoadVx(0x1, 0x22))
        );
        assert_eq!(
            Instruction::try_from(0x73FF),
            Ok(Instruction::AddVx(0x3, 0xFF))
        );
        assert_eq!(
            Instruction::try_from(0x8120),
            Ok(Instruction::Set(0x1, 0x2))
        );
        assert_eq!(Instruction::try_from(0x8AB1), Ok(Instruction::Or(0xA, 0xB)));
        assert_eq!(
            Instruction::try_from(0x8AB2),
            Ok(Instruction::And(0xA, 0xB))
        );
        assert_eq!(
            Instruction::try_from(0x8AB3),
            Ok(Instruction::Xor(0xA, 0xB))
        );
        assert_eq!(
            Instruction::try_from(0x8AB4),
            Ok(Instruction::Add(0xA, 0xB))
        );
        assert_eq!(
            Instruction::try_from(0x8AB5),
            Ok(Instruction::Sub(0xA, 0xB))
        );
        assert_eq!(
            Instruction::try_from(0x8AB6),
            Ok(Instruction::ShiftRightVx(0xA))
        );
        assert_eq!(
            Instruction::try_from(0x8AB7),
            Ok(Instruction::SubN(0xA, 0xB))
        );
        assert_eq!(
            Instruction::try_from(0x8ABE),
            Ok(Instruction::ShiftLeftVx(0xA))
        );
        assert_eq!(Instruction::try_from(0xABCD), Ok(Instruction::LoadI(0xBCD)));
        assert_eq!(
            Instruction::try_from(0xD12A),
            Ok(Instruction::DrawSprite(0x1, 0x2, 0xA))
        );
    }
}
