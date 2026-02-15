//! Stake commands for managing TAO delegation.

use crate::cli::utils::{
    confirm, create_table_with_headers, format_address, format_tao, keypair_to_signer, print_error,
    print_info, print_success, print_warning, prompt_password_optional, resolve_endpoint, spinner,
    tao_to_rao,
};
use crate::cli::Cli;
use crate::wallet::Wallet;
use clap::{Args, Subcommand};

/// Stake command container
#[derive(Args, Clone)]
pub struct StakeCommand {
    #[command(subcommand)]
    pub command: StakeCommands,
}

/// Available stake operations
#[derive(Subcommand, Clone)]
pub enum StakeCommands {
    /// Add stake to a hotkey on a subnet
    Add {
        /// Wallet name
        #[arg(short, long)]
        wallet: String,
        /// Hotkey name
        #[arg(short = 'k', long)]
        hotkey: String,
        /// Subnet ID
        #[arg(short, long)]
        netuid: u16,
        /// Amount in TAO to stake
        #[arg(short, long)]
        amount: f64,
    },

    /// Remove stake from a hotkey on a subnet
    Remove {
        /// Wallet name
        #[arg(short, long)]
        wallet: String,
        /// Hotkey name
        #[arg(short = 'k', long)]
        hotkey: String,
        /// Subnet ID
        #[arg(short, long)]
        netuid: u16,
        /// Amount in TAO to unstake
        #[arg(short, long)]
        amount: f64,
    },

    /// Show stake information
    Show {
        /// Wallet name (shows all if not specified)
        #[arg(short, long)]
        wallet: Option<String>,
        /// Show all wallets
        #[arg(long)]
        all: bool,
    },

    /// Move stake between hotkeys or subnets
    Move {
        /// Wallet name
        #[arg(short, long)]
        wallet: String,
        /// Source hotkey name
        #[arg(long)]
        from_hotkey: String,
        /// Destination hotkey name
        #[arg(long)]
        to_hotkey: String,
        /// Source subnet ID
        #[arg(long)]
        origin_netuid: u16,
        /// Destination subnet ID
        #[arg(long)]
        dest_netuid: u16,
        /// Amount in TAO to move
        #[arg(short, long)]
        amount: f64,
    },

    /// List all stake for a coldkey
    List {
        /// Wallet name
        #[arg(short, long)]
        wallet: String,
    },
}

/// Execute stake commands
pub async fn execute(cmd: StakeCommand, cli: &Cli) -> anyhow::Result<()> {
    match cmd.command {
        StakeCommands::Add {
            wallet,
            hotkey,
            netuid,
            amount,
        } => add_stake(&wallet, &hotkey, netuid, amount, cli).await,
        StakeCommands::Remove {
            wallet,
            hotkey,
            netuid,
            amount,
        } => remove_stake(&wallet, &hotkey, netuid, amount, cli).await,
        StakeCommands::Show { wallet, all } => show_stake(wallet.as_deref(), all, cli).await,
        StakeCommands::Move {
            wallet,
            from_hotkey,
            to_hotkey,
            origin_netuid,
            dest_netuid,
            amount,
        } => {
            move_stake(
                &wallet,
                &from_hotkey,
                &to_hotkey,
                origin_netuid,
                dest_netuid,
                amount,
                cli,
            )
            .await
        }
        StakeCommands::List { wallet } => list_stake(&wallet, cli).await,
    }
}

