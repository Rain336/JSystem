//! # Nintendo BCSV / JMap Format
//!
//! BCSV / JMap is a custom file format made by nintendo for GameCube and Wii games.
//! It stores data in a table format with typed columns.

mod data;
mod definition;
mod header;
mod table;

pub use data::*;
pub use definition::*;
pub use table::*;
use thiserror::Error;

/// Re-exports from the byteorder crate.
pub mod byteorder {
    pub use byteorder::{BigEndian, ByteOrder, LittleEndian, NativeEndian};
}

/// An old name hash function used mainly in GameCube games.
pub fn old_hash(name: &[u8]) -> u32 {
    name.iter().fold(0, |result, c| {
        ((result << 8).wrapping_add(*c as u32)) % 33554393
    })
}

/// A newer name hash function used in Wii games.
pub fn jgadget_hash(name: &[u8]) -> u32 {
    name.iter().fold(0, |result, c| {
        result.wrapping_mul(0x1F).wrapping_add(*c as u32)
    })
}

type Result<T> = std::result::Result<T, BcsvError>;

/// Errors that get returned from this library.
#[derive(Debug, Error)]
pub enum BcsvError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("invalid data type {0}, only types between 0 - 7 are known")]
    InvaildDataType(u8),
    #[error("inviald row length, expected {expected} but got {actual}")]
    InvaildRowLength { expected: usize, actual: usize },
    #[error("invalid column type at column {column}, expected {expected} but got {actual}")]
    InvaildRowType {
        column: usize,
        expected: DataType,
        actual: DataType,
    },
    #[error("inline strings are depricated and encoding is unsupported. Use offset strings")]
    InlineStringUnsupported,
    #[error("the table has too many columns, a maximum of 65536 columns is allowed")]
    TooManyColumns,
}
