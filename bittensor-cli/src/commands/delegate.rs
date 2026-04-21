//! Delegate command group — staking to delegates, removing stake, listing
//! delegates, setting delegate take, and viewing delegations.

use std::str::FromStr;

use anyhow::{Context, Result};
use clap::Subcommand;

use bittensor_chain::client::SubtensorClient;
use bittensor_wallet::prelude::Wallet;

use crate::config::Config;

/// Delegate subcommands.
#[derive(Debug, Subcommand)]
pub enum DelegateCommand {
    /// Add stake to a delegate (hotkey)
    Add {
        /// Hotkey SS58 address of the delegate
        #[arg(long)]
        hotkey: String,

        /// Amount in TAO to stake (e.g. "1.5")
        amount: String,

        /// Netuid to stake on
        #[arg(long, default_value_t = 0)]
        netuid: u16,

        /// Password to decrypt coldkey (prompted if not provided)
        #[arg(long)]
        password: Option<String>,
    },

    /// Remove stake from a delegate (hotkey)
    Remove {
        /// Hotkey SS58 address of the delegate
        #[arg(long)]
        hotkey: String,

        /// Amount in TAO to unstake (e.g. "1.5")
        amount: String,

        /// Netuid to unstake from
        #[arg(long, default_value_t = 0)]
        netuid: u16,

        /// Password to decrypt coldkey (prompted if not provided)
        #[arg(long)]
        password: Option<String>,
    },

    /// List all delegates on the network
    List,

    /// Set the delegate take for a hotkey
    Take {
        /// Hotkey SS58 address of the delegate
        #[arg(long)]
        hotkey: String,

        /// Take value (u16)
        take: u16,

        /// Password to decrypt coldkey (prompted if not provided)
        #[arg(long)]
        password: Option<String>,
    },

    /// Show delegations from the wallet's coldkey
    #[command(name = "my-delegates")]
    MyDelegates {
        /// Password to decrypt coldkey (prompted if not provided)
        #[arg(long)]
        password: Option<String>,
    },
}

impl DelegateCommand {
    /// Dispatch the delegate subcommand.
    pub async fn execute(self, config: &Config) -> Result<()> {
        match self {
            Self::Add { hotkey, amount, netuid, password } => {
                exec_delegate_add(config, &hotkey, &amount, netuid, password).await
            }
            Self::Remove { hotkey, amount, netuid, password } => {
                exec_delegate_remove(config, &hotkey, &amount, netuid, password).await
            }
            Self::List => exec_delegate_list(config).await,
            Self::Take { hotkey, take, password } => {
                exec_delegate_take(config, &hotkey, take, password).await
            }
            Self::MyDelegates { password } => exec_my_delegates(config, password).await,
        }
    }
}

// ---------------------------------------------------------------------------
// Individual command implementations
// ---------------------------------------------------------------------------

async fn exec_delegate_add(
    config: &Config,
    hotkey: &str,
    amount: &str,
    netuid: u16,
    password: Option<String>,
) -> Result<()> {
    let pwd = prompt_password(password)?;
    let mut wallet = Wallet::with_path(&config.wallet_name, config.wallet_dir());
    let signer = wallet.get_coldkey_pair(&pwd).context("failed to decrypt coldkey")?;

    let hotkey_account =
        subxt::utils::AccountId32::from_str(hotkey).context("invalid hotkey SS58 address")?;

    let rao = parse_tao_to_rao(amount)?;

    let client = SubtensorClient::from_config(config.network.clone())
        .await
        .context("failed to connect to chain")?;
    let rpc = client.rpc();

    let inner_kp = subxt_signer::sr25519::Keypair::from_secret_key(*signer.seed())?;

    let result = bittensor_chain::extrinsics::staking::add_stake(
        rpc,
        &inner_kp,
        hotkey_account,
        netuid,
        rao,
    )
    .await
    .context("add_stake extrinsic failed")?;

    println!("Delegate add submitted successfully.");
    println!("  Block hash:      {}", result.block_hash);
    println!("  Extrinsic hash:  {}", result.extrinsic_hash);
    Ok(())
}

async fn exec_delegate_remove(
    config: &Config,
    hotkey: &str,
    amount: &str,
    netuid: u16,
    password: Option<String>,
) -> Result<()> {
    let pwd = prompt_password(password)?;
    let mut wallet = Wallet::with_path(&config.wallet_name, config.wallet_dir());
    let signer = wallet.get_coldkey_pair(&pwd).context("failed to decrypt coldkey")?;

    let hotkey_account =
        subxt::utils::AccountId32::from_str(hotkey).context("invalid hotkey SS58 address")?;

    let rao = parse_tao_to_rao(amount)?;

    let client = SubtensorClient::from_config(config.network.clone())
        .await
        .context("failed to connect to chain")?;
    let rpc = client.rpc();

    let inner_kp = subxt_signer::sr25519::Keypair::from_secret_key(*signer.seed())?;

    let result = bittensor_chain::extrinsics::staking::remove_stake(
        rpc,
        &inner_kp,
        hotkey_account,
        netuid,
        rao,
    )
    .await
    .context("remove_stake extrinsic failed")?;

    println!("Delegate remove submitted successfully.");
    println!("  Block hash:      {}", result.block_hash);
    println!("  Extrinsic hash:  {}", result.extrinsic_hash);
    Ok(())
}