/// Add stake to a hotkey
async fn add_stake(
    wallet_name: &str,
    hotkey_name: &str,
    netuid: u16,
    amount: f64,
    cli: &Cli,
) -> anyhow::Result<()> {
    use crate::chain::{BittensorClient, ExtrinsicWait};
    use crate::validator::staking::add_stake as stake_add;
    use sp_core::crypto::AccountId32;
    use std::str::FromStr;

    if amount <= 0.0 {
        print_error("Amount must be positive");
        return Err(anyhow::anyhow!("Invalid amount"));
    }

    let endpoint = resolve_endpoint(&cli.network, cli.endpoint.as_deref());

    let wallet = match Wallet::new(wallet_name, hotkey_name, None) {
        Ok(w) => w,
        Err(e) => {
            print_error(&format!("Invalid wallet name '{}': {}", wallet_name, e));
            return Err(anyhow::anyhow!("Invalid wallet name: {}", e));
        }
    };
    if !wallet.coldkey_exists() {
        print_error(&format!("Wallet '{}' not found", wallet_name));
        return Err(anyhow::anyhow!("Wallet not found"));
    }

    let coldkey_password = prompt_password_optional("Coldkey password (enter if unencrypted)");
    let coldkey = wallet
        .coldkey_keypair(coldkey_password.as_deref())
        .map_err(|e| anyhow::anyhow!("Failed to unlock coldkey: {}", e))?;
    let signer = keypair_to_signer(&coldkey);

    let hotkey_password = prompt_password_optional("Hotkey password (enter if unencrypted)");
    let hotkey = wallet
        .hotkey_keypair(hotkey_password.as_deref())
        .map_err(|e| anyhow::anyhow!("Failed to unlock hotkey: {}", e))?;
    let hotkey_account = AccountId32::from_str(hotkey.ss58_address())
        .map_err(|e| anyhow::anyhow!("Invalid hotkey address: {:?}", e))?;

    let rao_amount = crate::utils::balance_newtypes::Rao::from(tao_to_rao(amount));

    print_info(&format!(
        "Adding stake: {} TAO ({} RAO)",
        amount,
        rao_amount.as_u128()
    ));
    print_info(&format!("Coldkey: {}", coldkey.ss58_address()));
    print_info(&format!("Hotkey: {}", hotkey.ss58_address()));
    print_info(&format!("Subnet: {}", netuid));

    if !confirm("Proceed with staking?", cli.no_prompt) {
        print_info("Staking cancelled");
        return Ok(());
    }

    let sp = spinner(&format!("Connecting to {}...", endpoint));
    let client = BittensorClient::new(&endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e))?;
    sp.finish_and_clear();

    let sp = spinner("Submitting stake transaction...");
    let result = stake_add(
        &client,
        &signer,
        &hotkey_account,
        netuid,
        rao_amount,
        ExtrinsicWait::Finalized,
    )
    .await;
    sp.finish_and_clear();

    match result {
        Ok(tx_hash) => {
            print_success("Stake added successfully!");
            print_info(&format!("Transaction hash: {}", tx_hash));
        }
        Err(e) => {
            print_error(&format!("Failed to add stake: {}", e));
            return Err(anyhow::anyhow!("Staking failed: {}", e));
        }
    }

    Ok(())
}

