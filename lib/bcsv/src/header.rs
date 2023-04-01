use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Result, Write};

pub struct BcsvHeader {
    pub row_count: u32,
    pub column_count: u32,
    pub data_offset: u32,
    pub row_size: u32,
}

impl BcsvHeader {
    pub fn read<T: ByteOrder>(mut reader: impl Read) -> Result<Self> {
        let mut integers = [0; 4];
        reader.read_u32_into::<T>(&mut integers)?;
        Ok(BcsvHeader {
            row_count: integers[0],
            column_count: integers[1],
            data_offset: integers[2],
            row_size: integers[3],
        })
    }

    pub fn write<T: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        writer.write_u32::<T>(self.row_count)?;
        writer.write_u32::<T>(self.column_count)?;
        writer.write_u32::<T>(self.data_offset)?;
        writer.write_u32::<T>(self.row_size)?;
        Ok(())
    }
}
