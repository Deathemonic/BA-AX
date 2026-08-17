use std::path::{Path, PathBuf};

use baad_utils::{debug, info, warn};
use fastcat::fconcat;
use rusqlite::Connection;
use tokio::fs;

use crate::error::ExtractError;
use crate::extractors::dump::dump_db_table;
use crate::extractors::options::ExtractOptions;

pub async fn extract_db(
    path: impl AsRef<Path>,
    output: impl AsRef<Path>,
    options: ExtractOptions<'_>
) -> Result<(), ExtractError> {
    let path = path.as_ref();
    let filename =
        path.file_name().ok_or(ExtractError::FileName)?.to_str().ok_or(ExtractError::FromString)?;

    let dir = output.as_ref().join(filename.trim_end_matches(".db"));

    debug!(from = filename, to = %dir.display(), "Extracting SQLite DB");

    fs::create_dir_all(&dir).await?;

    let conn = Connection::open(path)?;

    conn.execute_batch("PRAGMA cipher_log_level = NONE;").ok();

    if let Some(license) = options.license {
        conn.execute_batch(&fconcat!("PRAGMA cipher_license = '", license, "';\n"))
            .map_err(|_| ExtractError::SqlCipherKey)?;
    }

    if let Some(key) = options.key {
        conn.execute_batch(&fconcat!("PRAGMA key = \"x'", key, "'\";\n"))
            .map_err(|_| ExtractError::SqlCipherKey)?;
    }

    let result = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%';");

    let mut stmt = match result {
        Ok(stmt) => stmt,
        Err(_) if options.key.is_some() => return Err(ExtractError::SqlCipherKey),
        Err(_) => return Err(ExtractError::SqlCipherRequired)
    };

    let table_names: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, rusqlite::Error>>()?;

    info!(table = table_names.len(), "Found tables in database");

    let mut writes: Vec<(PathBuf, Vec<u8>)> = Vec::with_capacity(table_names.len());

    for table_name in &table_names {
        let blobs = match query_table_bytes(&conn, table_name) {
            Ok(blobs) if blobs.is_empty() => {
                warn!(table_name, "Table is empty, skipping");
                continue;
            }
            Ok(blobs) => blobs,
            Err(e) => {
                warn!(table = table_name, error = %e, "Failed to query table");
                continue;
            }
        };

        if options.flatbuffer && dump_db_table(&dir, table_name, &blobs, options.output).await? {
            continue;
        }

        let combined = blobs.concat();
        writes.push((dir.join(fconcat!(table_name, ".bytes")), combined));
    }

    let tasks: Vec<_> = writes
        .into_iter()
        .map(|(path, bytes)| tokio::spawn(async move { fs::write(path, bytes).await }))
        .collect();

    for task in tasks {
        match task.await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => warn!(error = %e, "Failed to write file"),
            Err(e) => warn!(error = %e, "Task panicked")
        }
    }

    info!(success = true, filename, "Extracted SQLite DB");
    Ok(())
}

fn query_table_bytes(conn: &Connection, table_name: &str) -> Result<Vec<Vec<u8>>, ExtractError> {
    let query = fconcat!("SELECT Bytes FROM '", table_name, "' ORDER BY rowid");
    let mut stmt = conn.prepare(&query)?;

    let rows = stmt.query_map([], |row| row.get::<_, Vec<u8>>(0))?;

    let mut blobs = Vec::new();
    for row in rows {
        let bytes = row?;
        if !bytes.is_empty() {
            blobs.push(bytes);
        }
    }

    Ok(blobs)
}
