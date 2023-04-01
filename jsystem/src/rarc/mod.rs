mod entry;
mod error;
mod header;
mod node;
mod utils;

use bytes::{Bytes, BytesMut};
use entry::RarcEntry;
pub use error::*;
use node::RarcNode;
use radix_trie::{Trie, TrieCommon};
use std::collections::HashMap;
use std::io::{Read, Seek, Write};

type RarcResult<T> = Result<T, RarcError>;

pub struct RarcArchive(Trie<String, (String, Option<Bytes>)>);

impl RarcArchive {
    pub fn read(reader: impl Read + Seek) -> RarcResult<Self> {
        let mut archive = yaz0::Yaz0Archive::new(reader)?;
        let expected_size = archive.expected_size();
        let mut buffer = BytesMut::with_capacity(expected_size);
        buffer.resize(expected_size, 0);
        archive.decompress_into(&mut buffer[..])?;
        let buffer = buffer.freeze();

        let header = match header::RarcHeader::read(&buffer[..0x40]) {
            Ok(h) => h,
            Err(magic) => return Err(RarcError::InvalidMagic(magic)),
        };

        let node_table = node::read_node_table(
            header.node_count,
            &buffer[(header.node_table_offset as usize)..(header.node_table_offset as usize + header.node_count as usize * 0x10)],
        );
        let entry_table = entry::read_entry_table(
            header.entry_count,
            &buffer[(header.entry_table_offset as usize)..(header.entry_table_offset as usize + header.entry_count as usize * 0x14)],
        );

        let mut result = Trie::new();

        utils::process_node(
            &mut result,
            &node_table[0],
            "",
            &node_table,
            &entry_table,
            &buffer[(header.string_table_offset as usize)..(header.string_table_offset as usize + header.string_table_size as usize)],
            &buffer.slice((header.data_offset as usize)..(header.data_offset as usize + header.data_length as usize)),
        );

        Ok(RarcArchive(result))
    }

    pub fn save(&self, writer: &mut impl Write, quality: usize) -> RarcResult<()> {
        let mut result = BytesMut::with_capacity(0x20);
        result.resize(0x40, 0);
        let mut header_buffer = result.split();
        let mut header = header::RarcHeader::default();

        let (entries_by_directory, mut nodes, next_id, data) =
            self.collect_entries_and_nodes(&mut result);

        header.node_count = nodes.len() as u32;
        header.node_table_offset = result.len() as u32 + 0x40;
        header.entry_count = entries_by_directory.values().map(|x| x.len() as u32).sum();
        header.entry_table_offset = header.node_table_offset + (header.node_count * 0x10);
        header.string_table_size = header.node_table_offset - 0x40;
        header.next_free_file_id = next_id;
        header.data_offset = header.entry_table_offset + (header.entry_count * 0x14);
        header.data_length = data.len() as u32;
        header.file_size = header.data_offset + header.data_length;
        header.mram_size = header.data_length;

        let mut string_buffer = result.split();
        result.reserve((header.data_offset - header.node_table_offset) as usize + data.len());
        result.resize((header.data_offset - header.node_table_offset) as usize, 0);

        let mut node_buffer = result.split_to((header.node_count as usize) * 0x10);
        let mut entry_buffer = result.split();
        result.extend_from_slice(&data[..]);

        let mut entry_offset = 0;
        let mut node_buffer_mut = &mut node_buffer[..];
        let mut entry_buffer_mut = &mut entry_buffer[..];
        for entries in entries_by_directory.values() {
            let node = nodes.get_mut(entries[0].data_offset as usize).unwrap();
            node.entry_count = entries.len() as u16;
            node.entry_offset = entry_offset;
            entry_offset += entries.len() as u32;
            node.write(&mut node_buffer_mut);

            for entry in entries {
                entry.write(&mut entry_buffer_mut);
            }
        }

        header.write(&mut header_buffer[..]);

        entry_buffer.unsplit(result);
        node_buffer.unsplit(entry_buffer);
        string_buffer.unsplit(node_buffer);
        header_buffer.unsplit(string_buffer);

        let writer = yaz0::Yaz0Writer::new(writer);
        writer.compress_and_write(
            &header_buffer,
            yaz0::CompressionLevel::Lookahead { quality },
        )?;
        Ok(())
    }

    fn create(&mut self, mut path: &str, data: Option<Bytes>) {
        if path.starts_with('/') {
            path = &path[1..];
        }

        if path.ends_with('/') {
            path = &path[..path.len() - 1];
        }

        if path.contains('/') {
            let mut buffer = String::with_capacity(path.len());
            for segment in path.split('/') {
                if !buffer.is_empty() {
                    buffer.push('/');
                }
                buffer.push_str(segment);

                if self.0.get(&buffer).is_none() {
                    self.0.insert(buffer.clone(), (segment.into(), None));
                }
            }
            if let Some((_, data_ref)) = self.0.get_mut(&buffer) {
                *data_ref = data;
            }
        } else if self.0.get(path).is_none() {
            self.0.insert(path.into(), (path.into(), data));
        }
    }