async fn exec_delegate_list(config: &Config) -> Result<()> {
    let client = SubtensorClient::from_config(config.network.clone())
        .await
        .context("failed to connect to chain")?;
    let rpc = client.rpc();

    let delegates = bittensor_chain::queries::delegate::get_delegates(rpc)
        .await
        .context("failed to query delegates")?;

    if delegates.is_empty() {
        println!("No delegates found on the network.");
    } else {
        println!("Delegates:");
        for d in &delegates {
            println!("  hotkey={} take={}", d.delegate_hotkey, d.take);
        }
    }

    Ok(())
}

async fn exec_delegate_take(
    config: &Config,
    hotkey: &str,
    take: u16,
    password: Option<String>,
) -> Result<()> {
    let pwd = prompt_password(password)?;
    let mut wallet = Wallet::with_path(&config.wallet_name, config.wallet_dir());
    let signer = wallet.get_coldkey_pair(&pwd).context("failed to decrypt coldkey")?;

    let hotkey_account =
        subxt::utils::AccountId32::from_str(hotkey).context("invalid hotkey SS58 address")?;

    let client = SubtensorClient::from_config(config.network.clone())
        .await
        .context("failed to connect to chain")?;
    let rpc = client.rpc();

    let inner_kp = subxt_signer::sr25519::Keypair::from_secret_key(*signer.seed())?;

    // Determine whether to increase or decrease take based on current value
    let current_take = bittensor_chain::queries::delegate::get_delegate_take(rpc, &hotkey_account)
        .await
        .context("failed to query delegate take")?;

    let result = if take > current_take {
        bittensor_chain::extrinsics::take::increase_take(rpc, &inner_kp, hotkey_account, take)
            .await
            .context("increase_take extrinsic failed")?
    } else {
        bittensor_chain::extrinsics::take::decrease_take(rpc, &inner_kp, hotkey_account, take)
            .await
            .context("decrease_take extrinsic failed")?
    };

    println!("Delegate take set successfully.");
    println!("  Block hash:      {}", result.block_hash);
    println!("  Extrinsic hash:  {}", result.extrinsic_hash);
    Ok(())
}

