mod archive;
mod header;
mod node;
mod string_table;

pub use archive::Archive;
pub use node::FileAttributes;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RarcError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    PodCastError(#[from] bytemuck::PodCastError),
    #[error("Rarc file has an invaild magic number")]
    InvaildMagic,
    #[error("The first directory node in not the root directory")]
    FirstDirectoryNotRoot,
    #[error("A file in a directory is missing. Expected Offset: {index}")]
    MissingFile { index: usize },
}

type Result<T> = std::result::Result<T, RarcError>;
