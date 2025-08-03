use crate::extractors::db::extract_db;

use anyhow::{Context, Result};
use baad_core::{debug, info};
use bacy::table_zip::TableZipFile;
use std::path::Path;
use tokio::fs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractionMode {
    MediaResources,
    Tables,
}

pub async fn extract_zip<P1: AsRef<Path>, P2: AsRef<Path>>(
    path: P1,
    output: P2,
    lowercase: bool,
) -> Result<()> {
    let path = path.as_ref();
    let buf = fs::read(path).await?;
    let filename = path
        .file_name()
        .context("Failed to get filename from path")?
        .to_str()
        .context("Failed to convert filename to string")?;

    let zip_filename = if lowercase {
        filename.to_lowercase()
    } else {
        filename.to_string()
    };

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

pub async fn extract<P1: AsRef<Path>, P2: AsRef<Path>>(
    input: P1,
    output: P2,
    mode: ExtractionMode,
    lowercase: bool,
) -> Result<()> {
    info!("Extracting {:?}...", mode);

    for entry in input.as_ref().read_dir()? {
        let entry = entry?;
        let path = entry.path();

        let extension = path.extension().context("Failed to get file extension")?;

        match extension.to_str().unwrap_or("") {
            "zip" => {
                extract_zip(path, &output, lowercase).await?;
            }
            "db" if mode == ExtractionMode::Tables => {
                extract_db(path, &output).await?;
            }
            _ => {
                if mode == ExtractionMode::MediaResources && extension != "zip" {
                    continue;
                }
            }
        }
    }

    Ok(())
}
