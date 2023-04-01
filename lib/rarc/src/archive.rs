use crate::header::RarcHeader;
use crate::node::{self, DirectoryNode, FileAttributes, FileNode};
use crate::string_table::StringTable;
use crate::{RarcError, Result};
use byteorder::ByteOrder;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, Write};
use std::path::Path;

enum IndexEntry {
    Directory {
        files: u16,
    },
    File {
        offset: u32,
        size: u32,
        attributes: FileAttributes,
    },
}

pub struct Archive<F: Read + Seek> {
    index: BTreeMap<String, IndexEntry>,
    reader: F,
}

impl<F: Read + Seek> Archive<F> {
    pub fn read<T: ByteOrder>(mut reader: F) -> Result<Self> {
        let header = RarcHeader::read::<T>(&mut reader)?;

        if header.directory_nodes == 0 {
            return Ok(Archive {
                index: BTreeMap::new(),
                reader,
            });
        }

        let mut index = BTreeMap::new();
        let table = StringTable::read(&mut reader, &header)?;

        let directories = node::read_directory_nodes(&mut reader, &header)?;
        let files = node::read_file_nodes(&mut reader, &header)?;

        if &directories[0].tag != b"ROOT" {
            return Err(RarcError::FirstDirectoryNotRoot);
        }

        read_directory::<T>(0, "/".into(), &directories, &files, &table, &mut index)?;

        Ok(Archive { index, reader })
    }

    pub fn write<T: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        todo!()
    }

    pub fn save<T: ByteOrder>(&self, path: impl AsRef<Path>) -> Result<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        self.write::<T>(writer)
    }

    pub fn open_file(&mut self, path: &str) -> Option<ArchivedFile<'_>> {
        match self.index.get_mut(path)? {
            IndexEntry::Directory { .. } => None,
            entry => Some(ArchivedFile { entry }),
        }
    }
}

impl Archive<BufReader<File>> {
    pub fn open<T: ByteOrder>(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Archive::read::<T>(reader)
    }
}

pub struct ArchivedFile<'a> {
    entry: &'a mut IndexEntry,
}

impl<'a> ArchivedFile<'a> {
    pub fn attributes(&self) -> FileAttributes {
        if let IndexEntry::File { attributes, .. } = &self.entry {
            *attributes
        } else {
            panic!("Expected File entry");
        }
    }

    pub fn set_attributes(&mut self, value: FileAttributes) {
        if let IndexEntry::File { attributes, .. } = self.entry {
            *attributes = value
        } else {
            panic!("Expected File entry");
        }
    }
}

fn read_directory<T: ByteOrder>(
    directory_index: usize,
    directory_path: String,
    directories: &[DirectoryNode],
    files: &[FileNode],
    table: &StringTable,
    index: &mut BTreeMap<String, IndexEntry>,
) -> Result<()> {
    let directory = &directories[directory_index];
    let file_start = T::read_u32(bytemuck::bytes_of(&directory.file_offset));
    let file_count = T::read_u16(bytemuck::bytes_of(&directory.file_count));

    for i in (file_start as usize)..(file_start as usize + file_count as usize) {
        let file = files.get(i).ok_or(RarcError::MissingFile { index: i })?;

        let name = match file.name::<T>(table) {
            Some(x) => x,
            None => continue,
        };

        if name == "." || name == ".." {
            continue;
        }

        let mut path = String::with_capacity(directory_path.len() + name.len() + 1);
        path.push_str(&directory_path);
        path.push('/');
        path.push_str(name);

        if file.index == u16::MAX {
            let directory_index = T::read_u32(bytemuck::bytes_of(&file.offset_or_index)) as usize;
            read_directory::<T>(directory_index, path, directories, files, table, index)?;
        } else {
            let offset = T::read_u32(bytemuck::bytes_of(&file.offset_or_index));
            let size = T::read_u32(bytemuck::bytes_of(&file.size));
            index.insert(
                path,
                IndexEntry::File {
                    offset,
                    size,
                    attributes: FileAttributes::from_bits_truncate(file.attributes),
                },
            );
        }
    }

    index.insert(directory_path, IndexEntry::Directory { files: file_count });

    Ok(())
}