    pub fn create_directory(&mut self, path: &str) {
        self.create(path, None)
    }

    pub fn create_file(&mut self, path: &str, data: Bytes) {
        self.create(path, Some(data))
    }

    pub fn is_directory(&self, path: &str) -> bool {
        matches!(self.0.get(path), Some((_, None)))
    }

    pub fn get_file(&self, path: &str) -> Option<Bytes> {
        match self.0.get(path) {
            Some((_, Some(data))) => Some(data.clone()),
            _ => None,
        }
    }

    pub fn remove(&mut self, path: &str) -> Option<Option<Bytes>> {
        match self.0.remove(path) {
            Some((_, x)) => Some(x),
            _ => None,
        }
    }

    pub fn mv(&mut self, old: &str, new: &str) {
        if let Some(data) = self.remove(old) {
            self.create(new, data)
        }
    }

    pub fn iter(&self) -> impl std::iter::Iterator<Item = (&str, Option<&Bytes>)> {
        self.0
            .iter()
            .map(|(path, (_, data))| (path.as_ref(), data.into()))
    }

    pub fn directories(&self) -> impl std::iter::Iterator<Item = &str> {
        self.0
            .iter()
            .filter(|(_, (_, data))| data.is_none())
            .map(|(path, _)| path.as_ref())
    }

    pub fn files(&self) -> impl std::iter::Iterator<Item = (&str, &Bytes)> {
        self.0
            .iter()
            .filter(|(_, (_, data))| data.is_some())
            .map(|(path, (_, data))| (path.as_ref(), data.as_ref().unwrap()))
    }

    fn collect_entries_and_nodes(
        &self,
        string_table: &mut BytesMut,
    ) -> (HashMap<&str, Vec<RarcEntry>>, Vec<RarcNode>, u16, Bytes) {
        let mut string_table = utils::StringTableWriter(string_table);
        let dot_offset = string_table.write_bytes(b".") as u16;
        const DOT_HASH: u16 = b'.' as u16;
        let dotdot_offset = string_table.write_bytes(b"..") as u16;
        const DOTDOT_HASH: u16 = b'.' as u16 * 3 + b'.' as u16;

        let mut data_writer = utils::DataWriter::new();

        let mut entries_by_dictonary = HashMap::new();
        let mut nodes = vec![RarcNode {
            tag: *b"ROOT",
            name_offset: string_table.write_bytes(b"ROOT") as u32,
            name_hash: utils::hash_str("ROOT"),
            entry_count: 0,
            entry_offset: 0,
        }];
        let mut id = 0;

        entries_by_dictonary.insert(
            "",
            vec![
                RarcEntry {
                    id: 0xFFFF,
                    name_hash: DOT_HASH,
                    ty: 0x0200,
                    name_offset: dot_offset,
                    data_offset: 0,
                    length: 0,
                },
                RarcEntry {
                    id: 0xFFFF,
                    name_hash: DOTDOT_HASH,
                    ty: 0x0200,
                    name_offset: dotdot_offset,
                    data_offset: 0,
                    length: 0,
                },
            ],
        );

        for (key, (name, data)) in self.0.iter() {
            let parent = if let Some(idx) = key.rfind('/') {
                &key[..idx]
            } else {
                ""
            };

            match data {
                Some(data) => {
                    entries_by_dictonary
                        .get_mut(parent)
                        .unwrap()
                        .push(RarcEntry {
                            id,
                            name_hash: utils::hash_str(name),
                            ty: 0x1100,
                            name_offset: string_table.write_str(name) as u16,
                            data_offset: data_writer.append(data),
                            length: data.len() as u32,
                        });

                    id += 1;
                }
                None => {
                    let node_id = nodes.len();
                    nodes.push(RarcNode {
                        tag: utils::create_tag(name),
                        name_offset: string_table.write_str(name) as u32,
                        name_hash: utils::hash_str(name),
                        entry_count: 0,
                        entry_offset: 0,
                    });

                    entries_by_dictonary
                        .get_mut(parent)
                        .unwrap()
                        .push(RarcEntry {
                            id: 0xFFFF,
                            name_hash: utils::hash_str(name),
                            ty: 0x0200,
                            name_offset: string_table.write_str(name) as u16,
                            data_offset: node_id as u32,
                            length: 0,
                        });

                    entries_by_dictonary.insert(
                        key,
                        vec![
                            RarcEntry {
                                id: 0xFFFF,
                                name_hash: DOT_HASH,
                                ty: 0x0200,
                                name_offset: dot_offset,
                                data_offset: node_id as u32,
                                length: 0,
                            },
                            RarcEntry {
                                id: 0xFFFF,
                                name_hash: DOTDOT_HASH,
                                ty: 0x0200,
                                name_offset: dotdot_offset,
                                data_offset: entries_by_dictonary.get(parent).unwrap()[0]
                                    .data_offset,
                                length: 0,
                            },
                        ],
                    );
                }
            }
        }

        (entries_by_dictonary, nodes, id, data_writer.finish())
    }
}

impl Default for RarcArchive {
    fn default() -> Self {
        RarcArchive(Trie::new())
    }
}
