use bytes::{Buf, BufMut};

pub struct RarcNode {
    pub tag: [u8; 4],
    pub name_offset: u32,
    pub name_hash: u16,
    pub entry_count: u16,
    pub entry_offset: u32,
}

impl RarcNode {
    pub fn write(&self, buffer: &mut &mut [u8]) {
        buffer.put_slice(&self.tag);
        buffer.put_u32(self.name_offset);
        buffer.put_u16(self.name_hash);
        buffer.put_u16(self.entry_count);
        buffer.put_u32(self.entry_offset);
    }
}

pub fn read_node_table(count: u32, mut offset: &[u8]) -> Vec<RarcNode> {
    let mut result = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let mut tag = [0u8; 4];
        offset.copy_to_slice(&mut tag[..]);
        let name_offset = offset.get_u32();
        let name_hash = offset.get_u16();
        let entry_count = offset.get_u16();
        let entry_offset = offset.get_u32();

        result.push(RarcNode {
            tag,
            name_offset,
            name_hash,
            entry_count,
            entry_offset,
        })
    }
    result
}
