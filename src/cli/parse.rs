use crate::cli::{Cli, Commands, ExtractType};
use crate::extractors::media::{extract_media, extract_zip};

use anyhow::Result;
use clap::Parser;
use tokio::fs;

pub async fn run() -> Result<()> {
    let args = Cli::parse();
    
    match args.command {
        Commands::Extract(extract_args) => {
            match extract_args.extract_type {
                ExtractType::Media(media_args) => {
                    if !media_args.output.exists() {
                        fs::create_dir_all(&media_args.output).await?;
                    }
                    
                    let metadata = fs::metadata(&media_args.input).await?;
                    
                    if metadata.is_file() {
                        extract_zip(media_args.input, &media_args.output).await?;
                    } else if metadata.is_dir() {
                        extract_media(media_args.input, media_args.output).await?;
                    }
                }
            }
        }
    }
    
    Ok(())
}