//! Subnet command group — create, list, info, hyperparameters, and
//! set-identity for Bittensor subnets.

use std::str::FromStr;

use anyhow::{Context, Result};
use clap::Subcommand;

use bittensor_chain::client::SubtensorClient;
use bittensor_wallet::prelude::Wallet;

use crate::config::Config;

/// Subnet subcommands.
#[derive(Debug, Subcommand)]
pub enum SubnetCommand {
    /// Create a new subnet
    Create {
        /// Password to decrypt coldkey (prompted if not provided)
        #[arg(long)]
        password: Option<String>,
    },

    /// List all subnets
    List,

    /// Show subnet info
    Info {
        /// Netuid of the subnet
        #[arg(long)]
        netuid: u16,
    },

    /// Show subnet hyperparameters
    Hyperparameters {
        /// Netuid of the subnet
        #[arg(long)]
        netuid: u16,
    },

    /// Set subnet identity
    #[command(name = "set-identity")]
    SetIdentity {
        /// Netuid of the subnet
        #[arg(long)]
        netuid: u16,

        /// Subnet name
        #[arg(long)]
        name: String,

        /// GitHub repository URL
        #[arg(long)]
        github_repo: String,

        /// Contact info
        #[arg(long)]
        contact: String,

        /// Subnet URL
        #[arg(long)]
        url: String,

        /// Discord link
        #[arg(long)]
        discord: String,

        /// Subnet description
        #[arg(long)]
        description: String,

        /// Password to decrypt coldkey (prompted if not provided)
        #[arg(long)]
        password: Option<String>,
    },
}

