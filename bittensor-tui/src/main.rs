//! bittensor-tui — Bittensor Terminal UI Dashboard
//!
//! Launch with `cargo run -p bittensor-tui`.
//! Use `--network finney|test|local` and `--refresh-rate <seconds>`.

use bittensor_core::config::NetworkConfig;
use bittensor_tui::app::App;
use clap::Parser;

/// Bittensor Terminal UI Dashboard.
#[derive(Parser, Debug)]
#[command(name = "bittensor-tui", version, about = "Bittensor Terminal UI Dashboard")]
struct Args {
    /// Network to connect to.
    #[arg(long, default_value = "finney", value_parser = ["finney", "test", "local"])]
    network: String,

    /// Data refresh rate in seconds.
    #[arg(long, default_value_t = 5)]
    refresh_rate: u64,
}

fn resolve_network(name: &str) -> NetworkConfig {
    match name {
        "finney" => NetworkConfig::finney(),
        "test" => NetworkConfig::test(),
        "local" => NetworkConfig::local(),
        _ => NetworkConfig::finney(),
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let network_config = resolve_network(&args.network);

    // Set up terminal
    let mut terminal = ratatui::init();

    // Build app and run
    let mut app = App::new();
    let result = app.run(&mut terminal, network_config, args.refresh_rate).await;

    // Always restore terminal before exiting
    ratatui::restore();

    result
}
