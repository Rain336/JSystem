use crate::BcsvError;
use std::fmt::{self, Display};
use std::str::FromStr;

/// The type of a column.
/// Types in BCSV have no siging information, so all types are assumed to be signed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DataType {
    /// A 32-bit integer [`i32`].
    Int32 = 0,

    /// A 32 byte inline string.
    /// This type should be seen as depricated and use of [`OffsetString`](`DataType::OffsetString`) should be prefered.
    InlineString = 1,

    /// An IEEE 32-bit single precision floating point value [`f32`].
    Float = 2,

    /// Another 32-bit integer.
    /// It is not really known why this exists, but most libraries use this type to interpret an unsinged 32-bit value [`u32`], so we do the same.
    UInt32 = 3,

    /// A 16-bit integer [`i16`].
    Int16 = 4,

    /// A 8-bit integer [`i8`].
    Int8 = 5,

    /// A 32-bit offset into the string pool.
    /// This is the preferd way to store string is BCSV, since it allows for arbitrarily sized strings.
    OffsetString = 6,

    /// A null type. Has a size of 0 and can be used to add empty columns.
    Null = 7,
}

impl DataType {
    /// Gets the size of a type in bytes.
    pub fn size(self) -> usize {
        match self {
            DataType::Int32 => 4,
            DataType::InlineString => 32,
            DataType::Float => 4,
            DataType::UInt32 => 4,
            DataType::Int16 => 2,
            DataType::Int8 => 1,
            DataType::OffsetString => 4,
            DataType::Null => 0,
        }
    }

    /// Creates a default [`DataValue`] from a type.
    pub fn default_value(self) -> DataValue {
        match self {
            DataType::Int32 => DataValue::Int32(0),
            DataType::InlineString => DataValue::InlineString(String::new()),
            DataType::Float => DataValue::Float(0.0),
            DataType::UInt32 => DataValue::UInt32(0),
            DataType::Int16 => DataValue::Int16(0),
            DataType::Int8 => DataValue::Int8(0),
            DataType::OffsetString => DataValue::OffsetString(String::new()),
            DataType::Null => DataValue::Null,
        }
    }
}

impl TryFrom<u8> for DataType {
    type Error = BcsvError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(DataType::Int32),
            1 => Ok(DataType::InlineString),
            2 => Ok(DataType::Float),
            3 => Ok(DataType::UInt32),
            4 => Ok(DataType::Int16),
            5 => Ok(DataType::Int8),
            6 => Ok(DataType::OffsetString),
            7 => Ok(DataType::Null),
            _ => Err(BcsvError::InvaildDataType(value)),
        }
    }
}

impl Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataType::Int32 => f.write_str("Int32"),
            DataType::InlineString => f.write_str("InlineString"),
            DataType::Float => f.write_str("Float"),
            DataType::UInt32 => f.write_str("UInt32"),
            DataType::Int16 => f.write_str("Int16"),
            DataType::Int8 => f.write_str("Int8"),
            DataType::OffsetString => f.write_str("OffsetString"),
            DataType::Null => f.write_str("Null"),
        }
    }
}

impl FromStr for DataType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Int32" => Ok(DataType::Int32),
            "InlineString" => Ok(DataType::InlineString),
            "Float" => Ok(DataType::Float),
            "UInt32" => Ok(DataType::UInt32),
            "Int16" => Ok(DataType::Int16),
            "Int8" => Ok(DataType::Int8),
            "OffsetString" => Ok(DataType::OffsetString),
            "Null" => Ok(DataType::Null),
            _ => Err(()),
        }
    }
}

/// A enum representing on the the values of [`DataType`].
#[derive(Clone)]
pub enum DataValue {
    Int32(i32),
    InlineString(String),
    Float(f32),
    UInt32(u32),
    Int16(i16),
    Int8(i8),
    OffsetString(String),
    Null,
}

impl DataValue {
    /// Gets the type from a value.
    pub fn ty(&self) -> DataType {
        match self {
            DataValue::Int32(_) => DataType::Int32,
            DataValue::InlineString(_) => DataType::InlineString,
            DataValue::Float(_) => DataType::Float,
            DataValue::UInt32(_) => DataType::UInt32,
            DataValue::Int16(_) => DataType::Int16,
            DataValue::Int8(_) => DataType::Int8,
            DataValue::OffsetString(_) => DataType::OffsetString,
            DataValue::Null => DataType::Null,
        }
    }
}

impl Display for DataValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataValue::Int32(x) => x.fmt(f),
            DataValue::InlineString(x) => x.fmt(f),
            DataValue::Float(x) => x.fmt(f),
            DataValue::UInt32(x) => x.fmt(f),
            DataValue::Int16(x) => x.fmt(f),
            DataValue::Int8(x) => x.fmt(f),
            DataValue::OffsetString(x) => x.fmt(f),
            DataValue::Null => f.write_str("null"),
        }
    }
}

impl Default for DataValue {
    fn default() -> Self {
        DataValue::Null
    }
}

impl From<i32> for DataValue {
    fn from(value: i32) -> Self {
        DataValue::Int32(value)
    }
}

impl From<u32> for DataValue {
    fn from(value: u32) -> Self {
        DataValue::UInt32(value)
    }
}

impl From<f32> for DataValue {
    fn from(value: f32) -> Self {
        DataValue::Float(value)
    }
}

impl From<i16> for DataValue {
    fn from(value: i16) -> Self {
        DataValue::Int16(value)
    }
}

impl From<i8> for DataValue {
    fn from(value: i8) -> Self {
        DataValue::Int8(value)
    }
}
