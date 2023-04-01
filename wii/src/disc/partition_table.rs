use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Seek, SeekFrom, Write};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PartitionType {
    Data,
    Update,
    ChannelInstaller,
    Other(u32),
}

impl From<u32> for PartitionType {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Data,
            1 => Self::Update,
            2 => Self::ChannelInstaller,
            x => Self::Other(x),
        }
    }
}

impl From<PartitionType> for u32 {
    fn from(val: PartitionType) -> Self {
        match val {
            PartitionType::Data => 0,
            PartitionType::Update => 1,
            PartitionType::ChannelInstaller => 2,
            PartitionType::Other(x) => x,
        }
    }
}

pub struct PartitionTableEntry {
    offset: u32,
    ty: PartitionType,
}

impl PartitionTableEntry {
    pub fn read(mut reader: impl Read) -> io::Result<Self> {
        let offset = reader.read_u32::<BigEndian>()? << 2;
        let ty = reader.read_u32::<BigEndian>()?.into();
        Ok(PartitionTableEntry { offset, ty })
    }

    pub fn save(&self, mut writer: impl Write) -> io::Result<()> {
        writer.write_u32::<BigEndian>(self.offset >> 2)?;
        writer.write_u32::<BigEndian>(self.ty.into())?;
        Ok(())
    }
}

pub fn read_partition_table(
    mut reader: impl Read + Seek,
) -> io::Result<Vec<PartitionTableEntry>> {
    let position = reader.seek(SeekFrom::Start(0x40000))?;
    let c1 = reader.read_u32::<BigEndian>()?;
    let o1 = reader.read_u32::<BigEndian>()? << 2;
    let c2 = reader.read_u32::<BigEndian>()?;
    let o2 = reader.read_u32::<BigEndian>()? << 2;
    let c3 = reader.read_u32::<BigEndian>()?;
    let o3 = reader.read_u32::<BigEndian>()? << 2;
    let c4 = reader.read_u32::<BigEndian>()?;
    let o4 = reader.read_u32::<BigEndian>()? << 2;

    let mut result = Vec::with_capacity(c1 as usize + c2 as usize + c3 as usize + c4 as usize);

    if c1 > 0 {
        reader.seek(SeekFrom::Start(o1 as u64))?;
        for _ in 0..c1 {
            result.push(PartitionTableEntry::read(&mut reader)?);
        }
    }

    if c2 > 0 {
        reader.seek(SeekFrom::Start(o2 as u64))?;
        for _ in 0..c2 {
            result.push(PartitionTableEntry::read(&mut reader)?);
        }
    }

    if c3 > 0 {
        reader.seek(SeekFrom::Start(o3 as u64))?;
        for _ in 0..c3 {
            result.push(PartitionTableEntry::read(&mut reader)?);
        }
    }

    if c4 > 0 {
        reader.seek(SeekFrom::Start(o4 as u64))?;
        for _ in 0..c4 {
            result.push(PartitionTableEntry::read(&mut reader)?);
        }
    }

    reader.seek(SeekFrom::Start(position))?;

    Ok(result)
}
