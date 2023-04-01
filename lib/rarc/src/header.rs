use crate::{RarcError, Result};
use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Seek, SeekFrom, Write};

pub struct RarcHeader {
    pub size: u32,
    pub data_offset: u32,
    pub data_length: u32,
    pub mram: u32,
    pub aram: u32,
    pub dvd: u32,

    pub directory_nodes: u32,
    pub directory_offset: u32,
    pub file_nodes: u32,
    pub file_offset: u32,
    pub string_size: u32,
    pub string_offset: u32,
    pub next_file: u16,
    pub keep_synced: bool,
}

const RARC_MAGIC: &[u8; 4] = b"RARC";

impl RarcHeader {
    pub fn read<T: ByteOrder>(mut reader: impl Read + Seek) -> Result<Self> {
        let mut magic = [0; 4];
        reader.read_exact(&mut magic)?;

        if &magic == RARC_MAGIC {
            return Err(RarcError::InvaildMagic);
        }

        let size = reader.read_u32::<T>()?;
        let data_header = reader.read_u32::<T>()?;
        let data_offset = reader.read_u32::<T>()?;
        let data_length = reader.read_u32::<T>()?;
        let mram = reader.read_u32::<T>()?;
        let aram = reader.read_u32::<T>()?;
        let dvd = reader.read_u32::<T>()?;

        if data_header != 0x20 {
            reader.seek(SeekFrom::Start(data_header as u64))?;
        }

        Ok(RarcHeader {
            size,
            data_offset,
            data_length,
            mram,
            aram,
            dvd,

            directory_nodes: reader.read_u32::<T>()?,
            directory_offset: reader.read_u32::<T>()?,
            file_nodes: reader.read_u32::<T>()?,
            file_offset: reader.read_u32::<T>()?,
            string_size: reader.read_u32::<T>()?,
            string_offset: reader.read_u32::<T>()?,
            next_file: reader.read_u16::<T>()?,
            keep_synced: reader.read_u8()? == 1,
        })
    }

    pub fn write<T: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        writer.write_all(RARC_MAGIC)?;
        writer.write_u32::<T>(self.size)?;
        writer.write_u32::<T>(0x20)?;
        writer.write_u32::<T>(self.data_offset)?;
        writer.write_u32::<T>(self.data_length)?;
        writer.write_u32::<T>(self.mram)?;
        writer.write_u32::<T>(self.aram)?;
        writer.write_u32::<T>(self.dvd)?;

        writer.write_u32::<T>(self.directory_nodes)?;
        writer.write_u32::<T>(self.directory_offset)?;
        writer.write_u32::<T>(self.file_nodes)?;
        writer.write_u32::<T>(self.file_offset)?;
        writer.write_u32::<T>(self.string_size)?;
        writer.write_u32::<T>(self.string_offset)?;
        writer.write_u16::<T>(self.next_file)?;
        writer.write_u8(u8::from(self.keep_synced))?;
        writer.write_all(&[0; 5])?;
        Ok(())
    }
}