/// Remove stake from a hotkey
async fn remove_stake(
    wallet_name: &str,
    hotkey_name: &str,
    netuid: u16,
    amount: f64,
    cli: &Cli,
) -> anyhow::Result<()> {
    use crate::chain::{BittensorClient, ExtrinsicWait};
    use crate::validator::staking::unstake;
    use sp_core::crypto::AccountId32;
    use std::str::FromStr;

    if amount <= 0.0 {
        print_error("Amount must be positive");
        return Err(anyhow::anyhow!("Invalid amount"));
    }

    let endpoint = resolve_endpoint(&cli.network, cli.endpoint.as_deref());

    let wallet = match Wallet::new(wallet_name, hotkey_name, None) {
        Ok(w) => w,
        Err(e) => {
            print_error(&format!("Invalid wallet name '{}': {}", wallet_name, e));
            return Err(anyhow::anyhow!("Invalid wallet name: {}", e));
        }
    };
    if !wallet.coldkey_exists() {
        print_error(&format!("Wallet '{}' not found", wallet_name));
        return Err(anyhow::anyhow!("Wallet not found"));
    }

    let coldkey_password = prompt_password_optional("Coldkey password (enter if unencrypted)");
    let coldkey = wallet
        .coldkey_keypair(coldkey_password.as_deref())
        .map_err(|e| anyhow::anyhow!("Failed to unlock coldkey: {}", e))?;
    let signer = keypair_to_signer(&coldkey);

    let hotkey_password = prompt_password_optional("Hotkey password (enter if unencrypted)");
    let hotkey = wallet
        .hotkey_keypair(hotkey_password.as_deref())
        .map_err(|e| anyhow::anyhow!("Failed to unlock hotkey: {}", e))?;
    let hotkey_account = AccountId32::from_str(hotkey.ss58_address())
        .map_err(|e| anyhow::anyhow!("Invalid hotkey address: {:?}", e))?;

    let rao_amount = crate::utils::balance_newtypes::Rao::from(tao_to_rao(amount));

    print_info(&format!(
        "Removing stake: {} TAO ({} RAO)",
        amount,
        rao_amount.as_u128()
    ));
    print_info(&format!("Coldkey: {}", coldkey.ss58_address()));
    print_info(&format!("Hotkey: {}", hotkey.ss58_address()));
    print_info(&format!("Subnet: {}", netuid));

    if !confirm("Proceed with unstaking?", cli.no_prompt) {
        print_info("Unstaking cancelled");
        return Ok(());
    }

    let sp = spinner(&format!("Connecting to {}...", endpoint));
    let client = BittensorClient::new(&endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e))?;
    sp.finish_and_clear();

    let sp = spinner("Submitting unstake transaction...");
    let result = unstake(
        &client,
        &signer,
        &hotkey_account,
        netuid,
        rao_amount,
        ExtrinsicWait::Finalized,
    )
    .await;
    sp.finish_and_clear();

    match result {
        Ok(tx_hash) => {
            print_success("Stake removed successfully!");
            print_info(&format!("Transaction hash: {}", tx_hash));
        }
        Err(e) => {
            print_error(&format!("Failed to remove stake: {}", e));
            return Err(anyhow::anyhow!("Unstaking failed: {}", e));
        }
    }

    Ok(())
}

/// Show stake information for wallets
async fn show_stake(wallet_name: Option<&str>, all: bool, cli: &Cli) -> anyhow::Result<()> {
    use crate::chain::BittensorClient;
    use crate::queries::stakes::get_stake_info_for_coldkey;
    use crate::wallet::list_wallets;
    use sp_core::crypto::AccountId32;
    use std::str::FromStr;

    let endpoint = resolve_endpoint(&cli.network, cli.endpoint.as_deref());

    let sp = spinner(&format!("Connecting to {}...", endpoint));
    let client = BittensorClient::new(&endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e))?;
    sp.finish_and_clear();

    let wallets: Vec<Wallet> = if let Some(name) = wallet_name {
        match Wallet::new(name, "default", None) {
            Ok(w) => vec![w],
            Err(e) => {
                print_error(&format!("Invalid wallet name '{}': {}", name, e));
                return Err(anyhow::anyhow!("Invalid wallet name: {}", e));
            }
        }
    } else if all {
        let names = list_wallets().map_err(|e| anyhow::anyhow!("Failed to list wallets: {}", e))?;
        names
            .iter()
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

    for wallet in &wallets {
        let coldkey_password =
            prompt_password_optional(&format!("Password for '{}' (enter to skip)", &wallet.name));

        let coldkey_addr = match wallet.coldkey_ss58(coldkey_password.as_deref()) {
            Ok(addr) => addr,
            Err(e) => {
                print_warning(&format!("Could not unlock '{}': {}", &wallet.name, e));
                continue;
            }
        };

        let coldkey_account = match AccountId32::from_str(&coldkey_addr) {
            Ok(acc) => acc,
            Err(e) => {
                print_warning(&format!("Invalid coldkey address: {:?}", e));
                continue;
            }
        };

        let sp = spinner(&format!(
            "Fetching stake info for {}...",
            format_address(&coldkey_addr)
        ));
        let stake_result = get_stake_info_for_coldkey(&client, &coldkey_account).await;
        sp.finish_and_clear();

        match stake_result {
            Ok(stakes) => {
                println!(
                    "\nWallet: {} ({})",
                    &wallet.name,
                    format_address(&coldkey_addr)
                );

                if stakes.is_empty() {
                    print_info("No stake found");
                    continue;
                }

                let mut table = create_table_with_headers(&["Hotkey", "Subnet", "Stake (TAO)"]);

                for stake_info in stakes {
                    table.add_row(vec![
                        format_address(&stake_info.hotkey.to_string()),
                        stake_info.netuid.to_string(),
                        format_tao(stake_info.stake),
                    ]);
                }

                println!("{table}");
            }
            Err(e) => {
                print_warning(&format!(
                    "Failed to fetch stake for {}: {}",
                    &wallet.name, e
                ));
            }
        }
    }

    Ok(())
}

