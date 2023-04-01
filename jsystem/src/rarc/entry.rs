use bytes::{Buf, BufMut};

pub struct RarcEntry {
    pub id: u16,
    pub name_hash: u16,
    pub ty: u16,
    pub name_offset: u16,
    pub data_offset: u32,
    pub length: u32,
}

impl RarcEntry {
    pub fn write(&self, buffer: &mut &mut [u8]) {
        buffer.put_u16(self.id);
        buffer.put_u16(self.name_hash);
        buffer.put_u16(self.ty);
        buffer.put_u16(self.name_offset);
        buffer.put_u32(self.data_offset);
        buffer.put_u32(self.length);
        buffer.put_u32(0);
    }
}

pub fn read_entry_table(count: u32, mut offset: &[u8]) -> Vec<RarcEntry> {
    let mut result = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let id = offset.get_u16();
        let name_hash = offset.get_u16();
        let ty = offset.get_u16();
        let name_offset = offset.get_u16();
        let data_offset = offset.get_u32();
        let length = offset.get_u32();
        offset.advance(0x04);

        result.push(RarcEntry {
            id,
            name_hash,
            ty,
            name_offset,
            data_offset,
            length,
        })
    }
    result
}
