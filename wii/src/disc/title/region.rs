use std::fmt::{self, Write};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RegionCode {
    German,
    Usa,
    France,
    Italy,
    Japan,
    Korea,
    PAL,
    Russia,
    Spanish,
    Taiwan,
    Australia,
    Unknown(u8),
}

impl From<u8> for RegionCode {
    fn from(value: u8) -> Self {
        match value {
            b'D' => Self::German,
            b'E' => Self::Usa,
            b'F' => Self::France,
            b'I' => Self::Italy,
            b'J' => Self::Japan,
            b'K' => Self::Korea,
            b'P' => Self::PAL,
            b'R' => Self::Russia,
            b'S' => Self::Spanish,
            b'T' => Self::Taiwan,
            b'U' => Self::Australia,
            c => Self::Unknown(c),
        }
    }
}

impl From<RegionCode> for u8 {
    fn from(val: RegionCode) -> Self {
        match val {
            RegionCode::German => b'D',
            RegionCode::Usa => b'E',
            RegionCode::France => b'F',
            RegionCode::Italy => b'I',
            RegionCode::Japan => b'J',
            RegionCode::Korea => b'K',
            RegionCode::PAL => b'P',
            RegionCode::Russia => b'R',
            RegionCode::Spanish => b'S',
            RegionCode::Taiwan => b'T',
            RegionCode::Australia => b'U',
            RegionCode::Unknown(c) => c,
        }
    }
}

impl fmt::Display for RegionCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let encoded: u8 = (*self).into();
        f.write_char(encoded as char)
    }
}