/// Move stake between hotkeys or subnets
async fn move_stake(
    wallet_name: &str,
    from_hotkey: &str,
    to_hotkey: &str,
    origin_netuid: u16,
    dest_netuid: u16,
    amount: f64,
    cli: &Cli,
) -> anyhow::Result<()> {
    use crate::chain::{BittensorClient, ExtrinsicWait};
    use crate::validator::staking::move_stake as stake_move;
    use sp_core::crypto::AccountId32;
    use std::str::FromStr;

    if amount <= 0.0 {
        print_error("Amount must be positive");
        return Err(anyhow::anyhow!("Invalid amount"));
    }

    let endpoint = resolve_endpoint(&cli.network, cli.endpoint.as_deref());

    let from_wallet = match Wallet::new(wallet_name, from_hotkey, None) {
        Ok(w) => w,
        Err(e) => {
            print_error(&format!("Invalid wallet name '{}': {}", wallet_name, e));
            return Err(anyhow::anyhow!("Invalid wallet name: {}", e));
        }
    };
    let to_wallet = match Wallet::new(wallet_name, to_hotkey, None) {
        Ok(w) => w,
        Err(e) => {
            print_error(&format!("Invalid wallet name '{}': {}", wallet_name, e));
            return Err(anyhow::anyhow!("Invalid wallet name: {}", e));
        }
    };

    if !from_wallet.coldkey_exists() {
        print_error(&format!("Wallet '{}' not found", wallet_name));
        return Err(anyhow::anyhow!("Wallet not found"));
    }

    let coldkey_password = prompt_password_optional("Coldkey password (enter if unencrypted)");
    let coldkey = from_wallet
        .coldkey_keypair(coldkey_password.as_deref())
        .map_err(|e| anyhow::anyhow!("Failed to unlock coldkey: {}", e))?;
    let signer = keypair_to_signer(&coldkey);

    let from_hotkey_password =
        prompt_password_optional("Source hotkey password (enter if unencrypted)");
    let from_hk = from_wallet
        .hotkey_keypair(from_hotkey_password.as_deref())
        .map_err(|e| anyhow::anyhow!("Failed to unlock source hotkey: {}", e))?;
    let from_hk_account = AccountId32::from_str(from_hk.ss58_address())
        .map_err(|e| anyhow::anyhow!("Invalid source hotkey address: {:?}", e))?;

    let to_hotkey_password =
        prompt_password_optional("Destination hotkey password (enter if unencrypted)");
    let to_hk = to_wallet
        .hotkey_keypair(to_hotkey_password.as_deref())
        .map_err(|e| anyhow::anyhow!("Failed to unlock destination hotkey: {}", e))?;
    let to_hk_account = AccountId32::from_str(to_hk.ss58_address())
        .map_err(|e| anyhow::anyhow!("Invalid destination hotkey address: {:?}", e))?;

    let rao_amount = crate::utils::balance_newtypes::Rao::from(tao_to_rao(amount));

    print_info(&format!(
        "Moving stake: {} TAO ({} RAO)",
        amount,
        rao_amount.as_u128()
    ));
    print_info(&format!(
        "From: {} (subnet {})",
        from_hk.ss58_address(),
        origin_netuid
    ));
    print_info(&format!(
        "To: {} (subnet {})",
        to_hk.ss58_address(),
        dest_netuid
    ));

    if !confirm("Proceed with stake move?", cli.no_prompt) {
        print_info("Move cancelled");
        return Ok(());
    }

    let sp = spinner(&format!("Connecting to {}...", endpoint));
    let client = BittensorClient::new(&endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e))?;
    sp.finish_and_clear();

    let sp = spinner("Submitting move stake transaction...");
    let result = stake_move(
        &client,
        &signer,
        &from_hk_account,
        &to_hk_account,
        origin_netuid,
        dest_netuid,
        rao_amount,
        ExtrinsicWait::Finalized,
    )
    .await;
    sp.finish_and_clear();

    match result {
        Ok(tx_hash) => {
            print_success("Stake moved successfully!");
            print_info(&format!("Transaction hash: {}", tx_hash));
        }
        Err(e) => {
            print_error(&format!("Failed to move stake: {}", e));
            return Err(anyhow::anyhow!("Move stake failed: {}", e));
        }
    }

    Ok(())
}

