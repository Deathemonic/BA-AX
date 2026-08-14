use std::path::Path;

use baad_utils::{info, warn};
use fastcat::fconcat;
use tokio::fs;

use crate::error::ExtractError;
use crate::loader::api;

pub async fn dump_bytes(
    dir: impl AsRef<Path>,
    name: &str,
    bytes: &mut [u8]
) -> Result<bool, ExtractError> {
    let api = api()?;
    let Some(table) = api.resolve_table(name)? else {
        warn!(file = name, "No flatbuffer table matches, keeping raw bytes");
        return Ok(false);
    };

    let json = match api.dump_table(&table, bytes) {
        Ok(json) => json,
        Err(error) => {
            warn!(table, cause = %error, "Failed to decode flatbuffer");
            return Ok(false);
        }
    };

    write(dir, &table, json).await?;

    info!(success = true, table, "Dumped");
    Ok(true)
}

pub async fn dump_db_table(
    dir: impl AsRef<Path>,
    table_name: &str,
    blobs: &[Vec<u8>]
) -> Result<bool, ExtractError> {
    let api = api()?;
    let Some(row_type) = api.resolve_row(table_name)? else {
        warn!(table = table_name, "No flatbuffer schema matches, keeping raw bytes");
        return Ok(false);
    };

    let rows = blobs.iter().map(Vec::as_slice).collect::<Vec<_>>();
    let json = match api.dump_rows(&row_type, &rows) {
        Ok(json) => json,
        Err(error) => {
            warn!(table = row_type, cause = %error, "Failed to decode flatbuffer");
            return Ok(false);
        }
    };

    write(dir, &row_type, json).await?;

    info!(success = true, row_type, rows = blobs.len(), "Dumped");
    Ok(true)
}

async fn write(dir: impl AsRef<Path>, name: &str, json: String) -> Result<(), ExtractError> {
    Ok(fs::write(dir.as_ref().join(fconcat!(name, ".json")), json).await?)
}
