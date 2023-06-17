use core::fmt;
use sdl2::render::{TextureValueError, UpdateTextureError};
use sdl2::video::WindowBuildError;
use sdl2::IntegerOrSdlError;
use std::error;
use std::error::Error as ErrorTrait;
use std::io;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    RuntimeError(CPUError),
    SystemError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SystemError(msg) => write!(f, "{}", msg),
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
            _ => None,
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

impl From<WindowBuildError> for Error {
    fn from(err: WindowBuildError) -> Error {
        Error::SystemError(format!("{}", err))
    }
}

impl From<IntegerOrSdlError> for Error {
    fn from(err: IntegerOrSdlError) -> Error {
        Error::SystemError(format!("{}", err))
    }
}

impl From<TextureValueError> for Error {
    fn from(err: TextureValueError) -> Error {
        Error::SystemError(format!("{}", err))
    }
}

impl From<UpdateTextureError> for Error {
    fn from(err: UpdateTextureError) -> Error {
        Error::SystemError(format!("{}", err))
    }
}

#[derive(Debug, PartialEq)]
pub enum CPUError {
    MemoryOverflow,
    StackOverflow,
    InvalidOpcode(u16),
    InvalidAddress(u16),
    InvalidVRegister(u8),
}

impl fmt::Display for CPUError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MemoryOverflow => write!(f, "Memory overflow"),
            Self::StackOverflow => write!(f, "Stack overflow"),
            Self::InvalidOpcode(op) => write!(f, "Invalid opcode: {:#04X}", op),
            Self::InvalidAddress(addr) => write!(f, "Invalid memory address: {:#04X}", addr),
            Self::InvalidVRegister(i) => write!(f, "Invalid V-Register: {:#01X}", i),
        }
    }
}

impl error::Error for CPUError {}
