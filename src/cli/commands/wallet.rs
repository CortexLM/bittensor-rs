//! Wallet commands for managing coldkeys and hotkeys.

use crate::cli::utils::{
    confirm, create_table_with_headers, format_address, format_tao, keypair_to_signer,
    print_error, print_info, print_success, print_warning, prompt_password,
    prompt_password_optional, resolve_endpoint, spinner, tao_to_rao,
};
use crate::cli::Cli;
use crate::wallet::{Mnemonic, Wallet};
use clap::{Args, Subcommand};

/// Wallet command container
#[derive(Args, Clone)]
pub struct WalletCommand {
    #[command(subcommand)]
    pub command: WalletCommands,
}

/// Available wallet operations
#[derive(Subcommand, Clone)]
pub enum WalletCommands {
    /// Create a new wallet (coldkey and hotkey)
    Create {
        /// Wallet name
        #[arg(short, long, default_value = "default")]
        name: String,
        /// Hotkey name
        #[arg(short = 'k', long, default_value = "default")]
        hotkey: String,
        /// Number of mnemonic words (12, 15, 18, 21, 24)
        #[arg(long, default_value = "12")]
        words: usize,
        /// Skip password for coldkey encryption
        #[arg(long)]
        no_password: bool,
    },

    /// Regenerate wallet from mnemonic phrase
    Regen {
        /// Wallet name
        #[arg(short, long)]
        name: String,
        /// Mnemonic phrase (space-separated words)
        #[arg(long)]
        mnemonic: String,
        /// Skip password for encryption
        #[arg(long)]
        no_password: bool,
    },

    /// List all wallets
    List {
        /// Custom wallet path
        #[arg(long)]
        path: Option<String>,
    },

    /// Show wallet overview (balances and registrations)
    Overview {
        /// Wallet name (default: all wallets)
        #[arg(short, long)]
        name: Option<String>,
        /// Show registrations on all subnets
        #[arg(long)]
        all: bool,
    },

    /// Show wallet balance
    Balance {
        /// Wallet name
        #[arg(short, long)]
        name: Option<String>,
        /// Show all wallets
        #[arg(long)]
        all: bool,
    },

    /// Transfer TAO to another address
    Transfer {
        /// Source wallet name
        #[arg(short, long)]
        name: String,
        /// Destination address (SS58 format)
        #[arg(short, long)]
        dest: String,
        /// Amount in TAO
        #[arg(short, long)]
        amount: f64,
    },

    /// Create a new hotkey
    NewHotkey {
        /// Wallet name
        #[arg(short, long)]
        name: String,
        /// Hotkey name
        #[arg(short = 'k', long)]
        hotkey: String,
        /// Number of mnemonic words (12, 15, 18, 21, 24)
        #[arg(long, default_value = "12")]
        words: usize,
        /// Skip password for hotkey encryption
        #[arg(long)]
        no_password: bool,
    },

    /// Create a new coldkey
    NewColdkey {
        /// Wallet name
        #[arg(short, long)]
        name: String,
        /// Number of mnemonic words (12, 15, 18, 21, 24)
        #[arg(long, default_value = "12")]
        words: usize,
        /// Skip password for encryption
        #[arg(long)]
        no_password: bool,
    },

    /// Regenerate coldkey from mnemonic
    RegenColdkey {
        /// Wallet name
        #[arg(short, long)]
        name: String,
        /// Mnemonic phrase
        #[arg(long)]
        mnemonic: String,
        /// Skip password for encryption
        #[arg(long)]
        no_password: bool,
    },

    /// Regenerate hotkey from mnemonic
    RegenHotkey {
        /// Wallet name
        #[arg(short, long)]
        name: String,
        /// Hotkey name
        #[arg(short = 'k', long)]
        hotkey: String,
        /// Mnemonic phrase
        #[arg(long)]
        mnemonic: String,
        /// Skip password for encryption
        #[arg(long)]
        no_password: bool,
    },

    /// Show wallet addresses
    Address {
        /// Wallet name
        #[arg(short, long, default_value = "default")]
        name: String,
        /// Hotkey name
        #[arg(short = 'k', long, default_value = "default")]
        hotkey: String,
    },
}

