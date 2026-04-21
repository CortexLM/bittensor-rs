//! Stake command group — staking, unstaking, moving, swapping, listing, and
//! auto-staking TAO on the Bittensor network.

use std::str::FromStr;

use anyhow::{Context, Result};
use clap::Subcommand;

use bittensor_chain::client::SubtensorClient;
use bittensor_wallet::prelude::Wallet;

use crate::config::Config;

/// Stake subcommands.
#[derive(Debug, Subcommand)]
pub enum StakeCommand {
    /// Stake TAO to a hotkey
    Add {
        /// Hotkey SS58 address to stake to
        #[arg(long)]
        hotkey: String,

        /// Netuid to stake on
        #[arg(long, default_value_t = 0)]
        netuid: u16,

        /// Amount in TAO (e.g. "1.5")
        amount: String,

        /// Password to decrypt coldkey (prompted if not provided)
        #[arg(long)]
        password: Option<String>,
    },

    /// Unstake TAO from a hotkey
    #[command(name = "remove")]
    Remove {
        /// Hotkey SS58 address to unstake from
        #[arg(long)]
        hotkey: String,

        /// Netuid to unstake from
        #[arg(long, default_value_t = 0)]
        netuid: u16,

        /// Amount in TAO to unstake
        amount: String,

        /// Password to decrypt coldkey (prompted if not provided)
        #[arg(long)]
        password: Option<String>,
    },

    /// Move stake between hotkeys
    #[command(name = "move")]
    Move {
        /// Origin hotkey SS58 address
        #[arg(long)]
        origin_hotkey: String,

        /// Destination hotkey SS58 address
        #[arg(long)]
        destination_hotkey: String,

        /// Origin netuid
        #[arg(long, default_value_t = 0)]
        origin_netuid: u16,

        /// Destination netuid
        #[arg(long, default_value_t = 0)]
        destination_netuid: u16,

        /// Amount in TAO to move
        amount: String,

        /// Password to decrypt coldkey (prompted if not provided)
        #[arg(long)]
        password: Option<String>,
    },

    /// Swap stake between hotkeys
    #[command(name = "swap")]
    Swap {
        /// Hotkey SS58 address
        #[arg(long)]
        hotkey: String,

        /// Origin netuid
        #[arg(long, default_value_t = 0)]
        origin_netuid: u16,

        /// Destination netuid
        #[arg(long, default_value_t = 0)]
        destination_netuid: u16,

        /// Amount in TAO to swap
        amount: String,

        /// Password to decrypt coldkey (prompted if not provided)
        #[arg(long)]
        password: Option<String>,
    },

    /// List all stakes for a wallet
    List,

    /// Query stake for a specific hotkey
    #[command(name = "get-stake")]
    GetStake {
        /// Hotkey SS58 address
        #[arg(long)]
        hotkey: String,

        /// Netuid
        #[arg(long, default_value_t = 0)]
        netuid: u16,
    },

    /// Enable or disable auto-staking for a hotkey
    #[command(name = "set-auto-stake")]
    SetAutoStake {
        /// Hotkey SS58 address
        #[arg(long)]
        hotkey: String,

        /// Netuid
        #[arg(long, default_value_t = 0)]
        netuid: u16,

        /// Password to decrypt coldkey (prompted if not provided)
        #[arg(long)]
        password: Option<String>,
    },
}

