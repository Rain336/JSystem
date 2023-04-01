use bytes::{Buf, BufMut, BytesMut};

pub struct BcsvHeader {
    pub entry_count: u32,
    pub field_count: u32,
    pub data_offset: u32,
    pub entry_size: u32,
}

impl BcsvHeader {
    pub fn read(mut buffer: &[u8]) -> Self {
        let entry_count = buffer.get_u32();
        let field_count = buffer.get_u32();
        let data_offset = buffer.get_u32();
        let entry_size = buffer.get_u32();

        BcsvHeader {
            entry_count,
            field_count,
            data_offset,
            entry_size,
        }
    }

    pub fn write(&self, buffer: &mut BytesMut) {
        buffer.put_u32(self.entry_count);
        buffer.put_u32(self.field_count);
        buffer.put_u32(self.data_offset);
        buffer.put_u32(self.entry_size);
    }
}
