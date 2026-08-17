use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use baad_utils::info;
use fastcat::fconcat;
use tokio::fs;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::error::ExtractError;
use crate::extractors::pack::PackFile;

pub async fn convert_pack(
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

    let pack = PackFile::open(input)?;
    let zip_path = output.join(fconcat!(filename.trim_end_matches(".molru"), ".zip"));
    let file = BufWriter::with_capacity(1024 * 1024, File::create(&zip_path)?);
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

    for (name, data) in pack.entries() {
        zip.start_file(name, options)?;
        zip.write_all(data)?;
    }

    zip.finish()?;

    info!(success = true, filename, "Converted pack");
    Ok(())
}