/// Execute wallet commands
pub async fn execute(cmd: WalletCommand, cli: &Cli) -> anyhow::Result<()> {
    match cmd.command {
        WalletCommands::Create {
            name,
            hotkey,
            words,
            no_password,
        } => create_wallet(&name, &hotkey, words, no_password, cli).await,
        WalletCommands::Regen {
            name,
            mnemonic,
            no_password,
        } => regen_wallet(&name, &mnemonic, no_password, cli).await,
        WalletCommands::List { path } => list_wallets(path.as_deref()).await,
        WalletCommands::Overview { name, all } => overview(name.as_deref(), all, cli).await,
        WalletCommands::Balance { name, all } => balance(name.as_deref(), all, cli).await,
        WalletCommands::Transfer { name, dest, amount } => {
            transfer(&name, &dest, amount, cli).await
        }
        WalletCommands::NewHotkey {
            name,
            hotkey,
            words,
            no_password,
        } => new_hotkey(&name, &hotkey, words, no_password).await,
        WalletCommands::NewColdkey {
            name,
            words,
            no_password,
        } => new_coldkey(&name, words, no_password).await,
        WalletCommands::RegenColdkey {
            name,
            mnemonic,
            no_password,
        } => regen_coldkey(&name, &mnemonic, no_password).await,
        WalletCommands::RegenHotkey {
            name,
            hotkey,
            mnemonic,
            no_password,
        } => regen_hotkey(&name, &hotkey, &mnemonic, no_password).await,
        WalletCommands::Address { name, hotkey } => show_address(&name, &hotkey).await,
    }
}

/// Create a new wallet with coldkey and hotkey
async fn create_wallet(
    name: &str,
    hotkey_name: &str,
    words: usize,
    no_password: bool,
    cli: &Cli,
) -> anyhow::Result<()> {
    // Validate word count
    if ![12, 15, 18, 21, 24].contains(&words) {
        print_error("Word count must be 12, 15, 18, 21, or 24");
        return Err(anyhow::anyhow!("Invalid word count"));
    }

    // Check if wallet already exists
    let mut wallet = match Wallet::new(name, hotkey_name, None) {
        Ok(w) => w,
        Err(e) => {
            print_error(&format!("Invalid wallet name '{}': {}", name, e));
            return Err(anyhow::anyhow!("Invalid wallet name: {}", e));
        }
    };
    if wallet.coldkey_exists() {
        print_warning(&format!("Wallet '{}' already exists", name));
        if !confirm("Overwrite existing wallet?", cli.no_prompt) {
            print_info("Aborted");
            return Ok(());
        }
    }

    // Generate mnemonics
    let coldkey_mnemonic = Mnemonic::generate_with_words(words)
        .map_err(|e| anyhow::anyhow!("Failed to generate coldkey mnemonic: {}", e))?;

    let hotkey_mnemonic = Mnemonic::generate_with_words(words)
        .map_err(|e| anyhow::anyhow!("Failed to generate hotkey mnemonic: {}", e))?;

    // Get password for coldkey
    let coldkey_password = if no_password {
        None
    } else {
        let pwd = prompt_password("Enter password for coldkey encryption");
        let confirm = prompt_password("Confirm password");
        if pwd != confirm {
            print_error("Passwords do not match");
            return Err(anyhow::anyhow!("Password mismatch"));
        }
        Some(pwd)
    };

    // Create coldkey
    let sp = spinner("Creating coldkey...");
    wallet
        .create_coldkey(
            coldkey_password.as_deref(),
            Some(coldkey_mnemonic.phrase()),
            false,
        )
        .map_err(|e| anyhow::anyhow!("Failed to create coldkey: {}", e))?;
    sp.finish_and_clear();

    // Create hotkey (typically no password)
    let sp = spinner("Creating hotkey...");
    wallet
        .create_hotkey(None, Some(hotkey_mnemonic.phrase()), false)
        .map_err(|e| anyhow::anyhow!("Failed to create hotkey: {}", e))?;
    sp.finish_and_clear();

    // Display results
    print_success(&format!("Wallet '{}' created successfully!", name));
    println!();

    print_warning("IMPORTANT: Save these mnemonic phrases securely!");
    println!();

    let coldkey_addr = wallet
        .coldkey_ss58(coldkey_password.as_deref())
        .map_err(|e| anyhow::anyhow!("Failed to get coldkey address: {}", e))?;
    let hotkey_addr = wallet
        .hotkey_ss58(None)
        .map_err(|e| anyhow::anyhow!("Failed to get hotkey address: {}", e))?;

    println!("Coldkey address: {}", coldkey_addr);
    println!("Coldkey mnemonic: {}", coldkey_mnemonic.phrase());
    println!();
    println!("Hotkey address: {}", hotkey_addr);
    println!("Hotkey mnemonic: {}", hotkey_mnemonic.phrase());

    Ok(())
}

