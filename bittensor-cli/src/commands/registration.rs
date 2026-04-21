//! Registration command group — POW registration, burned registration, and
//! root network registration on the Bittensor network.

use std::str::FromStr;

use anyhow::{Context, Result};
use clap::Subcommand;

use bittensor_chain::client::SubtensorClient;
use bittensor_wallet::prelude::Wallet;

use crate::config::Config;

/// Registration subcommands.
#[derive(Debug, Subcommand)]
pub enum RegistrationCommand {
    /// Register on a subnet via Proof-of-Work
    Register {
        /// Netuid to register on
        #[arg(long, default_value_t = 1)]
        netuid: u16,

        /// Password to decrypt coldkey (prompted if not provided)
        #[arg(long)]
        password: Option<String>,
    },

    /// Register on a subnet by burning TAO
    #[command(name = "burned-register")]
    BurnedRegister {
        /// Netuid to register on
        #[arg(long, default_value_t = 1)]
        netuid: u16,

        /// Password to decrypt coldkey (prompted if not provided)
        #[arg(long)]
        password: Option<String>,
    },

    /// Register on the root network
    #[command(name = "root-register")]
    RootRegister {
        /// Password to decrypt coldkey (prompted if not provided)
        #[arg(long)]
        password: Option<String>,
    },
}

/// Root command (top-level subcommand, not under Registration).
#[derive(Debug, Subcommand)]
pub enum RootCommand {
    /// Register on the root network
    #[command(name = "register")]
    Register {
        /// Password to decrypt coldkey (prompted if not provided)
        #[arg(long)]
        password: Option<String>,
    },

    /// Set weights on the root network
    #[command(name = "set-weights")]
    SetWeights {
        /// Netuid for the weights
        #[arg(long)]
        netuid: u16,

        /// Destination UIDs (comma-separated, e.g. "1,2,3")
        dests: String,

        /// Weight values (comma-separated, e.g. "100,200,300")
        weights: String,

        /// Version key (default 0)
        #[arg(long, default_value_t = 0)]
        version_key: u64,

        /// Password to decrypt coldkey (prompted if not provided)
        #[arg(long)]
        password: Option<String>,
    },

    /// Get weights set by a UID on the root network
    #[command(name = "get-weights")]
    GetWeights {
        /// Netuid for the weights
        #[arg(long)]
        netuid: u16,

        /// UID to query
        uid: u16,
    },

    /// Claim root authority for subnets
    #[command(name = "claim")]
    Claim {
        /// Subnet IDs to claim (comma-separated, e.g. "1,3,7")
        subnets: String,

        /// Password to decrypt coldkey (prompted if not provided)
        #[arg(long)]
        password: Option<String>,
    },
}

impl RegistrationCommand {
    /// Dispatch the registration subcommand.
    pub async fn execute(self, config: &Config) -> Result<()> {
        match self {
            Self::Register { netuid, password } => exec_register(config, netuid, password).await,
            Self::BurnedRegister { netuid, password } => {
                exec_burned_register(config, netuid, password).await
            }
            Self::RootRegister { password } => exec_root_register(config, password).await,
        }
    }
}

impl RootCommand {
    /// Dispatch the root subcommand.
    pub async fn execute(self, config: &Config) -> Result<()> {
        match self {
            Self::Register { password } => exec_root_register(config, password).await,
            Self::SetWeights { netuid, dests, weights, version_key, password } => {
                exec_set_weights(config, netuid, &dests, &weights, version_key, password).await
            }
            Self::GetWeights { netuid, uid } => exec_get_weights(config, netuid, uid).await,
            Self::Claim { subnets, password } => exec_claim_root(config, &subnets, password).await,
        }
    }
}

// ---------------------------------------------------------------------------
// Individual command implementations
// ---------------------------------------------------------------------------

