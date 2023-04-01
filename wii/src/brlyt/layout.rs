use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};

use crate::WiiResult;
use std::io::{BufRead, Seek, Write};

pub struct LayoutSection {
    pub centered: bool,
    pub width: f32,
    pub height: f32,
}

impl LayoutSection {
    pub(crate) fn read<T: ByteOrder>(reader: &mut (impl BufRead + Seek)) -> WiiResult<Self> {
        let centered = reader.read_u8()? != 0;
        let width = reader.read_f32::<T>()?;
        let height = reader.read_f32::<T>()?;

        Ok(LayoutSection {
            centered,
            width,
            height,
        })
    }

    pub(crate) fn write<T: ByteOrder>(&self, writer: &mut impl Write) -> WiiResult<()> {
        writer.write_u32::<T>(0x6C797431)?;
        writer.write_u32::<T>(0x14)?;
        writer.write_u8(if self.centered { 0x01 } else { 0x00 })?;
        writer.write_all(&[0; 3])?;
        writer.write_f32::<T>(self.width)?;
        writer.write_f32::<T>(self.height)?;
        Ok(())
    }
}
