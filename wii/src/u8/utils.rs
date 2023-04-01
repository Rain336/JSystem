use super::node::U8Node;
use crate::WiiResult;
use radix_trie::Trie;
use std::io::{BufRead, ErrorKind, Seek, SeekFrom};

pub type ArchiveTrie = Trie<String, Option<Box<[u8]>>>;

pub struct StringTableWriter(Vec<u8>);

impl StringTableWriter {
    pub fn new() -> Self {
        StringTableWriter(vec![0])
    }

    pub fn write_str(&mut self, value: &str) -> u32 {
        if value.is_empty() {
            return 0;
        }

        let offset = self.0.len();
        if value.is_ascii() {
            self.0.reserve(value.len() + 1);
            self.0.extend_from_slice(value.as_bytes());
        } else {
            self.0
                .extend(value.chars().filter(char::is_ascii).map(|x| x as u8));
        }
        self.0.push(0);
        offset as u32
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn finish(self) -> Vec<u8> {
        self.0
    }
}

pub fn process_directory<R: BufRead + Seek>(
    node: U8Node,
    target: &mut ArchiveTrie,
    reader: &mut R,
    string_table: u64,
    prefix: &str,
    mut idx: u32,
) -> WiiResult<u32> {
    loop {
        if idx == node.size {
            break;
        }
        idx += 1;

        let current = U8Node::read(reader)?;
        let name = read_string(reader, string_table + current.name_offset as u64, prefix)?;

        if current.ty == 0 {
            let reset = reader.stream_position()?;
            reader.seek(SeekFrom::Start(current.data_offset as u64))?;

            let mut data = vec![0; current.size as usize];
            reader.read_exact(&mut data)?;

            reader.seek(SeekFrom::Start(reset))?;

            target.insert(name, Some(data.into_boxed_slice()));
        } else if current.ty == 1 {
            idx = process_directory(current, target, reader, string_table, &name, idx)?;
            target.insert(name, None);
        }
    }
    Ok(idx)
}

fn read_string<R: BufRead + Seek>(reader: &mut R, offset: u64, prefix: &str) -> WiiResult<String> {
    let mut result = if prefix.is_empty() {
        String::new()
    } else {
        let mut result = String::with_capacity(prefix.len() + 1);
        result.push_str(prefix);
        result.push('/');
        result
    };

    let reset = reader.stream_position()?;
    reader.seek(SeekFrom::Start(offset))?;

    loop {
        let buffer = match reader.fill_buf() {
            Ok(x) => x,
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => return Err(e.into()),
        };
        if let Some(idx) = buffer.iter().position(|x| *x == 0) {
            let buffer = &buffer[..idx];
            result.push_str(std::str::from_utf8(buffer)?);
            reader.consume(idx);
            break;
        }

        result.push_str(std::str::from_utf8(buffer)?);
        let len = buffer.len();
        reader.consume(len);
    }

    reader.seek(SeekFrom::Start(reset))?;
    Ok(result)
}
