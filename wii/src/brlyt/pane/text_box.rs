use super::BasicPane;

pub struct TextBoxPane {
    basic: BasicPane,
    string_size: u16,
    max_string_size: u16,
    material_index: u16,
    font_index: u16,
    origin: u8,
    alignment: u8,
    text: String,
    top_color: [u8; 4],
    bottom_color: [u8; 4],
    font_size_x: f32,
    font_size_y: f32,
    char_size: f32,
    line_size: f32,
}
