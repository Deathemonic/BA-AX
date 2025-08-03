use crate::cli::{Cli, Commands, ExtractType, MediaArgs, TableArgs};
use baax::extractors::db::extract_db;
use baax::extractors::zip::{extract, extract_zip};
use baax::extractors::ExtractionMode;

use anyhow::{anyhow, Result};
use baad_core::config::{init_logging, LoggingConfig};
use baad_core::info;
use clap::Parser;
use tokio::fs;

pub async fn run() -> Result<()> {
    let args = Cli::parse();

    let mut logging_config = LoggingConfig::default();
    if !args.verbose {
        logging_config.verbose_mode = false;
        logging_config.enable_debug = false;
    }
    init_logging(logging_config)?;

    match args.command {
        Commands::Extract(extract_args) => match extract_args.extract_type {
            ExtractType::Media(media_args) => {
                handle_media(media_args).await?;
            }
            ExtractType::Table(table_args) => {
                handle_table(table_args).await?;
            }
        },
    }

    Ok(())
}

async fn handle_media(media_args: MediaArgs) -> Result<()> {
    info!("Extracting MediaResources...");

    if !media_args.output.exists() {
        fs::create_dir_all(&media_args.output).await?;
    }

    let metadata = fs::metadata(&media_args.input).await?;

    if metadata.is_file() {
        extract_zip(media_args.input, media_args.output, true).await?;
    } else if metadata.is_dir() {
        extract(media_args.input, media_args.output, ExtractionMode::MediaResources, true).await?;
    }

    Ok(())
}

async fn handle_table(table_args: TableArgs) -> Result<()> {
    info!("Extracting Tables...");

    if !table_args.output.exists() {
        fs::create_dir_all(&table_args.output).await?;
    }

    let metadata = fs::metadata(&table_args.input).await?;

    if metadata.is_file() {
        let extension = table_args
            .input
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        match extension {
            "zip" => {
                extract_zip(table_args.input, table_args.output, false).await?;
            }
            "db" => {
                extract_db(table_args.input, table_args.output).await?;
            }
            _ => {
                return Err(anyhow!("Unsupported file type: {}", extension));
            }
        }
    } else if metadata.is_dir() {
        extract(table_args.input, table_args.output, ExtractionMode::Tables, false).await?;
    }

    Ok(())
}
