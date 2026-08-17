use std::path::Path;
use std::process;

use baad_utils::info;
use baax::converters::flatbuffer::convert_flatbuffer;
use baax::converters::pack::convert_pack;
use baax::extractors::zip::{extract, extract_file};
use baax::extractors::{ExtractOptions, ExtractionMode};
use baax::loader;
use clap::CommandFactory;
use eyre::Result;
use tokio::fs;

use crate::args::{
    Args,
    BaseExtractArgs,
    Commands,
    ConvertType,
    ExtractType,
    FlatbufferArgs,
    MediaArgs,
    TableArgs
};

pub struct CommandHandler {
    args: Args
}

impl CommandHandler {
    const fn new(args: Args) -> Self { Self { args } }

    async fn handle(self) -> Result<()> {
        let Some(command) = self.args.command else {
            Args::command().print_help()?;
            process::exit(0);
        };

        match command {
            Commands::Extract { extract_type } => Self::handle_extract(extract_type).await,
            Commands::Convert { convert_type } => Self::handle_convert(convert_type).await
        }
    }

    async fn handle_extract(extract_type: ExtractType) -> Result<()> {
        match extract_type {
            ExtractType::Media(media_args) => Self::execute_media_extraction(media_args).await,
            ExtractType::Table(table_args) => Self::execute_table_extraction(table_args).await
        }
    }

    async fn handle_convert(convert_type: ConvertType) -> Result<()> {
        match convert_type {
            ConvertType::Flatbuffers(flatbuffer_args) => {
                Self::execute_flatbuffer_conversion(flatbuffer_args).await
            }
            ConvertType::Pack(pack_args) => Self::execute_pack_conversion(pack_args).await
        }
    }

    async fn execute_media_extraction(args: MediaArgs) -> Result<()> {
        if !args.base.output.exists() {
            fs::create_dir_all(&args.base.output).await?;
        }

        let options = ExtractOptions::new(ExtractionMode::MediaResources)
            .with_lowercase(true)
            .with_format(args.format.into());

        info!("Extracting {}...", options.mode);
        Self::execute_extraction(&args.base.input, &args.base.output, options).await?;

        Ok(())
    }

    async fn execute_table_extraction(args: TableArgs) -> Result<()> {
        let mode = ExtractionMode::Tables;
        info!("Extracting {}...", mode);

        if !args.base.output.exists() {
            fs::create_dir_all(&args.base.output).await?;
        }

        if let Some(path) = args.flatbuffers.as_deref() {
            loader::load(path)?;
            info!(version = loader::version()?, "Loaded flatbuffer plugin");
        }

        let options = ExtractOptions::new(mode)
            .with_key(args.key.as_deref())
            .with_license(args.license.as_deref())
            .with_flatbuffer(args.flatbuffers.is_some())
            .with_output(args.format.into());

        Self::execute_extraction(&args.base.input, &args.base.output, options).await?;

        Ok(())
    }

    async fn execute_flatbuffer_conversion(args: FlatbufferArgs) -> Result<()> {
        loader::load(&args.flat)?;
        info!(version = loader::version()?, "Loaded flatbuffer plugin");
        convert_flatbuffer(&args.base.input, &args.base.output, args.format.into()).await?;

        Ok(())
    }

    async fn execute_pack_conversion(args: BaseExtractArgs) -> Result<()> {
        convert_pack(&args.input, &args.output).await?;

        Ok(())
    }

    async fn execute_extraction(
        input: impl AsRef<Path>,
        output: impl AsRef<Path>,
        options: ExtractOptions<'_>
    ) -> Result<()> {
        let metadata = fs::metadata(&input).await?;

        if metadata.is_file() {
            extract_file(input, output, options).await?;
        } else if metadata.is_dir() {
            extract(input, output, options).await?;
        }

        Ok(())
    }
}

pub async fn run(args: Args) -> Result<()> {
    let handler = CommandHandler::new(args);
    handler.handle().await
}
