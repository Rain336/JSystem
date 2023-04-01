use crate::{BcsvError, DataType, DataValue, Result};
use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use std::io::{BufRead, Read, Seek, SeekFrom, Write};

/// The definition of a column, as written in the BCSV.
pub struct ColumnDefinition {
    /// The name hash of the column.
    pub name: u32,

    /// A bitmask that gets applied to every integer value.
    pub bitmask: u32,

    /// The offset into an entry to the start of this column.
    pub offset: u16,

    /// A shift that gets applied to every integer value.
    pub shift: u8,

    /// The type of this column.
    pub ty: DataType,
}

impl ColumnDefinition {
    pub(crate) fn read<T: ByteOrder>(mut reader: impl Read) -> Result<Self> {
        Ok(ColumnDefinition {
            name: reader.read_u32::<T>()?,
            bitmask: reader.read_u32::<T>()?,
            offset: reader.read_u16::<T>()?,
            shift: reader.read_u8()?,
            ty: reader.read_u8()?.try_into()?,
        })
    }

    pub(crate) fn write<T: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        writer.write_u32::<T>(self.name)?;
        writer.write_u32::<T>(self.bitmask)?;
        writer.write_u16::<T>(self.offset)?;
        writer.write_u8(self.shift)?;
        writer.write_u8(self.ty as u8)?;
        Ok(())
    }

    pub(crate) fn read_entry<T: ByteOrder>(
        &self,
        mut reader: impl BufRead + Seek,
        string_offset: u64,
    ) -> Result<DataValue> {
        match self.ty {
            DataType::Int32 => {
                let value = reader.read_i32::<T>()?;
                Ok(DataValue::Int32(
                    (value & self.bitmask as i32) >> self.shift,
                ))
            }
            DataType::InlineString => {
                let mut buffer = [0; 32];
                reader.read_exact(&mut buffer)?;
                let (text, _) = encoding_rs::SHIFT_JIS.decode_without_bom_handling(&buffer);
                Ok(DataValue::InlineString(text.into_owned()))
            }
            DataType::Float => Ok(reader.read_f32::<T>()?.into()),
            DataType::UInt32 => {
                let value = reader.read_u32::<T>()?;
                Ok(DataValue::UInt32((value & self.bitmask) >> self.shift))
            }
            DataType::Int16 => {
                let value = reader.read_i16::<T>()?;
                Ok(DataValue::Int16(
                    (value & self.bitmask as i16) >> self.shift,
                ))
            }
            DataType::Int8 => {
                let value = reader.read_i8()?;
                Ok(DataValue::Int8((value & self.bitmask as i8) >> self.shift))
            }
            DataType::OffsetString => {
                let offset = reader.read_u32::<T>()? as u64;

                let position = reader.stream_position()?;
                reader.seek(SeekFrom::Start(string_offset + offset))?;

                let mut result = Vec::new();
                reader.read_until(0, &mut result)?;

                reader.seek(SeekFrom::Start(position))?;

                let (text, _) = encoding_rs::SHIFT_JIS.decode_without_bom_handling(&result);
                Ok(DataValue::OffsetString(text.into_owned()))
            }
            DataType::Null => Ok(DataValue::Null),
        }
    }

    pub(crate) fn write_entry<T: ByteOrder>(
        &self,
        value: &DataValue,
        mut writer: impl Write,
        pool: &mut Vec<u8>,
    ) -> Result<()> {
        match value {
            DataValue::Int32(x) => {
                let value = (*x << self.shift) & self.bitmask as i32;
                writer.write_i32::<T>(value)?
            }
            DataValue::InlineString(_) => return Err(BcsvError::InlineStringUnsupported),
            DataValue::Float(x) => writer.write_f32::<T>(*x)?,
            DataValue::UInt32(x) => {
                let value = (*x << self.shift) & self.bitmask;
                writer.write_u32::<T>(value)?
            }
            DataValue::Int16(x) => {
                let value = (*x << self.shift) & self.bitmask as i16;
                writer.write_i16::<T>(value)?
            }
            DataValue::Int8(x) => {
                let value = (*x << self.shift) & self.bitmask as i8;
                writer.write_i8(value)?
            }
            DataValue::OffsetString(x) => {
                let (text, _, _) = encoding_rs::SHIFT_JIS.encode(x);

                let offset = pool.len();
                pool.extend_from_slice(&text);

                writer.write_u32::<T>(offset as u32)?;
            }
            DataValue::Null => {}
        }
        Ok(())
    }
}