impl StakeCommand {
    /// Dispatch the stake subcommand.
    pub async fn execute(self, config: &Config) -> Result<()> {
        match self {
            Self::Add { hotkey, netuid, amount, password } => {
                exec_stake_add(config, &hotkey, netuid, &amount, password).await
            }
            Self::Remove { hotkey, netuid, amount, password } => {
                exec_stake_remove(config, &hotkey, netuid, &amount, password).await
            }
            Self::Move {
                origin_hotkey,
                destination_hotkey,
                origin_netuid,
                destination_netuid,
                amount,
                password,
            } => {
                exec_stake_move(
                    config,
                    &origin_hotkey,
                    &destination_hotkey,
                    origin_netuid,
                    destination_netuid,
                    &amount,
                    password,
                )
                .await
            }
            Self::Swap { hotkey, origin_netuid, destination_netuid, amount, password } => {
                exec_stake_swap(
                    config,
                    &hotkey,
                    origin_netuid,
                    destination_netuid,
                    &amount,
                    password,
                )
                .await
            }
            Self::List => exec_stake_list(config).await,
            Self::GetStake { hotkey, netuid } => exec_stake_get(config, &hotkey, netuid).await,
            Self::SetAutoStake { hotkey, netuid, password } => {
                exec_set_auto_stake(config, &hotkey, netuid, password).await
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Individual command implementations
// ---------------------------------------------------------------------------

async fn exec_stake_add(
    config: &Config,
    hotkey: &str,
    netuid: u16,
    amount: &str,
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

    println!("Stake add submitted successfully.");
    println!("  Block hash:      {}", result.block_hash);
    println!("  Extrinsic hash:  {}", result.extrinsic_hash);
    Ok(())
}

async fn exec_stake_remove(
    config: &Config,
    hotkey: &str,
    netuid: u16,
    amount: &str,
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

    println!("Stake remove submitted successfully.");
    println!("  Block hash:      {}", result.block_hash);
    println!("  Extrinsic hash:  {}", result.extrinsic_hash);
    Ok(())
}

async fn exec_stake_move(
    config: &Config,
    origin_hotkey: &str,
    destination_hotkey: &str,
    origin_netuid: u16,
    destination_netuid: u16,
    amount: &str,
    password: Option<String>,
) -> Result<()> {
    let pwd = prompt_password(password)?;
    let mut wallet = Wallet::with_path(&config.wallet_name, config.wallet_dir());
    let signer = wallet.get_coldkey_pair(&pwd).context("failed to decrypt coldkey")?;

    let origin_account = subxt::utils::AccountId32::from_str(origin_hotkey)
        .context("invalid origin hotkey SS58 address")?;
    let dest_account = subxt::utils::AccountId32::from_str(destination_hotkey)
        .context("invalid destination hotkey SS58 address")?;

    let rao = parse_tao_to_rao(amount)?;

    let client = SubtensorClient::from_config(config.network.clone())
        .await
        .context("failed to connect to chain")?;
    let rpc = client.rpc();

    let inner_kp = subxt_signer::sr25519::Keypair::from_secret_key(*signer.seed())?;

    let result = bittensor_chain::extrinsics::staking::move_stake(
        rpc,
        &inner_kp,
        origin_account,
        dest_account,
        origin_netuid,
        destination_netuid,
        rao,
    )
    .await
    .context("move_stake extrinsic failed")?;

    println!("Stake move submitted successfully.");
    println!("  Block hash:      {}", result.block_hash);
    println!("  Extrinsic hash:  {}", result.extrinsic_hash);
    Ok(())
}

async fn exec_stake_swap(
    config: &Config,
    hotkey: &str,
    origin_netuid: u16,
    destination_netuid: u16,
    amount: &str,
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

    let result = bittensor_chain::extrinsics::staking::swap_stake(
        rpc,
        &inner_kp,
        hotkey_account,
        origin_netuid,
        destination_netuid,
        rao,
    )
    .await
    .context("swap_stake extrinsic failed")?;

    println!("Stake swap submitted successfully.");
    println!("  Block hash:      {}", result.block_hash);
    println!("  Extrinsic hash:  {}", result.extrinsic_hash);
    Ok(())
}

async fn exec_stake_list(config: &Config) -> Result<()> {
    let mut wallet = Wallet::with_path(&config.wallet_name, config.wallet_dir());
    let addr = wallet.get_coldkeypub().context("coldkeypub not found — does the wallet exist?")?;

    let account_id = subxt::utils::AccountId32::from_str(&addr).context("invalid SS58 address")?;

    let client = SubtensorClient::from_config(config.network.clone())
        .await
        .context("failed to connect to chain")?;
    let rpc = client.rpc();

    let stakes = bittensor_chain::queries::account::get_stake_info_for_coldkey(rpc, &account_id)
        .await
        .context("failed to query stake info")?;

    if stakes.is_empty() {
        println!("No stakes found for wallet '{}'.", config.wallet_name);
    } else {
        println!("Stakes for wallet '{}':", config.wallet_name);
        for si in &stakes {
            println!("  hotkey={} stake={} TAO", si.hotkey, si.stake);
        }
    }

    Ok(())
}

async fn exec_stake_get(config: &Config, hotkey: &str, netuid: u16) -> Result<()> {
    let mut wallet = Wallet::with_path(&config.wallet_name, config.wallet_dir());
    let addr = wallet.get_coldkeypub().context("coldkeypub not found — does the wallet exist?")?;

    let coldkey_account =
        subxt::utils::AccountId32::from_str(&addr).context("invalid coldkey SS58 address")?;
    let hotkey_account =
        subxt::utils::AccountId32::from_str(hotkey).context("invalid hotkey SS58 address")?;

    let client = SubtensorClient::from_config(config.network.clone())
        .await
        .context("failed to connect to chain")?;
    let rpc = client.rpc();

    let stake = bittensor_chain::queries::account::get_stake(
        rpc,
        &coldkey_account,
        &hotkey_account,
        netuid,
    )
    .await
    .context("failed to query stake")?;

    println!("Stake for hotkey={hotkey} netuid={netuid}: {stake} TAO");
    Ok(())
}

async fn exec_set_auto_stake(
    config: &Config,
    hotkey: &str,
    netuid: u16,
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

    let result = bittensor_chain::extrinsics::staking::set_auto_stake(
        rpc,
        &inner_kp,
        netuid,
        hotkey_account,
    )
    .await
    .context("set_auto_stake extrinsic failed")?;

    println!("Auto-stake set successfully.");
    println!("  Block hash:      {}", result.block_hash);
    println!("  Extrinsic hash:  {}", result.extrinsic_hash);
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
    fn parse_tao_small_decimal() {
        assert_eq!(parse_tao_to_rao("0.001").unwrap(), 1_000_000);
    }

    #[test]
    fn parse_tao_very_small() {
        assert_eq!(parse_tao_to_rao("0.000000001").unwrap(), 1);
    }

    #[test]
    fn parse_tao_negative_fails() {
        assert!(parse_tao_to_rao("-1.0").is_err());
    }

    #[test]
    fn parse_tao_invalid_string_fails() {
        assert!(parse_tao_to_rao("abc").is_err());
    }

    #[test]
    fn parse_tao_large_amount() {
        assert_eq!(parse_tao_to_rao("1000").unwrap(), 1_000_000_000_000);
    }

    #[test]
    fn stake_command_debug_format() {
        let cmd = StakeCommand::List;
        assert!(format!("{cmd:?}").contains("List"));
    }

    #[test]
    fn parse_stake_add_command() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "btcli-rs",
            "stake",
            "add",
            "--hotkey",
            "5Hotkey123",
            "--netuid",
            "1",
            "5.0",
            "--password",
            "secret",
        ])
        .unwrap();
        match cli.command {
            crate::Command::Stake {
                command: StakeCommand::Add { hotkey, netuid, amount, password },
            } => {
                assert_eq!(hotkey, "5Hotkey123");
                assert_eq!(netuid, 1);
                assert_eq!(amount, "5.0");
                assert_eq!(password.unwrap(), "secret");
            }
            other => panic!("expected Stake::Add, got {other:?}"),
        }
    }

    #[test]
    fn parse_stake_remove_command() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "btcli-rs",
            "stake",
            "remove",
            "--hotkey",
            "5Hotkey456",
            "--netuid",
            "3",
            "2.5",
        ])
        .unwrap();
        match cli.command {
            crate::Command::Stake {
                command: StakeCommand::Remove { hotkey, netuid, amount, .. },
            } => {
                assert_eq!(hotkey, "5Hotkey456");
                assert_eq!(netuid, 3);
                assert_eq!(amount, "2.5");
            }
            other => panic!("expected Stake::Remove, got {other:?}"),
        }
    }

    #[test]
    fn parse_stake_move_command() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "btcli-rs",
            "stake",
            "move",
            "--origin-hotkey",
            "5OriginHK",
            "--destination-hotkey",
            "5DestHK",
            "--origin-netuid",
            "1",
            "--destination-netuid",
            "2",
            "10.0",
            "--password",
            "pass",
        ])
        .unwrap();
        match cli.command {
            crate::Command::Stake {
                command:
                    StakeCommand::Move {
                        origin_hotkey,
                        destination_hotkey,
                        origin_netuid,
                        destination_netuid,
                        amount,
                        password,
                    },
            } => {
                assert_eq!(origin_hotkey, "5OriginHK");
                assert_eq!(destination_hotkey, "5DestHK");
                assert_eq!(origin_netuid, 1);
                assert_eq!(destination_netuid, 2);
                assert_eq!(amount, "10.0");
                assert_eq!(password.unwrap(), "pass");
            }
            other => panic!("expected Stake::Move, got {other:?}"),
        }
    }

    #[test]
    fn parse_stake_swap_command() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "btcli-rs",
            "stake",
            "swap",
            "--hotkey",
            "5SwapHK",
            "--origin-netuid",
            "1",
            "--destination-netuid",
            "5",
            "3.0",
        ])
        .unwrap();
        match cli.command {
            crate::Command::Stake {
                command:
                    StakeCommand::Swap { hotkey, origin_netuid, destination_netuid, amount, .. },
            } => {
                assert_eq!(hotkey, "5SwapHK");
                assert_eq!(origin_netuid, 1);
                assert_eq!(destination_netuid, 5);
                assert_eq!(amount, "3.0");
            }
            other => panic!("expected Stake::Swap, got {other:?}"),
        }
    }

    #[test]
    fn parse_stake_list_command() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from(["btcli-rs", "stake", "list"]).unwrap();
        match cli.command {
            crate::Command::Stake { command: StakeCommand::List } => {}
            other => panic!("expected Stake::List, got {other:?}"),
        }
    }

    #[test]
    fn parse_stake_get_stake_command() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "btcli-rs",
            "stake",
            "get-stake",
            "--hotkey",
            "5GKHK",
            "--netuid",
            "7",
        ])
        .unwrap();
        match cli.command {
            crate::Command::Stake { command: StakeCommand::GetStake { hotkey, netuid } } => {
                assert_eq!(hotkey, "5GKHK");
                assert_eq!(netuid, 7);
            }
            other => panic!("expected Stake::GetStake, got {other:?}"),
        }
    }

    #[test]
    fn parse_set_auto_stake_command() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "btcli-rs",
            "stake",
            "set-auto-stake",
            "--hotkey",
            "5AutoHK",
            "--netuid",
            "1",
            "--password",
            "pw",
        ])
        .unwrap();
        match cli.command {
            crate::Command::Stake {
                command: StakeCommand::SetAutoStake { hotkey, netuid, password },
            } => {
                assert_eq!(hotkey, "5AutoHK");
                assert_eq!(netuid, 1);
                assert_eq!(password.unwrap(), "pw");
            }
            other => panic!("expected Stake::SetAutoStake, got {other:?}"),
        }
    }

    #[test]
    fn stake_command_all_variants_parseable() {
        // Verify all StakeCommand variants are parseable via CLI
        let variants: Vec<Vec<&str>> = vec![
            vec!["btcli-rs", "stake", "add", "--hotkey", "5HK", "1.0"],
            vec!["btcli-rs", "stake", "remove", "--hotkey", "5HK", "1.0"],
            vec![
                "btcli-rs",
                "stake",
                "move",
                "--origin-hotkey",
                "5O",
                "--destination-hotkey",
                "5D",
                "1.0",
            ],
            vec!["btcli-rs", "stake", "swap", "--hotkey", "5HK", "1.0"],
            vec!["btcli-rs", "stake", "list"],
            vec!["btcli-rs", "stake", "get-stake", "--hotkey", "5HK"],
            vec!["btcli-rs", "stake", "set-auto-stake", "--hotkey", "5HK"],
        ];
        use clap::Parser;
        for args in &variants {
            let result = crate::Cli::try_parse_from(args);
            assert!(result.is_ok(), "variant {:?} should be parseable", args);
        }
    }

    // --- Integration-style tests using temp dirs (no real chain calls) ---

    #[tokio::test]
    async fn stake_list_no_wallet() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "ghost".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        // exec_stake_list will fail gracefully because coldkeypub doesn't exist
        let result = exec_stake_list(&config).await;
        assert!(result.is_err(), "stake list on nonexistent wallet should fail");
    }

    #[tokio::test]
    async fn stake_get_no_wallet() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "ghost".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let result = exec_stake_get(&config, "5FakeHK", 0).await;
        assert!(result.is_err(), "stake get on nonexistent wallet should fail");
    }
}
