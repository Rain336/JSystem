use std::num::Wrapping;

use bytes::{BufMut, Bytes, BytesMut};
use radix_trie::Trie;

struct StringTableReader<'a>(&'a [u8]);

impl StringTableReader<'_> {
    fn read_str(&self, offset: usize) -> String {
        let start = &self.0[offset..];
        let mut length = 0;
        while start[length] != 0 {
            length += 1;
        }
        let (result, _) = encoding_rs::SHIFT_JIS.decode_without_bom_handling(&start[..length]);
        result.into_owned()
    }

    fn is_skipable(&self, offset: usize) -> bool {
        if self.0[offset] != b'.' {
            return false;
        }

        if self.0[offset + 1] == b'.' {
            self.0[offset + 2] == 0
        } else {
            self.0[offset + 1] == 0
        }
    }
}

pub struct StringTableWriter<'a>(pub &'a mut BytesMut);

impl StringTableWriter<'_> {
    pub fn write_str(&mut self, value: &str) -> usize {
        let (encoded, _, _) = encoding_rs::SHIFT_JIS.encode(value);
        self.write_bytes(&encoded)
    }

    pub fn write_bytes(&mut self, value: &[u8]) -> usize {
        let position = self.0.len();
        self.0.extend_from_slice(value);
        self.0.put_u8(0);
        position
    }
}

pub struct DataWriter(BytesMut);

impl DataWriter {
    pub fn new() -> Self {
        DataWriter(BytesMut::new())
    }

    pub fn append(&mut self, data: &[u8]) -> u32 {
        let result = self.0.len() as u32;
        self.0.extend_from_slice(data);
        result
    }

    pub fn finish(self) -> Bytes {
        self.0.freeze()
    }
}

pub fn process_node(
    target: &mut Trie<String, (String, Option<Bytes>)>,
    node: &super::RarcNode,
    prefix: &str,
    node_table: &[super::RarcNode],
    entry_table: &[super::RarcEntry],
    string_table: &[u8],
    data: &Bytes,
) {
    let entires = &entry_table[(node.entry_offset as usize)..(node.entry_count as usize)];
    let string_table = StringTableReader(string_table);

    for entry in entires {
        if string_table.is_skipable(entry.name_offset as usize) {
            continue;
        }

        let name = string_table.read_str(entry.name_offset as usize);
        let path = join_path(prefix, &name);
        match entry.ty {
            0x1100 => {
                target.insert(
                    path,
                    (
                        name,
                        Some(data.slice(
                            (entry.data_offset as usize)
                                ..(entry.data_offset as usize + entry.length as usize),
                        )),
                    ),
                );
            }
            0x0200 => {
                let node = &node_table[entry.data_offset as usize];
                process_node(
                    target,
                    node,
                    &path,
                    node_table,
                    entry_table,
                    string_table.0,
                    data,
                );
                target.insert(path, (name, None));
            }
            _ => {}
        }
    }
}

pub fn join_path(left: &str, right: &str) -> String {
    if left.is_empty() {
        return right.into();
    }

    let mut result = String::with_capacity(left.len() + right.len() + 1);
    result.push_str(left);
    result.push('/');
    result.push_str(right);
    result
}

pub fn create_tag(name: &str) -> [u8; 4] {
    let mut result = [b' '; 4];
    for (target, c) in result.iter_mut().zip(name.chars()) {
        *target = c as u8;
    }
    result
}

pub fn hash_str(value: &str) -> u16 {
    let mut result = Wrapping(0u16);
    for c in value.chars() {
        result *= Wrapping(3);
        result += Wrapping(c as u16);
    }
    result.0
}
