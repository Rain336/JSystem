use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub enum RarcError {
    Yaz0Error(yaz0::Error),
    InvalidMagic(u32),
}

impl Display for RarcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use RarcError::*;
        match self {
            Yaz0Error(err) => err.fmt(f),
            InvalidMagic(invalid) => f.write_fmt(format_args!(
                "Magic number mismatched. Expected: 0x52415243 Got: {:#x}",
                invalid
            )),
        }
    }
}

impl Error for RarcError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use RarcError::*;
        match self {
            Yaz0Error(err) => Some(err),
            InvalidMagic(_) => None,
        }
    }
}

impl From<yaz0::Error> for RarcError {
    fn from(error: yaz0::Error) -> Self {
        RarcError::Yaz0Error(error)
    }
}
