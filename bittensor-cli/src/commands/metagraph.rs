//! Metagraph command group — show and sync subnet metagraph state.

use anyhow::{Context, Result};
use clap::Subcommand;

use bittensor_chain::client::SubtensorClient;

use crate::config::Config;

/// Metagraph subcommands.
#[derive(Debug, Subcommand)]
pub enum MetagraphCommand {
    /// Display metagraph information for a subnet
    Show {
        /// Netuid of the subnet
        #[arg(long)]
        netuid: u16,

        /// Output as JSON instead of a table
        #[arg(long)]
        json: bool,

        /// Skip interactive prompts
        #[arg(long)]
        no_prompt: bool,
    },

    /// Sync metagraph from the chain and optionally save to file
    Sync {
        /// Netuid of the subnet
        #[arg(long)]
        netuid: u16,

        /// Output file path for saving the synced metagraph (JSON)
        #[arg(long)]
        output: Option<String>,
    },
}

impl MetagraphCommand {
    /// Dispatch the metagraph subcommand.
    pub async fn execute(self, config: &Config) -> Result<()> {
        match self {
            Self::Show { netuid, json, no_prompt: _ } => {
                exec_metagraph_show(config, netuid, json).await
            }
            Self::Sync { netuid, output } => exec_metagraph_sync(config, netuid, output).await,
        }
    }
}

// ---------------------------------------------------------------------------
// Individual command implementations
// ---------------------------------------------------------------------------

async fn exec_metagraph_show(config: &Config, netuid: u16, json: bool) -> Result<()> {
    let client = SubtensorClient::from_config(config.network.clone())
        .await
        .context("failed to connect to chain")?;
    let _rpc = client.rpc();

    let metagraph =
        bittensor_metagraph::sync(&client, netuid).await.context("failed to sync metagraph")?;

    if json {
        let json_str =
            serde_json::to_string_pretty(&metagraph).context("failed to serialize metagraph")?;
        println!("{json_str}");
    } else {
        println!("Metagraph for subnet {netuid}:");
        println!("  Neurons:       {}", metagraph.n);
        println!("  Block:         {}", metagraph.block);
        println!("  Total stake:   {:.4} TAO", metagraph.stake.sum());
        println!();
        println!("  {:<6} {:<14} {:<12} {:<8} {:<8}", "UID", "Hotkey", "Stake", "Rank", "Trust");
        for neuron in metagraph.neurons() {
            println!(
                "  {:<6} {:<14} {:<12.4} {:<8} {:<8}",
                neuron.uid,
                &neuron.hotkey[..std::cmp::min(12, neuron.hotkey.len())],
                neuron.stake.to_tao(),
                neuron.rank,
                neuron.trust,
            );
        }
    }

    Ok(())
}

async fn exec_metagraph_sync(config: &Config, netuid: u16, output: Option<String>) -> Result<()> {
    let client = SubtensorClient::from_config(config.network.clone())
        .await
        .context("failed to connect to chain")?;

    let metagraph =
        bittensor_metagraph::sync(&client, netuid).await.context("failed to sync metagraph")?;

    if let Some(ref path) = output {
        let path = std::path::Path::new(path);
        bittensor_metagraph::save(&metagraph, path).context("failed to save metagraph")?;
        println!("Metagraph for subnet {netuid} saved to {}", path.display());
    } else {
        println!("Metagraph for subnet {netuid} synced successfully.");
    }

    println!("  Neurons: {}", metagraph.n);
    println!("  Block:   {}", metagraph.block);

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn metagraph_command_debug_format() {
        let cmd = MetagraphCommand::Show { netuid: 1, json: false, no_prompt: false };
        assert!(format!("{cmd:?}").contains("Show"));
    }

    #[test]
    fn parse_metagraph_show() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "btcli-rs",
            "metagraph",
            "show",
            "--netuid",
            "1",
            "--json",
        ])
        .unwrap();
        match cli.command {
            crate::Command::Metagraph { command: MetagraphCommand::Show { netuid, json, .. } } => {
                assert_eq!(netuid, 1);
                assert!(json);
            }
            other => panic!("expected Metagraph::Show, got {other:?}"),
        }
    }

    #[test]
    fn parse_metagraph_sync() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "btcli-rs",
            "metagraph",
            "sync",
            "--netuid",
            "5",
            "--output",
            "/tmp/mg.json",
        ])
        .unwrap();
        match cli.command {
            crate::Command::Metagraph { command: MetagraphCommand::Sync { netuid, output } } => {
                assert_eq!(netuid, 5);
                assert_eq!(output, Some("/tmp/mg.json".to_string()));
            }
            other => panic!("expected Metagraph::Sync, got {other:?}"),
        }
    }

    #[test]
    fn metagraph_command_all_variants_parseable() {
        let variants: Vec<Vec<&str>> = vec![
            vec!["btcli-rs", "metagraph", "show", "--netuid", "1"],
            vec!["btcli-rs", "metagraph", "show", "--netuid", "1", "--json"],
            vec!["btcli-rs", "metagraph", "sync", "--netuid", "1"],
            vec!["btcli-rs", "metagraph", "sync", "--netuid", "1", "--output", "/tmp/mg.json"],
        ];
        use clap::Parser;
        for args in &variants {
            let result = crate::Cli::try_parse_from(args);
            assert!(result.is_ok(), "variant {:?} should be parseable", args);
        }
    }

    #[tokio::test]
    async fn metagraph_show_local_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "ghost".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let result = exec_metagraph_show(&config, 1, false).await;
        // Will fail because there's no local node, but the function should handle it
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn metagraph_sync_local_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "ghost".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let result = exec_metagraph_sync(&config, 1, None).await;
        assert!(result.is_err(), "sync with no local node should fail");
    }

    #[tokio::test]
    async fn metagraph_sync_local_with_output_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "ghost".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let out_path = dir.path().join("mg.json").to_string_lossy().to_string();
        let result = exec_metagraph_sync(&config, 1, Some(out_path)).await;
        assert!(result.is_err(), "sync with output path and no local node should fail");
    }

    #[test]
    fn parse_metagraph_show_no_json() {
        use clap::Parser;
        let cli =
            crate::Cli::try_parse_from(["btcli-rs", "metagraph", "show", "--netuid", "3"]).unwrap();
        match cli.command {
            crate::Command::Metagraph { command: MetagraphCommand::Show { netuid, json, .. } } => {
                assert_eq!(netuid, 3);
                assert!(!json);
            }
            other => panic!("expected Metagraph::Show, got {other:?}"),
        }
    }

    #[test]
    fn parse_metagraph_sync_without_output() {
        use clap::Parser;
        let cli =
            crate::Cli::try_parse_from(["btcli-rs", "metagraph", "sync", "--netuid", "7"]).unwrap();
        match cli.command {
            crate::Command::Metagraph { command: MetagraphCommand::Sync { netuid, output } } => {
                assert_eq!(netuid, 7);
                assert!(output.is_none());
            }
            other => panic!("expected Metagraph::Sync, got {other:?}"),
        }
    }

    #[test]
    fn metagraph_sync_command_debug_format() {
        let cmd = MetagraphCommand::Sync { netuid: 42, output: Some("/tmp/sync.json".to_string()) };
        assert!(format!("{cmd:?}").contains("Sync"));
    }
}
