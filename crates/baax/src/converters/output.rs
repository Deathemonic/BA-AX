use std::path::Path;

use fastcat::fconcat;
use strum::{Display, EnumString};
use tokio::fs;

use crate::converters::sheet::Sheet;
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

pub enum Output {
    Json(String),
    Sheet(Sheet)
}

impl OutputFormat {
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

impl Output {
    const fn format(&self) -> OutputFormat {
        match self {
            Self::Json(_) => OutputFormat::Json,
            Self::Sheet(_) => OutputFormat::Xlsx
        }
    }

    fn encode(self, name: &str) -> Result<Vec<u8>, ExtractError> {
        match self {
            Self::Json(json) => Ok(json.into_bytes()),
            Self::Sheet(sheet) => to_xlsx(&sheet, name)
        }
    }
}

pub fn base_name(name: &str) -> &str {
    SUFFIXES
        .iter()
        .find_map(|suffix| name.strip_suffix(suffix).filter(|base| !base.is_empty()))
        .unwrap_or(name)
}

pub async fn write(dir: impl AsRef<Path>, name: &str, output: Output) -> Result<(), ExtractError> {
    let format = output.format();
    let stem = format.stem(name);
    let path = dir.as_ref().join(fconcat!(stem, format.extension()));
    let bytes = output.encode(stem)?;

    Ok(fs::write(path, bytes).await?)
}
