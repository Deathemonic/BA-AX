use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "baax")]
#[command(about = "Blue Archive Asset Extractor")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Extract(ExtractArgs),
}

#[derive(Args)]
pub struct ExtractArgs {
    #[command(subcommand)]
    pub extract_type: ExtractType,
}

#[derive(Subcommand)]
pub enum ExtractType {
    Media(MediaArgs),
}

#[derive(Args)]
pub struct MediaArgs {
    /// Input file or folder to extract media from
    #[arg(short, long, value_name = "INPUT")]
    pub input: PathBuf,

    /// Output file or folder for extracted media
    #[arg(short, long, value_name = "OUTPUT")]
    pub output: PathBuf,
}