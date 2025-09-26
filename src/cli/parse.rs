use crate::cli::args::{Args, Commands, ExtractType, MediaArgs, TableArgs};
use baax::extractors::db::extract_db;
use baax::extractors::zip::{extract, extract_zip};
use baax::extractors::ExtractionMode;

use baad_core::info;
use clap::CommandFactory;
use eyre::{eyre, Result};
use tokio::fs;

pub struct CommandHandler {
    args: Args,
}

impl CommandHandler {
    fn new(args: Args) -> Result<Self> {
        Ok(Self { args })
    }

    async fn handle(&self) -> Result<()> {
        match &self.args.command {
            Some(Commands::Extract { extract_type }) => self.handle_extract(extract_type).await,
            None => {
                Args::command().print_help()?;
                std::process::exit(0);
            }
        }
    }

    async fn handle_extract(&self, extract_type: &ExtractType) -> Result<()> {
        match extract_type {
            ExtractType::Media(media_args) => self.execute_media_extraction(media_args).await,
            ExtractType::Table(table_args) => self.execute_table_extraction(table_args).await,
        }
    }

    async fn execute_media_extraction(&self, args: &MediaArgs) -> Result<()> {
        info!("Extracting MediaResources...");

        if !args.base.output.exists() {
            fs::create_dir_all(&args.base.output).await?;
        }

        let metadata = fs::metadata(&args.base.input).await?;

        if metadata.is_file() {
            extract_zip(args.base.input.clone(), args.base.output.clone(), true).await?;
        } else if metadata.is_dir() {
            extract(
                args.base.input.clone(),
                args.base.output.clone(),
                ExtractionMode::MediaResources,
                true,
            )
            .await?;
        }

        Ok(())
    }

    async fn execute_table_extraction(&self, args: &TableArgs) -> Result<()> {
        info!("Extracting Tables...");

        if !args.base.output.exists() {
            fs::create_dir_all(&args.base.output).await?;
        }

        let metadata = fs::metadata(&args.base.input).await?;

        if metadata.is_file() {
            let extension = args
                .base
                .input
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("");

            match extension {
                "zip" => {
                    extract_zip(args.base.input.clone(), args.base.output.clone(), false).await?;
                }
                "db" => {
                    extract_db(args.base.input.clone(), args.base.output.clone()).await?;
                }
                _ => {
                    return Err(eyre!("Unsupported file type: {}", extension));
                }
            }
        } else if metadata.is_dir() {
            extract(
                args.base.input.clone(),
                args.base.output.clone(),
                ExtractionMode::Tables,
                false,
            )
            .await?;
        }

        Ok(())
    }
}

pub async fn run(args: Args) -> Result<()> {
    if args.command.is_none() {
        Args::command().print_help()?;
        std::process::exit(0);
    }

    let handler = CommandHandler::new(args)?;
    handler.handle().await
}
