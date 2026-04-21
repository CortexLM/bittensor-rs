//! Transfer command group — transferring TAO between addresses.

use std::str::FromStr;

use anyhow::{Context, Result};
use clap::Subcommand;

use bittensor_chain::client::SubtensorClient;
use bittensor_wallet::prelude::Wallet;

use crate::config::Config;

/// Transfer subcommands.
#[derive(Debug, Subcommand)]
pub enum TransferCommand {
    /// Transfer TAO to another address
    Transfer {
        /// Destination SS58 address
        dest: String,

        /// Amount in TAO (e.g. "1.5")
        amount: String,

        /// Password to decrypt coldkey (prompted if not provided)
        #[arg(long)]
        password: Option<String>,
    },

    /// Batch transfer TAO to multiple recipients
    #[command(name = "multiple")]
    Multiple {
        /// Destination SS58 addresses (comma-separated)
        #[arg(long)]
        destinations: String,

        /// Amounts in TAO (comma-separated, one per destination)
        #[arg(long)]
        amounts: String,

        /// Password to decrypt coldkey (prompted if not provided)
        #[arg(long)]
        password: Option<String>,
    },
}

impl TransferCommand {
    /// Dispatch the transfer subcommand.
    pub async fn execute(self, config: &Config) -> Result<()> {
        match self {
            Self::Transfer { dest, amount, password } => {
                exec_transfer(config, &dest, &amount, password).await
            }
            Self::Multiple { destinations, amounts, password } => {
                exec_transfer_multiple(config, &destinations, &amounts, password).await
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Individual command implementations
// ---------------------------------------------------------------------------

async fn exec_transfer(
    config: &Config,
    dest: &str,
    amount: &str,
    password: Option<String>,
) -> Result<()> {
    let pwd = prompt_password(password)?;
    let mut wallet = Wallet::with_path(&config.wallet_name, config.wallet_dir());
    let signer = wallet.get_coldkey_pair(&pwd).context("failed to decrypt coldkey")?;

    let dest_account =
        subxt::utils::AccountId32::from_str(dest).context("invalid destination SS58 address")?;

    let rao = parse_tao_to_rao(amount)?;

    let client = SubtensorClient::from_config(config.network.clone())
        .await
        .context("failed to connect to chain")?;
    let rpc = client.rpc();

    let inner_kp = subxt_signer::sr25519::Keypair::from_secret_key(*signer.seed())?;

    let result = bittensor_chain::extrinsics::transfer::transfer(rpc, &inner_kp, dest_account, rao)
        .await
        .context("transfer extrinsic failed")?;

    println!("Transfer submitted successfully.");
    println!("  Block hash:      {}", result.block_hash);
    println!("  Extrinsic hash:  {}", result.extrinsic_hash);
    Ok(())
}

async fn exec_transfer_multiple(
    config: &Config,
    destinations: &str,
    amounts: &str,
    password: Option<String>,
) -> Result<()> {
    let pwd = prompt_password(password)?;
    let mut wallet = Wallet::with_path(&config.wallet_name, config.wallet_dir());
    let signer = wallet.get_coldkey_pair(&pwd).context("failed to decrypt coldkey")?;

    let dest_list: Vec<&str> = destinations.split(',').map(|s| s.trim()).collect();
    let amount_list: Vec<&str> = amounts.split(',').map(|s| s.trim()).collect();

    if dest_list.len() != amount_list.len() {
        anyhow::bail!(
            "number of destinations ({}) does not match number of amounts ({})",
            dest_list.len(),
            amount_list.len()
        );
    }

    let client = SubtensorClient::from_config(config.network.clone())
        .await
        .context("failed to connect to chain")?;
    let rpc = client.rpc();

    let inner_kp = subxt_signer::sr25519::Keypair::from_secret_key(*signer.seed())?;

    for (dest, amount) in dest_list.iter().zip(amount_list.iter()) {
        let dest_account = subxt::utils::AccountId32::from_str(dest)
            .with_context(|| format!("invalid SS58 address: {dest}"))?;

        let rao = parse_tao_to_rao(amount)?;

        let result =
            bittensor_chain::extrinsics::transfer::transfer(rpc, &inner_kp, dest_account, rao)
                .await
                .with_context(|| format!("transfer to {dest} failed"))?;

        println!("Transfer to {dest} submitted successfully.");
        println!("  Block hash:      {}", result.block_hash);
        println!("  Extrinsic hash:  {}", result.extrinsic_hash);
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

/// Parse a TAO amount string to rao (u64).
/// 1 TAO = 1_000_000_000 rao (9 decimal places).
fn parse_tao_to_rao(amount: &str) -> Result<u64> {
    let tao: f64 = amount.parse().context("invalid amount — must be a number (e.g. 1.5)")?;
    if tao < 0.0 {
        anyhow::bail!("amount must be non-negative");
    }
    let rao = (tao * 1_000_000_000.0).round() as u64;
    Ok(rao)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn parse_tao_zero() {
        assert_eq!(parse_tao_to_rao("0").unwrap(), 0);
    }

    #[test]
    fn parse_tao_one() {
        assert_eq!(parse_tao_to_rao("1").unwrap(), 1_000_000_000);
    }

    #[test]
    fn parse_tao_decimal() {
        assert_eq!(parse_tao_to_rao("1.5").unwrap(), 1_500_000_000);
    }

    #[test]
    fn parse_tao_negative_fails() {
        assert!(parse_tao_to_rao("-1.0").is_err());
    }

    #[test]
    fn parse_tao_invalid_fails() {
        assert!(parse_tao_to_rao("abc").is_err());
    }

    #[test]
    fn transfer_command_debug_format() {
        let cmd = TransferCommand::Transfer {
            dest: "5Dest".into(),
            amount: "1.0".into(),
            password: None,
        };
        assert!(format!("{cmd:?}").contains("Transfer"));
    }

    #[test]
    fn parse_transfer_command() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "btcli-rs",
            "transfer",
            "transfer",
            "5Dest123",
            "10.0",
            "--password",
            "secret",
        ])
        .unwrap();
        match cli.command {
            crate::Command::Transfer {
                command: TransferCommand::Transfer { dest, amount, password },
            } => {
                assert_eq!(dest, "5Dest123");
                assert_eq!(amount, "10.0");
                assert_eq!(password.unwrap(), "secret");
            }
            other => panic!("expected Transfer::Transfer, got {other:?}"),
        }
    }

    #[test]
    fn parse_transfer_multiple_command() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "btcli-rs",
            "transfer",
            "multiple",
            "--destinations",
            "5A,5B,5C",
            "--amounts",
            "1.0,2.0,3.0",
            "--password",
            "pw",
        ])
        .unwrap();
        match cli.command {
            crate::Command::Transfer {
                command: TransferCommand::Multiple { destinations, amounts, password },
            } => {
                assert_eq!(destinations, "5A,5B,5C");
                assert_eq!(amounts, "1.0,2.0,3.0");
                assert_eq!(password.unwrap(), "pw");
            }
            other => panic!("expected Transfer::Multiple, got {other:?}"),
        }
    }

