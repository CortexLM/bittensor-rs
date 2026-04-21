//! Weights command group — setting and querying neuron weights on Bittensor subnets.

use anyhow::{Context, Result};
use clap::Subcommand;

use bittensor_chain::client::SubtensorClient;
use bittensor_wallet::prelude::Wallet;

use crate::config::Config;

/// Weights subcommands.
#[derive(Debug, Subcommand)]
pub enum WeightsCommand {
    /// Set weights on a subnet
    #[command(name = "set-weights")]
    SetWeights {
        /// Netuid of the subnet
        #[arg(long)]
        netuid: u16,

        /// Destination UIDs (comma-separated, e.g. "1,2,3")
        dests: String,

        /// Weight values (comma-separated, e.g. "100,200,300")
        weights: String,

        /// Version key (default 0)
        #[arg(long, default_value_t = 0)]
        version_key: u64,

        /// Wallet name override
        #[arg(long)]
        wallet_name: Option<String>,

        /// Wallet path override
        #[arg(long)]
        wallet_path: Option<String>,

        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,

        /// Password to decrypt coldkey (prompted if not provided)
        #[arg(long)]
        password: Option<String>,
    },

    /// Get weights set by a UID on a subnet
    #[command(name = "get-weights")]
    GetWeights {
        /// Netuid of the subnet
        #[arg(long)]
        netuid: u16,

        /// UID to query (omit to list all UIDs)
        #[arg(long)]
        uid: Option<u16>,
    },
}

impl WeightsCommand {
    /// Dispatch the weights subcommand.
    pub async fn execute(self, config: &Config) -> Result<()> {
        match self {
            Self::SetWeights {
                netuid,
                dests,
                weights,
                version_key,
                wallet_name,
                wallet_path,
                yes,
                password,
            } => {
                exec_set_weights(
                    config,
                    netuid,
                    &dests,
                    &weights,
                    version_key,
                    wallet_name,
                    wallet_path,
                    yes,
                    password,
                )
                .await
            }
            Self::GetWeights { netuid, uid } => exec_get_weights(config, netuid, uid).await,
        }
    }
}

// ---------------------------------------------------------------------------
// Individual command implementations
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
async fn exec_set_weights(
    config: &Config,
    netuid: u16,
    dests: &str,
    weights: &str,
    version_key: u64,
    wallet_name: Option<String>,
    wallet_path: Option<String>,
    yes: bool,
    password: Option<String>,
) -> Result<()> {
    let dest_vec = parse_comma_u16(dests)?;
    let weight_vec = parse_comma_u16(weights)?;

    if dest_vec.len() != weight_vec.len() {
        anyhow::bail!(
            "number of dests ({}) does not match number of weights ({})",
            dest_vec.len(),
            weight_vec.len()
        );
    }

    if !yes {
        println!("You are about to set weights on subnet {netuid}:");
        println!("  dests:   {dest_vec:?}");
        println!("  weights: {weight_vec:?}");
        println!("  version: {version_key}");
        println!();
        let answer = dialoguer::Confirm::new()
            .with_prompt("Proceed?")
            .default(false)
            .interact()
            .context("failed to read confirmation")?;
        if !answer {
            println!("Aborted.");
            return Ok(());
        }
    }

    let pwd = prompt_password(password)?;
    let w_name = wallet_name.unwrap_or_else(|| config.wallet_name.clone());
    let w_path = wallet_path.map(std::path::PathBuf::from).unwrap_or_else(|| config.wallet_dir());
    let mut wallet = Wallet::with_path(&w_name, w_path);
    let signer = wallet.get_coldkey_pair(&pwd).context("failed to decrypt coldkey")?;

    let client = SubtensorClient::from_config(config.network.clone())
        .await
        .context("failed to connect to chain")?;
    let rpc = client.rpc();

    let inner_kp = subxt_signer::sr25519::Keypair::from_secret_key(*signer.seed())?;

    let result = bittensor_chain::extrinsics::weights::set_weights(
        rpc,
        &inner_kp,
        netuid,
        dest_vec,
        weight_vec,
        version_key,
    )
    .await
    .context("set_weights extrinsic failed")?;

    println!("Set weights submitted successfully.");
    println!("  Netuid:         {netuid}");
    println!("  Block hash:      {}", result.block_hash);
    println!("  Extrinsic hash:  {}", result.extrinsic_hash);
    Ok(())
}

