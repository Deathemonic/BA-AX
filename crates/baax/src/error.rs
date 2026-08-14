use thiserror::Error;

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
pub enum ExtractError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

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

    #[error("Unsupported file type: {0}")]
    UnsupportedFileType(String)
}