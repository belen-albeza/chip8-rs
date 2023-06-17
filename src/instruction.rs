use crate::error::CPUError;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Instruction {
    Jump(u16), // 1nnn
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

        match nibbles {
            (0x1, _, _, _) => Ok(Self::Jump(nnn)),
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
        assert_eq!(Instruction::try_from(0x1123), Ok(Instruction::Jump(0x123)))
    }
}
