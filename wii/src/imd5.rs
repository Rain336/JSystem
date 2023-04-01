use crate::{FileFormat, WiiError, WiiResult};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use md5::{Digest, Md5};
use std::io::{BufRead, Cursor, Seek, SeekFrom, Write};

pub struct IMD5<F: FileFormat<F>>(F);

impl<F: FileFormat<F>> IMD5<F> {
    pub fn inner(&self) -> &F {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut F {
        &mut self.0
    }

    pub fn into_inner(self) -> F {
        self.0
    }
}

impl<F: FileFormat<F>> FileFormat<IMD5<F>> for IMD5<F> {
    fn read(reader: &mut (impl BufRead + Seek)) -> WiiResult<Self> {
        if reader.read_u32::<BigEndian>()? != 0x424E5320 {
            return Err(WiiError::InvalidMagic);
        }

        let size = reader.read_u32::<BigEndian>()?;

        reader.seek(SeekFrom::Current(8))?;

        let mut hash = [0; 16];
        reader.read_exact(&mut hash)?;

        let mut data = vec![0; size as usize];
        reader.read_exact(&mut data)?;

        let computed: &[u8] = &Md5::digest(&data);

        if computed != hash {
            return Err(WiiError::HashMismatch);
        }

        Ok(IMD5(F::read(&mut Cursor::new(data))?))
    }

    fn write(&self, writer: &mut impl Write) -> WiiResult<()> {
        let mut buffer = Vec::new();
        self.0.write(&mut buffer)?;

        let hash = Md5::digest(&buffer);

        writer.write_u32::<BigEndian>(0x424E5320)?;
        writer.write_u32::<BigEndian>(buffer.len() as u32)?;
        writer.write_all(&[0; 8])?;
        writer.write_all(&hash)?;
        writer.write_all(&buffer)?;

        Ok(())
    }
}

impl<F: FileFormat<F>> From<F> for IMD5<F> {
    fn from(value: F) -> Self {
        IMD5(value)
    }
}
