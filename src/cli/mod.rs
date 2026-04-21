//! CLI tool for Bittensor (btcli equivalent)
//!
//! This module provides a command-line interface for interacting with the
//! Bittensor network, similar to the Python btcli tool.
//!
//! # Commands
//!
//! - `wallet` - Wallet creation, management, and operations
//! - `stake` - Stake management (add, remove, move)
//! - `subnet` - Subnet information and registration
//! - `root` - Root network operations
//! - `weights` - Weight commit, reveal, and set operations

use clap::{Parser, Subcommand};

pub mod commands;
pub mod utils;

/// Bittensor CLI - Rust implementation
#[derive(Parser)]
#[command(name = "btcli")]
#[command(author = "Cortex Foundation")]
#[command(version = "0.1.0")]
#[command(about = "Bittensor CLI - Rust implementation", long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Network to connect to (finney, test, local, or custom URL)
    #[arg(short, long, default_value = "finney", global = true)]
    pub network: String,

    /// Custom RPC endpoint (overrides --network)
    #[arg(long, global = true)]
    pub endpoint: Option<String>,

    /// Don't prompt for confirmations (auto-approve)
    #[arg(long, global = true)]
    pub no_prompt: bool,
}

/// Available CLI commands
#[derive(Subcommand)]
pub enum Commands {
    /// Wallet operations (create, list, transfer, etc.)
    #[command(alias = "w")]
    Wallet(commands::wallet::WalletCommand),

    /// Stake operations (add, remove, move stake)
    #[command(alias = "s")]
    Stake(commands::stake::StakeCommand),

    /// Subnet operations (list, info, register)
    #[command(alias = "sn")]
    Subnet(commands::subnet::SubnetCommand),

    /// Root network operations
    #[command(alias = "r")]
    Root(commands::root::RootCommand),

    /// Weight operations (commit, reveal, set)
    #[command(alias = "wt")]
    Weights(commands::weights::WeightsCommand),
}

/// Run the CLI application
pub async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Wallet(cmd) => commands::wallet::execute(cmd.clone(), &cli).await,
        Commands::Stake(cmd) => commands::stake::execute(cmd.clone(), &cli).await,
        Commands::Subnet(cmd) => commands::subnet::execute(cmd.clone(), &cli).await,
        Commands::Root(cmd) => commands::root::execute(cmd.clone(), &cli).await,
        Commands::Weights(cmd) => commands::weights::execute(cmd.clone(), &cli).await,
    }
}
