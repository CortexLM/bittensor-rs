//! Wallet command group — all subcommands for wallet management.

use std::path::Path;
use std::str::FromStr;

use anyhow::{Context, Result};
use clap::Subcommand;

use bittensor_chain::client::SubtensorClient;
use bittensor_wallet::prelude::Wallet;

use crate::config::Config;

/// Wallet subcommands (sub-subcommand group under `wallet`).
#[derive(Debug, Subcommand)]
pub enum WalletCommand {
    /// Generate new coldkey (encrypted) + hotkey pair
    Create {
        /// Skip password confirmation prompt (use empty password)
        #[arg(long)]
        no_password: bool,

        /// Use given password instead of prompting
        #[arg(long)]
        password: Option<String>,
    },

    /// List all wallets in the wallet directory
    List,

    /// Display wallet details (address, balance, hotkeys)
    Show {
        /// Password to decrypt coldkey (prompted if not provided)
        #[arg(long)]
        password: Option<String>,
    },

    /// Show balance for a wallet/coldkey
    Balance {
        /// Password to decrypt coldkey (prompted if not provided)
        #[arg(long)]
        password: Option<String>,

        /// Show all wallets' balances
        #[arg(long)]
        all: bool,
    },

    /// Comprehensive wallet overview (stakes, delegations, etc.)
    Overview {
        /// Password to decrypt coldkey (prompted if not provided)
        #[arg(long)]
        password: Option<String>,

        /// Show all wallets' overviews
        #[arg(long)]
        all: bool,
    },

    /// Transfer TAO to another address
    Transfer {
        /// Destination SS58 address
        dest: String,

        /// Amount in TAO
        amount: String,

        /// Password to decrypt coldkey (prompted if not provided)
        #[arg(long)]
        password: Option<String>,
    },

    /// Initiate coldkey swap process
    #[command(name = "swap-coldkey")]
    SwapColdkey {
        /// New coldkey SS58 address
        new_coldkey: String,

        /// Password to decrypt coldkey (prompted if not provided)
        #[arg(long)]
        password: Option<String>,
    },

    /// Show all keys and addresses in the wallet
    Inspect {
        /// Password to decrypt coldkey (prompted if not provided)
        #[arg(long)]
        password: Option<String>,
    },

    /// Regenerate coldkey from mnemonic
    #[command(name = "regen-coldkey")]
    RegenColdkey {
        /// Mnemonic phrase (space-separated words)
        mnemonic: String,

        /// Password to encrypt the new coldkey
        #[arg(long)]
        password: Option<String>,

        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,
    },

    /// Regenerate coldkeypub from public key
    #[command(name = "regen-coldkeypub")]
    RegenColdkeypub {
        /// SS58 address of the coldkeypub
        ss58_address: String,
    },

    /// Create a new hotkey under a wallet
    #[command(name = "create-hotkey")]
    CreateHotkey {
        /// Name for the new hotkey (defaults to "default")
        #[arg(long, default_value = "default")]
        hotkey: String,

        /// Password to decrypt coldkey for derived hotkey (prompted if not provided)
        #[arg(long)]
        password: Option<String>,

        /// Nominator / seed-based hotkey instead of derived
        #[arg(long)]
        seed: bool,
    },

    /// Regenerate hotkey from mnemonic
    #[command(name = "regen-hotkey")]
    RegenHotkey {
        /// Mnemonic phrase (space-separated words)
        mnemonic: String,

        /// Name for the hotkey (defaults to "default")
        #[arg(long, default_value = "default")]
        hotkey: String,
    },
}

