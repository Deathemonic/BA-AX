use std::path::{Path, PathBuf};

use baad_core::{debug, info, warn};
use tokio::fs;

use crate::error::ExtractError;

pub async fn extract_db<P1: AsRef<Path>, P2: AsRef<Path>>(
    path: P1,
    output: P2,
    key: Option<&str>,
    license: Option<&str>
) -> Result<(), ExtractError> {
    use rusqlite::Connection;

    let path = path.as_ref();
    let filename =
        path.file_name().ok_or(ExtractError::FileName)?.to_str().ok_or(ExtractError::FromString)?;

    let dir = output.as_ref().join(filename.trim_end_matches(".db"));

    debug!(from = filename, to = %dir.display(), "Extracting SQLite DB");

    fs::create_dir_all(&dir).await?;

    let conn = Connection::open(path)?;

    if let Some(license) = license {
        conn.execute_batch(&format!("PRAGMA cipher_license = '{license}';\n"))
            .map_err(|_| ExtractError::SqlCipherKey)?;
    }

    if let Some(key) = key {
        conn.execute_batch(&format!("PRAGMA key = \"x'{key}'\";\n"))
            .map_err(|_| ExtractError::SqlCipherKey)?;
    }

    let result = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%';");

    let mut stmt = match result {
        Ok(stmt) => stmt,
        Err(_) if key.is_some() => return Err(ExtractError::SqlCipherKey),
        Err(_) => return Err(ExtractError::SqlCipherRequired)
    };

    let table_names: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, rusqlite::Error>>()?;

    info!("Found {} tables in database", table_names.len());

    let mut writes: Vec<(PathBuf, Vec<u8>)> = Vec::with_capacity(table_names.len());

    for table_name in &table_names {
        match query_table_bytes(&conn, table_name) {
            Ok(Some(bytes)) => {
                let file_path = dir.join(format!("{table_name}.bytes"));
                writes.push((file_path, bytes));
            }
            Ok(None) => {
                warn!(table = table_name, "Table is empty, skipping");
            }
            Err(e) => {
                warn!(table = table_name, error = %e, "Failed to query table");
            }
        }
    }

    let tasks: Vec<_> = writes
        .into_iter()
        .map(|(path, bytes)| tokio::spawn(async move { fs::write(path, bytes).await }))
        .collect();

    let mut written = 0usize;
    for task in tasks {
        match task.await {
            Ok(Ok(())) => written += 1,
            Ok(Err(e)) => warn!(error = %e, "Failed to write file"),
            Err(e) => warn!(error = %e, "Task panicked")
        }
    }

    info!(success = true, filename, written, "Extracted SQLite DB");
    Ok(())
}

fn query_table_bytes(
    conn: &rusqlite::Connection,
    table_name: &str
) -> Result<Option<Vec<u8>>, ExtractError> {
    let query = format!("SELECT Bytes FROM '{table_name}' LIMIT 1");
    let mut stmt = conn.prepare(&query)?;

    let mut rows = stmt.query_map([], |row| row.get::<_, Vec<u8>>(0))?;

    match rows.next() {
        Some(Ok(bytes)) => Ok(Some(bytes)),
        Some(Err(e)) => Err(e.into()),
        None => Ok(None)
    }
}