impl SubnetCommand {
    /// Dispatch the subnet subcommand.
    pub async fn execute(self, config: &Config) -> Result<()> {
        match self {
            Self::Create { password } => exec_subnet_create(config, password).await,
            Self::List => exec_subnet_list(config).await,
            Self::Info { netuid } => exec_subnet_info(config, netuid).await,
            Self::Hyperparameters { netuid } => exec_subnet_hyperparameters(config, netuid).await,
            Self::SetIdentity {
                netuid,
                name,
                github_repo,
                contact,
                url,
                discord,
                description,
                password,
            } => {
                exec_subnet_set_identity(
                    config,
                    &SubnetIdentityArgs {
                        netuid,
                        name: &name,
                        github_repo: &github_repo,
                        contact: &contact,
                        url: &url,
                        discord: &discord,
                        description: &description,
                    },
                    password,
                )
                .await
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Individual command implementations
// ---------------------------------------------------------------------------

async fn exec_subnet_create(config: &Config, password: Option<String>) -> Result<()> {
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

    let result =
        bittensor_chain::extrinsics::registration::register_subnet(rpc, &inner_kp, hotkey_account)
            .await
            .context("register_subnet extrinsic failed")?;

    println!("Subnet creation submitted successfully.");
    println!("  Block hash:      {}", result.block_hash);
    println!("  Extrinsic hash:  {}", result.extrinsic_hash);
    Ok(())
}

async fn exec_subnet_list(config: &Config) -> Result<()> {
    let client = SubtensorClient::from_config(config.network.clone())
        .await
        .context("failed to connect to chain")?;
    let rpc = client.rpc();

    let total = bittensor_chain::queries::subnet::get_total_subnets(rpc)
        .await
        .context("failed to query total subnets")?;

    println!("Total subnets: {total}");
    for netuid in 0..=total {
        if bittensor_chain::queries::subnet::subnet_exists(rpc, netuid)
            .await
            .context("failed to query subnet existence")?
        {
            let name = bittensor_chain::queries::subnet::get_subnet_name(rpc, netuid)
                .await
                .context("failed to query subnet name")?
                .unwrap_or_default();
            println!("  netuid={netuid} name={name}");
        }
    }

    Ok(())
}

async fn exec_subnet_info(config: &Config, netuid: u16) -> Result<()> {
    let client = SubtensorClient::from_config(config.network.clone())
        .await
        .context("failed to connect to chain")?;
    let rpc = client.rpc();

    let info = bittensor_chain::queries::subnet::get_subnet_info(rpc, netuid)
        .await
        .context("failed to query subnet info")?
        .context("subnet not found")?;

    println!("Subnet Info (netuid={netuid}):");
    println!("  Name:          {}", info.name);
    println!("  Owner hotkey:  {}", info.owner_hotkey);
    println!("  Tempo:         {}", info.tempo);
    println!("  Maximum UID:   {}", info.maximum_uid);

    if let Some(id) = &info.subnet_identity {
        println!("  Identity name: {}", id.name);
        println!("  Identity symbol: {}", id.symbol);
    }

    Ok(())
}

async fn exec_subnet_hyperparameters(config: &Config, netuid: u16) -> Result<()> {
    let client = SubtensorClient::from_config(config.network.clone())
        .await
        .context("failed to connect to chain")?;
    let rpc = client.rpc();

    let hp = bittensor_chain::queries::subnet::get_subnet_hyperparameters(rpc, netuid)
        .await
        .context("failed to query subnet hyperparameters")?
        .context("subnet not found")?;

    println!("Subnet Hyperparameters (netuid={netuid}):");
    println!("  Rho:                   {}", hp.rho);
    println!("  Kappa:                 {}", hp.kappa);
    println!("  Difficulty:            {}", hp.difficulty);
    println!("  Burn:                  {}", hp.burn);
    println!("  Immunity ratio:        {}", hp.immunity_ratio);
    println!("  Min burn:              {}", hp.min_burn);
    println!("  Max burn:              {}", hp.max_burn);
    println!("  Weights rate limit:    {}", hp.weights_rate_limit);
    println!("  Weights version:       {}", hp.weights_version);
    println!("  Max weight limit:      {}", hp.max_weight_limit);
    println!("  Scaling law power:     {}", hp.scaling_law_power);
    println!("  Subnetwork N:          {}", hp.subnetwork_n);
    println!("  Max N:                 {}", hp.max_n);
    println!("  Blocks since last step: {}", hp.blocks_since_last_step);
    println!("  Tempo:                 {}", hp.tempo);
    println!("  Adjustment alpha:      {}", hp.adjustment_alpha);
    println!("  Adjustment interval:   {}", hp.adjustment_interval);
    println!("  Bonds moving avg:      {}", hp.bonds_moving_avg);
    println!("  Alpha high:           {}", hp.alpha_high);
    println!("  Alpha low:            {}", hp.alpha_low);
    println!("  Liquid alpha enabled:  {}", hp.liquid_alpha_enabled);

    Ok(())
}

/// Arguments for setting subnet identity.
struct SubnetIdentityArgs<'a> {
    netuid: u16,
    name: &'a str,
    github_repo: &'a str,
    contact: &'a str,
    url: &'a str,
    discord: &'a str,
    description: &'a str,
}

async fn exec_subnet_set_identity(
    config: &Config,
    args: &SubnetIdentityArgs<'_>,
    password: Option<String>,
) -> Result<()> {
    let pwd = prompt_password(password)?;
    let mut wallet = Wallet::with_path(&config.wallet_name, config.wallet_dir());
    let signer = wallet.get_coldkey_pair(&pwd).context("failed to decrypt coldkey")?;

    let client = SubtensorClient::from_config(config.network.clone())
        .await
        .context("failed to connect to chain")?;
    let rpc = client.rpc();

    let inner_kp = subxt_signer::sr25519::Keypair::from_secret_key(*signer.seed())?;

    let result = bittensor_chain::extrinsics::registration::set_subnet_identity(
        rpc,
        &inner_kp,
        bittensor_chain::extrinsics::registration::SetSubnetIdentityParams {
            netuid: args.netuid,
            subnet_name: args.name.as_bytes().to_vec(),
            github_repo: args.github_repo.as_bytes().to_vec(),
            subnet_contact: args.contact.as_bytes().to_vec(),
            subnet_url: args.url.as_bytes().to_vec(),
            discord: args.discord.as_bytes().to_vec(),
            description: args.description.as_bytes().to_vec(),
            logo_url: Vec::new(),
            additional: Vec::new(),
        },
    )
    .await
    .context("set_subnet_identity extrinsic failed")?;

    println!("Subnet identity set successfully.");
    println!("  Netuid:         {}", args.netuid);
    println!("  Block hash:      {}", result.block_hash);
    println!("  Extrinsic hash:  {}", result.extrinsic_hash);
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers (shared with tests)
// ---------------------------------------------------------------------------

fn prompt_password(password: Option<String>) -> Result<String> {
    match password {
        Some(p) => Ok(p),
        None => Ok(rpassword::prompt_password("Enter coldkey password: ")?),
    }
}

/// Parse a comma-separated string into a Vec<u16>.
// Dead code allowed: utility reserved for future subnet CLI subcommands (e.g., batch set-identity)
#[allow(dead_code)]
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
    use tempfile::TempDir;

    #[test]
    fn subnet_command_debug_format() {
        let cmd = SubnetCommand::List;
        assert!(format!("{cmd:?}").contains("List"));
    }

    #[test]
    fn parse_subnet_create() {
        use clap::Parser;
        let cli =
            crate::Cli::try_parse_from(["btcli-rs", "subnet", "create", "--password", "secret"])
                .unwrap();
        match cli.command {
            crate::Command::Subnet { command: SubnetCommand::Create { password } } => {
                assert_eq!(password.unwrap(), "secret");
            }
            other => panic!("expected Subnet::Create, got {other:?}"),
        }
    }

    #[test]
    fn parse_subnet_list() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from(["btcli-rs", "subnet", "list"]).unwrap();
        match cli.command {
            crate::Command::Subnet { command: SubnetCommand::List } => {}
            other => panic!("expected Subnet::List, got {other:?}"),
        }
    }

    #[test]
    fn parse_subnet_info() {
        use clap::Parser;
        let cli =
            crate::Cli::try_parse_from(["btcli-rs", "subnet", "info", "--netuid", "18"]).unwrap();
        match cli.command {
            crate::Command::Subnet { command: SubnetCommand::Info { netuid } } => {
                assert_eq!(netuid, 18);
            }
            other => panic!("expected Subnet::Info, got {other:?}"),
        }
    }

    #[test]
    fn parse_subnet_hyperparameters() {
        use clap::Parser;
        let cli =
            crate::Cli::try_parse_from(["btcli-rs", "subnet", "hyperparameters", "--netuid", "1"])
                .unwrap();
        match cli.command {
            crate::Command::Subnet { command: SubnetCommand::Hyperparameters { netuid } } => {
                assert_eq!(netuid, 1);
            }
            other => panic!("expected Subnet::Hyperparameters, got {other:?}"),
        }
    }

    #[test]
    fn parse_subnet_set_identity() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "btcli-rs",
            "subnet",
            "set-identity",
            "--netuid",
            "5",
            "--name",
            "my-subnet",
            "--github-repo",
            "https://github.com/example/repo",
            "--contact",
            "admin@example.com",
            "--url",
            "https://example.com",
            "--discord",
            "https://discord.gg/example",
            "--description",
            "A test subnet",
            "--password",
            "pw",
        ])
        .unwrap();
        match cli.command {
            crate::Command::Subnet {
                command:
                    SubnetCommand::SetIdentity {
                        netuid,
                        name,
                        github_repo,
                        contact,
                        url,
                        discord,
                        description,
                        password,
                    },
            } => {
                assert_eq!(netuid, 5);
                assert_eq!(name, "my-subnet");
                assert_eq!(github_repo, "https://github.com/example/repo");
                assert_eq!(contact, "admin@example.com");
                assert_eq!(url, "https://example.com");
                assert_eq!(discord, "https://discord.gg/example");
                assert_eq!(description, "A test subnet");
                assert_eq!(password.unwrap(), "pw");
            }
            other => panic!("expected Subnet::SetIdentity, got {other:?}"),
        }
    }

    #[test]
    fn subnet_command_all_variants_parseable() {
        let variants: Vec<Vec<&str>> = vec![
            vec!["btcli-rs", "subnet", "create"],
            vec!["btcli-rs", "subnet", "list"],
            vec!["btcli-rs", "subnet", "info", "--netuid", "1"],
            vec!["btcli-rs", "subnet", "hyperparameters", "--netuid", "1"],
            vec![
                "btcli-rs",
                "subnet",
                "set-identity",
                "--netuid",
                "1",
                "--name",
                "n",
                "--github-repo",
                "g",
                "--contact",
                "c",
                "--url",
                "u",
                "--discord",
                "d",
                "--description",
                "desc",
            ],
        ];
        use clap::Parser;
        for args in &variants {
            let result = crate::Cli::try_parse_from(args);
            assert!(result.is_ok(), "variant {:?} should be parseable", args);
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
    async fn subnet_create_no_wallet_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "ghost".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let result = exec_subnet_create(&config, Some("pw".into())).await;
        assert!(result.is_err(), "subnet create with no wallet should fail");
    }
}
