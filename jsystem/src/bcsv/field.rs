use super::{BcsvResult, DataType};

pub struct BcsvFieldDefinition {
    pub name_hash: u32,
    pub bitmask: u32,
    pub entry_offset: u16,
    pub shift_amount: u8,
    pub data_type: DataType,
}

impl BcsvFieldDefinition {
    pub fn read(buffer: &mut &[u8]) -> BcsvResult<Self> {
        let name_hash = buffer.get_u32();
        let bitmask = buffer.get_u32();
        let entry_offset = buffer.get_u16();
        let shift_amount = buffer.get_u8();
        let data_type = buffer.get_u8().try_into()?;

        Ok(BcsvFieldDefinition {
            name_hash,
            bitmask,
            entry_offset,
            shift_amount,
            data_type,
        })
    }

    pub fn write(&self, buffer: &mut BytesMut) {
        buffer.put_u32(self.name_hash);
        buffer.put_u32(self.bitmask);
        buffer.put_u16(self.entry_offset);
        buffer.put_u8(self.shift_amount);
        buffer.put_u8(self.data_type as u8);
    }
}
