use crate::utils::DataBufferWriter;
use crate::{utils, WiiError, WiiResult};
use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use std::io::{BufRead, Seek, SeekFrom, Write};

pub enum UserDataEntry {
    String { name: String, value: String },
    Int { name: String, value: Vec<i32> },
    Float { name: String, value: Vec<f32> },
}

impl UserDataEntry {
    pub(crate) fn read<T: ByteOrder>(mut reader: impl BufRead + Seek) -> WiiResult<Self> {
        let start = reader.stream_position()?;

        let name_offset = reader.read_u32::<T>()?;
        let data_offset = reader.read_u32::<T>()?;
        let count = reader.read_u16::<T>()?;
        let ty = reader.read_u8()?;
        reader.read_u8()?;

        reader.seek(SeekFrom::Start(start + name_offset as u64))?;
        let name = utils::read_string(&mut reader)?;

        reader.seek(SeekFrom::Start(start + data_offset as u64))?;

        let result = match ty {
            0 => UserDataEntry::String {
                name,
                value: utils::read_string_exact(&mut reader, count as usize)?,
            },
            1 => {
                let mut value = Vec::with_capacity(count as usize);
                for _ in 0..count {
                    value.push(reader.read_i32::<T>()?);
                }
                UserDataEntry::Int { name, value }
            }
            2 => {
                let mut value = Vec::with_capacity(count as usize);
                for _ in 0..count {
                    value.push(reader.read_f32::<T>()?);
                }
                UserDataEntry::Float { name, value }
            }
            _ => return Err(WiiError::InvalidType),
        };

        reader.seek(SeekFrom::Start(start + 12))?;

        Ok(result)
    }

    pub(crate) fn write<T: ByteOrder, W: Write>(
        &self,
        writer: &mut W,
        data: &mut DataBufferWriter,
        end: u32,
    ) -> WiiResult<()> {
        let name_offset = data.write_str_null(self.name()) + end;
        let (data_offset, count, ty) = match self {
            UserDataEntry::String { value, .. } => {
                (data.write_bytes(value.as_bytes()) + end, value.len(), 0u8)
            }
            UserDataEntry::Int { value, .. } => {
                (data.write_i32_slice::<T>(value) + end, value.len(), 1u8)
            }
            UserDataEntry::Float { value, .. } => {
                (data.write_f32_slice::<T>(value) + end, value.len(), 2u8)
            }
        };

        writer.write_u32::<T>(name_offset)?;
        writer.write_u32::<T>(data_offset)?;
        writer.write_u16::<T>(count as u16)?;
        writer.write_u8(ty)?;
        writer.write_u8(0)?;

        Ok(())
    }

    pub(crate) fn size(&self) -> usize {
        match self {
            UserDataEntry::String { name, value } => 14 + name.len() + 1 + value.len(),
            UserDataEntry::Int { name, value } => 14 + name.len() + 1 + value.len() * 4,
            UserDataEntry::Float { name, value } => 14 + name.len() + 1 + value.len() * 4,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            UserDataEntry::String { name, .. } => name,
            UserDataEntry::Int { name, .. } => name,
            UserDataEntry::Float { name, .. } => name,
        }
    }

    pub fn name_mut(&mut self) -> &mut String {
        match self {
            UserDataEntry::String { name, .. } => name,
            UserDataEntry::Int { name, .. } => name,
            UserDataEntry::Float { name, .. } => name,
        }
    }
}

pub struct UserDataSection(Vec<UserDataEntry>);

impl UserDataSection {
    pub(crate) fn read<T: ByteOrder>(mut reader: impl BufRead + Seek, size: u32) -> WiiResult<Self> {
        let count = reader.read_u16::<T>()?;
        reader.read_u16::<T>()?;

        let mut result = Vec::with_capacity(count as usize);
        for _ in 0..count {
            result.push(UserDataEntry::read::<T>(&mut reader)?);
        }

        reader.seek(SeekFrom::Current(size as i64 - 12 - count as i64 * 12))?;

        Ok(UserDataSection(result))
    }

    pub(crate) fn write<T: ByteOrder, W: Write>(&self, writer: &mut W) -> WiiResult<()> {
        let size: usize = self.0.iter().map(|x| x.size()).sum();

        writer.write_u32::<T>(0x75736431)?;
        writer.write_u32::<T>(size as u32 + 12)?;
        writer.write_u16::<T>(self.0.len() as u16)?;
        writer.write_u16::<T>(0)?;

        let mut end = self.0.len() as u32 * 12;
        let mut data = DataBufferWriter::with_capacity(size - end as usize);
        for entry in &self.0 {
            entry.write::<T, W>(writer, &mut data, end)?;
            end -= 14;
        }

        writer.write_all(&data.finish())?;

        Ok(())
    }
}
