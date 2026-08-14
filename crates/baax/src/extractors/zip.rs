use std::path::Path;

use baad_utils::{debug, info};
use tokio::fs;

use crate::error::ExtractError;
use crate::extractors::db::extract_db;
use crate::extractors::options::{ExtractOptions, ExtractionMode};
use crate::extractors::table::TableZipFile;

pub async fn extract_zip(
    path: impl AsRef<Path>,
    output: impl AsRef<Path>,
    lowercase: bool
) -> Result<(), ExtractError> {
    let path = path.as_ref();
    let buf = fs::read(path).await?;
    let filename =
        path.file_name().ok_or(ExtractError::FileName)?.to_str().ok_or(ExtractError::FromString)?;

    let zip_filename = if lowercase { filename.to_lowercase() } else { filename.to_string() };

    let mut zip = TableZipFile::new(buf, zip_filename)?;
    let dir = output.as_ref().join(filename.trim_end_matches(".zip"));

    debug!(from=filename, to=%dir.display(), "Extracting");

    fs::create_dir_all(&dir).await?;

    for (name, buf) in zip.extract_all()? {
        fs::write(dir.join(name), buf).await?;
    }

    info!(success = true, filename, "Extracted");
    Ok(())
}

pub async fn extract(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
    options: ExtractOptions<'_>
) -> Result<(), ExtractError> {
    info!("Extracting {:?}...", options.mode);

    for entry in input.as_ref().read_dir()? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && supports_file(&path, options.mode) {
            extract_supported_file(path, &output, options).await?;
        }
    }

    Ok(())
}

pub async fn extract_file(
    path: impl AsRef<Path>,
    output: impl AsRef<Path>,
    options: ExtractOptions<'_>
) -> Result<(), ExtractError> {
    let path_ref = path.as_ref();

    if !supports_file(path_ref, options.mode) {
        let extension = path_ref.extension().and_then(|ext| ext.to_str()).unwrap_or("");
        return Err(ExtractError::UnsupportedFileType(extension.to_string()));
    }

    extract_supported_file(path, output, options).await?;
    Ok(())
}

fn supports_file(path: &Path, mode: ExtractionMode) -> bool {
    matches!(
        (path.extension().and_then(|ext| ext.to_str()), mode),
        (Some("zip"), _) | (Some("db"), ExtractionMode::Tables)
    )
}

async fn extract_supported_file(
    path: impl AsRef<Path>,
    output: impl AsRef<Path>,
    options: ExtractOptions<'_>
) -> Result<(), ExtractError> {
    match path.as_ref().extension().and_then(|ext| ext.to_str()) {
        Some("zip") => extract_zip(path, output, options.lowercase).await,
        Some("db") if options.mode == ExtractionMode::Tables => {
            extract_db(path, output, options.key, options.license).await
        }
        _ => unreachable!("extract_supported_file called with unsupported file")
    }
}