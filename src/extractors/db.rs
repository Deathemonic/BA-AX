use crate::helpers::ExtractError;

use baad_core::{debug, error, info, warn};
use std::path::Path;
use tokio::fs;

pub async fn extract_db<P1: AsRef<Path>, P2: AsRef<Path>>(
    path: P1,
    output: P2,
) -> Result<(), ExtractError> {
    use rusqlite::Connection;

    let path = path.as_ref();
    let filename = path
        .file_name()
        .ok_or(ExtractError::FileName)?
        .to_str()
        .ok_or(ExtractError::FromString)?;

    let dir = output.as_ref().join(filename.trim_end_matches(".db"));

    debug!(from=filename, to=%dir.display(), "Extracting SQLite DB");

    fs::create_dir_all(&dir).await?;

    let conn = Connection::open(path)?;

    let mut stmt = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%';",
    )?;
    let table_names: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, rusqlite::Error>>()?;

    info!("Found {} tables in database", table_names.len());

    for table_name in table_names {
        debug!("Processing table: {}", table_name);

        match extract_db_bytes(&conn, &table_name, &dir).await {
            Ok(count) => {
                info!(table = table_name, count, "Extracted table successfully");
            }
            Err(e) => {
                error!(table=table_name, error=%e, "Failed to extract table");
            }
        }
    }

    info!(success = true, filename, "Extracted SQLite DB");
    Ok(())
}

async fn extract_db_bytes(
    conn: &rusqlite::Connection,
    table_name: &str,
    output_dir: &Path,
) -> Result<usize, ExtractError> {
    let query = format!("SELECT Bytes FROM '{table_name}'");
    let mut stmt = conn.prepare(&query)?;

    let mut count = 0;
    let rows = stmt.query_map([], |row| {
        let bytes: Vec<u8> = row.get(0)?;
        Ok(bytes)
    })?;

    for (index, row_result) in rows.enumerate() {
        match row_result {
            Ok(bytes) => {
                let filename = format!("{table_name}_{index:04}.bytes");
                let file_path = output_dir.join(filename);

                tokio::fs::write(file_path, bytes).await?;
                count += 1;
            }
            Err(e) => {
                warn!(table=table_name, index, error=%e, "Failed to extract row");
            }
        }
    }

    Ok(count)
}
