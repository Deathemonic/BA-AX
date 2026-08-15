use std::path::Path;

use fastcat::fconcat;
use strum::{Display, EnumString};
use tokio::fs;

use crate::converters::xlsx::to_xlsx;
use crate::error::ExtractError;

const SUFFIXES: [&str; 5] = ["ExcelTable", "DBSchema", "Excel", "Table", "DB"];

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Json,
    Xlsx
}

impl OutputFormat {
    fn encode(self, json: String, name: &str) -> Result<Vec<u8>, ExtractError> {
        match self {
            Self::Json => Ok(json.into_bytes()),
            Self::Xlsx => to_xlsx(&json, name)
        }
    }

    const fn extension(self) -> &'static str {
        match self {
            Self::Json => ".json",
            Self::Xlsx => ".xlsx"
        }
    }

    fn stem(self, name: &str) -> &str {
        match self {
            Self::Json => name,
            Self::Xlsx => base_name(name)
        }
    }
}

pub fn base_name(name: &str) -> &str {
    SUFFIXES
        .iter()
        .find_map(|suffix| name.strip_suffix(suffix).filter(|base| !base.is_empty()))
        .unwrap_or(name)
}

pub async fn write(
    dir: impl AsRef<Path>,
    name: &str,
    json: String,
    format: OutputFormat
) -> Result<(), ExtractError> {
    let stem = format.stem(name);
    let bytes = format.encode(json, stem)?;
    let path = dir.as_ref().join(fconcat!(stem, format.extension()));

    Ok(fs::write(path, bytes).await?)
}
