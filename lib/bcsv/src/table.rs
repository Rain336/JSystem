use crate::header::BcsvHeader;
use crate::{BcsvError, ColumnDefinition, DataType, DataValue, Result};
use byteorder::{ByteOrder, WriteBytesExt};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Seek, Write};
use std::iter::FusedIterator;
use std::ops::{Index, IndexMut};
use std::path::Path;

/// A BCSV table that can be read, edited and saved.
/// BCSV is a typed format, so each column has a concrete type and this is where [`DataType`] adn [`DataValue`] come into play.
/// The table takes and returns [`DataValue`]s when editing and enfoces the correct type of each column.
pub struct Table {
    fields: Vec<ColumnDefinition>,
    table: Vec<DataValue>,
}

impl Table {
    /// Creates a new empty BCSV table that can later be saved using [`Table::save`]
    pub fn new() -> Self {
        Table {
            fields: Vec::new(),
            table: Vec::new(),
        }
    }

    /// Reads in a BCSV from the given reader.
    /// The reader has to be buffered and support seeking.
    /// For files just wrap your [`File`](`std::fs::File`) in a [`BufReader`](`std::io::BufReader`).
    /// For byte slices, wrap your `&[u8]` in a [`Cursor`](`std::io::Cursor`) to make is seekable.
    pub fn read<T: ByteOrder>(mut reader: impl BufRead + Seek) -> Result<Self> {
        let header = BcsvHeader::read::<T>(&mut reader)?;

        let mut fields = Vec::with_capacity(header.column_count as usize);
        for _ in 0..header.column_count {
            fields.push(ColumnDefinition::read::<T>(&mut reader)?);
        }

        let size = header.row_count as usize * header.column_count as usize;
        let string_offset = header.data_offset as u64 + size as u64;

        let mut table = Vec::with_capacity(size);
        for _ in 0..header.row_count {
            for definition in &fields {
                table.push(definition.read_entry::<T>(&mut reader, string_offset)?);
            }
        }

        Ok(Table { fields, table })
    }

    /// Reads in the BCSV at the given path.
    pub fn open<T: ByteOrder>(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Table::read::<T>(reader)
    }

    /// Write this table as a BCSV into the given writer.
    /// The writer is not buffered, but it is highly recomended to do so for files.
    /// Just wrap your [`File`](`std::fs::File`) in a [`BufWriter`](`std::io::BufWriter`).
    pub fn write<T: ByteOrder>(&self, mut writer: impl Write) -> Result<()> {
        let header = BcsvHeader {
            row_count: self.row_count() as u32,
            column_count: self.fields.len() as u32,
            data_offset: 0x10 + self.fields.len() as u32 * 0x0C,
            row_size: self.fields.iter().map(|x| x.ty.size() as u32).sum(),
        };
        header.write::<T>(&mut writer)?;

        for definition in &self.fields {
            definition.write::<T>(&mut writer)?;
        }

        let mut pool = Vec::new();
        for row in 0..header.row_count {
            for (column, definition) in self.fields.iter().enumerate() {
                definition.write_entry::<T>(
                    &self.table[row as usize * self.fields.len() + column],
                    &mut writer,
                    &mut pool,
                )?;
            }
        }

        writer.write_all(&pool)?;

        let size = header.data_offset as usize
            + (header.row_count as usize * header.row_size as usize)
            + pool.len();
        let padding = ((size + 31) & !31) - size;

        for _ in 0..padding {
            writer.write_u8(b'@')?;
        }

        Ok(())
    }

