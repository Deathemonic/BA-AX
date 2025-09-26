use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "baax")]
#[command(about = "Blue Archive Asset Extractor")]
#[command(version)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    Extract {
        #[command(subcommand)]
        extract_type: ExtractType,
    },
}

#[derive(Subcommand)]
pub enum ExtractType {
    /// Extract media resources
    Media(MediaArgs),
    /// Extract table data
    Table(TableArgs),
}

#[derive(Parser)]
pub struct BaseExtractArgs {
    // Input file or folder to extract media from
    #[arg(short, long, value_name = "INPUT")]
    pub input: PathBuf,

    /// Output file or folder for extracted media
    #[arg(short, long, value_name = "OUTPUT")]
    pub output: PathBuf,
}

#[derive(Parser)]
pub struct MediaArgs {
    #[command(flatten)]
    pub base: BaseExtractArgs,
}

#[derive(Parser)]
pub struct TableArgs {
    #[command(flatten)]
    pub base: BaseExtractArgs,
}