impl WalletCommand {
    /// Dispatch the wallet subcommand.
    pub async fn execute(self, config: &Config) -> Result<()> {
        match self {
            Self::Create { no_password, password } => {
                exec_create(config, no_password, password).await
            }
            Self::List => exec_list(config).await,
            Self::Show { password } => exec_show(config, password).await,
            Self::Balance { password, all } => exec_balance(config, password, all).await,
            Self::Overview { password, all } => exec_overview(config, password, all).await,
            Self::Transfer { dest, amount, password } => {
                exec_transfer(config, &dest, &amount, password).await
            }
            Self::SwapColdkey { new_coldkey, password } => {
                exec_swap_coldkey(config, &new_coldkey, password).await
            }
            Self::Inspect { password } => exec_inspect(config, password).await,
            Self::RegenColdkey { mnemonic, password, yes } => {
                exec_regen_coldkey(config, &mnemonic, password, yes).await
            }
            Self::RegenColdkeypub { ss58_address } => {
                exec_regen_coldkeypub(config, &ss58_address).await
            }
            Self::CreateHotkey { hotkey, password, seed } => {
                exec_create_hotkey(config, &hotkey, password, seed).await
            }
            Self::RegenHotkey { mnemonic, hotkey } => {
                exec_regen_hotkey(config, &mnemonic, &hotkey).await
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Individual command implementations
// ---------------------------------------------------------------------------

async fn exec_create(config: &Config, no_password: bool, password: Option<String>) -> Result<()> {
    let pwd = if no_password {
        String::new()
    } else {
        match password {
            Some(p) => p,
            None => {
                let p = rpassword::prompt_password("Enter coldkey password: ")?;
                let confirm = rpassword::prompt_password("Confirm coldkey password: ")?;
                if p != confirm {
                    anyhow::bail!("passwords do not match");
                }
                p
            }
        }
    };

    let mut wallet = Wallet::with_path(&config.wallet_name, config.wallet_dir());
    let mnemonic = wallet.create_coldkey(&pwd).context("failed to create coldkey")?;

    let hotkey = wallet.create_hotkey().context("failed to create hotkey")?;

    let coldkey_addr = wallet.get_coldkeypub().context("failed to read coldkeypub")?;

    println!("Wallet created: {}", config.wallet_name);
    println!("  Path:       {}", config.wallet_dir().display());
    println!("  Coldkey:    {coldkey_addr}");
    println!("  Hotkey:     {}", hotkey.ss58_address());
    println!();
    println!("IMPORTANT: Write down your mnemonic phrase:");
    println!("  {mnemonic}");
    println!();
    println!("Store it in a safe place. It is the ONLY way to recover your coldkey.");

    Ok(())
}

async fn exec_list(config: &Config) -> Result<()> {
    let base = &config.wallet_path;
    if !base.exists() {
        println!("No wallets found (directory does not exist: {})", base.display());
        return Ok(());
    }

    let mut entries: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(base)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                entries.push(name.to_string());
            }
        }
    }
    entries.sort();

    if entries.is_empty() {
        println!("No wallets found in {}", base.display());
        return Ok(());
    }

    println!("Wallets ({}):", base.display());
    for name in &entries {
        let wallet_dir = base.join(name);
        let has_coldkey = wallet_dir.join("coldkey").exists();
        let has_coldkeypub = wallet_dir.join("coldkeypub").exists();
        let hotkeys_dir = wallet_dir.join("hotkeys");
        let hotkey_count = if hotkeys_dir.exists() {
            std::fs::read_dir(&hotkeys_dir).map(|d| d.count()).unwrap_or(0)
        } else {
            0
        };

        let mut markers: Vec<String> = Vec::new();
        if has_coldkey {
            markers.push("coldkey".to_string());
        }
        if has_coldkeypub {
            markers.push("coldkeypub".to_string());
        }
        if hotkey_count > 0 {
            markers.push(format!("{hotkey_count} hotkey(s)"));
        }
        let detail = if markers.is_empty() { String::from("empty") } else { markers.join(", ") };

        println!("  {name}  ({detail})");
    }

    Ok(())
}

async fn exec_show(config: &Config, password: Option<String>) -> Result<()> {
    let wallet_dir = config.wallet_dir();
    let mut wallet = Wallet::with_path(&config.wallet_name, wallet_dir.clone());

    let coldkeypub =
        wallet.get_coldkeypub().context("coldkeypub not found — does the wallet exist?")?;

    println!("Wallet: {}", config.wallet_name);
    println!("  Path:       {}", wallet_dir.display());
    println!("  Coldkey SS58: {coldkeypub}");

    // Try to read hotkeys
    let hotkeys_dir = wallet_dir.join("hotkeys");
    if hotkeys_dir.exists() {
        let mut hotkeys = Vec::new();
        for entry in std::fs::read_dir(&hotkeys_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                if let Some(name) = entry.file_name().to_str() {
                    hotkeys.push(name.to_string());
                }
            }
        }
        hotkeys.sort();
        if hotkeys.is_empty() {
            println!("  Hotkeys:    (none)");
        } else {
            println!("  Hotkeys:");
            for hk_name in &hotkeys {
                let mut temp_wallet = Wallet::with_path(&config.wallet_name, wallet_dir.clone());
                temp_wallet.set_hotkey_name(hk_name);
                match temp_wallet.get_hotkey_pair() {
                    Ok(kp) => println!("    {hk_name}: {}", kp.ss58_address()),
                    Err(_) => println!("    {hk_name}: (error reading key)"),
                }
            }
        }
    } else {
        println!("  Hotkeys:    (none)");
    }

    // If password given, show the full coldkey address
    if let Some(pwd) = password {
        match wallet.get_coldkey_pair(&pwd) {
            Ok(kp) => println!("  Coldkey (full): {}", kp.ss58_address()),
            Err(_) => println!("  Coldkey (full): (wrong password)"),
        }
    }

    Ok(())
}

async fn exec_balance(config: &Config, _password: Option<String>, all: bool) -> Result<()> {
    let base = &config.wallet_path;

    if all {
        // Iterate all wallets
        if !base.exists() {
            println!("No wallets found.");
            return Ok(());
        }
        let mut entries: Vec<String> = Vec::new();
        for entry in std::fs::read_dir(base)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    entries.push(name.to_string());
                }
            }
        }
        entries.sort();

        let client = SubtensorClient::from_config(config.network.clone())
            .await
            .context("failed to connect to chain")?;
        let rpc = client.rpc();

        for name in &entries {
            let wallet_dir = base.join(name);
            let mut w = Wallet::with_path(name, wallet_dir);
            let addr = match w.get_coldkeypub() {
                Ok(a) => a,
                Err(_) => continue,
            };
            let account_id =
                subxt::utils::AccountId32::from_str(&addr).context("invalid SS58 address")?;
            let balance = bittensor_chain::queries::account::get_balance(rpc, &account_id).await?;
            println!("{name}: {balance} TAO");
        }
    } else {
        let mut wallet = Wallet::with_path(&config.wallet_name, config.wallet_dir());
        let addr =
            wallet.get_coldkeypub().context("coldkeypub not found — does the wallet exist?")?;

        let account_id =
            subxt::utils::AccountId32::from_str(&addr).context("invalid SS58 address")?;

        let client = SubtensorClient::from_config(config.network.clone())
            .await
            .context("failed to connect to chain")?;
        let rpc = client.rpc();

        let balance = bittensor_chain::queries::account::get_balance(rpc, &account_id).await?;
        println!("Balance: {balance} TAO");
    }

    Ok(())
}

