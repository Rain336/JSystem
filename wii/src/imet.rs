use crate::{FileFormat, WiiError, WiiResult};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use md5::{Digest, Md5};
use std::io::{BufRead, Read, Seek, SeekFrom, Write};

pub struct IMET<F: FileFormat<F>> {
    icon_bin_size: u32,
    banner_bin_size: u32,
    sound_bin_size: u32,
    name_jp: String,
    name_en: String,
    name_de: String,
    name_fr: String,
    name_es: String,
    name_it: String,
    name_nl: String,
    name_unk1: String,
    name_unk2: String,
    name_kr: String,
    inner: F,
}

impl<F: FileFormat<F>> IMET<F> {
    fn write_hashable_data(&self) -> WiiResult<Vec<u8>> {
        let mut result = vec![0; 0x600];
        let mut writer = &mut result[..];

        writer.write_u32::<BigEndian>(0x494d4554)?;
        writer.write_u32::<BigEndian>(0x600)?;
        writer.write_u32::<BigEndian>(3)?;
        writer.write_u32::<BigEndian>(self.icon_bin_size)?;
        writer.write_u32::<BigEndian>(self.banner_bin_size)?;
        writer.write_u32::<BigEndian>(self.sound_bin_size)?;
        writer.write_u32::<BigEndian>(0)?;
        write_string(&mut writer, &self.name_jp)?;
        write_string(&mut writer, &self.name_en)?;
        write_string(&mut writer, &self.name_de)?;
        write_string(&mut writer, &self.name_fr)?;
        write_string(&mut writer, &self.name_es)?;
        write_string(&mut writer, &self.name_it)?;
        write_string(&mut writer, &self.name_nl)?;
        write_string(&mut writer, &self.name_unk1)?;
        write_string(&mut writer, &self.name_unk2)?;
        write_string(&mut writer, &self.name_kr)?;

        Ok(result)
    }
}

impl<F: FileFormat<F>> FileFormat<IMET<F>> for IMET<F> {
    fn read(reader: &mut (impl BufRead + Seek)) -> WiiResult<Self> {
        reader.seek(SeekFrom::Current(64))?;

        if reader.read_u32::<BigEndian>()? != 0x494d4554 {
            return Err(WiiError::InvalidMagic);
        }

        reader.read_u32::<BigEndian>()?;
        reader.read_u32::<BigEndian>()?;

        let icon_bin_size = reader.read_u32::<BigEndian>()?;
        let banner_bin_size = reader.read_u32::<BigEndian>()?;
        let sound_bin_size = reader.read_u32::<BigEndian>()?;

        reader.read_u32::<BigEndian>()?;

        let name_jp = read_string(reader)?;
        let name_en = read_string(reader)?;
        let name_de = read_string(reader)?;
        let name_fr = read_string(reader)?;
        let name_es = read_string(reader)?;
        let name_it = read_string(reader)?;
        let name_nl = read_string(reader)?;
        let name_unk1 = read_string(reader)?;
        let name_unk2 = read_string(reader)?;
        let name_kr = read_string(reader)?;

        reader.seek(SeekFrom::Current(588 + 16))?;

        let inner = F::read(reader)?;

        Ok(IMET {
            icon_bin_size,
            banner_bin_size,
            sound_bin_size,
            name_jp,
            name_en,
            name_de,
            name_fr,
            name_es,
            name_it,
            name_nl,
            name_unk1,
            name_unk2,
            name_kr,
            inner,
        })
    }

    fn write(&self, writer: &mut impl Write) -> WiiResult<()> {
        let mut data = self.write_hashable_data()?;
        let hash = Md5::digest(&data);
        data[0x5F0..].copy_from_slice(&hash);

        writer.write_all(&[0; 64])?;
        writer.write_all(&data)?;
        self.inner.write(writer)?;

        Ok(())
    }
}

fn read_string(reader: &mut impl Read) -> WiiResult<String> {
    let mut buffer = [0u16; 42];
    reader.read_u16_into::<BigEndian>(&mut buffer)?;

    let mut result = String::from_utf16(&buffer)?;
    let len = result.trim_end_matches('\0').len();
    result.truncate(len);
    Ok(result)
}

fn write_string(writer: &mut impl Write, value: &str) -> WiiResult<()> {
    let buffer: Box<[u8]> = value.encode_utf16().flat_map(|x| x.to_be_bytes()).collect();
    writer.write_all(&buffer)?;
    Ok(())
}
