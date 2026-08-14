use std::ffi::NulError;
use std::path::PathBuf;
use std::str::Utf8Error;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum FlatError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Utf8(#[from] Utf8Error),

    #[error("Invalid .flat container")]
    Invalid,

    #[error("Unsupported .flat version: {0}")]
    UnsupportedVersion(u16),

    #[error("No plugin for {host} (available: {available})")]
    TripleNotFound { host: Box<str>, available: Box<str> },

    #[error(".flat plugin checksum mismatch")]
    ChecksumMismatch,

    #[error("Failed to decompress .flat plugin")]
    Decompress
}

#[derive(Error, Debug)]
pub enum TableZipError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Zip(#[from] zip::result::ZipError),

    #[error(transparent)]
    Base64Encode(#[from] base64::EncodeSliceError)
}

#[derive(Error, Debug)]
pub enum FfiError {
    #[error(transparent)]
    Library(#[from] libloading::Error),

    #[error(transparent)]
    Nul(#[from] NulError),

    #[error(transparent)]
    Utf8(#[from] Utf8Error),

    #[error("Flatbuffer plugin returned a null {0} pointer")]
    NullResult(&'static str),

    #[error("Flatbuffer plugin is already loaded")]
    AlreadyLoaded,

    #[error("Flatbuffer plugin was not found: {0}")]
    NotFound(PathBuf),

    #[error("Flatbuffer plugin is not loaded")]
    NotLoaded,

    #[error("Flatbuffer plugin error: {0}")]
    Plugin(Box<str>),

    #[error(transparent)]
    Flat(#[from] FlatError)
}

#[derive(Error, Debug)]
pub enum ExtractError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Ffi(#[from] FfiError),

    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),

    #[error(transparent)]
    TableZip(#[from] TableZipError),

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
