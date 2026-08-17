use std::ffi::NulError;
use std::io;
use std::path::PathBuf;
use std::str::Utf8Error;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum FlatError {
    #[error(transparent)]
    Io(#[from] io::Error),

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
pub enum FfiError {
    #[error(transparent)]
    Library(#[from] libloading::Error),

    #[error(transparent)]
    Nul(#[from] NulError),

    #[error(transparent)]
    Utf8(#[from] Utf8Error),

    #[error("Flatbuffer plugin returned a null {0} pointer")]
    NullResult(&'static str),

    #[error("Flatbuffer plugin sink version mismatch: {0}")]
    SinkVersion(u32),

    #[error("Flatbuffer plugin sink size mismatch: {0}")]
    SinkSize(u32),

    #[error("Flatbuffer plugin does not support sink streaming")]
    SinkUnsupported,

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