async fn exec_register(config: &Config, netuid: u16, password: Option<String>) -> Result<()> {
    let pwd = prompt_password(password)?;
    let mut wallet = Wallet::with_path(&config.wallet_name, config.wallet_dir());
    let signer = wallet.get_coldkey_pair(&pwd).context("failed to decrypt coldkey")?;

    let coldkey_account = subxt::utils::AccountId32::from_str(
        &wallet.get_coldkeypub().context("failed to read coldkeypub")?,
    )
    .context("invalid coldkey SS58 address")?;

    let hotkey_account = subxt::utils::AccountId32::from_str(
        &wallet.get_hotkey_pair().context("failed to read hotkey")?.ss58_address(),
    )
    .context("invalid hotkey SS58 address")?;

    let client = SubtensorClient::from_config(config.network.clone())
        .await
        .context("failed to connect to chain")?;
    let rpc = client.rpc();

    // Fetch current block info for POW
    let block_number = bittensor_chain::queries::network::get_network_block(rpc)
        .await
        .context("failed to get current block number")?;

    let block_hash_bytes = bittensor_chain::queries::network::get_block_hash(rpc, block_number)
        .await
        .context("failed to get block hash")?
        .unwrap_or_default();

    let block_hash: [u8; 32] = block_hash_bytes.into();

    // Fetch subnet difficulty
    let hyperparams = bittensor_chain::queries::subnet::get_subnet_hyperparameters(rpc, netuid)
        .await
        .context("failed to get subnet hyperparameters")?
        .context("subnet not found")?;

    let difficulty = hyperparams.difficulty as u64;

    // Solve POW
    let nonce_seed = signer.public_key().0;
    let solution =
        bittensor_core::pow::solve_pow(&nonce_seed, difficulty, block_hash, block_number)
            .context("POW solve failed")?;

    // Submit registration extrinsic
    let inner_kp = subxt_signer::sr25519::Keypair::from_secret_key(*signer.seed())?;

    let result = bittensor_chain::extrinsics::registration::register(
        rpc,
        &inner_kp,
        bittensor_chain::extrinsics::registration::RegisterParams {
            netuid,
            block_number,
            nonce: solution.nonce,
            work: solution.seal.to_vec(),
            hotkey: hotkey_account,
            coldkey: coldkey_account,
        },
    )
    .await
    .context("register extrinsic failed")?;

    println!("POW Registration submitted successfully.");
    println!("  Netuid:         {netuid}");
    println!("  Block hash:      {}", result.block_hash);
    println!("  Extrinsic hash:  {}", result.extrinsic_hash);
    Ok(())
}

async fn exec_burned_register(
    config: &Config,
    netuid: u16,
    password: Option<String>,
) -> Result<()> {
    let pwd = prompt_password(password)?;
    let mut wallet = Wallet::with_path(&config.wallet_name, config.wallet_dir());
    let signer = wallet.get_coldkey_pair(&pwd).context("failed to decrypt coldkey")?;

    let hotkey_account = subxt::utils::AccountId32::from_str(
        &wallet.get_hotkey_pair().context("failed to read hotkey")?.ss58_address(),
    )
    .context("invalid hotkey SS58 address")?;

    let client = SubtensorClient::from_config(config.network.clone())
        .await
        .context("failed to connect to chain")?;
    let rpc = client.rpc();

    let inner_kp = subxt_signer::sr25519::Keypair::from_secret_key(*signer.seed())?;

    let result = bittensor_chain::extrinsics::registration::burned_register(
        rpc,
        &inner_kp,
        netuid,
        hotkey_account,
    )
    .await
    .context("burned_register extrinsic failed")?;

    println!("Burned registration submitted successfully.");
    println!("  Netuid:         {netuid}");
    println!("  Block hash:      {}", result.block_hash);
    println!("  Extrinsic hash:  {}", result.extrinsic_hash);
    Ok(())
}

async fn exec_root_register(config: &Config, password: Option<String>) -> Result<()> {
    let pwd = prompt_password(password)?;
    let mut wallet = Wallet::with_path(&config.wallet_name, config.wallet_dir());
    let signer = wallet.get_coldkey_pair(&pwd).context("failed to decrypt coldkey")?;

    let hotkey_account = subxt::utils::AccountId32::from_str(
        &wallet.get_hotkey_pair().context("failed to read hotkey")?.ss58_address(),
    )
    .context("invalid hotkey SS58 address")?;

    let client = SubtensorClient::from_config(config.network.clone())
        .await
        .context("failed to connect to chain")?;
    let rpc = client.rpc();

    let inner_kp = subxt_signer::sr25519::Keypair::from_secret_key(*signer.seed())?;

    let result = bittensor_chain::extrinsics::root::root_register(rpc, &inner_kp, hotkey_account)
        .await
        .context("root_register extrinsic failed")?;

    println!("Root registration submitted successfully.");
    println!("  Block hash:      {}", result.block_hash);
    println!("  Extrinsic hash:  {}", result.extrinsic_hash);
    Ok(())
}

