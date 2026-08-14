use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "baax")]
#[command(about = "Blue Archive Asset Extractor")]
#[command(version)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Enable verbose output
    #[arg(short, long, value_name = "LEVEL", num_args = 0..=1, default_missing_value = "minimal", require_equals = true)]
    pub verbose: Option<VerboseLevel>
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum VerboseLevel {
    /// Enable debug logs
    Minimal,

    /// Enable trace logs
    Full
}

#[derive(Subcommand)]
pub enum Commands {
    Extract {
        #[command(subcommand)]
        extract_type: ExtractType
    }
}

#[derive(Subcommand)]
pub enum ExtractType {
    /// Extract media resources
    Media(MediaArgs),

    /// Extract table data
    Table(TableArgs),

    /// Extract pack files
    Pack(PackArgs)
}

#[derive(Parser)]
pub struct BaseExtractArgs {
    // Input file or folder to extract media from
    #[arg(short, long, value_name = "INPUT")]
    pub input: PathBuf,

    /// Output file or folder for extracted media
    #[arg(short, long, value_name = "OUTPUT")]
    pub output: PathBuf
}

#[derive(Parser)]
pub struct MediaArgs {
    #[command(flatten)]
    pub base: BaseExtractArgs
}

#[derive(Parser)]
pub struct TableArgs {
    #[command(flatten)]
    pub base: BaseExtractArgs,

    /// SQLCipher key for encrypted ExcelDB (hex string)
    #[arg(short, long, value_name = "KEY")]
    pub key: Option<String>,

    /// SQLCipher license
    #[arg(short, long, value_name = "LICENSE")]
    pub license: Option<String>,

    /// Decode FlatBuffers
    #[arg(short, long, value_name = "LIBRARY")]
    pub flatbuffer: Option<PathBuf>
}

#[derive(Parser)]
pub struct PackArgs {
    #[command(flatten)]
    pub base: BaseExtractArgs
}
