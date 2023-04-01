use bytes::{Buf, BufMut};

pub struct RarcHeader {
    pub file_size: u32,
    pub data_header_offset: u32,
    pub data_offset: u32,
    pub data_length: u32,
    pub mram_size: u32,
    pub aram_size: u32,
    pub dvd_size: u32,
    pub node_count: u32,
    pub node_table_offset: u32,
    pub entry_count: u32,
    pub entry_table_offset: u32,
    pub string_table_size: u32,
    pub string_table_offset: u32,
    pub next_free_file_id: u16,
    pub keep_file_ids_sync: bool,
}

impl RarcHeader {
    pub fn read(mut buffer: &[u8]) -> Result<Self, u32> {
        let magic = buffer.get_u32();
        if magic != 0x52415243 {
            return Err(magic);
        }

        let file_size = buffer.get_u32();
        let data_header_offset = buffer.get_u32();
        let data_offset = buffer.get_u32() + 0x20;
        let data_length = buffer.get_u32();
        let mram_size = buffer.get_u32();
        let aram_size = buffer.get_u32();
        let dvd_size = buffer.get_u32();

        let node_count = buffer.get_u32();
        let node_table_offset = buffer.get_u32() + 0x20;
        let entry_count = buffer.get_u32();
        let entry_table_offset = buffer.get_u32() + 0x20;
        let string_table_size = buffer.get_u32();
        let string_table_offset = buffer.get_u32() + 0x20;
        let next_free_file_id = buffer.get_u16();
        let keep_file_ids_sync = buffer.get_u8() != 0x00;

        Ok(RarcHeader {
            file_size,
            data_header_offset,
            data_offset,
            data_length,
            mram_size,
            aram_size,
            dvd_size,
            node_count,
            node_table_offset,
            entry_count,
            entry_table_offset,
            string_table_size,
            string_table_offset,
            next_free_file_id,
            keep_file_ids_sync,
        })
    }

    pub fn write(&self, mut target: &mut [u8]) {
        target.put_u32(0x52415243);

        target.put_u32(self.file_size);
        target.put_u32(self.data_header_offset);
        target.put_u32(self.data_offset - 0x20);
        target.put_u32(self.data_length);
        target.put_u32(self.mram_size);
        target.put_u32(self.aram_size);
        target.put_u32(self.dvd_size);

        target.put_u32(self.node_count);
        target.put_u32(self.node_table_offset - 0x20);
        target.put_u32(self.entry_count);
        target.put_u32(self.entry_table_offset - 0x20);
        target.put_u32(self.string_table_size);
        target.put_u32(self.string_table_offset - 0x20);
        target.put_u16(self.next_free_file_id);
        target.put_u8(if self.keep_file_ids_sync {
            1
        } else {
            0
        });
    }
}

impl Default for RarcHeader {
    fn default() -> Self {
        RarcHeader {
            file_size: 0,
            data_header_offset: 0x20,
            data_offset: 0,
            data_length: 0,
            mram_size: 0,
            aram_size: 0,
            dvd_size: 0,
            node_count: 0,
            node_table_offset: 0,
            entry_count: 0,
            entry_table_offset: 0,
            string_table_size: 0,
            string_table_offset: 0x40,
            next_free_file_id: 0,
            keep_file_ids_sync: false,
        }
    }
}