/// Regenerate wallet from mnemonic
async fn regen_wallet(
    name: &str,
    mnemonic: &str,
    no_password: bool,
    cli: &Cli,
) -> anyhow::Result<()> {
    // Validate mnemonic
    if !Mnemonic::validate(mnemonic) {
        print_error("Invalid mnemonic phrase");
        return Err(anyhow::anyhow!("Invalid mnemonic"));
    }

    let mut wallet = match Wallet::new(name, "default", None) {
        Ok(w) => w,
        Err(e) => {
            print_error(&format!("Invalid wallet name '{}': {}", name, e));
            return Err(anyhow::anyhow!("Invalid wallet name: {}", e));
        }
    };

    if wallet.coldkey_exists() {
        print_warning(&format!("Wallet '{}' already exists", name));
        if !confirm("Overwrite existing wallet?", cli.no_prompt) {
            print_info("Aborted");
            return Ok(());
        }
    }

    let password = if no_password {
        None
    } else {
        let pwd = prompt_password("Enter password for encryption");
        let confirm = prompt_password("Confirm password");
        if pwd != confirm {
            print_error("Passwords do not match");
            return Err(anyhow::anyhow!("Password mismatch"));
        }
        Some(pwd)
    };

    let sp = spinner("Regenerating wallet from mnemonic...");
    wallet
        .create_coldkey(password.as_deref(), Some(mnemonic), false)
        .map_err(|e| anyhow::anyhow!("Failed to regenerate coldkey: {}", e))?;
    sp.finish_and_clear();

    let addr = wallet
        .coldkey_ss58(password.as_deref())
        .map_err(|e| anyhow::anyhow!("Failed to get address: {}", e))?;

    print_success(&format!("Wallet '{}' regenerated successfully!", name));
    println!("Coldkey address: {}", addr);

    Ok(())
}

/// List all wallets
async fn list_wallets(path: Option<&str>) -> anyhow::Result<()> {
    use crate::wallet::{list_wallets as get_wallet_names, wallet_path};
    use std::path::Path;

    let wallet_names = if let Some(p) = path {
        crate::wallet::list_wallets_at(Path::new(p))
            .map_err(|e| anyhow::anyhow!("Failed to list wallets: {}", e))?
    } else {
        get_wallet_names()
            .map_err(|e| anyhow::anyhow!("Failed to list wallets: {}", e))?
    };

    if wallet_names.is_empty() {
        print_info("No wallets found");
        return Ok(());
    }

    let mut table = create_table_with_headers(&["Wallet", "Coldkey Path"]);

    for wallet_name in &wallet_names {
        table.add_row(vec![
            wallet_name.clone(),
            wallet_path(wallet_name).display().to_string(),
        ]);
    }

    println!("{table}");
    Ok(())
}

