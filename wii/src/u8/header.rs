use crate::{WiiError, WiiResult};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Seek, SeekFrom, Write};

pub struct U8Header {
    pub root_node: u32,
    pub header_size: u32,
    pub data_offset: u32,
}

impl U8Header {
    pub fn read<R: Read + Seek>(reader: &mut R) -> WiiResult<Self> {
        if reader.read_u32::<BigEndian>()? != 0x55AA382D {
            return Err(WiiError::InvalidMagic);
        }

        let root_node = reader.read_u32::<BigEndian>()?;
        let header_size = reader.read_u32::<BigEndian>()?;
        let data_offset = reader.read_u32::<BigEndian>()?;

        reader.seek(SeekFrom::Current(16))?;

        Ok(U8Header {
            root_node,
            header_size,
            data_offset,
        })
    }

    pub fn write(&self, writer: &mut impl Write) -> WiiResult<()> {
        writer.write_u32::<BigEndian>(0x55AA382D)?;
        writer.write_u32::<BigEndian>(self.root_node)?;
        writer.write_u32::<BigEndian>(self.header_size)?;
        writer.write_u32::<BigEndian>(self.data_offset)?;
        writer.write_all(&[0; 16])?;
        Ok(())
    }
}
