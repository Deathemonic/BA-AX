use baax::cli::parse;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    parse::run().await
}
