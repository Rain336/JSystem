use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Write};

pub struct NKitHeader {
    version: [u8; 4],
    original_crc: u32,
    nkit_crc: u32,
    source_image_length: u32,
    forced_junk_id: u32,
    update_partition_crc: u32,
}

impl NKitHeader {
    pub fn read(reader: &mut impl Read) -> io::Result<Option<Self>> {
        let mut magic = [0; 4];
        reader.read_exact(&mut magic)?;
        if &magic != b"NKIT" {
            return Ok(None);
        }

        let mut version = [0; 4];
        reader.read_exact(&mut version)?;
        let original_crc = reader.read_u32::<BigEndian>()?;
        let nkit_crc = reader.read_u32::<BigEndian>()?;
        let source_image_length = reader.read_u32::<BigEndian>()?;
        let forced_junk_id = reader.read_u32::<BigEndian>()?;
        let update_partition_crc = reader.read_u32::<BigEndian>()?;

        Ok(Some(NKitHeader {
            version,
            original_crc,
            nkit_crc,
            source_image_length,
            forced_junk_id,
            update_partition_crc,
        }))
    }

    pub fn write(&self, writer: &mut impl Write) -> io::Result<()> {
        writer.write_all(b"NKIT")?;
        writer.write_all(&self.version)?;
        writer.write_u32::<BigEndian>(self.original_crc)?;
        writer.write_u32::<BigEndian>(self.nkit_crc)?;
        writer.write_u32::<BigEndian>(self.source_image_length)?;
        writer.write_u32::<BigEndian>(self.forced_junk_id)?;
        writer.write_u32::<BigEndian>(self.update_partition_crc)?;
        Ok(())
    }
}
