mod nkit;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
pub use nkit::*;
use std::io::{self, Read, Seek, SeekFrom, Write};
use crate::disc::title::TitleId;

pub struct WiiDiscHeader {
    title_id: TitleId,
    marker_code: [u8; 2],
    disc_number: u8,
    disc_version: u8,
    audio_streaming: bool,
    streaming_buffer_size: u8,
    wii_magicword: bool,
    gamecube_magicword: bool,
    game_title: [u8; 64],
    hash_varification: bool,
    disc_encryption: bool,
    nkit_header: Option<NKitHeader>,
}

impl WiiDiscHeader {
    pub fn read<R: Read + Seek>(reader: &mut R) -> io::Result<Self> {
        let mut title_id = [0; 4];
        reader.read_exact(&mut title_id)?;
        let mut marker_code = [0; 2];
        reader.read_exact(&mut marker_code)?;
        let disc_number = reader.read_u8()?;
        let disc_version = reader.read_u8()?;
        let audio_streaming = reader.read_u8()? != 0;
        let streaming_buffer_size = reader.read_u8()?;

        reader.seek(SeekFrom::Current(14))?;
        let wii_magicword = reader.read_u32::<BigEndian>()? == 0x5D1C9EA3;
        let gamecube_magicword = reader.read_u32::<BigEndian>()? == 0xC2339F3D;
        let mut game_title = [0; 64];
        reader.read_exact(&mut game_title)?;
        let hash_varification = reader.read_u8()? != 0x01;
        let disc_encryption = reader.read_u8()? != 0x01;

        reader.seek(SeekFrom::Start(200))?;
        let nkit_header = NKitHeader::read(reader)?;

        reader.seek(SeekFrom::Start(1024))?;

        Ok(WiiDiscHeader {
            title_id: title_id.into(),
            marker_code,
            disc_number,
            disc_version,
            audio_streaming,
            streaming_buffer_size,
            wii_magicword,
            gamecube_magicword,
            game_title,
            hash_varification,
            disc_encryption,
            nkit_header,
        })
    }

    pub fn write(&self, writer: &mut impl Write) -> io::Result<()> {
        let title_id: [u8; 4] = self.title_id.into();
        writer.write_all(&title_id)?;
        writer.write_all(&self.marker_code)?;
        writer.write_u8(self.disc_number)?;
        writer.write_u8(self.disc_version)?;
        writer.write_u8(if self.audio_streaming { 0x01 } else { 0x00 })?;
        writer.write_u8(self.streaming_buffer_size)?;

        writer.write_all(&[0; 14])?;
        writer.write_u32::<BigEndian>(if self.wii_magicword { 0x5D1C9EA3 } else { 0 })?;
        writer.write_u32::<BigEndian>(if self.gamecube_magicword { 0xC2339F3D } else { 0 })?;
        writer.write_all(&self.game_title)?;
        writer.write_u8(if self.hash_varification { 0x00 } else { 0x01 })?;
        writer.write_u8(if self.disc_encryption { 0x00 } else { 0x01 })?;
    
        if let Some(nkit) = &self.nkit_header {
            writer.write_all(&[0; 414])?;
            nkit.write(writer)?;
            writer.write_all(&[0; 484])?;
        } else {
            writer.write_all(&[0; 926])?;
        }

        Ok(())
    }
}
