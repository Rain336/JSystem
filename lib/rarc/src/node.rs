use crate::header::RarcHeader;
use crate::string_table::StringTable;
use crate::{RarcError, Result};
use bitflags::bitflags;
use bytemuck::{Pod, Zeroable};
use byteorder::ByteOrder;
use std::io::{Read, Seek, SeekFrom};

#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct DirectoryNode {
    pub tag: [u8; 4],
    pub name_offset: u32,
    pub name_hash: u16,
    pub file_count: u16,
    pub file_offset: u32,
}

pub fn read_directory_nodes(
    mut reader: impl Read + Seek,
    header: &RarcHeader,
) -> Result<Vec<DirectoryNode>> {
    reader.seek(SeekFrom::Start(header.directory_offset as u64 + 0x20))?;

    let mut result = vec![0; header.directory_nodes as usize * 0x10];
    reader.read_exact(&mut result)?;

    bytemuck::allocation::try_cast_vec(result).map_err(|(e, _)| RarcError::PodCastError(e))
}

bitflags! {
    pub struct FileAttributes: u8 {
        const FILE = 0x01;
        const DIRECTORY = 0x02;
        const COMPRESSED = 0x04;
        const PRELOAD_TO_MRAM = 0x10;
        const PRELOAD_TO_ARAM = 0x20;
        const LOAD_FROM_DVD = 0x40;
        const YAZ0_COMPRESSED = 0x80;
    }
}

#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct FileNode {
    pub index: u16,
    pub name_hash: u16,
    pub attributes: u8,
    pub padding: u8,
    pub name_offset: u16,
    pub offset_or_index: u32,
    pub size: u32,
}

impl FileNode {
    pub fn name<'a, T: ByteOrder>(&self, table: &'a StringTable) -> Option<&'a str> {
        let offset = T::read_u16(bytemuck::bytes_of(&self.name_offset));
        table.string_at(offset as usize)
    }
}

pub fn read_file_nodes(mut reader: impl Read + Seek, header: &RarcHeader) -> Result<Vec<FileNode>> {
    reader.seek(SeekFrom::Start(header.file_offset as u64 + 0x20))?;

    let mut result = vec![0; header.file_nodes as usize * 0x10];
    reader.read_exact(&mut result)?;

    bytemuck::allocation::try_cast_vec(result).map_err(|(e, _)| RarcError::PodCastError(e))
}
