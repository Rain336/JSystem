use super::DataType;
use thiserror::Error;

pub type BcsvResult<T> = Result<T, BcsvError>;

#[derive(Debug, Error)]
pub enum BcsvError {
    #[error("Unkown data type id: {0}")]
    InvalidDataType(u8),
    #[error(
        "Row doesn't have expected length. Expcted length: {expected} Actual length: {actual}"
    )]
    InvalidRowLength { expected: usize, actual: usize },
    #[error("The value at {index} has an invalid data type. Expcted type: {expected} Actual type: {actual}",)]
    InvalidRowDataType {
        index: usize,
        expected: DataType,
        actual: DataType,
    },
}