/// List all stakes for a coldkey
async fn list_stake(wallet_name: &str, cli: &Cli) -> anyhow::Result<()> {
    use crate::chain::BittensorClient;
    use crate::queries::stakes::get_stake_info_for_coldkey;
    use sp_core::crypto::AccountId32;
    use std::str::FromStr;

    let endpoint = resolve_endpoint(&cli.network, cli.endpoint.as_deref());

    let wallet = match Wallet::new(wallet_name, "default", None) {
        Ok(w) => w,
        Err(e) => {
            print_error(&format!("Invalid wallet name '{}': {}", wallet_name, e));
            return Err(anyhow::anyhow!("Invalid wallet name: {}", e));
        }
    };
    if !wallet.coldkey_exists() {
        print_error(&format!("Wallet '{}' not found", wallet_name));
        return Err(anyhow::anyhow!("Wallet not found"));
    }

    let coldkey_password = prompt_password_optional("Coldkey password (enter if unencrypted)");
    let coldkey_addr = wallet
        .coldkey_ss58(coldkey_password.as_deref())
        .map_err(|e| anyhow::anyhow!("Failed to unlock coldkey: {}", e))?;
    let coldkey_account = AccountId32::from_str(&coldkey_addr)
        .map_err(|e| anyhow::anyhow!("Invalid coldkey address: {:?}", e))?;

    let sp = spinner(&format!("Connecting to {}...", endpoint));
    let client = BittensorClient::new(&endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e))?;
    sp.finish_and_clear();

    let sp = spinner("Fetching stake information...");
    let stakes = get_stake_info_for_coldkey(&client, &coldkey_account)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch stakes: {}", e))?;
    sp.finish_and_clear();

    println!(
        "\nStake for wallet '{}' ({})",
        wallet_name,
        format_address(&coldkey_addr)
    );

    if stakes.is_empty() {
        print_info("No stake found");
        return Ok(());
    }

    let mut table = create_table_with_headers(&["Hotkey", "Subnet", "Stake (TAO)"]);
    let mut total_stake: u128 = 0;

    for stake_info in &stakes {
        table.add_row(vec![
            format_address(&stake_info.hotkey.to_string()),
            stake_info.netuid.to_string(),
            format_tao(stake_info.stake),
        ]);
        total_stake += stake_info.stake;
    }

    println!("{table}");
    println!("\nTotal stake: {}", format_tao(total_stake));

    Ok(())
}
