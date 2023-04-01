use crate::utils::DataBufferWriter;
use crate::{utils, WiiResult};
use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use std::io::{BufRead, Seek, SeekFrom, Write};

pub struct FontSection(Vec<String>);

impl FontSection {
    pub(crate) fn read<T: ByteOrder>(mut reader: impl BufRead + Seek) -> WiiResult<Self> {
        let start = reader.stream_position()? - 8;
        let count = reader.read_u16::<T>()?;
        reader.read_u16::<T>()?;

        let mut result = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let offset = reader.read_u32::<T>()? as u64;
            reader.read_u32::<T>()?;

            let reset = reader.stream_position()?;
            reader.seek(SeekFrom::Start(start + offset))?;
            result.push(utils::read_string(&mut reader)?);

            reader.seek(SeekFrom::Start(reset))?;
        }

        Ok(FontSection(result))
    }

    pub(crate) fn write<T: ByteOrder>(&self, mut writer: impl Write) -> WiiResult<()> {
        let len = self.0.iter().map(|x| x.len() + 1).sum();

        writer.write_u32::<T>(0x74786C31)?;
        writer.write_u32::<T>(12 + self.0.len() as u32 * 8 + len as u32)?;
        writer.write_u16::<T>(self.0.len() as u16)?;
        writer.write_u16::<T>(0)?;

        let mut data = DataBufferWriter::with_capacity(len);
        let start = 12 + self.0.len() as u32 * 8;
        for x in &self.0 {
            writer.write_u32::<T>(start + data.write_str_null(x))?;
            writer.write_u32::<T>(0)?;
        }

        writer.write_all(&data.finish())?;

        Ok(())
    }
}
