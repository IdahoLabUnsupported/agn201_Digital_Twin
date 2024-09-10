use jester_core::errors::ProcessorError;
use std::io;
use std::num::TryFromIntError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ISUProcessorError {
    #[error("unknown processor error")]
    Unknown,
    #[error("blank path supplied")]
    BlankPath,
    #[error("io error")]
    IOError(#[from] io::Error),
    #[error("runtime error")]
    ThreadError,
    #[error("sqlite error {0}")]
    SQLiteError(#[from] sqlx::Error),
    #[error("not implemented")]
    NotImplementedError,
    #[error("csv parsing error")]
    CSVError(#[from] csv::Error),
    #[error("time parsing error")]
    TimeParsingError(#[from] chrono::ParseError),
    #[error("no channel error")]
    NoChannelError,
    #[error("parse int error")]
    ParseIntError(#[from] TryFromIntError),
    #[error("rusqlite error {0}")]
    Rusqlite(#[from] rusqlite::Error),
}

impl From<ISUProcessorError> for ProcessorError {
    fn from(e: ISUProcessorError) -> Self {
        ProcessorError::PluginError(anyhow::Error::new(e))
    }
}