    /// Saves this BCSv to the given path.
    pub fn save<T: ByteOrder>(&self, path: impl AsRef<Path>) -> Result<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        self.write::<T>(writer)
    }

    /// Appends a row of values onto this table.
    /// The values will be moved out of the given vec on success.
    pub fn push_row(&mut self, values: &mut Vec<DataValue>) -> Result<()> {
        if values.len() != self.fields.len() {
            Err(BcsvError::InvaildRowLength {
                expected: self.fields.len(),
                actual: values.len(),
            })
        } else {
            for (column, (defintion, value)) in self.fields.iter().zip(values.iter()).enumerate() {
                if defintion.ty != value.ty() {
                    return Err(BcsvError::InvaildRowType {
                        column,
                        expected: defintion.ty,
                        actual: value.ty(),
                    });
                }
            }
            self.table.append(values);
            Ok(())
        }
    }

    /// Pushes a new column onto the table.
    /// The name is a hashed [`u32`], using one of the two hash functions supplied with this crate.
    /// See [`crate::jgadget_hash`] and [`crate::old_hash`] and chose wich one you prefer.
    /// The given value will be coppied into each row at the new columns position,
    /// if you would prefer to supply a diffrent value for each row, see [`Table::push_column_with`].
    pub fn push_column(&mut self, name: u32, ty: DataType, value: &DataValue) -> Result<()> {
        if self.fields.len() == u16::MAX as usize {
            return Err(BcsvError::TooManyColumns);
        }

        if ty != value.ty() {
            return Err(BcsvError::InvaildRowType {
                column: self.fields.len(),
                expected: ty,
                actual: value.ty(),
            });
        }

        self.table.reserve(self.row_count());
        let rows = self.row_count();
        for row in 0..rows {
            self.table.insert(
                row * self.fields.len() + row + self.fields.len(),
                value.clone(),
            );
        }

        let offset = self.fields.len() as u16;
        self.fields.push(ColumnDefinition {
            name,
            bitmask: u32::MAX,
            offset,
            shift: 0,
            ty,
        });

        Ok(())
    }

    /// Pushes a new column onto the table.
    /// The name is a hashed [`u32`], using one of the two hash functions supplied with this crate.
    /// See [`crate::jgadget_hash`] and [`crate::old_hash`] and chose wich one you prefer.
    /// The given FnMut will be invoked per row to create a value for that row.
    /// If you want every row entry to just get a copy of one value, see [`Table::push_column`].
    pub fn push_column_with(
        &mut self,
        name: u32,
        ty: DataType,
        mut f: impl FnMut() -> DataValue,
    ) -> Result<()> {
        if self.fields.len() == u16::MAX as usize {
            return Err(BcsvError::TooManyColumns);
        }

        let rows = self.row_count();
        for row in 0..rows {
            let data = f();

            if ty != data.ty() {
                return Err(BcsvError::InvaildRowType {
                    column: self.fields.len(),
                    expected: ty,
                    actual: data.ty(),
                });
            }

            self.table
                .insert(row * self.fields.len() + row + self.fields.len(), data);
        }

        let offset = self.fields.len() as u16;
        self.fields.push(ColumnDefinition {
            name,
            bitmask: u32::MAX,
            offset,
            shift: 0,
            ty,
        });

        Ok(())
    }

    /// Gets the number of rows in this table.
    pub fn row_count(&self) -> usize {
        self.table.len() / self.fields.len()
    }

    /// Gets the number of columns in this table.
    pub fn column_count(&self) -> usize {
        self.fields.len()
    }

    /// Gets a referance to the value at the given row and column, if it exists.
    pub fn get(&self, row: usize, column: usize) -> Option<&DataValue> {
        self.table.get(row * self.fields.len() + column)
    }

    /// Gets a mutable referance to the value at the given row and column, if it exists.
    pub fn get_mut(&mut self, row: usize, column: usize) -> Option<&mut DataValue> {
        self.table.get_mut(row * self.fields.len() + column)
    }

    /// Gets a row from this table as a slice of values.
    pub fn row(&self, row: usize) -> Option<&[DataValue]> {
        let start = row * self.fields.len();
        let end = start + self.fields.len();
        if end > self.table.len() {
            None
        } else {
            Some(&self.table[start..end])
        }
    }

    /// Gets a row from this table as a mutable slice of values.
    pub fn row_mut(&mut self, row: usize) -> Option<&mut [DataValue]> {
        let start = row * self.fields.len();
        let end = start + self.fields.len();
        if end > self.table.len() {
            None
        } else {
            Some(&mut self.table[start..end])
        }
    }

    /// Creates an iterator over a column of values.
    pub fn column(&self, column: usize) -> Option<ColumnIter<'_>> {
        if column > self.fields.len() {
            None
        } else {
            Some(ColumnIter {
                table: &self.table,
                offset: column,
                increment: self.fields.len(),
            })
        }
    }

    /// Creates a mutable iterator over a column of values.
    pub fn column_mut(&mut self, column: usize) -> Option<ColumnIterMut<'_>> {
        if column > self.fields.len() {
            None
        } else {
            Some(ColumnIterMut {
                table: &mut self.table[column..],
                increment: self.fields.len() - 1,
            })
        }
    }

    /// Removes a row from this table, returning it's values in a vec.
    pub fn remove_row(&mut self, row: usize) -> Option<Vec<DataValue>> {
        if row > self.row_count() {
            None
        } else {
            let start = row * self.fields.len();
            let end = start + self.fields.len();
            Some(self.table.drain(start..end).collect())
        }
    }

    /// Removes a column from this table, returning it's values in a vec.
    pub fn remove_column(&mut self, column: usize) -> Option<(ColumnDefinition, Vec<DataValue>)> {
        if column > self.fields.len() {
            None
        } else {
            let rows = self.row_count();
            let mut result = Vec::with_capacity(rows);
            for row in 0..rows {
                let index = row * self.fields.len() + column - row;
                result.push(self.table.remove(index));
            }

            Some((self.fields.remove(column), result))
        }
    }

    /// Creates an iterator over the column definitions of this table.
    /// This is useful of you read the table in and want to find out how it's values are encoded.
    pub fn definitions(&self) -> &[ColumnDefinition] {
        &self.fields
    }

    /// Returns an iterator over the tables's rows.
    pub fn iter(&self) -> RowIter<'_> {
        RowIter {
            table: &self.table,
            fields: self.column_count(),
        }
    }

    /// Returns a mutable iterator over the tables's rows.
    pub fn iter_mut(&mut self) -> RowIterMut<'_> {
        RowIterMut {
            fields: self.column_count(),
            table: &mut self.table,
        }
    }
}