async fn exec_my_delegates(config: &Config, password: Option<String>) -> Result<()> {
    let _pwd = prompt_password(password)?;
    let mut wallet = Wallet::with_path(&config.wallet_name, config.wallet_dir());
    let coldkey_addr =
        wallet.get_coldkeypub().context("coldkeypub not found — does the wallet exist?")?;

    let coldkey_account = subxt::utils::AccountId32::from_str(&coldkey_addr)
        .context("invalid coldkey SS58 address")?;

    let client = SubtensorClient::from_config(config.network.clone())
        .await
        .context("failed to connect to chain")?;
    let rpc = client.rpc();

    let delegated = bittensor_chain::queries::delegate::get_delegated_info(rpc, &coldkey_account)
        .await
        .context("failed to query delegated info")?;

    if delegated.is_empty() {
        println!("No delegations found for wallet '{}'.", config.wallet_name);
    } else {
        println!("Delegations for wallet '{}':", config.wallet_name);
        for d in &delegated {
            println!("  hotkey={} take={}", d.delegate_hotkey, d.take);
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
    use bittensor_wallet::prelude::Wallet;
    use tempfile::TempDir;

    #[test]
    fn delegate_command_debug_format() {
        let cmd = DelegateCommand::List;
        assert!(format!("{cmd:?}").contains("List"));
    }

    #[test]
    fn parse_delegate_add() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "btcli-rs",
            "delegate",
            "add",
            "--hotkey",
            "5HK123",
            "--netuid",
            "3",
            "10.0",
            "--password",
            "secret",
        ])
        .unwrap();
        match cli.command {
            crate::Command::Delegate {
                command: DelegateCommand::Add { hotkey, amount, netuid, password },
            } => {
                assert_eq!(hotkey, "5HK123");
                assert_eq!(amount, "10.0");
                assert_eq!(netuid, 3);
                assert_eq!(password.unwrap(), "secret");
            }
            other => panic!("expected Delegate::Add, got {other:?}"),
        }
    }

    #[test]
    fn parse_delegate_remove() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "btcli-rs", "delegate", "remove", "--hotkey", "5HK456", "--netuid", "1", "5.0",
        ])
        .unwrap();
        match cli.command {
            crate::Command::Delegate {
                command: DelegateCommand::Remove { hotkey, amount, netuid, .. },
            } => {
                assert_eq!(hotkey, "5HK456");
                assert_eq!(amount, "5.0");
                assert_eq!(netuid, 1);
            }
            other => panic!("expected Delegate::Remove, got {other:?}"),
        }
    }

    #[test]
    fn parse_delegate_list() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from(["btcli-rs", "delegate", "list"]).unwrap();
        match cli.command {
            crate::Command::Delegate { command: DelegateCommand::List } => {}
            other => panic!("expected Delegate::List, got {other:?}"),
        }
    }

    #[test]
    fn parse_delegate_take() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "btcli-rs",
            "delegate",
            "take",
            "--hotkey",
            "5HK789",
            "18",
            "--password",
            "pw",
        ])
        .unwrap();
        match cli.command {
            crate::Command::Delegate {
                command: DelegateCommand::Take { hotkey, take, password },
            } => {
                assert_eq!(hotkey, "5HK789");
                assert_eq!(take, 18);
                assert_eq!(password.unwrap(), "pw");
            }
            other => panic!("expected Delegate::Take, got {other:?}"),
        }
    }

    #[test]
    fn parse_delegate_my_delegates() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "btcli-rs",
            "delegate",
            "my-delegates",
            "--password",
            "pw2",
        ])
        .unwrap();
        match cli.command {
            crate::Command::Delegate { command: DelegateCommand::MyDelegates { password } } => {
                assert_eq!(password.unwrap(), "pw2");
            }
            other => panic!("expected Delegate::MyDelegates, got {other:?}"),
        }
    }

    #[test]
    fn delegate_command_all_variants_parseable() {
        let variants: Vec<Vec<&str>> = vec![
            vec!["btcli-rs", "delegate", "add", "--hotkey", "5HK", "1.0"],
            vec!["btcli-rs", "delegate", "remove", "--hotkey", "5HK", "1.0"],
            vec!["btcli-rs", "delegate", "list"],
            vec!["btcli-rs", "delegate", "take", "--hotkey", "5HK", "10"],
            vec!["btcli-rs", "delegate", "my-delegates"],
        ];
        use clap::Parser;
        for args in &variants {
            let result = crate::Cli::try_parse_from(args);
            assert!(result.is_ok(), "variant {:?} should be parseable", args);
        }
    }

    #[test]
    fn parse_tao_delegate_zero() {
        assert_eq!(parse_tao_to_rao("0").unwrap(), 0);
    }

    #[test]
    fn parse_tao_delegate_one() {
        assert_eq!(parse_tao_to_rao("1").unwrap(), 1_000_000_000);
    }

    #[test]
    fn parse_tao_delegate_decimal() {
        assert_eq!(parse_tao_to_rao("2.5").unwrap(), 2_500_000_000);
    }

    #[test]
    fn parse_tao_delegate_negative_fails() {
        assert!(parse_tao_to_rao("-1.0").is_err());
    }

    #[tokio::test]
    async fn delegate_add_no_wallet_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "ghost".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let result = exec_delegate_add(&config, "5HK", "1.0", 0, Some("pw".into())).await;
        assert!(result.is_err(), "delegate add with no wallet should fail");
    }

    #[tokio::test]
    async fn delegate_remove_no_wallet_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "ghost".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let result = exec_delegate_remove(&config, "5HK", "1.0", 0, Some("pw".into())).await;
        assert!(result.is_err(), "delegate remove with no wallet should fail");
    }

    #[tokio::test]
    async fn delegate_take_no_wallet_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "ghost".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let result = exec_delegate_take(&config, "5HK", 10, Some("pw".into())).await;
        assert!(result.is_err(), "delegate take with no wallet should fail");
    }

    #[tokio::test]
    async fn my_delegates_no_wallet_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "ghost".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let result = exec_my_delegates(&config, Some("pw".into())).await;
        assert!(result.is_err(), "my delegates with no wallet should fail");
    }

    #[tokio::test]
    async fn delegate_list_local_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "ghost".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let result = exec_delegate_list(&config).await;
        assert!(result.is_err(), "delegate list with no local node should fail");
    }

    #[tokio::test]
    async fn delegate_add_created_wallet_chain_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "test-wallet".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let mut wallet = Wallet::with_path(&config.wallet_name, config.wallet_dir());
        wallet.create_coldkey("").expect("create coldkey");
        wallet.create_hotkey().expect("create hotkey");
        let result = exec_delegate_add(
            &config,
            "5FakeHKAddress111111111111111111111",
            "1.0",
            0,
            Some("".into()),
        )
        .await;
        assert!(result.is_err(), "delegate add with created wallet but no chain should fail");
    }

    #[tokio::test]
    async fn delegate_remove_created_wallet_chain_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "test-wallet".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let mut wallet = Wallet::with_path(&config.wallet_name, config.wallet_dir());
        wallet.create_coldkey("").expect("create coldkey");
        wallet.create_hotkey().expect("create hotkey");
        let result = exec_delegate_remove(
            &config,
            "5FakeHKAddress111111111111111111111",
            "1.0",
            0,
            Some("".into()),
        )
        .await;
        assert!(result.is_err(), "delegate remove with created wallet but no chain should fail");
    }
}
