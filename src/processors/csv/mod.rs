mod deserialize;
pub mod reader;

use std::io;
use thiserror::Error;

pub type CsvProcessorResult<T> = Result<T, CsvProcessorError>;

#[derive(Error, Debug)]
pub enum CsvProcessorError {
    #[error("error processing csv: {0}")]
    Csv(#[from] csv::Error),

    #[error("error processing csv: {0}")]
    Io(#[from] io::Error),

    #[error("invalid format: {0}")]
    InvalidFormat(String),
}