async fn exec_set_weights(
    config: &Config,
    netuid: u16,
    dests: &str,
    weights: &str,
    version_key: u64,
    password: Option<String>,
) -> Result<()> {
    let pwd = prompt_password(password)?;
    let mut wallet = Wallet::with_path(&config.wallet_name, config.wallet_dir());
    let signer = wallet.get_coldkey_pair(&pwd).context("failed to decrypt coldkey")?;

    let dest_vec = parse_comma_u16(dests)?;
    let weight_vec = parse_comma_u16(weights)?;

    if dest_vec.len() != weight_vec.len() {
        anyhow::bail!(
            "number of dests ({}) does not match number of weights ({})",
            dest_vec.len(),
            weight_vec.len()
        );
    }

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

async fn exec_get_weights(config: &Config, netuid: u16, uid: u16) -> Result<()> {
    let client = SubtensorClient::from_config(config.network.clone())
        .await
        .context("failed to connect to chain")?;
    let rpc = client.rpc();

    let weights = bittensor_chain::queries::weights::get_weights(rpc, netuid, uid)
        .await
        .context("failed to query weights")?;

    println!("Weights for netuid={netuid} uid={uid}:");
    for (dest, weight) in &weights {
        println!("  dest={dest} weight={weight}");
    }

    Ok(())
}

async fn exec_claim_root(config: &Config, subnets: &str, password: Option<String>) -> Result<()> {
    let pwd = prompt_password(password)?;
    let mut wallet = Wallet::with_path(&config.wallet_name, config.wallet_dir());
    let signer = wallet.get_coldkey_pair(&pwd).context("failed to decrypt coldkey")?;

    let subnet_vec = parse_comma_u16(subnets)?;

    let client = SubtensorClient::from_config(config.network.clone())
        .await
        .context("failed to connect to chain")?;
    let rpc = client.rpc();

    let inner_kp = subxt_signer::sr25519::Keypair::from_secret_key(*signer.seed())?;

    let result = bittensor_chain::extrinsics::root::claim_root(rpc, &inner_kp, subnet_vec)
        .await
        .context("claim_root extrinsic failed")?;

    println!("Claim root submitted successfully.");
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

/// Parse a comma-separated string into a Vec<u16>.
fn parse_comma_u16(input: &str) -> Result<Vec<u16>> {
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
    fn registration_command_debug_format() {
        let cmd = RegistrationCommand::Register { netuid: 1, password: None };
        assert!(format!("{cmd:?}").contains("Register"));
    }

    #[test]
    fn root_command_debug_format() {
        let cmd = RootCommand::Register { password: None };
        assert!(format!("{cmd:?}").contains("Register"));
    }

    #[test]
    fn parse_register_command() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "btcli-rs",
            "register",
            "register",
            "--netuid",
            "3",
            "--password",
            "secret",
        ])
        .unwrap();
        match cli.command {
            crate::Command::Register {
                command: RegistrationCommand::Register { netuid, password },
            } => {
                assert_eq!(netuid, 3);
                assert_eq!(password.unwrap(), "secret");
            }
            other => panic!("expected Register::Register, got {other:?}"),
        }
    }

    #[test]
    fn parse_register_command_default_netuid() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from(["btcli-rs", "register", "register"]).unwrap();
        match cli.command {
            crate::Command::Register { command: RegistrationCommand::Register { netuid, .. } } => {
                assert_eq!(netuid, 1);
            }
            other => panic!("expected Register::Register, got {other:?}"),
        }
    }

    #[test]
    fn parse_burned_register_command() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "btcli-rs",
            "register",
            "burned-register",
            "--netuid",
            "5",
            "--password",
            "pw",
        ])
        .unwrap();
        match cli.command {
            crate::Command::Register {
                command: RegistrationCommand::BurnedRegister { netuid, password },
            } => {
                assert_eq!(netuid, 5);
                assert_eq!(password.unwrap(), "pw");
            }
            other => panic!("expected Register::BurnedRegister, got {other:?}"),
        }
    }

    #[test]
    fn parse_root_register_via_register_group() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "btcli-rs",
            "register",
            "root-register",
            "--password",
            "pw2",
        ])
        .unwrap();
        match cli.command {
            crate::Command::Register {
                command: RegistrationCommand::RootRegister { password },
            } => {
                assert_eq!(password.unwrap(), "pw2");
            }
            other => panic!("expected Register::RootRegister, got {other:?}"),
        }
    }

    #[test]
    fn parse_root_register_top_level() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from(["btcli-rs", "root", "register", "--password", "pw3"])
            .unwrap();
        match cli.command {
            crate::Command::Root { command: RootCommand::Register { password } } => {
                assert_eq!(password.unwrap(), "pw3");
            }
            other => panic!("expected Root::Register, got {other:?}"),
        }
    }

    #[test]
    fn registration_command_all_variants_parseable() {
        let variants: Vec<Vec<&str>> = vec![
            vec!["btcli-rs", "register", "register", "--netuid", "1"],
            vec!["btcli-rs", "register", "burned-register", "--netuid", "1"],
            vec!["btcli-rs", "register", "root-register"],
            vec!["btcli-rs", "root", "register"],
            vec!["btcli-rs", "root", "set-weights", "--netuid", "1", "1,2", "100,200"],
            vec!["btcli-rs", "root", "get-weights", "--netuid", "1", "5"],
            vec!["btcli-rs", "root", "claim", "1,3,7"],
        ];
        use clap::Parser;
        for args in &variants {
            let result = crate::Cli::try_parse_from(args);
            assert!(result.is_ok(), "variant {:?} should be parseable", args);
        }
    }

    #[test]
    fn parse_root_set_weights() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "btcli-rs",
            "root",
            "set-weights",
            "--netuid",
            "3",
            "1,2,3",
            "100,200,300",
            "--version-key",
            "42",
            "--password",
            "pw",
        ])
        .unwrap();
        match cli.command {
            crate::Command::Root {
                command: RootCommand::SetWeights { netuid, dests, weights, version_key, password },
            } => {
                assert_eq!(netuid, 3);
                assert_eq!(dests, "1,2,3");
                assert_eq!(weights, "100,200,300");
                assert_eq!(version_key, 42);
                assert_eq!(password.unwrap(), "pw");
            }
            other => panic!("expected Root::SetWeights, got {other:?}"),
        }
    }

    #[test]
    fn parse_root_get_weights() {
        use clap::Parser;
        let cli =
            crate::Cli::try_parse_from(["btcli-rs", "root", "get-weights", "--netuid", "1", "7"])
                .unwrap();
        match cli.command {
            crate::Command::Root { command: RootCommand::GetWeights { netuid, uid } } => {
                assert_eq!(netuid, 1);
                assert_eq!(uid, 7);
            }
            other => panic!("expected Root::GetWeights, got {other:?}"),
        }
    }

    #[test]
    fn parse_root_claim() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "btcli-rs",
            "root",
            "claim",
            "1,3,7",
            "--password",
            "secret",
        ])
        .unwrap();
        match cli.command {
            crate::Command::Root { command: RootCommand::Claim { subnets, password } } => {
                assert_eq!(subnets, "1,3,7");
                assert_eq!(password.unwrap(), "secret");
            }
            other => panic!("expected Root::Claim, got {other:?}"),
        }
    }

    #[test]
    fn parse_comma_u16_simple() {
        let vals = parse_comma_u16("1,2,3").unwrap();
        assert_eq!(vals, vec![1, 2, 3]);
    }

    #[test]
    fn parse_comma_u16_single() {
        let vals = parse_comma_u16("42").unwrap();
        assert_eq!(vals, vec![42]);
    }

    #[test]
    fn parse_comma_u16_with_spaces() {
        let vals = parse_comma_u16("1, 2, 3").unwrap();
        assert_eq!(vals, vec![1, 2, 3]);
    }

    #[test]
    fn parse_comma_u16_invalid_fails() {
        assert!(parse_comma_u16("1,abc,3").is_err());
    }

    #[test]
    fn parse_comma_u16_empty() {
        let vals = parse_comma_u16("").unwrap();
        assert!(vals.is_empty());
    }

    #[tokio::test]
    async fn register_no_wallet_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "ghost".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let result = exec_register(&config, 1, Some("pw".into())).await;
        assert!(result.is_err(), "register with no wallet should fail");
    }

    #[tokio::test]
    async fn burned_register_no_wallet_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "ghost".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let result = exec_burned_register(&config, 1, Some("pw".into())).await;
        assert!(result.is_err(), "burned_register with no wallet should fail");
    }

    #[tokio::test]
    async fn root_register_no_wallet_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "ghost".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let result = exec_root_register(&config, Some("pw".into())).await;
        assert!(result.is_err(), "root_register with no wallet should fail");
    }

    #[tokio::test]
    async fn root_set_weights_no_wallet_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "ghost".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let result = exec_set_weights(&config, 1, "1,2", "10,20", 0, Some("pw".into())).await;
        assert!(result.is_err(), "root set_weights with no wallet should fail");
    }

    #[tokio::test]
    async fn root_claim_no_wallet_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "ghost".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let result = exec_claim_root(&config, "1,3", Some("pw".into())).await;
        assert!(result.is_err(), "root claim with no wallet should fail");
    }

    #[tokio::test]
    async fn root_get_weights_local_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "ghost".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let result = exec_get_weights(&config, 1, 5).await;
        assert!(result.is_err(), "root get_weights with no local node should fail");
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
    async fn register_created_wallet_chain_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "test-wallet".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let mut wallet = Wallet::with_path(&config.wallet_name, config.wallet_dir());
        wallet.create_coldkey("").expect("create coldkey");
        wallet.create_hotkey().expect("create hotkey");
        let result = exec_register(&config, 1, Some("".into())).await;
        assert!(result.is_err(), "register with created wallet but no chain should fail");
    }

    #[tokio::test]
    async fn burned_register_created_wallet_chain_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "test-wallet".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let mut wallet = Wallet::with_path(&config.wallet_name, config.wallet_dir());
        wallet.create_coldkey("").expect("create coldkey");
        wallet.create_hotkey().expect("create hotkey");
        let result = exec_burned_register(&config, 1, Some("".into())).await;
        assert!(result.is_err(), "burned_register with created wallet but no chain should fail");
    }

    #[tokio::test]
    async fn root_register_created_wallet_chain_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "test-wallet".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let mut wallet = Wallet::with_path(&config.wallet_name, config.wallet_dir());
        wallet.create_coldkey("").expect("create coldkey");
        wallet.create_hotkey().expect("create hotkey");
        let result = exec_root_register(&config, Some("".into())).await;
        assert!(result.is_err(), "root_register with created wallet but no chain should fail");
    }

    #[tokio::test]
    async fn root_set_weights_created_wallet_chain_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "test-wallet".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let mut wallet = Wallet::with_path(&config.wallet_name, config.wallet_dir());
        wallet.create_coldkey("").expect("create coldkey");
        wallet.create_hotkey().expect("create hotkey");
        let result = exec_set_weights(&config, 1, "1,2", "10,20", 0, Some("".into())).await;
        assert!(result.is_err(), "root set_weights with created wallet but no chain should fail");
    }

    #[tokio::test]
    async fn root_claim_created_wallet_chain_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "test-wallet".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let mut wallet = Wallet::with_path(&config.wallet_name, config.wallet_dir());
        wallet.create_coldkey("").expect("create coldkey");
        wallet.create_hotkey().expect("create hotkey");
        let result = exec_claim_root(&config, "1,3", Some("".into())).await;
        assert!(result.is_err(), "root claim with created wallet but no chain should fail");
    }
}
