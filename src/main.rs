mod cli;

#[tokio::main]
async fn main() {
    if let Err(e) = cli::parse::run().await {
        baad_core::error::log_error_chain(&e);
        std::process::exit(1);
    }
}