async fn exec_get_weights(config: &Config, netuid: u16, uid: Option<u16>) -> Result<()> {
    let client = SubtensorClient::from_config(config.network.clone())
        .await
        .context("failed to connect to chain")?;
    let rpc = client.rpc();

    match uid {
        Some(uid) => {
            let weights = bittensor_chain::queries::weights::get_weights(rpc, netuid, uid)
                .await
                .context("failed to query weights")?;

            println!("Weights for netuid={netuid} uid={uid}:");
            for (dest, weight) in &weights {
                println!("  dest={dest} weight={weight}");
            }
        }
        None => {
            let neuron_count = bittensor_chain::queries::get_neuron_count(rpc, netuid)
                .await
                .context("failed to query neuron count")?;

            println!("Weights for netuid={netuid} ({neuron_count} neurons):");
            for uid_val in 0..neuron_count {
                let weights = bittensor_chain::queries::weights::get_weights(rpc, netuid, uid_val)
                    .await
                    .context("failed to query weights")?;
                if !weights.is_empty() {
                    println!("  uid={uid_val}:");
                    for (dest, weight) in &weights {
                        println!("    dest={dest} weight={weight}");
                    }
                }
            }
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn prompt_password(password: Option<String>) -> Result<String> {
    match password {
        Some(p) => Ok(p),
        None => Ok(rpassword::prompt_password("Enter coldkey password: ")?),
    }
}

/// Parse a comma-separated string into a Vec<u16>.
pub(crate) fn parse_comma_u16(input: &str) -> Result<Vec<u16>> {
    let mut result = Vec::new();
    for part in input.split(',') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        let val: u16 =
            trimmed.parse().with_context(|| format!("invalid u16 value: '{trimmed}'"))?;
        result.push(val);
    }
    Ok(result)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use bittensor_wallet::prelude::Wallet;
    use tempfile::TempDir;

    #[test]
    fn weights_command_debug_format() {
        let cmd = WeightsCommand::GetWeights { netuid: 1, uid: Some(5) };
        assert!(format!("{cmd:?}").contains("GetWeights"));
    }

    #[test]
    fn parse_set_weights_command() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "btcli-rs",
            "weights",
            "set-weights",
            "--netuid",
            "3",
            "1,2,3",
            "100,200,300",
            "--version-key",
            "42",
            "--yes",
        ])
        .unwrap();
        match cli.command {
            crate::Command::Weights {
                command: WeightsCommand::SetWeights { netuid, dests, weights, version_key, yes, .. },
            } => {
                assert_eq!(netuid, 3);
                assert_eq!(dests, "1,2,3");
                assert_eq!(weights, "100,200,300");
                assert_eq!(version_key, 42);
                assert!(yes);
            }
            other => panic!("expected Weights::SetWeights, got {other:?}"),
        }
    }

    #[test]
    fn parse_get_weights_command() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "btcli-rs",
            "weights",
            "get-weights",
            "--netuid",
            "1",
            "--uid",
            "5",
        ])
        .unwrap();
        match cli.command {
            crate::Command::Weights { command: WeightsCommand::GetWeights { netuid, uid } } => {
                assert_eq!(netuid, 1);
                assert_eq!(uid, Some(5));
            }
            other => panic!("expected Weights::GetWeights, got {other:?}"),
        }
    }

    #[test]
    fn parse_get_weights_all() {
        use clap::Parser;
        let cli =
            crate::Cli::try_parse_from(["btcli-rs", "weights", "get-weights", "--netuid", "7"])
                .unwrap();
        match cli.command {
            crate::Command::Weights { command: WeightsCommand::GetWeights { netuid, uid } } => {
                assert_eq!(netuid, 7);
                assert!(uid.is_none());
            }
            other => panic!("expected Weights::GetWeights, got {other:?}"),
        }
    }

    #[test]
    fn weights_command_all_variants_parseable() {
        let variants: Vec<Vec<&str>> = vec![
            vec!["btcli-rs", "weights", "set-weights", "--netuid", "1", "1,2", "10,20", "--yes"],
            vec!["btcli-rs", "weights", "get-weights", "--netuid", "1"],
            vec!["btcli-rs", "weights", "get-weights", "--netuid", "1", "--uid", "5"],
        ];
        use clap::Parser;
        for args in &variants {
            let result = crate::Cli::try_parse_from(args);
            assert!(result.is_ok(), "variant {:?} should be parseable", args);
        }
    }

    #[test]
    fn parse_comma_u16_weights_simple() {
        let vals = parse_comma_u16("1,2,3").unwrap();
        assert_eq!(vals, vec![1, 2, 3]);
    }

    #[test]
    fn parse_comma_u16_weights_single() {
        let vals = parse_comma_u16("42").unwrap();
        assert_eq!(vals, vec![42]);
    }

    #[test]
    fn parse_comma_u16_weights_with_spaces() {
        let vals = parse_comma_u16("1, 2, 3").unwrap();
        assert_eq!(vals, vec![1, 2, 3]);
    }

    #[test]
    fn parse_comma_u16_weights_invalid_fails() {
        assert!(parse_comma_u16("1,abc,3").is_err());
    }

    #[test]
    fn parse_comma_u16_weights_empty() {
        let vals = parse_comma_u16("").unwrap();
        assert!(vals.is_empty());
    }

    #[tokio::test]
    async fn set_weights_no_wallet_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "ghost".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let result =
            exec_set_weights(&config, 1, "1,2", "10,20", 0, None, None, true, Some("pw".into()))
                .await;
        assert!(result.is_err(), "set_weights with no wallet should fail");
    }

    #[tokio::test]
    async fn set_weights_mismatched_dests_weights_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "ghost".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let result =
            exec_set_weights(&config, 1, "1,2,3", "10,20", 0, None, None, true, Some("pw".into()))
                .await;
        assert!(result.is_err(), "set_weights with mismatched dests/weights should fail");
    }

    #[tokio::test]
    async fn get_weights_all_local_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "ghost".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let result = exec_get_weights(&config, 1, None).await;
        assert!(result.is_err(), "get_weights with no local node should fail");
    }

    #[tokio::test]
    async fn get_weights_by_uid_local_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "ghost".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let result = exec_get_weights(&config, 1, Some(5)).await;
        assert!(result.is_err(), "get_weights by uid with no local node should fail");
    }

    #[test]
    fn parse_comma_u16_overflow() {
        assert!(parse_comma_u16("65536").is_err(), "u16 overflow should fail");
        assert!(parse_comma_u16("1,70000").is_err(), "u16 overflow in list should fail");
    }

    #[test]
    fn parse_comma_u16_trailing_comma() {
        let vals = parse_comma_u16("1,2,").unwrap();
        assert_eq!(vals, vec![1, 2], "trailing comma should be ignored");
    }

    #[test]
    fn parse_comma_u16_leading_comma() {
        let vals = parse_comma_u16(",1,2").unwrap();
        assert_eq!(vals, vec![1, 2], "leading comma should be ignored");
    }

    #[tokio::test]
    async fn set_weights_created_wallet_chain_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "test-wallet".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let mut wallet = Wallet::with_path(&config.wallet_name, config.wallet_dir());
        wallet.create_coldkey("").expect("create coldkey");
        wallet.create_hotkey().expect("create hotkey");
        let result =
            exec_set_weights(&config, 1, "1,2", "10,20", 0, None, None, true, Some("".into()))
                .await;
        assert!(result.is_err(), "set_weights with created wallet but no chain should fail");
    }

    #[tokio::test]
    async fn root_set_weights_created_wallet_chain_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "test-wallet".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let wallet_dir_str = config.wallet_dir().to_string_lossy().to_string();
        let mut wallet = Wallet::with_path(&config.wallet_name, config.wallet_dir());
        wallet.create_coldkey("").expect("create coldkey");
        wallet.create_hotkey().expect("create hotkey");
        let result = exec_set_weights(
            &config,
            1,
            "1,2",
            "10,20",
            0,
            Some(config.wallet_name.clone()),
            Some(wallet_dir_str),
            true,
            Some("".into()),
        )
        .await;
        assert!(result.is_err(), "root set_weights with created wallet but no chain should fail");
    }
}
