//! Bittensor CLI binary entrypoint.
//!
//! This is the main entry point for the btcli command-line tool,
//! providing wallet, stake, subnet, root, and weight management commands.

use bittensor_rs::cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize basic logging for CLI
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::WARN.into()),
        )
        .with_target(false)
        .init();

    // Run the CLI
    cli::run().await
}
