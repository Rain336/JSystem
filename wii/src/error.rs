use std::error::Error;
use std::fmt::Display;
use std::io;
use std::str::Utf8Error;
use std::string::FromUtf16Error;

pub type WiiResult<T> = Result<T, WiiError>;

#[derive(Debug)]
pub enum WiiError {
    InvalidMagic,
    HashMismatch,
    InvalidType,
    IoError(io::Error),
    Utf16Error(FromUtf16Error),
    Utf8Error(Utf8Error),
}

impl Display for WiiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WiiError::InvalidMagic => f.write_str("Magic number didn't match."),
            WiiError::HashMismatch => f.write_str("The read and calculated hashes didn't match."),
            WiiError::InvalidType => f.write_str("Invalid or unknown type supplied."),
            WiiError::IoError(e) => std::fmt::Debug::fmt(e, f),
            WiiError::Utf16Error(e) => std::fmt::Display::fmt(e, f),
            WiiError::Utf8Error(e) => std::fmt::Display::fmt(e, f),
        }
    }
}

impl From<io::Error> for WiiError {
    fn from(e: io::Error) -> Self {
        WiiError::IoError(e)
    }
}

impl From<FromUtf16Error> for WiiError {
    fn from(e: FromUtf16Error) -> Self {
        WiiError::Utf16Error(e)
    }
}

impl From<Utf8Error> for WiiError {
    fn from(e: Utf8Error) -> Self {
        WiiError::Utf8Error(e)
    }
}

impl Error for WiiError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            WiiError::InvalidMagic => None,
            WiiError::HashMismatch => None,
            WiiError::InvalidType => None,
            WiiError::IoError(e) => Some(e),
            WiiError::Utf16Error(e) => Some(e),
            WiiError::Utf8Error(e) => Some(e),
        }
    }
}