    #[test]
    fn transfer_command_all_variants_parseable() {
        let variants: Vec<Vec<&str>> = vec![
            vec!["btcli-rs", "transfer", "transfer", "5Dest", "1.0"],
            vec![
                "btcli-rs",
                "transfer",
                "multiple",
                "--destinations",
                "5A,5B",
                "--amounts",
                "1.0,2.0",
            ],
        ];
        use clap::Parser;
        for args in &variants {
            let result = crate::Cli::try_parse_from(args);
            assert!(result.is_ok(), "variant {:?} should be parseable", args);
        }
    }

    #[tokio::test]
    async fn transfer_no_wallet_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "ghost".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let result = exec_transfer(&config, "5Dest", "1.0", Some("pw".into())).await;
        assert!(result.is_err(), "transfer with no wallet should fail");
    }

    #[tokio::test]
    async fn transfer_multiple_no_wallet_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "ghost".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let result = exec_transfer_multiple(&config, "5A,5B", "1.0,2.0", Some("pw".into())).await;
        assert!(result.is_err(), "transfer multiple with no wallet should fail");
    }

    #[tokio::test]
    async fn transfer_zero_amount_no_wallet_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "ghost".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let result = exec_transfer(&config, "5Dest", "0", Some("pw".into())).await;
        assert!(result.is_err(), "transfer with zero amount and no wallet should fail");
    }

    #[tokio::test]
    async fn transfer_multiple_zero_amounts_no_wallet_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "ghost".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let result = exec_transfer_multiple(&config, "5A,5B", "0,0", Some("pw".into())).await;
        assert!(result.is_err(), "transfer multiple with zero amounts and no wallet should fail");
    }

    #[test]
    fn parse_multiple_destinations_amounts() {
        let dests: Vec<&str> = "5A,5B,5C".split(',').map(|s| s.trim()).collect();
        let amts: Vec<&str> = "1.0,2.0,3.0".split(',').map(|s| s.trim()).collect();
        assert_eq!(dests.len(), 3);
        assert_eq!(amts.len(), 3);
        assert_eq!(dests[0], "5A");
        assert_eq!(amts[2], "3.0");
    }

    #[test]
    fn parse_mismatched_destinations_amounts() {
        let dests: Vec<&str> = "5A,5B".split(',').map(|s| s.trim()).collect();
        let amts: Vec<&str> = "1.0,2.0,3.0".split(',').map(|s| s.trim()).collect();
        assert_ne!(dests.len(), amts.len());
    }

    // --- Created-wallet chain-fails tests (exercise keypair decryption path) ---

    async fn setup_wallet(name: &str, dir: &std::path::Path) -> Config {
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: name.to_string(),
            wallet_path: dir.to_path_buf(),
        };
        let mut wallet =
            bittensor_wallet::prelude::Wallet::with_path(&config.wallet_name, config.wallet_dir());
        wallet.create_coldkey("").expect("create coldkey");
        wallet.create_hotkey().expect("create hotkey");
        config
    }

    #[tokio::test]
    async fn transfer_created_wallet_chain_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = setup_wallet("xfer-test", dir.path()).await;
        let result = exec_transfer(&config, "5DestFake", "1.0", Some("".into())).await;
        assert!(result.is_err(), "transfer with no local node should fail");
    }

    #[tokio::test]
    async fn transfer_multiple_created_wallet_chain_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = setup_wallet("xfer-multi", dir.path()).await;
        let result = exec_transfer_multiple(&config, "5A,5B", "1.0,2.0", Some("".into())).await;
        assert!(result.is_err(), "transfer multiple with no local node should fail");
    }

    #[tokio::test]
    async fn transfer_multiple_mismatched_with_wallet_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = setup_wallet("xfer-mismatch", dir.path()).await;
        // This exercises the bail path after wallet decryption succeeds
        let result = exec_transfer_multiple(&config, "5A,5B,5C", "1.0,2.0", Some("".into())).await;
        assert!(result.is_err(), "transfer multiple with mismatched counts should fail");
    }
}
