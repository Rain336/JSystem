use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Write};

#[derive(Debug)]
pub struct U8Node {
    pub ty: u8,
    pub name_offset: u32,
    pub data_offset: u32,
    pub size: u32,
}

impl U8Node {
    pub fn read(reader: &mut impl Read) -> io::Result<Self> {
        let ty = reader.read_u8()?;
        let name_offset = reader.read_u24::<BigEndian>()?;
        let data_offset = reader.read_u32::<BigEndian>()?;
        let size = reader.read_u32::<BigEndian>()?;

        Ok(U8Node {
            ty,
            name_offset,
            data_offset,
            size,
        })
    }

    pub fn write(&self, writer: &mut impl Write) -> io::Result<()> {
        writer.write_u8(self.ty)?;
        writer.write_u24::<BigEndian>(self.name_offset)?;
        writer.write_u32::<BigEndian>(self.data_offset)?;
        writer.write_u32::<BigEndian>(self.size)?;
        Ok(())
    }
}
