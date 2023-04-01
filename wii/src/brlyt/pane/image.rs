use super::BasicPane;
use crate::brlyt::UVSet;

pub struct ImagePane {
    basic: BasicPane,
    top_left: [u8; 4],
    top_right: [u8; 4],
    bottom_left: [u8; 4],
    bottom_right: [u8; 4],
    material_index: u16,
    uv_sets: Vec<UVSet>,
}