/// Show wallet overview
async fn overview(name: Option<&str>, _all: bool, cli: &Cli) -> anyhow::Result<()> {
    use crate::chain::BittensorClient;
    use crate::queries::balances::get_balance;
    use crate::queries::stakes::get_stake_info_for_coldkey;
    use crate::wallet::list_wallets as get_wallet_names;
    use sp_core::crypto::AccountId32;
    use std::str::FromStr;

    let endpoint = resolve_endpoint(&cli.network, cli.endpoint.as_deref());

    let sp = spinner(&format!("Connecting to {}...", endpoint));
    let client = BittensorClient::new(&endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e))?;
    sp.finish_and_clear();

    let wallets: Vec<Wallet> = if let Some(wallet_name) = name {
        match Wallet::new(wallet_name, "default", None) {
            Ok(w) => vec![w],
            Err(e) => {
                print_error(&format!("Invalid wallet name '{}': {}", wallet_name, e));
                return Err(anyhow::anyhow!("Invalid wallet name: {}", e));
            }
        }
    } else {
        let names = get_wallet_names()
            .map_err(|e| anyhow::anyhow!("Failed to list wallets: {}", e))?;
        names.iter()
            .filter_map(|n| Wallet::new(n, "default", None).ok())
            .collect()
    };

    if wallets.is_empty() {
        print_info("No wallets found");
        return Ok(());
    }

    let mut table = create_table_with_headers(&["Wallet", "Coldkey", "Free Balance", "Staked"]);

    for wallet in &wallets {
        let password = prompt_password_optional(&format!(
            "Password for '{}' (enter to skip)",
            &wallet.name
        ));

        let coldkey_addr = match wallet.coldkey_ss58(password.as_deref()) {
            Ok(addr) => addr,
            Err(e) => {
                print_warning(&format!(
                    "Could not unlock '{}': {}",
                    &wallet.name,
                    e
                ));
                continue;
            }
        };

        let sp = spinner(&format!("Fetching balance for {}...", format_address(&coldkey_addr)));
        
        // Parse SS58 to AccountId32
        let account = AccountId32::from_str(&coldkey_addr)
            .map_err(|e| anyhow::anyhow!("Invalid SS58 address: {}", e))?;
        
        let balance_result = get_balance(&client, &account).await;
        let stake_result = get_stake_info_for_coldkey(&client, &account).await;
        sp.finish_and_clear();

        let free = balance_result.unwrap_or(0);
        let staked: u128 = stake_result
            .map(|stakes| stakes.iter().map(|s| s.stake).sum())
            .unwrap_or(0);

        table.add_row(vec![
            wallet.name.to_string(),
            format_address(&coldkey_addr),
            format_tao(free),
            format_tao(staked),
        ]);
    }

    println!("\n{table}");
    Ok(())
}

/// Show wallet balance
async fn balance(name: Option<&str>, all: bool, cli: &Cli) -> anyhow::Result<()> {
    use crate::chain::BittensorClient;
    use crate::queries::balances::get_balance;
    use crate::queries::stakes::get_stake_info_for_coldkey;
    use crate::wallet::list_wallets as get_wallet_names;
    use sp_core::crypto::AccountId32;
    use std::str::FromStr;

    let endpoint = resolve_endpoint(&cli.network, cli.endpoint.as_deref());

    let sp = spinner(&format!("Connecting to {}...", endpoint));
    let client = BittensorClient::new(&endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e))?;
    sp.finish_and_clear();

    let wallets: Vec<Wallet> = if let Some(wallet_name) = name {
        match Wallet::new(wallet_name, "default", None) {
            Ok(w) => vec![w],
            Err(e) => {
                print_error(&format!("Invalid wallet name '{}': {}", wallet_name, e));
                return Err(anyhow::anyhow!("Invalid wallet name: {}", e));
            }
        }
    } else if all {
        let names = get_wallet_names()
            .map_err(|e| anyhow::anyhow!("Failed to list wallets: {}", e))?;
        names.iter()
            .filter_map(|n| Wallet::new(n, "default", None).ok())
            .collect()
    } else {
        match Wallet::new("default", "default", None) {
            Ok(w) => vec![w],
            Err(e) => {
                print_error(&format!("Invalid wallet name: {}", e));
                return Err(anyhow::anyhow!("Invalid wallet name: {}", e));
            }
        }
    };

    if wallets.is_empty() {
        print_info("No wallets found");
        return Ok(());
    }

    let mut table =
        create_table_with_headers(&["Wallet", "Coldkey", "Free Balance", "Staked", "Total"]);

    for wallet in &wallets {
        let password = prompt_password_optional(&format!(
            "Password for '{}' (enter to skip)",
            &wallet.name
        ));

        let coldkey_addr = match wallet.coldkey_ss58(password.as_deref()) {
            Ok(addr) => addr,
            Err(e) => {
                print_warning(&format!("Could not unlock '{}': {}", &wallet.name, e));
                continue;
            }
        };

        let sp = spinner(&format!("Fetching balance for {}...", format_address(&coldkey_addr)));
        
        // Parse SS58 to AccountId32
        let account = AccountId32::from_str(&coldkey_addr)
            .map_err(|e| anyhow::anyhow!("Invalid SS58 address: {}", e))?;
        
        let balance_result = get_balance(&client, &account).await;
        let stake_result = get_stake_info_for_coldkey(&client, &account).await;
        sp.finish_and_clear();

        let free = balance_result.unwrap_or(0);
        let staked: u128 = stake_result
            .map(|stakes| stakes.iter().map(|s| s.stake).sum())
            .unwrap_or(0);
        let total = free + staked;

        table.add_row(vec![
            wallet.name.to_string(),
            format_address(&coldkey_addr),
            format_tao(free),
            format_tao(staked),
            format_tao(total),
        ]);
    }

    println!("\n{table}");
    Ok(())
}

