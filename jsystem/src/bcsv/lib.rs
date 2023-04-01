mod data;
mod error;
mod field;
mod header;
mod row;

pub use data::*;
pub use error::*;

use bytes::{Bytes, BytesMut};
use field::BcsvFieldDefinition;
use row::{BcsvRow, StringWriter};
use std::ops::{Index, IndexMut};
use std::u32;

pub fn create_name_hash(name: &str) -> u32 {
    name.chars().fold(0, |result, c| {
        result.wrapping_mul(0x1F).wrapping_add(c as u32)
    })
}

#[derive(Default)]
pub struct BcsvTable(Vec<BcsvRow>, Vec<BcsvFieldDefinition>);

impl BcsvTable {
    pub fn read(buffer: &[u8]) -> BcsvResult<Self> {
        let header = header::BcsvHeader::read(&buffer[..0x10]);

        let mut fields = Vec::with_capacity(header.field_count as usize);
        let mut field_buffer = &buffer[0x10..];
        for _ in 0..header.field_count {
            fields.push(field::BcsvFieldDefinition::read(&mut field_buffer)?);
        }

        let mut rows = Vec::with_capacity(header.entry_count as usize);
        let string_offset = header.data_offset as usize
            + (header.entry_count as usize * header.entry_size as usize);
        let strings = &buffer[string_offset..];
        for _ in 0..header.entry_count {
            rows.push(BcsvRow::read(field_buffer, &fields, strings));
            field_buffer = &field_buffer[(header.entry_size as usize)..];
        }

        Ok(BcsvTable(rows, fields))
    }

    pub fn save(&self) -> Bytes {
        let header = header::BcsvHeader {
            entry_count: self.0.len() as u32,
            field_count: self.1.len() as u32,
            data_offset: 0x10 + self.1.len() as u32 * 0x0C,
            entry_size: self.1.iter().map(|x| x.data_type.size()).sum::<usize>() as u32,
        };

        let mut buffer = BytesMut::with_capacity(
            header.data_offset as usize + header.entry_size as usize * header.entry_count as usize,
        );

        header.write(&mut buffer);

        for field in self.1.iter() {
            field.write(&mut buffer);
        }

        let mut strings = StringWriter::new();
        for row in self.0.iter() {
            row.write(&mut buffer, &self.1, &mut strings)
        }

        buffer.extend_from_slice(&strings.finish());

        buffer.freeze()
    }

    pub fn push_row(&mut self, values: Vec<DataValue>) -> BcsvResult<()> {
        let row = BcsvRow(values);
        row.validate(&self.1)?;
        self.0.push(row);
        Ok(())
    }

    pub fn push_column(
        &mut self,
        name_hash: u32,
        data_type: DataType,
        default: &DataValue,
    ) -> BcsvResult<()> {
        let entry_offset = match self.1.last() {
            Some(x) => x.entry_offset + x.data_type.size() as u16,
            None => 0,
        };

        let field = BcsvFieldDefinition {
            name_hash,
            bitmask: u32::MAX,
            entry_offset,
            shift_amount: 0,
            data_type,
        };

        if default.to_type() != data_type {
            return Err(BcsvError::InvalidRowDataType(
                self.1.len(),
                data_type,
                default.to_type(),
            ));
        }

        self.1.push(field);
        for row in self.0.iter_mut() {
            row.0.push(default.clone());
        }

        Ok(())
    }

    pub fn row_count(&self) -> usize {
        self.0.len()
    }

    pub fn column_count(&self) -> usize {
        self.1.len()
    }

    pub fn get(&self, row: usize, column: usize) -> Option<&DataValue> {
        self.0.get(row).and_then(|x| x.0.get(column))
    }

    pub fn get_mut(&mut self, row: usize, column: usize) -> Option<&mut DataValue> {
        self.0.get_mut(row).and_then(|x| x.0.get_mut(column))
    }

    pub fn row(&self, idx: usize) -> Option<&[DataValue]> {
        match self.0.get(idx) {
            Some(x) => Some(&x.0),
            None => None,
        }
    }

    pub fn row_mut(&mut self, idx: usize) -> Option<&mut [DataValue]> {
        match self.0.get_mut(idx) {
            Some(x) => Some(&mut x.0),
            None => None,
        }
    }

    pub fn column(&self, idx: usize) -> Option<impl Iterator<Item = &DataValue>> {
        if idx > self.1.len() {
            None
        } else {
            Some(self.0.iter().map(move |x| &x.0[idx]))
        }
    }

    pub fn column_mut(&mut self, idx: usize) -> Option<impl Iterator<Item = &mut DataValue>> {
        if idx > self.1.len() {
            None
        } else {
            Some(self.0.iter_mut().map(move |x| &mut x.0[idx]))
        }
    }

    pub fn remove_row(&mut self, idx: usize) -> Option<Vec<DataValue>> {
        if idx > self.0.len() {
            None
        } else {
            Some(self.0.remove(idx).0)
        }
    }

    pub fn remove_column(&mut self, idx: usize) {
        if idx > self.1.len() {
            return;
        }

        self.1.remove(idx);
        for row in self.0.iter_mut() {
            row.0.remove(idx);
        }
    }

    pub fn definitions<'a>(&'a self) -> impl Iterator<Item = (u32, DataType)> + 'a {
        self.1.iter().map(|x| (x.name_hash, x.data_type))
    }

    pub fn iter(&self) -> impl Iterator<Item = &[DataValue]> {
        self.0.iter().map(|x| &x.0[..])
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut [DataValue]> {
        self.0.iter_mut().map(|x| &mut x.0[..])
    }
}

impl Index<(usize, usize)> for BcsvTable {
    type Output = DataValue;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        self.get(index.0, index.1).unwrap()
    }
}

impl IndexMut<(usize, usize)> for BcsvTable {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        self.get_mut(index.0, index.1).unwrap()
    }
}

impl Index<usize> for BcsvTable {
    type Output = [DataValue];

    fn index(&self, index: usize) -> &Self::Output {
        self.row(index).unwrap()
    }
}

impl IndexMut<usize> for BcsvTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.row_mut(index).unwrap()
    }
}
