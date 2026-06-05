mod cli;

use baad_core::config::{LoggingConfig, init_logging};
use clap::Parser;
use cli::{Args, parse};
use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let config = LoggingConfig {
        verbose_mode: args.verbose,
        enable_debug: args.verbose,
        enable_async_writer: false,
        ..LoggingConfig::default()
    };
    init_logging(config)?;

    parse::run(args).await
}
