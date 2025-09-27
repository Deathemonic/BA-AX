use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExtractError {
    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("{0}")]
    TableZip(#[from] bacy::error::TableZipError),

    #[error("Failed to get file extension")]
    FileExtension,

    #[error("Failed to get filename from path")]
    FileName,

    #[error("Failed to convert file to string")]
    FromString
}