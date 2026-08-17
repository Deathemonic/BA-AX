use std::path::PathBuf;

use baax::converters::OutputFormat;
use baax::extractors::ArchiveFormat;
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
    },

    Convert {
        #[command(subcommand)]
        convert_type: ConvertType
    }
}

#[derive(Subcommand)]
pub enum ConvertType {
    /// Convert FlatBuffer bytes to JSON
    Flatbuffers(FlatbufferArgs),

    /// Convert molru pack to zip
    Pack(BaseExtractArgs)
}

#[derive(Subcommand)]
pub enum ExtractType {
    /// Extract media resources
    Media(MediaArgs),

    /// Extract table data
    Table(TableArgs)
}

#[derive(Debug, Default, Clone, Copy, ValueEnum)]
pub enum Format {
    /// Detect the archive format from the file extension
    #[default]
    Auto,

    /// Force zip extraction
    Zip,

    /// Force molru extraction
    Pack
}

impl From<Format> for ArchiveFormat {
    fn from(format: Format) -> Self {
        match format {
            Format::Auto => Self::Auto,
            Format::Zip => Self::Zip,
            Format::Pack => Self::Pack
        }
    }
}

#[derive(Debug, Default, Clone, Copy, ValueEnum)]
pub enum DataFormat {
    /// Write decoded tables as JSON
    #[default]
    Json,

    /// Write decoded tables as Excel spreadsheets
    Xlsx
}

impl From<DataFormat> for OutputFormat {
    fn from(format: DataFormat) -> Self {
        match format {
            DataFormat::Json => Self::Json,
            DataFormat::Xlsx => Self::Xlsx
        }
    }
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
    pub base: BaseExtractArgs,

    /// Archive format to extract
    #[arg(short, long, value_name = "FORMAT", default_value = "auto")]
    pub format: Format
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
    pub flatbuffers: Option<PathBuf>,

    /// Output format for decoded tables
    #[arg(long, value_name = "FORMAT", default_value = "json", requires = "flatbuffer")]
    pub format: DataFormat
}

#[derive(Parser)]
pub struct FlatbufferArgs {
    #[command(flatten)]
    pub base: BaseExtractArgs,

    /// Flatbuffer plugin library or .flat bundle
    #[arg(long, value_name = "LIBRARY")]
    pub flat: PathBuf,

    /// Output format for decoded tables
    #[arg(long, value_name = "FORMAT", default_value = "json", requires = "flat")]
    pub format: DataFormat
}
