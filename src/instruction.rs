use crate::error::CPUError;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Instruction {
    // 1nnn -> Jump to address nnnn
    Jump(u16),
    // 6xkk -> Vx = kk
    LoadVx(u8, u8),
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
            (0x1, _, _, _) => Ok(Self::Jump(nnn)),
            (0x6, x, _, _) => Ok(Self::LoadVx(x, kk)),
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
    }
}