impl Default for Table {
    fn default() -> Self {
        Table::new()
    }
}

impl Index<(usize, usize)> for Table {
    type Output = DataValue;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.table[index.0 * self.fields.len() + index.1]
    }
}

impl IndexMut<(usize, usize)> for Table {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.table[index.0 * self.fields.len() + index.1]
    }
}

impl Index<usize> for Table {
    type Output = [DataValue];

    fn index(&self, index: usize) -> &Self::Output {
        self.row(index).unwrap()
    }
}

impl IndexMut<usize> for Table {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.row_mut(index).unwrap()
    }
}

impl<'a> IntoIterator for &'a Table {
    type Item = &'a [DataValue];

    type IntoIter = RowIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut Table {
    type Item = &'a mut [DataValue];

    type IntoIter = RowIterMut<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// An iterator over a table's columns
pub struct ColumnIter<'a> {
    table: &'a [DataValue],
    offset: usize,
    increment: usize,
}

impl<'a> Iterator for ColumnIter<'a> {
    type Item = &'a DataValue;

    fn next(&mut self) -> Option<Self::Item> {
        let offset = self.offset;
        self.offset += self.increment;
        self.table.get(offset)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let hint = self.table.len() / self.increment;
        (hint, Some(hint))
    }
}

impl<'a> FusedIterator for ColumnIter<'a> {}
impl<'a> ExactSizeIterator for ColumnIter<'a> {}

/// A mutable iterator over a table's columns
pub struct ColumnIterMut<'a> {
    table: &'a mut [DataValue],
    increment: usize,
}

impl<'a> Iterator for ColumnIterMut<'a> {
    type Item = &'a mut DataValue;

    fn next(&mut self) -> Option<Self::Item> {
        if self.table.is_empty() {
            return None;
        }

        let slice = std::mem::take(&mut self.table);
        let (left, right) = slice.split_first_mut()?;
        self.table = &mut right[self.increment..];

        Some(left)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let hint = self.table.len() / self.increment;
        (hint, Some(hint))
    }
}

impl<'a> FusedIterator for ColumnIterMut<'a> {}
impl<'a> ExactSizeIterator for ColumnIterMut<'a> {}

/// An iterator over a table's rows
pub struct RowIter<'a> {
    table: &'a [DataValue],
    fields: usize,
}

impl<'a> Iterator for RowIter<'a> {
    type Item = &'a [DataValue];

    fn next(&mut self) -> Option<Self::Item> {
        if self.table.len() < self.fields {
            return None;
        }

        let (result, table) = self.table.split_at(self.fields);
        self.table = table;

        Some(result)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.table.len() / self.fields;
        (size, Some(size))
    }
}

impl<'a> FusedIterator for RowIter<'a> {}
impl<'a> ExactSizeIterator for RowIter<'a> {}

/// A mutable iterator over a table's rows
pub struct RowIterMut<'a> {
    table: &'a mut [DataValue],
    fields: usize,
}

impl<'a> Iterator for RowIterMut<'a> {
    type Item = &'a mut [DataValue];

    fn next(&mut self) -> Option<Self::Item> {
        if self.table.len() < self.fields {
            return None;
        }

        let table = std::mem::take(&mut self.table);
        let (result, table) = table.split_at_mut(self.fields);
        self.table = table;

        Some(result)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.table.len() / self.fields;
        (size, Some(size))
    }
}

impl<'a> FusedIterator for RowIterMut<'a> {}
impl<'a> ExactSizeIterator for RowIterMut<'a> {}
