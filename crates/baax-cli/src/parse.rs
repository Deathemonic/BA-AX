use baad_utils::info;
use baax::extractors::zip::{extract, extract_file};
use baax::extractors::{ExtractOptions, ExtractionMode};
use baax::loader;
use clap::CommandFactory;
use eyre::Result;
use tokio::fs;

use crate::args::{Args, Commands, ExtractType, MediaArgs, PackArgs, TableArgs};

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
        let mode = ExtractionMode::MediaResources;
        info!("Extracting {}...", mode);

        if !args.base.output.exists() {
            fs::create_dir_all(&args.base.output).await?;
        }

        let metadata = fs::metadata(&args.base.input).await?;
        let options = ExtractOptions::new(mode).with_lowercase(true);

        if metadata.is_file() {
            extract_file(args.base.input, &args.base.output, options).await?;
        } else if metadata.is_dir() {
            extract(args.base.input, args.base.output, options).await?;
        }

        Ok(())
    }

    async fn execute_table_extraction(args: TableArgs) -> Result<()> {
        let mode = ExtractionMode::Tables;
        info!("Extracting {}...", mode);

        if !args.base.output.exists() {
            fs::create_dir_all(&args.base.output).await?;
        }

        if let Some(path) = args.flatbuffer.as_deref() {
            loader::load(path)?;
            info!(version = loader::version()?, "Loaded flatbuffer plugin");
        }

        let metadata = fs::metadata(&args.base.input).await?;
        let options = ExtractOptions::new(mode)
            .with_key(args.key.as_deref())
            .with_license(args.license.as_deref())
            .with_flatbuffer(args.flatbuffer.is_some());

        if metadata.is_file() {
            extract_file(args.base.input, &args.base.output, options).await?;
        } else if metadata.is_dir() {
            extract(args.base.input, args.base.output, options).await?;
        }

        Ok(())
    }

    async fn execute_pack_extraction(args: PackArgs) -> Result<()> {
        let mode = ExtractionMode::Packs;
        info!("Extracting {}...", mode);

        if !args.base.output.exists() {
            fs::create_dir_all(&args.base.output).await?;
        }

        let metadata = fs::metadata(&args.base.input).await?;
        let options = ExtractOptions::new(mode);

        if metadata.is_file() {
            extract_file(args.base.input, &args.base.output, options).await?;
        } else if metadata.is_dir() {
            extract(args.base.input, args.base.output, options).await?;
        }

        Ok(())
    }
}

pub async fn run(args: Args) -> Result<()> {
    let handler = CommandHandler::new(args)?;
    handler.handle().await
}