/// Transfer TAO to another address
async fn transfer(name: &str, dest: &str, amount: f64, cli: &Cli) -> anyhow::Result<()> {
    use crate::chain::{BittensorClient, ExtrinsicWait};
    use crate::validator::transfer::transfer as do_transfer;
    use sp_core::crypto::AccountId32;
    use std::str::FromStr;

    if amount <= 0.0 {
        print_error("Amount must be positive");
        return Err(anyhow::anyhow!("Invalid amount"));
    }

    let endpoint = resolve_endpoint(&cli.network, cli.endpoint.as_deref());

    let wallet = match Wallet::new(name, "default", None) {
        Ok(w) => w,
        Err(e) => {
            print_error(&format!("Invalid wallet name '{}': {}", name, e));
            return Err(anyhow::anyhow!("Invalid wallet name: {}", e));
        }
    };
    if !wallet.coldkey_exists() {
        print_error(&format!("Wallet '{}' not found", name));
        return Err(anyhow::anyhow!("Wallet not found"));
    }

    let password = prompt_password_optional("Coldkey password (enter if unencrypted)");
    let coldkey = wallet
        .coldkey_keypair(password.as_deref())
        .map_err(|e| anyhow::anyhow!("Failed to unlock coldkey: {}", e))?;
    let signer = keypair_to_signer(&coldkey);

    let dest_account = AccountId32::from_str(dest)
        .map_err(|e| anyhow::anyhow!("Invalid destination address: {:?}", e))?;

    let rao_amount = tao_to_rao(amount);

    print_info(&format!(
        "Transfer {} TAO ({} RAO)",
        amount, rao_amount
    ));
    print_info(&format!("From: {}", coldkey.ss58_address()));
    print_info(&format!("To: {}", dest));

    if !confirm("Proceed with transfer?", cli.no_prompt) {
        print_info("Transfer cancelled");
        return Ok(());
    }

    let sp = spinner(&format!("Connecting to {}...", endpoint));
    let client = BittensorClient::new(&endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e))?;
    sp.finish_and_clear();

    let sp = spinner("Submitting transfer...");
    let result = do_transfer(&client, &signer, &dest_account, rao_amount, true, ExtrinsicWait::Finalized).await;
    sp.finish_and_clear();

    match result {
        Ok(tx_hash) => {
            print_success("Transfer successful!");
            print_info(&format!("Transaction hash: {}", tx_hash));
        }
        Err(e) => {
            print_error(&format!("Transfer failed: {}", e));
            return Err(anyhow::anyhow!("Transfer failed: {}", e));
        }
    }

    Ok(())
}

