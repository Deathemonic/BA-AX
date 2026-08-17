use std::path::Path;

use baad_utils::info;
use tokio::fs;

use crate::converters::output::OutputFormat;
use crate::error::ExtractError;
use crate::extractors::dump::{dump_bytes, dump_db_table};

pub async fn convert_flatbuffer(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
    format: OutputFormat
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
    if !dump(output, filename, &mut bytes, format).await? {
        return Err(ExtractError::UnsupportedFileType);
    }

    info!(success = true, filename, "Converted flatbuffer");
    Ok(())
}

async fn dump(
    output: impl AsRef<Path>,
    filename: &str,
    bytes: &mut [u8],
    format: OutputFormat
) -> Result<bool, ExtractError> {
    let table_name = filename.trim_end_matches(".bytes");
    if table_name.ends_with("DBSchema") {
        return dump_db_table(output, table_name, &split_rows(bytes), format).await;
    }

    dump_bytes(output, filename, bytes, format).await
}

fn split_rows(bytes: &[u8]) -> Vec<Vec<u8>> {
    let mut rows = Vec::new();
    let mut offset = 0_usize;

    while offset < bytes.len() {
        let Some(end) = row_end(&bytes[offset..]) else {
            return vec![bytes.to_vec()];
        };

        rows.push(bytes[offset..offset + end].to_vec());
        offset += end;
    }

    rows
}

fn row_end(bytes: &[u8]) -> Option<usize> {
    let root = u32::from_le_bytes(bytes.get(..4)?.try_into().ok()?) as usize;
    let table = root;
    let table_offset = i32::from_le_bytes(bytes.get(table..table + 4)?.try_into().ok()?) as usize;
    let vtable = table.checked_sub(table_offset)?;
    let object_size =
        u16::from_le_bytes(bytes.get(vtable + 2..vtable + 4)?.try_into().ok()?) as usize;

    table.checked_add(object_size).filter(|&end| end <= bytes.len() && end > 0)
}
