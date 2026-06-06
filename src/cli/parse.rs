use baad_core::info;
use baax::extractors::ExtractionMode;
use baax::extractors::pack::{extract_all_packs, extract_pack};
use baax::extractors::zip::{extract, extract_file};
use clap::CommandFactory;
use eyre::Result;
use tokio::fs;

use crate::cli::args::{Args, Commands, ExtractType, MediaArgs, PackArgs, TableArgs};

pub struct CommandHandler {
    args: Args
}

impl CommandHandler {
    fn new(args: Args) -> Result<Self> { Ok(Self { args }) }

    async fn handle(self) -> Result<()> {
        let Some(command) = self.args.command else {
            Args::command().print_help()?;
            std::process::exit(0);
        };

        match command {
            Commands::Extract { extract_type } => Self::handle_extract(extract_type).await
        }
    }

    async fn handle_extract(extract_type: ExtractType) -> Result<()> {
        match extract_type {
            ExtractType::Media(media_args) => Self::execute_media_extraction(media_args).await,
            ExtractType::Table(table_args) => Self::execute_table_extraction(table_args).await,
            ExtractType::Pack(pack_args) => Self::execute_pack_extraction(pack_args).await
        }
    }

    async fn execute_media_extraction(args: MediaArgs) -> Result<()> {
        info!("Extracting MediaResources...");

        if !args.base.output.exists() {
            fs::create_dir_all(&args.base.output).await?;
        }

        let metadata = fs::metadata(&args.base.input).await?;

        if metadata.is_file() {
            extract_file(
                args.base.input,
                &args.base.output,
                ExtractionMode::MediaResources,
                true,
                None,
                None
            )
            .await?;
        } else if metadata.is_dir() {
            extract(
                args.base.input,
                args.base.output,
                ExtractionMode::MediaResources,
                true,
                None,
                None
            )
            .await?;
        }

        Ok(())
    }

    async fn execute_table_extraction(args: TableArgs) -> Result<()> {
        info!("Extracting Tables...");

        if !args.base.output.exists() {
            fs::create_dir_all(&args.base.output).await?;
        }

        let metadata = fs::metadata(&args.base.input).await?;

        if metadata.is_file() {
            extract_file(
                args.base.input,
                &args.base.output,
                ExtractionMode::Tables,
                false,
                args.key.as_deref(),
                args.license.as_deref()
            )
            .await?;
        } else if metadata.is_dir() {
            extract(
                args.base.input,
                args.base.output,
                ExtractionMode::Tables,
                false,
                args.key.as_deref(),
                args.license.as_deref()
            )
            .await?;
        }

        Ok(())
    }

    async fn execute_pack_extraction(args: PackArgs) -> Result<()> {
        info!("Extracting Packs...");

        if !args.base.output.exists() {
            fs::create_dir_all(&args.base.output).await?;
        }

        let metadata = fs::metadata(&args.base.input).await?;

        if metadata.is_file() {
            extract_pack(args.base.input, args.base.output).await?;
        } else if metadata.is_dir() {
            extract_all_packs(args.base.input, args.base.output).await?;
        }

        Ok(())
    }
}

pub async fn run(args: Args) -> Result<()> {
    let handler = CommandHandler::new(args)?;
    handler.handle().await
}
