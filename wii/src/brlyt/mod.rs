mod font;
mod layout;
mod material;
mod pane;
mod texture;
mod user_data;

pub use font::*;
pub use layout::*;
pub use material::*;
pub use pane::*;
pub use texture::*;
pub use user_data::*;

pub struct UVSet {
    top_left: [f32; 2],
    top_right: [f32; 2],
    bottom_left: [f32; 2],
    bottom_right: [f32; 2],
}

pub enum RevolutionLayoutSection {
    Layout(LayoutSection),
    UserData(UserDataSection),
    Texture(TextureSection),
    Material(MaterialSection),
    Font(FontSection),
    Pane(BasicPane),
}

pub struct BinaryRevolutionLayout {
    version: u16,
    sections: Vec<RevolutionLayoutSection>,
}
