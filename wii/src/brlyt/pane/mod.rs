mod image;
mod text_box;

use crate::WiiResult;
use byteorder::ByteOrder;
pub use image::*;
use std::io::{BufRead, Seek};
pub use text_box::*;

pub struct BoundaryPane(BasicPane);

pub enum Pane {
    Basic(BasicPane),
    Image(ImagePane),
    Boundary(BoundaryPane),
    TextBox(TextBoxPane),
}

pub struct BasicPane {
    flags: u8,
    origin_type: u8,
    alpha: u8,
    name: String,
    user_info: String,
    translation: [f32; 3],
    rotation: [f32; 3],
    scale: [f32; 2],
    width: f32,
    height: f32,
    children: Vec<Pane>,
}

impl BasicPane {
    pub(crate) fn read<T: ByteOrder>(reader: impl BufRead + Seek) -> WiiResult<Self> {
        todo!()
    }
}
