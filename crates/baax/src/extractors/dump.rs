use std::path::Path;

use baad_utils::{info, warn};
use baax_plugin::error::FfiError;
use baax_plugin::loader::{Api, api};

use crate::converters::output::{Output, OutputFormat, write};
use crate::converters::sheet::Sheet;
use crate::error::ExtractError;

pub async fn dump_bytes(
    dir: impl AsRef<Path>,
    name: &str,
    bytes: &mut [u8],
    format: OutputFormat
) -> Result<bool, ExtractError> {
    let api = api()?;
    let Some(table) = api.resolve_table(name)? else {
        warn!(file = name, "No flatbuffer table matches, keeping raw bytes");
        return Ok(false);
    };

    let output = match table_output(api, &table, bytes, format) {
        Ok(output) => output,
        Err(error) => {
            warn!(table, cause = %error, "Failed to decode flatbuffer");
            return Ok(false);
        }
    };

    write(dir, &table, output).await?;

    info!(success = true, table, "Dumped");
    Ok(true)
}

pub async fn dump_db_table(
    dir: impl AsRef<Path>,
    table_name: &str,
    blobs: &[Vec<u8>],
    format: OutputFormat
) -> Result<bool, ExtractError> {
    let api = api()?;
    let Some(row_type) = api.resolve_row(table_name)? else {
        warn!(table_name, "No flatbuffer schema matches, keeping raw bytes");
        return Ok(false);
    };

    let rows = blobs.iter().map(Vec::as_slice).collect::<Vec<_>>();
    let output = match rows_output(api, &row_type, &rows, format) {
        Ok(output) => output,
        Err(error) => {
            warn!(table = row_type, cause = %error, "Failed to decode flatbuffer");
            return Ok(false);
        }
    };

    write(dir, &row_type, output).await?;

    info!(success = true, row_type, "Dumped");
    Ok(true)
}

fn table_output(
    api: &Api,
    table: &str,
    bytes: &mut [u8],
    format: OutputFormat
) -> Result<Output, FfiError> {
    if !sinkable(api, format)? {
        return api.dump_table(table, bytes).map(Output::Json);
    }

    let mut sheet = Sheet::new();
    api.visit_table(table, bytes, &mut sheet)?;

    Ok(Output::Sheet(sheet))
}

fn rows_output(
    api: &Api,
    row_type: &str,
    rows: &[&[u8]],
    format: OutputFormat
) -> Result<Output, FfiError> {
    if !sinkable(api, format)? {
        return api.dump_rows(row_type, rows).map(Output::Json);
    }

    let mut sheet = Sheet::new();
    api.visit_rows(row_type, rows, &mut sheet)?;

    Ok(Output::Sheet(sheet))
}

fn sinkable(api: &Api, format: OutputFormat) -> Result<bool, FfiError> {
    if format == OutputFormat::Json {
        return Ok(false);
    }

    if api.supports_sink() { Ok(true) } else { Err(FfiError::SinkUnsupported) }
}
