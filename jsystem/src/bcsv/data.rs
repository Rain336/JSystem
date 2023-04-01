use super::BcsvError;
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DataType {
    Int32 = 0,
    InlineString = 1,
    Float = 2,
    UInt32 = 3,
    Int16 = 4,
    Byte = 5,
    String = 6,
    Null = 7,
}

impl DataType {
    pub fn size(&self) -> usize {
        match self {
            DataType::Int32 => 4,
            DataType::InlineString => 32,
            DataType::Float => 4,
            DataType::UInt32 => 4,
            DataType::Int16 => 2,
            DataType::Byte => 1,
            DataType::String => 4,
            DataType::Null => 0,
        }
    }
}

impl TryFrom<u8> for DataType {
    type Error = super::BcsvError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(DataType::Int32),
            1 => Ok(DataType::InlineString),
            2 => Ok(DataType::Float),
            3 => Ok(DataType::UInt32),
            4 => Ok(DataType::Int16),
            5 => Ok(DataType::Byte),
            6 => Ok(DataType::String),
            7 => Ok(DataType::Null),
            id => Err(BcsvError::InvalidDataType(id)),
        }
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataType::Int32 => f.write_str("Int32"),
            DataType::InlineString => f.write_str("InlineString"),
            DataType::Float => f.write_str("Float"),
            DataType::UInt32 => f.write_str("UInt32"),
            DataType::Int16 => f.write_str("Int16"),
            DataType::Byte => f.write_str("Byte"),
            DataType::String => f.write_str("String"),
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
            "Byte" => Ok(DataType::Byte),
            "String" => Ok(DataType::String),
            "Null" => Ok(DataType::Null),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum DataValue {
    Int32(i32),
    InlineString(String),
    Float(f32),
    UInt32(u32),
    Int16(i16),
    Byte(u8),
    String(String),
    Null,
}

impl DataValue {
    pub fn to_type(&self) -> DataType {
        use DataValue::*;
        match self {
            Int32(_) => DataType::Int32,
            InlineString(_) => DataType::InlineString,
            Float(_) => DataType::Float,
            UInt32(_) => DataType::UInt32,
            Int16(_) => DataType::Int16,
            Byte(_) => DataType::Byte,
            String(_) => DataType::String,
            Null => DataType::Null,
        }
    }
}

impl fmt::Display for DataValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataValue::Int32(x) => x.fmt(f),
            DataValue::InlineString(x) => x.fmt(f),
            DataValue::Float(x) => x.fmt(f),
            DataValue::UInt32(x) => x.fmt(f),
            DataValue::Int16(x) => x.fmt(f),
            DataValue::Byte(x) => x.fmt(f),
            DataValue::String(x) => x.fmt(f),
            DataValue::Null => f.write_str("null"),
        }
    }
}

impl From<i32> for DataValue {
    fn from(value: i32) -> Self {
        Self::Int32(value)
    }
}

impl From<f32> for DataValue {
    fn from(value: f32) -> Self {
        Self::Float(value)
    }
}

impl From<u32> for DataValue {
    fn from(value: u32) -> Self {
        Self::UInt32(value)
    }
}

impl From<i16> for DataValue {
    fn from(value: i16) -> Self {
        Self::Int16(value)
    }
}

impl From<u8> for DataValue {
    fn from(value: u8) -> Self {
        Self::Byte(value)
    }
}

impl From<String> for DataValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}
