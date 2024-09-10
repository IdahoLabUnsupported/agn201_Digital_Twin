use std::io;
use thiserror::Error;
use tokio::task::JoinError;

// Project specific errors and wrappers of other libraries errors so we can always return ours but
// still be able to use ? notation
#[derive(Error, Debug)]
pub enum MachineLearningError {
    #[error("unwrap option error")]
    UnwrapOption,
    #[error("database error")]
    Database,
    #[error("thread error {0}")]
    Thread(String),
    #[error("thread join error {0}")]
    ThreadJoin(#[from] JoinError),
    #[error("no data sources provided")]
    NoDataSources,
    #[error("io error {0}")]
    IO(#[from] io::Error),
    #[error("duckdb underlying error: {0}")]
    DuckDB(#[from] duckdb::Error),
    #[error("logger error")]
    Logger(#[from] log::SetLoggerError),
    #[error("deeplynx api error: {0}")]
    API(#[from] crate::deep_lynx::APIError),
    #[error("yaml parsing error: {0}")]
    YAMLParsing(#[from] serde_yaml::Error),
    #[error("number parsing error: {0}")]
    NumberParsing(#[from] std::num::ParseIntError),
    #[error("blob conversion error, THIS SHOULD NEVER HAPPEN")]
    BlobConversion(#[from] std::str::Utf8Error),
}
