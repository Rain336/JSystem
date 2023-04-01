#![warn(clippy::missing_errors_doc)]
#![warn(clippy::missing_panics_doc)]
#![warn(clippy::missing_safety_doc)]
#![warn(clippy::needless_doctest_main)]
#![warn(clippy::tabs_in_doc_comments)]
#![warn(clippy::doc_markdown)]

pub mod brlyt;
#[cfg(feature = "disc")]
pub mod disc;
mod error;
#[cfg(feature = "imd5")]
pub mod imd5;
#[cfg(feature = "imet")]
pub mod imet;
#[cfg(feature = "u8")]
pub mod u8;
mod utils;

pub use error::*;
use std::io::{BufRead, Seek, Write};

pub trait FileFormat<T: FileFormat<T>> {
    fn read(reader: &mut (impl BufRead + Seek)) -> WiiResult<T>;
    fn write(&self, writer: &mut impl Write) -> WiiResult<()>;
}
