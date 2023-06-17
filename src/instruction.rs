use crate::error::CPUError;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Instruction {
    // 00e0 -> clear screen
    ClearScreen,
    // 1nnn -> PC = nnn
    Jump(u16),
    // 6xkk -> Vx = kk
    LoadVx(u8, u8),
    // 7xkk -> Vx += kk
    AddVx(u8, u8),
    // Annn -> I = nnn
    LoadI(u16),
    // Dxyn -> Draw n-byte sprite starting at I at (Vx,Vy). VF=collision
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
            (0x1, _, _, _) => Ok(Self::Jump(nnn)),
            (0x6, x, _, _) => Ok(Self::LoadVx(x, kk)),
            (0x7, x, _, _) => Ok(Self::AddVx(x, kk)),
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
        assert_eq!(Instruction::try_from(0x1123), Ok(Instruction::Jump(0x123)));
        assert_eq!(
            Instruction::try_from(0x6122),
            Ok(Instruction::LoadVx(0x1, 0x22))
        );
        assert_eq!(
            Instruction::try_from(0x73FF),
            Ok(Instruction::AddVx(0x3, 0xFF))
        );
        assert_eq!(Instruction::try_from(0xABCD), Ok(Instruction::LoadI(0xBCD)));
        assert_eq!(Instruction::try_from(0x00E0), Ok(Instruction::ClearScreen));
        assert_eq!(
            Instruction::try_from(0xD12A),
            Ok(Instruction::DrawSprite(0x1, 0x2, 0xA))
        );
    }
}
