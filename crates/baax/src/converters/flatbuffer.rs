use std::path::Path;

use baad_utils::info;
use tokio::fs;

use crate::error::ExtractError;
use crate::extractors::dump::dump_bytes;

pub async fn convert_flatbuffer(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>
) -> Result<(), ExtractError> {
    let input = input.as_ref();
    let output = output.as_ref();
    let filename = input
        .file_name()
        .ok_or(ExtractError::FileName)?
        .to_str()
        .ok_or(ExtractError::FromString)?;

    fs::create_dir_all(output).await?;

    let mut bytes = fs::read(input).await?;
    if !dump_bytes(output, filename, &mut bytes).await? {
        return Err(ExtractError::UnsupportedFileType);
    }

    info!(success = true, filename, "Converted flatbuffer");
    Ok(())
}
