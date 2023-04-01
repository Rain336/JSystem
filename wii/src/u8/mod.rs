mod header;
mod node;
mod utils;

use crate::utils::DataBufferWriter;
use crate::{FileFormat, WiiResult};
use header::U8Header;
use node::U8Node;
use radix_trie::{Trie, TrieCommon};
use std::io::{BufRead, Seek, SeekFrom, Write};
use utils::StringTableWriter;

pub struct U8Archive(utils::ArchiveTrie);

impl U8Archive {
    fn create(&mut self, mut path: &str, data: Option<Box<[u8]>>) {
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
                    self.0.insert(buffer.clone(), None);
                }
            }
            if let Some(data_ref) = self.0.get_mut(&buffer) {
                *data_ref = data;
            }
        } else if self.0.get(path).is_none() {
            self.0.insert(path.into(), data);
        }
    }

    pub fn create_directory(&mut self, path: &str) {
        self.create(path, None)
    }

    pub fn create_file(&mut self, path: &str, data: Box<[u8]>) {
        self.create(path, Some(data))
    }

    #[must_use]
    pub fn is_directory(&self, path: &str) -> bool {
        self.0.get(path) == None
    }

    #[must_use]
    pub fn get_file(&self, path: &str) -> Option<&[u8]> {
        match self.0.get(path) {
            Some(Some(data)) => Some(data),
            _ => None,
        }
    }

    pub fn remove(&mut self, path: &str) -> Option<Option<Box<[u8]>>> {
        self.0.remove(path)
    }

    pub fn mv(&mut self, old: &str, new: &str) {
        if let Some(data) = self.remove(old) {
            self.create(new, data)
        }
    }

    pub fn iter(&self) -> impl std::iter::Iterator<Item = (&str, Option<&[u8]>)> {
        self.0.iter().map(|(path, x)| (path.as_ref(), x.as_deref()))
    }

    pub fn directories(&self) -> impl std::iter::Iterator<Item = &str> {
        self.0
            .iter()
            .filter(|(_, x)| x.is_none())
            .map(|(path, _)| path.as_ref())
    }

    pub fn files(&self) -> impl std::iter::Iterator<Item = (&str, &[u8])> {
        self.0
            .iter()
            .filter_map(|(path, x)| x.as_ref().map(|x| (path.as_ref(), x.as_ref())))
    }
}

impl FileFormat<U8Archive> for U8Archive {
    fn read(reader: &mut (impl BufRead + Seek)) -> WiiResult<Self> {
        let header = U8Header::read(reader)?;

        reader.seek(SeekFrom::Start(header.root_node as u64))?;
        let root = U8Node::read(reader)?;

        let string_table = header.root_node as u64 + root.size as u64 * 12;
        let mut result = Trie::new();
        utils::process_directory(root, &mut result, reader, string_table, "", 1)?;

        Ok(U8Archive(result))
    }

    fn write(&self, writer: &mut impl Write) -> WiiResult<()> {
        let mut nodes: Vec<U8Node> = Vec::with_capacity(self.0.iter().count());
        let mut string_table = StringTableWriter::new();
        let mut data_buffer = DataBufferWriter::with_capacity(
            self.0
                .iter()
                .map(|(_, data)| data.as_deref().map(|x| x.len()).unwrap_or_default())
                .sum(),
        );

        nodes.push(U8Node {
            ty: 0x01,
            name_offset: 0,
            data_offset: 0,
            size: 0,
        });

        let mut stack = vec![("", 0)];
        for (path, data) in self.0.iter() {
            let name_offset = string_table.write_str(match path.rfind('/') {
                Some(idx) => &path[(idx + 1)..],
                None => "",
            });

            while !path.starts_with(stack.last().unwrap().0) {
                nodes.get_mut(stack.pop().unwrap().1).unwrap().size = nodes.len() as u32;
            }

            nodes.push(match data {
                Some(data) => U8Node {
                    ty: 0x00,
                    name_offset,
                    data_offset: data_buffer.write_bytes(data),
                    size: data.len() as u32,
                },
                None => {
                    stack.push((path, nodes.len()));
                    U8Node {
                        ty: 0x01,
                        name_offset,
                        data_offset: 0,
                        size: 0,
                    }
                }
            });
        }

        nodes.get_mut(0).unwrap().size = nodes.len() as u32;

        let header = U8Header {
            root_node: 32,
            header_size: (nodes.len() * 12 + string_table.len()) as u32,
            data_offset: (nodes.len() * 12 + string_table.len() + 12) as u32,
        };

        header.write(writer)?;
        for node in nodes {
            node.write(writer)?;
        }
        writer.write_all(&string_table.finish())?;
        writer.write_all(&data_buffer.finish())?;

        Ok(())
    }
}

impl Default for U8Archive {
    fn default() -> Self {
        U8Archive(Trie::new())
    }
}
