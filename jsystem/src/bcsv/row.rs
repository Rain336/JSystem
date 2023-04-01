use super::{BcsvError, BcsvFieldDefinition, BcsvResult, DataType, DataValue};
use bytes::{Buf, BufMut, Bytes, BytesMut};

pub struct BcsvRow(pub Vec<DataValue>);

impl BcsvRow {
    pub fn read(buffer: &[u8], fields: &[BcsvFieldDefinition], strings: &[u8]) -> Self {
        let mut result = Vec::with_capacity(fields.len());
        for field in fields {
            let mut entry_start = &buffer[(field.entry_offset as usize)..];
            result.push(match field.data_type {
                DataType::Int32 => {
                    let value = entry_start.get_i32();
                    DataValue::Int32((value & field.bitmask as i32) >> field.shift_amount)
                }
                DataType::InlineString => DataValue::Unknown,
                DataType::Float => DataValue::Float(entry_start.get_f32()),
                DataType::UInt32 => DataValue::UInt32(entry_start.get_u32()),
                DataType::Int16 => DataValue::Int16(entry_start.get_i16()),
                DataType::Byte => DataValue::Byte(entry_start.get_u8()),
                DataType::String => {
                    let string_start = &strings[(entry_start.get_u32() as usize)..];
                    let mut len = 0;
                    while string_start[len] != 0 {
                        len += 1;
                    }
                    let (string, _) =
                        encoding_rs::SHIFT_JIS.decode_without_bom_handling(&string_start[..len]);
                    DataValue::String(string.into_owned())
                }
                DataType::Null => DataValue::Null,
            });
        }

        BcsvRow(result)
    }

    pub fn write(
        &self,
        buffer: &mut BytesMut,
        fields: &[BcsvFieldDefinition],
        strings: &mut StringWriter,
    ) {
        for (field, definition) in self.0.iter().zip(fields) {
            match field {
                DataValue::Int32(x) => {
                    if definition.bitmask != u32::MAX {
                        buffer.put_i32((*x << definition.shift_amount) & definition.bitmask as i32)
                    } else {
                        buffer.put_i32(*x)
                    }
                }
                DataValue::Float(x) => buffer.put_f32(*x),
                DataValue::UInt32(x) => buffer.put_u32(*x),
                DataValue::Int16(x) => buffer.put_i16(*x),
                DataValue::Byte(x) => buffer.put_u8(*x),
                DataValue::String(x) => {
                    let offset = strings.write_str(x);
                    buffer.put_u32(offset as u32);
                }
                _ => {}
            }
        }
    }

    pub fn validate(&self, fields: &[BcsvFieldDefinition]) -> BcsvResult<()> {
        if self.0.len() != fields.len() {
            return Err(BcsvError::InvalidRowLength(fields.len(), self.0.len()));
        }

        for (i, (value, field)) in self.0.iter().zip(fields).enumerate() {
            if field.data_type != value.to_type() {
                return Err(BcsvError::InvalidRowDataType(
                    i,
                    field.data_type,
                    value.to_type(),
                ));
            }
        }

        Ok(())
    }
}

pub struct StringWriter(BytesMut);

impl StringWriter {
    pub fn new() -> Self {
        StringWriter(BytesMut::new())
    }

    pub fn write_str(&mut self, value: &str) -> usize {
        let (data, _, _) = encoding_rs::SHIFT_JIS.encode(value);

        self.0.reserve(data.len() + 1);

        let result = self.0.len();

        self.0.extend_from_slice(&data);
        self.0.put_u8(0);

        result
    }

    pub fn finish(self) -> Bytes {
        self.0.freeze()
    }
}
