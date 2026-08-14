use std::path::Path;
use std::str::FromStr;

use baad_utils::{debug, info};
use strum::EnumString;
use tokio::fs;

use crate::error::ExtractError;
use crate::extractors::db::extract_db;
use crate::extractors::dump::dump_bytes;
use crate::extractors::options::{ArchiveFormat, ExtractOptions, ExtractionMode};
use crate::extractors::pack::PackFile;
use crate::extractors::table::TableZipFile;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "lowercase")]
enum FileKind {
    Zip,
    Db,
    Molru
}

impl FileKind {
    fn from_path(path: &Path) -> Option<Self> {
        path.extension().and_then(|ext| ext.to_str()).and_then(|ext| Self::from_str(ext).ok())
    }

    const fn format(self) -> ArchiveFormat {
        match self {
            FileKind::Zip | FileKind::Db => ArchiveFormat::Zip,
            FileKind::Molru => ArchiveFormat::Pack
        }
    }

    fn is_supported(self, options: ExtractOptions<'_>) -> bool {
        let format_matches =
            options.format == ArchiveFormat::Auto || options.format == self.format();

        format_matches
            && matches!(
                (self, options.mode),
                (FileKind::Zip, ExtractionMode::MediaResources | ExtractionMode::Tables)
                    | (FileKind::Db, ExtractionMode::Tables)
                    | (FileKind::Molru, ExtractionMode::MediaResources | ExtractionMode::Packs)
            )
    }
}

pub async fn extract_zip(
    path: impl AsRef<Path>,
    output: impl AsRef<Path>,
    lowercase: bool,
    flatbuffer: bool
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

    for (name, mut buf) in zip.extract_all()? {
        if flatbuffer && dump_bytes(&dir, &name, &mut buf).await? {
            continue;
        }

        fs::write(dir.join(name), buf).await?;
    }

    info!(success = true, filename, "Extracted");
    Ok(())
}

pub async fn extract_pack(
    path: impl AsRef<Path>,
    output: impl AsRef<Path>
) -> Result<(), ExtractError> {
    let path = path.as_ref();
    let filename =
        path.file_name().ok_or(ExtractError::FileName)?.to_str().ok_or(ExtractError::FromString)?;

    let pack = PackFile::open(path)?;
    let dir = output.as_ref().join(filename.trim_end_matches(".molru"));

    debug!(from = filename, to = %dir.display(), "Extracting");

    fs::create_dir_all(&dir).await?;

    for (name, data) in pack.entries() {
        let out = dir.join(name);
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(out, data).await?;
    }

    info!(success = true, filename, "Extracted");
    Ok(())
}

pub async fn extract(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
    options: ExtractOptions<'_>
) -> Result<(), ExtractError> {
    info!("Extracting {}...", options.mode);

    for entry in input.as_ref().read_dir()? {
        let entry = entry?;
        let path = entry.path();

        let supported =
            path.is_file() && FileKind::from_path(&path).is_some_and(|k| k.is_supported(options));

        if supported {
            extract_file(path, &output, options).await?;
        }
    }

    Ok(())
}

pub async fn extract_file(
    path: impl AsRef<Path>,
    output: impl AsRef<Path>,
    options: ExtractOptions<'_>
) -> Result<(), ExtractError> {
    let kind = FileKind::from_path(path.as_ref()).ok_or(ExtractError::UnsupportedFileType)?;

    if !kind.is_supported(options) {
        return Err(ExtractError::UnsupportedFileType);
    }

    match kind {
        FileKind::Zip => extract_zip(path, output, options.lowercase, options.flatbuffer).await,
        FileKind::Db => extract_db(path, output, options).await,
        FileKind::Molru => extract_pack(path, output).await
    }
}