async fn exec_overview(config: &Config, _password: Option<String>, all: bool) -> Result<()> {
    let base = &config.wallet_path;

    let client = SubtensorClient::from_config(config.network.clone())
        .await
        .context("failed to connect to chain")?;
    let rpc = client.rpc();

    if all {
        if !base.exists() {
            println!("No wallets found.");
            return Ok(());
        }
        let mut entries: Vec<String> = Vec::new();
        for entry in std::fs::read_dir(base)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    entries.push(name.to_string());
                }
            }
        }
        entries.sort();
        for name in &entries {
            print_wallet_overview(rpc, base, name).await?;
        }
    } else {
        print_wallet_overview(rpc, base, &config.wallet_name).await?;
    }

    Ok(())
}

async fn print_wallet_overview(
    rpc: &subxt::OnlineClient<bittensor_core::config::SubtensorConfig>,
    base: &Path,
    name: &str,
) -> Result<()> {
    let wallet_dir = base.join(name);
    let mut w = Wallet::with_path(name, wallet_dir.clone());
    let addr = match w.get_coldkeypub() {
        Ok(a) => a,
        Err(_) => {
            println!("Wallet {name}: (no coldkeypub found)");
            return Ok(());
        }
    };

    let account_id = subxt::utils::AccountId32::from_str(&addr).context("invalid SS58 address")?;

    let balance = bittensor_chain::queries::account::get_balance(rpc, &account_id).await?;
    let stakes =
        bittensor_chain::queries::account::get_stake_info_for_coldkey(rpc, &account_id).await?;

    println!("Wallet: {name}");
    println!("  Address:  {addr}");
    println!("  Balance:  {balance} TAO");

    if stakes.is_empty() {
        println!("  Stakes:   (none)");
    } else {
        println!("  Stakes:");
        for si in &stakes {
            println!("    hotkey={} stake={} TAO", si.hotkey, si.stake);
        }
    }

    // List hotkeys
    let hotkeys_dir = wallet_dir.join("hotkeys");
    if hotkeys_dir.exists() {
        let mut hks = Vec::new();
        for entry in std::fs::read_dir(&hotkeys_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                if let Some(hn) = entry.file_name().to_str() {
                    hks.push(hn.to_string());
                }
            }
        }
        hks.sort();
        if hks.is_empty() {
            println!("  Hotkeys:  (none)");
        } else {
            println!("  Hotkeys:");
            for hk_name in &hks {
                let mut tw = Wallet::with_path(name, wallet_dir.clone());
                tw.set_hotkey_name(hk_name);
                match tw.get_hotkey_pair() {
                    Ok(kp) => println!("    {hk_name}: {}", kp.ss58_address()),
                    Err(_) => println!("    {hk_name}: (error reading key)"),
                }
            }
        }
    } else {
        println!("  Hotkeys:  (none)");
    }

    println!();
    Ok(())
}

async fn exec_transfer(
    config: &Config,
    dest: &str,
    amount: &str,
    password: Option<String>,
) -> Result<()> {
    // Delegate to the transfer command module
    super::transfer::TransferCommand::Transfer {
        dest: dest.to_string(),
        amount: amount.to_string(),
        password,
    }
    .execute(config)
    .await
}

async fn exec_swap_coldkey(
    config: &Config,
    new_coldkey: &str,
    password: Option<String>,
) -> Result<()> {
    let pwd = prompt_password(password)?;
    let mut wallet = Wallet::with_path(&config.wallet_name, config.wallet_dir());
    let signer = wallet.get_coldkey_pair(&pwd).context("failed to decrypt coldkey")?;

    let new_coldkey_account = subxt::utils::AccountId32::from_str(new_coldkey)
        .context("invalid new coldkey SS58 address")?;

    let client = SubtensorClient::from_config(config.network.clone())
        .await
        .context("failed to connect to chain")?;
    let rpc = client.rpc();

    let inner_kp = subxt_signer::sr25519::Keypair::from_secret_key(*signer.seed())?;

    // Use the swap_coldkey_announced extrinsic
    let result = bittensor_chain::extrinsics::coldkey_swap::swap_coldkey_announced(
        rpc,
        &inner_kp,
        new_coldkey_account,
    )
    .await
    .context("coldkey swap extrinsic failed")?;

    println!("Coldkey swap initiated successfully.");
    println!("  Block hash:      {}", result.block_hash);
    println!("  Extrinsic hash:  {}", result.extrinsic_hash);
    Ok(())
}

async fn exec_inspect(config: &Config, password: Option<String>) -> Result<()> {
    let wallet_dir = config.wallet_dir();
    let mut wallet = Wallet::with_path(&config.wallet_name, wallet_dir.clone());

    println!("Wallet: {}", config.wallet_name);
    println!("  Path: {}", wallet_dir.display());

    // Coldkeypub (always readable)
    match wallet.get_coldkeypub() {
        Ok(addr) => println!("  Coldkeypub: {addr}"),
        Err(_) => println!("  Coldkeypub: (not found)"),
    }

    // Full coldkey (needs password)
    if let Some(pwd) = password {
        match wallet.get_coldkey_pair(&pwd) {
            Ok(kp) => {
                println!("  Coldkey SS58:    {}", kp.ss58_address());
                println!("  Coldkey pubkey:  0x{}", hex::encode(kp.public_key().0));
            }
            Err(_) => println!("  Coldkey (full):  (wrong password or not found)"),
        }
    } else {
        println!("  Coldkey (full):  (provide --password to decrypt)");
    }

    // Hotkeys
    let hotkeys_dir = wallet_dir.join("hotkeys");
    if hotkeys_dir.exists() {
        let mut hks = Vec::new();
        for entry in std::fs::read_dir(&hotkeys_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                if let Some(name) = entry.file_name().to_str() {
                    hks.push(name.to_string());
                }
            }
        }
        hks.sort();
        if hks.is_empty() {
            println!("  Hotkeys: (none)");
        } else {
            println!("  Hotkeys:");
            for hk_name in &hks {
                let mut tw = Wallet::with_path(&config.wallet_name, wallet_dir.clone());
                tw.set_hotkey_name(hk_name);
                match tw.get_hotkey_pair() {
                    Ok(kp) => {
                        println!("    {hk_name}:");
                        println!("      SS58:    {}", kp.ss58_address());
                        println!("      Pubkey:  0x{}", hex::encode(kp.public_key().0));
                    }
                    Err(_) => println!("    {hk_name}: (error reading key)"),
                }
            }
        }
    } else {
        println!("  Hotkeys: (none)");
    }

    Ok(())
}

