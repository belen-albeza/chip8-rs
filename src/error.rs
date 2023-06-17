use core::fmt;
use std::error;
use std::error::Error as ErrorTrait;
use std::io;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    RuntimeError(CPUError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            _ => match self.source() {
                Some(err) => write!(f, "{}", err),
                None => write!(f, "{:?}", self),
            },
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::IOError(ref e) => Some(e),
            Self::RuntimeError(ref e) => Some(e),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IOError(err)
    }
}

impl From<CPUError> for Error {
    fn from(err: CPUError) -> Error {
        Error::RuntimeError(err)
    }
}

#[derive(Debug, PartialEq)]
pub enum CPUError {
    MemoryOverflow,
    InvalidOpcode(u16),
    InvalidAddress(u16),
}

impl fmt::Display for CPUError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MemoryOverflow => write!(f, "Memory overflow"),
            Self::InvalidOpcode(op) => write!(f, "Invalid opcode: {:#04X}", op),
            Self::InvalidAddress(addr) => write!(f, "Invalid memory address: {:#04X}", addr),
        }
    }
}

impl error::Error for CPUError {}
