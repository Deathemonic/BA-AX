use std::io;

use baax_plugin::FfiError;
use thiserror::Error;
use zip::result::ZipError;

#[derive(Error, Debug)]
pub enum TableZipError {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Zip(#[from] ZipError),

    #[error(transparent)]
    Base64Encode(#[from] base64::EncodeSliceError)
}

#[derive(Error, Debug)]
pub enum ExtractError {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Ffi(#[from] FfiError),

    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),

    #[error(transparent)]
    TableZip(#[from] TableZipError),

    #[error(transparent)]
    Zip(#[from] ZipError),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Xlsx(#[from] rust_xlsxwriter::XlsxError),

    #[error("Failed to get file extension")]
    FileExtension,

    #[error("Failed to get filename from path")]
    FileName,

    #[error("Failed to convert file to string")]
    FromString,

    #[error("Invalid format")]
    InvalidFormat,

    #[error("Decryption failed")]
    Crypto,

    #[error("Database is encrypted provide a key")]
    SqlCipherRequired,

    #[error("Could not decrypt database")]
    SqlCipherKey,

    #[error("Unsupported file type")]
    UnsupportedFileType
}
