use std::io::{BufRead, ErrorKind, Seek};

use byteorder::{ByteOrder, WriteBytesExt};

use crate::WiiResult;

pub fn read_string(mut reader: impl BufRead + Seek) -> WiiResult<String> {
    let mut result = String::new();

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

    Ok(result)
}

pub fn read_string_exact(mut reader: impl BufRead + Seek, mut len: usize) -> WiiResult<String> {
    let mut result = String::with_capacity(len);

    loop {
        let buffer = match reader.fill_buf() {
            Ok(x) => x,
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => return Err(e.into()),
        };

        if buffer.len() > len {
            result.push_str(std::str::from_utf8(&buffer[..len])?);
            break;
        }
        
        result.push_str(std::str::from_utf8(buffer)?);
        len -= buffer.len();
    }

    Ok(result)
}

pub struct DataBufferWriter(Vec<u8>);

impl DataBufferWriter {
    pub fn new() -> Self {
        DataBufferWriter(Vec::new())
    }

    pub fn with_capacity(len: usize) -> Self {
        DataBufferWriter(Vec::with_capacity(len))
    }

    pub fn write_bytes(&mut self, data: &[u8]) -> u32 {
        let offset = self.0.len() as u32;
        self.0.extend_from_slice(data);
        offset
    }

    pub fn write_str_null(&mut self, value: &str) -> u32 {
        let result = self.0.len() as u32;
        self.0.reserve(self.0.len() + 1);
        self.0.extend_from_slice(value.as_bytes());
        self.0.push(0);
        result
    }

    pub fn write_i32_slice<T: ByteOrder>(&mut self, value: &[i32]) -> u32 {
        let result = self.0.len() as u32;
        self.0.reserve(value.len() * 4);
        for x in value {
            self.0.write_i32::<T>(*x).unwrap();
        }
        result
    }

    pub fn write_f32_slice<T: ByteOrder>(&mut self, value: &[f32]) -> u32 {
        let result = self.0.len() as u32;
        self.0.reserve(value.len() * 4);
        for x in value {
            self.0.write_f32::<T>(*x).unwrap();
        }
        result
    }

    pub fn finish(self) -> Vec<u8> {
        self.0
    }
}
