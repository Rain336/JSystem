use crate::header::RarcHeader;
use crate::Result;
use std::io::{Read, Seek, SeekFrom};

pub struct StringTable {
    buffer: Vec<u8>,
}

impl StringTable {
    pub fn read(mut reader: impl Read + Seek, header: &RarcHeader) -> Result<Self> {
        reader.seek(SeekFrom::Start(header.string_offset as u64 + 0x20))?;

        let mut buffer = vec![0; header.string_size as usize];
        reader.read_exact(&mut buffer)?;

        Ok(StringTable { buffer })
    }

    pub fn string_at(&self, start: usize) -> Option<&str> {
        let buffer = match self.buffer[start..].iter().position(|x| *x == 0) {
            Some(end) => &self.buffer[start..end],
            None => &self.buffer[start..],
        };

        if buffer.is_ascii() {
            Some(unsafe { std::str::from_utf8_unchecked(buffer) })
        } else {
            None
        }
    }
}