/// Create a new hotkey
async fn new_hotkey(
    name: &str,
    hotkey_name: &str,
    words: usize,
    no_password: bool,
) -> anyhow::Result<()> {
    if ![12, 15, 18, 21, 24].contains(&words) {
        print_error("Word count must be 12, 15, 18, 21, or 24");
        return Err(anyhow::anyhow!("Invalid word count"));
    }

    let mut wallet = match Wallet::new(name, hotkey_name, None) {
        Ok(w) => w,
        Err(e) => {
            print_error(&format!("Invalid wallet name '{}': {}", name, e));
            return Err(anyhow::anyhow!("Invalid wallet name: {}", e));
        }
    };

    if !wallet.coldkey_exists() {
        print_error(&format!("Wallet '{}' does not exist", name));
        return Err(anyhow::anyhow!("Wallet not found"));
    }

    let mnemonic = Mnemonic::generate_with_words(words)
        .map_err(|e| anyhow::anyhow!("Failed to generate mnemonic: {}", e))?;

    let password = if no_password {
        None
    } else {
        let pwd = prompt_password("Enter password for hotkey encryption (enter for none)");
        if pwd.is_empty() {
            None
        } else {
            Some(pwd)
        }
    };

    let sp = spinner("Creating hotkey...");
    wallet
        .create_hotkey(password.as_deref(), Some(mnemonic.phrase()), false)
        .map_err(|e| anyhow::anyhow!("Failed to create hotkey: {}", e))?;
    sp.finish_and_clear();

    let addr = wallet
        .hotkey_ss58(password.as_deref())
        .map_err(|e| anyhow::anyhow!("Failed to get hotkey address: {}", e))?;

    print_success(&format!(
        "Hotkey '{}' created for wallet '{}'",
        hotkey_name, name
    ));
    println!();
    print_warning("Save this mnemonic phrase securely!");
    println!("Hotkey address: {}", addr);
    println!("Hotkey mnemonic: {}", mnemonic.phrase());

    Ok(())
}

/// Create a new coldkey
async fn new_coldkey(name: &str, words: usize, no_password: bool) -> anyhow::Result<()> {
    if ![12, 15, 18, 21, 24].contains(&words) {
        print_error("Word count must be 12, 15, 18, 21, or 24");
        return Err(anyhow::anyhow!("Invalid word count"));
    }

    let mut wallet = match Wallet::new(name, "default", None) {
        Ok(w) => w,
        Err(e) => {
            print_error(&format!("Invalid wallet name '{}': {}", name, e));
            return Err(anyhow::anyhow!("Invalid wallet name: {}", e));
        }
    };

    let mnemonic = Mnemonic::generate_with_words(words)
        .map_err(|e| anyhow::anyhow!("Failed to generate mnemonic: {}", e))?;

    let password = if no_password {
        None
    } else {
        let pwd = prompt_password("Enter password for encryption");
        let confirm = prompt_password("Confirm password");
        if pwd != confirm {
            print_error("Passwords do not match");
            return Err(anyhow::anyhow!("Password mismatch"));
        }
        Some(pwd)
    };

    let sp = spinner("Creating coldkey...");
    wallet
        .create_coldkey(password.as_deref(), Some(mnemonic.phrase()), false)
        .map_err(|e| anyhow::anyhow!("Failed to create coldkey: {}", e))?;
    sp.finish_and_clear();

    let addr = wallet
        .coldkey_ss58(password.as_deref())
        .map_err(|e| anyhow::anyhow!("Failed to get coldkey address: {}", e))?;

    print_success(&format!("Coldkey '{}' created!", name));
    println!();
    print_warning("IMPORTANT: Save this mnemonic phrase securely!");
    println!("Coldkey address: {}", addr);
    println!("Coldkey mnemonic: {}", mnemonic.phrase());

    Ok(())
}