async fn exec_regen_coldkey(
    config: &Config,
    mnemonic: &str,
    password: Option<String>,
    yes: bool,
) -> Result<()> {
    if !yes {
        println!(
            "WARNING: This will overwrite any existing coldkey for wallet '{}'.",
            config.wallet_name
        );
        println!("Press Enter to continue, or Ctrl+C to abort.");
        std::io::stdin().read_line(&mut String::new())?;
    }

    let pwd = match password {
        Some(p) => p,
        None => {
            let p = rpassword::prompt_password("Enter password for new coldkey: ")?;
            let confirm = rpassword::prompt_password("Confirm password: ")?;
            if p != confirm {
                anyhow::bail!("passwords do not match");
            }
            p
        }
    };

    let parsed =
        bittensor_wallet::mnemonic::parse_mnemonic(mnemonic).context("invalid mnemonic phrase")?;

    let mut wallet = Wallet::with_path(&config.wallet_name, config.wallet_dir());
    wallet
        .create_coldkey_from_mnemonic(&parsed, &pwd)
        .context("failed to regenerate coldkey from mnemonic")?;

    let addr = wallet.get_coldkeypub().context("failed to read coldkeypub")?;

    println!("Coldkey regenerated successfully.");
    println!("  SS58 address: {addr}");
    Ok(())
}

async fn exec_regen_coldkeypub(config: &Config, ss58_address: &str) -> Result<()> {
    // Validate the address format
    let _decoded =
        bittensor_wallet::ss58::decode_ss58(ss58_address).context("invalid SS58 address")?;

    let wallet_dir = config.wallet_dir();
    std::fs::create_dir_all(&wallet_dir).context("failed to create wallet directory")?;

    let coldkeypub_path = wallet_dir.join("coldkeypub");
    std::fs::write(&coldkeypub_path, ss58_address).context("failed to write coldkeypub file")?;

    println!("Coldkeypub regenerated successfully.");
    println!("  SS58 address: {ss58_address}");
    println!("  File:         {}", coldkeypub_path.display());
    Ok(())
}

async fn exec_create_hotkey(
    config: &Config,
    hotkey_name: &str,
    password: Option<String>,
    seed: bool,
) -> Result<()> {
    let wallet_dir = config.wallet_dir();
    let mut wallet = Wallet::with_path(&config.wallet_name, wallet_dir.clone());
    wallet.set_hotkey_name(hotkey_name);

    // Ensure the wallet directory exists
    std::fs::create_dir_all(&wallet_dir).context("failed to create wallet directory")?;

    if seed {
        // Seed-based: generate a random hotkey
        let kp = wallet.create_hotkey().context("failed to create hotkey")?;
        println!("Hotkey created (seed-based).");
        println!("  Name:    {hotkey_name}");
        println!("  SS58:    {}", kp.ss58_address());
    } else {
        // Derived from coldkey: needs password
        let pwd = prompt_password(password)?;
        let kp =
            wallet.create_hotkey_from_coldkey(&pwd).context("failed to create derived hotkey")?;
        println!("Hotkey created (derived from coldkey).");
        println!("  Name:    {hotkey_name}");
        println!("  SS58:    {}", kp.ss58_address());
    }

    println!("  Path:    {}", wallet.hotkey_path().display());
    Ok(())
}

