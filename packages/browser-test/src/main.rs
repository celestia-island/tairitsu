//! CLI entry point for tairitsu-browser-test

use clap::Parser;
use tairitsu_browser_test::cli::Cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cli = Cli::parse();
    cli.run().await
}