/// Regenerate coldkey from mnemonic
async fn regen_coldkey(name: &str, mnemonic: &str, no_password: bool) -> anyhow::Result<()> {
    if !Mnemonic::validate(mnemonic) {
        print_error("Invalid mnemonic phrase");
        return Err(anyhow::anyhow!("Invalid mnemonic"));
    }

    let mut wallet = match Wallet::new(name, "default", None) {
        Ok(w) => w,
        Err(e) => {
            print_error(&format!("Invalid wallet name '{}': {}", name, e));
            return Err(anyhow::anyhow!("Invalid wallet name: {}", e));
        }
    };

    let password = if no_password {
        None
    } else {
        let pwd = prompt_password("Enter password for encryption");
        let confirm = prompt_password("Confirm password");
        if pwd != confirm {
            print_error("Passwords do not match");
            return Err(anyhow::anyhow!("Password mismatch"));
        }
        Some(pwd)
    };

    let sp = spinner("Regenerating coldkey...");
    wallet
        .create_coldkey(password.as_deref(), Some(mnemonic), false)
        .map_err(|e| anyhow::anyhow!("Failed to regenerate coldkey: {}", e))?;
    sp.finish_and_clear();

    let addr = wallet
        .coldkey_ss58(password.as_deref())
        .map_err(|e| anyhow::anyhow!("Failed to get coldkey address: {}", e))?;

    print_success(&format!("Coldkey '{}' regenerated!", name));
    println!("Coldkey address: {}", addr);

    Ok(())
}

/// Regenerate hotkey from mnemonic
async fn regen_hotkey(
    name: &str,
    hotkey_name: &str,
    mnemonic: &str,
    no_password: bool,
) -> anyhow::Result<()> {
    if !Mnemonic::validate(mnemonic) {
        print_error("Invalid mnemonic phrase");
        return Err(anyhow::anyhow!("Invalid mnemonic"));
    }

    let mut wallet = match Wallet::new(name, hotkey_name, None) {
        Ok(w) => w,
        Err(e) => {
            print_error(&format!("Invalid wallet name '{}': {}", name, e));
            return Err(anyhow::anyhow!("Invalid wallet name: {}", e));
        }
    };

    if !wallet.coldkey_exists() {
        print_error(&format!("Wallet '{}' does not exist", name));
        return Err(anyhow::anyhow!("Wallet not found"));
    }

    let password = if no_password {
        None
    } else {
        let pwd = prompt_password("Enter password for encryption (enter for none)");
        if pwd.is_empty() {
            None
        } else {
            Some(pwd)
        }
    };

    let sp = spinner("Regenerating hotkey...");
    wallet
        .create_hotkey(password.as_deref(), Some(mnemonic), false)
        .map_err(|e| anyhow::anyhow!("Failed to regenerate hotkey: {}", e))?;
    sp.finish_and_clear();

    let addr = wallet
        .hotkey_ss58(password.as_deref())
        .map_err(|e| anyhow::anyhow!("Failed to get hotkey address: {}", e))?;

    print_success(&format!(
        "Hotkey '{}' regenerated for wallet '{}'!",
        hotkey_name, name
    ));
    println!("Hotkey address: {}", addr);

    Ok(())
}

/// Show wallet addresses
async fn show_address(name: &str, hotkey_name: &str) -> anyhow::Result<()> {
    let wallet = match Wallet::new(name, hotkey_name, None) {
        Ok(w) => w,
        Err(e) => {
            print_error(&format!("Invalid wallet name '{}': {}", name, e));
            return Err(anyhow::anyhow!("Invalid wallet name: {}", e));
        }
    };

    if !wallet.coldkey_exists() {
        print_error(&format!("Wallet '{}' not found", name));
        return Err(anyhow::anyhow!("Wallet not found"));
    }

    let coldkey_password = prompt_password_optional("Coldkey password (enter if unencrypted)");
    let hotkey_password = prompt_password_optional("Hotkey password (enter if unencrypted)");

    let coldkey_addr = wallet.coldkey_ss58(coldkey_password.as_deref());
    let hotkey_addr = wallet.hotkey_ss58(hotkey_password.as_deref());

    println!();
    match coldkey_addr {
        Ok(addr) => println!("Coldkey address: {}", addr),
        Err(e) => print_warning(&format!("Could not get coldkey address: {}", e)),
    }

    match hotkey_addr {
        Ok(addr) => println!("Hotkey address: {}", addr),
        Err(e) => print_warning(&format!("Could not get hotkey address: {}", e)),
    }

    Ok(())
}