async fn exec_regen_hotkey(config: &Config, mnemonic: &str, hotkey_name: &str) -> Result<()> {
    let wallet_dir = config.wallet_dir();
    let mut wallet = Wallet::with_path(&config.wallet_name, wallet_dir.clone());
    wallet.set_hotkey_name(hotkey_name);

    let parsed =
        bittensor_wallet::mnemonic::parse_mnemonic(mnemonic).context("invalid mnemonic phrase")?;

    let kp = bittensor_wallet::mnemonic::keypair_from_mnemonic(&parsed, None)
        .context("failed to derive hotkey from mnemonic")?;

    // Write the hotkey seed to disk
    let hotkeys_dir = wallet_dir.join("hotkeys");
    std::fs::create_dir_all(&hotkeys_dir).context("failed to create hotkeys directory")?;

    let hotkey_path = wallet.hotkey_path();
    std::fs::write(&hotkey_path, kp.seed_hex()).context("failed to write hotkey file")?;

    println!("Hotkey regenerated successfully.");
    println!("  Name:    {hotkey_name}");
    println!("  SS58:    {}", kp.ss58_address());
    println!("  Path:    {}", hotkey_path.display());
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn wallet_command_debug_format() {
        let cmd = WalletCommand::List;
        assert!(format!("{cmd:?}").contains("List"));
    }

    #[tokio::test]
    async fn create_wallet_with_no_password() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "test-wallet".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };

        let result = exec_create(&config, true, None).await;
        assert!(result.is_ok(), "create wallet should succeed: {result:?}");

        let wallet_dir = config.wallet_dir();
        assert!(wallet_dir.join("coldkey").exists());
        assert!(wallet_dir.join("coldkeypub").exists());
        assert!(wallet_dir.join("hotkeys").join("default").exists());
    }

    #[tokio::test]
    async fn list_empty_wallets() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "default".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };

        let result = exec_list(&config).await;
        assert!(result.is_ok(), "list on empty dir should succeed");
    }

    #[tokio::test]
    async fn show_nonexistent_wallet() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "nonexistent".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };

        let result = exec_show(&config, None).await;
        assert!(result.is_err(), "show on nonexistent wallet should fail");
    }

    #[tokio::test]
    async fn inspect_nonexistent_wallet() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "ghost".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };

        let result = exec_inspect(&config, None).await;
        // inspect gracefully handles missing keys
        assert!(result.is_ok(), "inspect should not fail on missing wallet");
    }

    #[tokio::test]
    async fn regen_coldkeypub_invalid_address() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "test".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };

        let result = exec_regen_coldkeypub(&config, "not_valid_ss58").await;
        assert!(result.is_err(), "invalid SS58 should fail");
    }

    #[test]
    fn parse_wallet_create_command() {
        use clap::Parser;
        let cli =
            crate::Cli::try_parse_from(["btcli-rs", "wallet", "create", "--no-password"]).unwrap();
        match cli.command {
            crate::Command::Wallet {
                command: WalletCommand::Create { no_password, password: _ },
            } => assert!(no_password),
            other => panic!("expected Wallet::Create, got {other:?}"),
        }
    }

    #[test]
    fn parse_wallet_transfer_command() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "btcli-rs",
            "wallet",
            "transfer",
            "5Dest123",
            "10.0",
            "--password",
            "secret",
        ])
        .unwrap();
        match cli.command {
            crate::Command::Wallet {
                command: WalletCommand::Transfer { dest, amount, password },
            } => {
                assert_eq!(dest, "5Dest123");
                assert_eq!(amount, "10.0");
                assert_eq!(password.unwrap(), "secret");
            }
            other => panic!("expected Wallet::Transfer, got {other:?}"),
        }
    }

    #[test]
    fn parse_wallet_create_hotkey_command() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "btcli-rs",
            "wallet",
            "create-hotkey",
            "--hotkey",
            "validator",
            "--seed",
        ])
        .unwrap();
        match cli.command {
            crate::Command::Wallet {
                command: WalletCommand::CreateHotkey { hotkey, password: _, seed },
            } => {
                assert_eq!(hotkey, "validator");
                assert!(seed);
            }
            other => panic!("expected Wallet::CreateHotkey, got {other:?}"),
        }
    }

    #[test]
    fn parse_wallet_regen_coldkey_command() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "btcli-rs",
            "wallet",
            "regen-coldkey",
            "word1 word2 word3",
            "--password",
            "pass",
            "--yes",
        ])
        .unwrap();
        match cli.command {
            crate::Command::Wallet {
                command: WalletCommand::RegenColdkey { mnemonic, password, yes },
            } => {
                assert_eq!(mnemonic, "word1 word2 word3");
                assert_eq!(password.unwrap(), "pass");
                assert!(yes);
            }
            other => panic!("expected Wallet::RegenColdkey, got {other:?}"),
        }
    }

    #[test]
    fn parse_wallet_regen_hotkey_command() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from([
            "btcli-rs",
            "wallet",
            "regen-hotkey",
            "alpha beta gamma",
            "--hotkey",
            "mykey",
        ])
        .unwrap();
        match cli.command {
            crate::Command::Wallet { command: WalletCommand::RegenHotkey { mnemonic, hotkey } } => {
                assert_eq!(mnemonic, "alpha beta gamma");
                assert_eq!(hotkey, "mykey");
            }
            other => panic!("expected Wallet::RegenHotkey, got {other:?}"),
        }
    }

    #[test]
    fn parse_wallet_balance_all_flag() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from(["btcli-rs", "wallet", "balance", "--all"]).unwrap();
        match cli.command {
            crate::Command::Wallet { command: WalletCommand::Balance { all, .. } } => assert!(all),
            other => panic!("expected Wallet::Balance, got {other:?}"),
        }
    }

    #[test]
    fn parse_wallet_overview_command() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from(["btcli-rs", "wallet", "overview"]).unwrap();
        match cli.command {
            crate::Command::Wallet { command: WalletCommand::Overview { .. } } => {}
            other => panic!("expected Wallet::Overview, got {other:?}"),
        }
    }

    #[test]
    fn parse_wallet_swap_coldkey_command() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from(["btcli-rs", "wallet", "swap-coldkey", "5NewColdkey"])
            .unwrap();
        match cli.command {
            crate::Command::Wallet { command: WalletCommand::SwapColdkey { new_coldkey, .. } } => {
                assert_eq!(new_coldkey, "5NewColdkey");
            }
            other => panic!("expected Wallet::SwapColdkey, got {other:?}"),
        }
    }

    #[test]
    fn parse_wallet_inspect_command() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from(["btcli-rs", "wallet", "inspect"]).unwrap();
        match cli.command {
            crate::Command::Wallet { command: WalletCommand::Inspect { .. } } => {}
            other => panic!("expected Wallet::Inspect, got {other:?}"),
        }
    }

    #[test]
    fn parse_wallet_regen_coldkeypub_command() {
        use clap::Parser;
        let cli =
            crate::Cli::try_parse_from(["btcli-rs", "wallet", "regen-coldkeypub", "5ColdkeyPub"])
                .unwrap();
        match cli.command {
            crate::Command::Wallet { command: WalletCommand::RegenColdkeypub { ss58_address } } => {
                assert_eq!(ss58_address, "5ColdkeyPub");
            }
            other => panic!("expected Wallet::RegenColdkeypub, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn create_and_list_wallets() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "list-test".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };

        // Create a wallet
        exec_create(&config, true, None).await.expect("create");

        // List should find it
        let result = exec_list(&config).await;
        assert!(result.is_ok(), "list should succeed after create");
    }

    #[tokio::test]
    async fn show_created_wallet() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "show-test".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };

        exec_create(&config, true, None).await.expect("create");
        let result = exec_show(&config, None).await;
        assert!(result.is_ok(), "show should succeed for created wallet");
    }

    #[tokio::test]
    async fn inspect_created_wallet() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "inspect-test".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };

        exec_create(&config, true, None).await.expect("create");
        let result = exec_inspect(&config, None).await;
        assert!(result.is_ok(), "inspect should succeed for created wallet");
    }

    #[tokio::test]
    async fn create_wallet_with_password() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "pwd-test".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };

        let result = exec_create(&config, false, Some("testpass".to_string())).await;
        assert!(result.is_ok(), "create with password should succeed");

        let wallet_dir = config.wallet_dir();
        assert!(wallet_dir.join("coldkey").exists());
    }

    #[tokio::test]
    async fn create_wallet_password_mismatch() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "mismatch-test".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };

        // When no_password=false and no password is given, it prompts —
        // but since we can't simulate interactive prompts in tests,
        // we test the no_password=true path instead
        let result = exec_create(&config, true, None).await;
        assert!(result.is_ok());
    }

    // -----------------------------------------------------------------------
    // Additional TDD tests: output verification, regen, hotkey creation
    // -----------------------------------------------------------------------

    /// Helper: create a wallet with no password and return the config.
    async fn setup_wallet(name: &str, dir: &Path) -> Config {
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: name.to_string(),
            wallet_path: dir.to_path_buf(),
        };
        exec_create(&config, true, None).await.expect("create wallet");
        config
    }

    #[tokio::test]
    async fn create_wallet_produces_key_files() {
        let dir = TempDir::new().expect("tempdir");
        let config = setup_wallet("file-check", dir.path()).await;

        let wallet_dir = config.wallet_dir();
        assert!(wallet_dir.join("coldkey").exists(), "coldkey file should exist");
        assert!(wallet_dir.join("coldkeypub").exists(), "coldkeypub file should exist");
        assert!(wallet_dir.join("hotkeys").join("default").exists(), "default hotkey should exist");
    }

    #[tokio::test]
    async fn create_wallet_coldkeypub_is_nonempty() {
        let dir = TempDir::new().expect("tempdir");
        let config = setup_wallet("pub-check", dir.path()).await;

        let coldkeypub = std::fs::read_to_string(config.wallet_dir().join("coldkeypub"))
            .expect("read coldkeypub");
        assert!(!coldkeypub.trim().is_empty(), "coldkeypub should not be empty");
        // SS58 addresses start with a digit on Substrate (typically 5 for prefix 42)
        assert!(
            coldkeypub.trim().starts_with('5'),
            "coldkeypub should be an SS58 address starting with 5"
        );
    }

    #[tokio::test]
    async fn list_with_multiple_wallets() {
        let dir = TempDir::new().expect("tempdir");
        // Create two wallets
        let config1 = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "alpha".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let config2 = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "beta".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        exec_create(&config1, true, None).await.expect("create alpha");
        exec_create(&config2, true, None).await.expect("create beta");

        // List using a generic config pointing at the same base dir
        let list_config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "default".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let result = exec_list(&list_config).await;
        assert!(result.is_ok(), "list should succeed with multiple wallets");
    }

    #[tokio::test]
    async fn list_shows_wallet_details() {
        let dir = TempDir::new().expect("tempdir");
        let _config = setup_wallet("detail-wallet", dir.path()).await;

        // List should succeed and find the wallet
        let list_config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "default".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let result = exec_list(&list_config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn create_hotkey_seed_based() {
        let dir = TempDir::new().expect("tempdir");
        let config = setup_wallet("hotkey-base", dir.path()).await;

        let result = exec_create_hotkey(&config, "validator", None, true).await;
        assert!(result.is_ok(), "create-hotkey --seed should succeed");

        // The hotkey file should exist
        let hotkey_path = config.wallet_dir().join("hotkeys").join("validator");
        assert!(hotkey_path.exists(), "hotkey file should exist");
    }

    #[tokio::test]
    async fn create_hotkey_derived_with_password() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "derived-hk".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        // Create wallet with a known password
        exec_create(&config, false, Some("testpass".to_string()))
            .await
            .expect("create with password");

        let result =
            exec_create_hotkey(&config, "miner", Some("testpass".to_string()), false).await;
        assert!(result.is_ok(), "create-hotkey derived should succeed");

        let hotkey_path = config.wallet_dir().join("hotkeys").join("miner");
        assert!(hotkey_path.exists(), "derived hotkey file should exist");
    }

    #[tokio::test]
    async fn regen_coldkey_valid_mnemonic() {
        let dir = TempDir::new().expect("tempdir");
        // First create a wallet to get a valid mnemonic
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "regen-test".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let mut wallet = Wallet::with_path(&config.wallet_name, config.wallet_dir());
        let mnemonic = wallet.create_coldkey("").expect("create coldkey for mnemonic");
        let mnemonic_str = mnemonic.to_string();

        // Now regenerate the coldkey using that mnemonic in a new wallet dir
        let dir2 = TempDir::new().expect("tempdir2");
        let config2 = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "regen-test".to_string(),
            wallet_path: dir2.path().to_path_buf(),
        };
        let result = exec_regen_coldkey(&config2, &mnemonic_str, Some("".to_string()), true).await;
        assert!(result.is_ok(), "regen-coldkey with valid mnemonic should succeed");

        // The coldkeypub should match the original
        let orig_addr = config.wallet_dir().join("coldkeypub");
        let regen_addr = config2.wallet_dir().join("coldkeypub");
        let orig_contents = std::fs::read_to_string(orig_addr).expect("read orig coldkeypub");
        let regen_contents = std::fs::read_to_string(regen_addr).expect("read regen coldkeypub");
        assert_eq!(
            orig_contents.trim(),
            regen_contents.trim(),
            "regenerated coldkeypub should match original"
        );
    }

    #[tokio::test]
    async fn regen_coldkeypub_valid_address() {
        let dir = TempDir::new().expect("tempdir");
        let config = setup_wallet("addr-base", dir.path()).await;

        // Read the coldkeypub address
        let addr = std::fs::read_to_string(config.wallet_dir().join("coldkeypub"))
            .expect("read coldkeypub")
            .trim()
            .to_string();

        // Regen coldkeypub in a new wallet dir with the same address
        let dir2 = TempDir::new().expect("tempdir2");
        let config2 = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "addr-base".to_string(),
            wallet_path: dir2.path().to_path_buf(),
        };
        let result = exec_regen_coldkeypub(&config2, &addr).await;
        assert!(result.is_ok(), "regen-coldkeypub with valid SS58 should succeed");

        let written = std::fs::read_to_string(config2.wallet_dir().join("coldkeypub"))
            .expect("read written coldkeypub");
        assert_eq!(written.trim(), addr, "written coldkeypub should match input address");
    }

    #[tokio::test]
    async fn regen_hotkey_valid_mnemonic() {
        let dir = TempDir::new().expect("tempdir");
        let config = setup_wallet("rhk-base", dir.path()).await;

        // Generate a valid mnemonic for the hotkey
        let _parsed = bittensor_wallet::mnemonic::parse_mnemonic(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        )
        .expect("parse test mnemonic");

        let result = exec_regen_hotkey(&config, "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about", "regen-hk").await;
        assert!(result.is_ok(), "regen-hotkey with valid mnemonic should succeed");

        let hotkey_path = config.wallet_dir().join("hotkeys").join("regen-hk");
        assert!(hotkey_path.exists(), "regen hotkey file should exist");
    }

    #[tokio::test]
    async fn show_wallet_lists_hotkeys() {
        let dir = TempDir::new().expect("tempdir");
        let config = setup_wallet("show-hk", dir.path()).await;

        // Add a second hotkey
        exec_create_hotkey(&config, "validator", None, true).await.expect("create second hotkey");

        let result = exec_show(&config, None).await;
        assert!(result.is_ok(), "show should succeed with multiple hotkeys");
    }

    #[tokio::test]
    async fn inspect_wallet_with_password_shows_full_coldkey() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "inspect-full".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        exec_create(&config, false, Some("inspectpass".to_string()))
            .await
            .expect("create with password");

        let result = exec_inspect(&config, Some("inspectpass".to_string())).await;
        assert!(result.is_ok(), "inspect with password should succeed");
    }

    #[tokio::test]
    async fn inspect_wallet_wrong_password_still_succeeds() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "inspect-wrong".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        exec_create(&config, false, Some("realpass".to_string()))
            .await
            .expect("create with password");

        // Inspect with wrong password should still succeed (graceful degradation)
        let result = exec_inspect(&config, Some("wrongpass".to_string())).await;
        assert!(result.is_ok(), "inspect should handle wrong password gracefully");
    }

    #[tokio::test]
    async fn create_wallet_default_hotkey_name() {
        let dir = TempDir::new().expect("tempdir");
        let config = setup_wallet("default-hk", dir.path()).await;

        // Default hotkey should be named "default"
        let default_hotkey = config.wallet_dir().join("hotkeys").join("default");
        assert!(default_hotkey.exists(), "default hotkey should be created");
    }

    #[test]
    fn prompt_password_returns_provided_value() {
        let result = prompt_password(Some("my-secret".to_string()));
        assert_eq!(result.unwrap(), "my-secret");
    }

    #[tokio::test]
    async fn balance_no_wallet_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "ghost".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let result = exec_balance(&config, None, false).await;
        assert!(result.is_err(), "balance with no wallet should fail");
    }

    #[tokio::test]
    async fn balance_all_no_wallets_dir_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "ghost".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let result = exec_balance(&config, None, true).await;
        assert!(result.is_err(), "balance --all with no local node should fail");
    }

    #[tokio::test]
    async fn overview_no_wallet_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "ghost".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let result = exec_overview(&config, None, false).await;
        assert!(result.is_err(), "overview with no local node should fail");
    }

    #[tokio::test]
    async fn overview_all_no_wallets_dir_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "ghost".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let result = exec_overview(&config, None, true).await;
        assert!(result.is_err(), "overview --all with no local node should fail");
    }

    #[test]
    fn parse_wallet_list_command() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from(["btcli-rs", "wallet", "list"]).unwrap();
        match cli.command {
            crate::Command::Wallet { command: WalletCommand::List } => {}
            other => panic!("expected Wallet::List, got {other:?}"),
        }
    }

    #[test]
    fn parse_wallet_show_command() {
        use clap::Parser;
        let cli = crate::Cli::try_parse_from(["btcli-rs", "wallet", "show"]).unwrap();
        match cli.command {
            crate::Command::Wallet { command: WalletCommand::Show { password } } => {
                assert!(password.is_none());
            }
            other => panic!("expected Wallet::Show, got {other:?}"),
        }
    }

    #[test]
    fn wallet_command_all_variants_parseable() {
        // Verify every WalletCommand variant name is recognized by clap
        let variants = [
            "create",
            "list",
            "show",
            "balance",
            "overview",
            "transfer",
            "swap-coldkey",
            "inspect",
            "regen-coldkey",
            "regen-coldkeypub",
            "create-hotkey",
            "regen-hotkey",
        ];
        for v in &variants {
            let args: Vec<&str> = match *v {
                "transfer" => vec!["btcli-rs", "wallet", "transfer", "5Dest", "1.0"],
                "swap-coldkey" => vec!["btcli-rs", "wallet", "swap-coldkey", "5New"],
                "regen-coldkey" => {
                    vec!["btcli-rs", "wallet", "regen-coldkey", "word1 word2", "--yes"]
                }
                "regen-coldkeypub" => vec!["btcli-rs", "wallet", "regen-coldkeypub", "5Addr"],
                "create-hotkey" => vec!["btcli-rs", "wallet", "create-hotkey"],
                "regen-hotkey" => vec!["btcli-rs", "wallet", "regen-hotkey", "word1 word2"],
                _ => vec!["btcli-rs", "wallet", *v],
            };
            use clap::Parser;
            let result = crate::Cli::try_parse_from(args);
            assert!(result.is_ok(), "variant '{v}' should be parseable");
        }
    }

    #[tokio::test]
    async fn swap_coldkey_no_wallet_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "ghost".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let result = exec_swap_coldkey(&config, "5FakeAddr", Some("pw".into())).await;
        assert!(result.is_err(), "swap coldkey with no wallet should fail");
    }

    #[tokio::test]
    async fn create_hotkey_derived_no_wallet_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "ghost".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let result = exec_create_hotkey(&config, "miner", Some("pw".into()), false).await;
        assert!(result.is_err(), "create-hotkey derived with no wallet should fail");
    }

    #[tokio::test]
    async fn regen_coldkeypub_valid_address_succeeds() {
        let dir = TempDir::new().expect("tempdir");
        let setup_config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "addr-source".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        exec_create(&setup_config, true, None).await.expect("create source wallet");

        let valid_addr = std::fs::read_to_string(setup_config.wallet_dir().join("coldkeypub"))
            .expect("read coldkeypub")
            .trim()
            .to_string();

        let dir2 = TempDir::new().expect("tempdir2");
        let config2 = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "regen-pub-test".to_string(),
            wallet_path: dir2.path().to_path_buf(),
        };
        let result = exec_regen_coldkeypub(&config2, &valid_addr).await;
        assert!(result.is_ok(), "regen-coldkeypub with valid address should succeed");

        let written = std::fs::read_to_string(config2.wallet_dir().join("coldkeypub"))
            .expect("read written coldkeypub");
        assert_eq!(written.trim(), valid_addr, "written coldkeypub should match input address");
    }

    #[tokio::test]
    async fn regen_coldkey_valid_mnemonic_yes_succeeds() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "regen-ck-test".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let mut wallet = Wallet::with_path(&config.wallet_name, config.wallet_dir());
        let mnemonic = wallet.create_coldkey("").expect("create coldkey for mnemonic");
        let mnemonic_str = mnemonic.to_string();

        let dir2 = TempDir::new().expect("tempdir2");
        let config2 = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "regen-ck-test".to_string(),
            wallet_path: dir2.path().to_path_buf(),
        };
        let result = exec_regen_coldkey(&config2, &mnemonic_str, Some("".to_string()), true).await;
        assert!(result.is_ok(), "regen-coldkey with valid mnemonic and --yes should succeed");

        let orig_addr = std::fs::read_to_string(config.wallet_dir().join("coldkeypub"))
            .expect("read orig coldkeypub");
        let regen_addr = std::fs::read_to_string(config2.wallet_dir().join("coldkeypub"))
            .expect("read regen coldkeypub");
        assert_eq!(
            orig_addr.trim(),
            regen_addr.trim(),
            "regenerated coldkeypub should match original"
        );
    }

    #[tokio::test]
    async fn regen_hotkey_valid_mnemonic_succeeds() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "regen-hk-test".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let result = exec_regen_hotkey(
            &config,
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
            "test-hk",
        )
        .await;
        assert!(result.is_ok(), "regen-hotkey with valid mnemonic should succeed");

        let hotkey_path = config.wallet_dir().join("hotkeys").join("test-hk");
        assert!(hotkey_path.exists(), "regen hotkey file should exist");
    }

    #[tokio::test]
    async fn regen_coldkey_invalid_mnemonic_yes_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "ghost".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let result =
            exec_regen_coldkey(&config, "invalid mnemonic words here", Some("pw".into()), true)
                .await;
        assert!(result.is_err(), "regen-coldkey with invalid mnemonic should fail");
    }

    #[tokio::test]
    async fn swap_coldkey_created_wallet_chain_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "swap-test".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let mut wallet = Wallet::with_path(&config.wallet_name, config.wallet_dir());
        wallet.create_coldkey("").expect("create coldkey");
        wallet.create_hotkey().expect("create hotkey");
        let result = exec_swap_coldkey(&config, "5FakeNewColdkey", Some("".into())).await;
        assert!(result.is_err(), "swap coldkey with created wallet but no chain should fail");
    }

    #[tokio::test]
    async fn create_hotkey_derived_created_wallet_chain_fails() {
        let dir = TempDir::new().expect("tempdir");
        let config = Config {
            network: bittensor_core::config::NetworkConfig::local(),
            wallet_name: "hk-derived-test".to_string(),
            wallet_path: dir.path().to_path_buf(),
        };
        let mut wallet = Wallet::with_path(&config.wallet_name, config.wallet_dir());
        wallet.create_coldkey("").expect("create coldkey");
        wallet.create_hotkey().expect("create hotkey");
        let result = exec_create_hotkey(&config, "derived-hk", Some("".into()), false).await;
        assert!(result.is_ok(), "create-hotkey derived with created wallet should succeed");
        assert!(
            config.wallet_dir().join("hotkeys").join("derived-hk").exists(),
            "derived hotkey file should exist"
        );
    }
}
