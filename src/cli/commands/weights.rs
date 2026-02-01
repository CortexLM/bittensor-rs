//! Weight commands for commit-reveal and direct weight setting.

use crate::cli::utils::{
    confirm, create_table_with_headers, format_address, keypair_to_signer, parse_f64_list,
    parse_u16_list, print_error, print_info, print_success, print_warning,
    prompt_password_optional, resolve_endpoint, spinner,
};
use crate::cli::Cli;
use crate::wallet::Wallet;
use clap::{Args, Subcommand};

/// Weights command container
#[derive(Args, Clone)]
pub struct WeightsCommand {
    #[command(subcommand)]
    pub command: WeightsCommands,
}

/// Available weight operations
#[derive(Subcommand, Clone)]
pub enum WeightsCommands {
    /// Commit weights (for commit-reveal)
    Commit {
        /// Wallet name
        #[arg(short, long)]
        wallet: String,
        /// Hotkey name
        #[arg(short = 'k', long)]
        hotkey: String,
        /// Subnet ID
        #[arg(short, long)]
        netuid: u16,
        /// Target UIDs (comma-separated, e.g., "1,2,3")
        #[arg(long)]
        uids: String,
        /// Weights (comma-separated, e.g., "0.3,0.5,0.2")
        #[arg(long)]
        weights: String,
    },

    /// Reveal committed weights
    Reveal {
        /// Wallet name
        #[arg(short, long)]
        wallet: String,
        /// Hotkey name
        #[arg(short = 'k', long)]
        hotkey: String,
        /// Subnet ID
        #[arg(short, long)]
        netuid: u16,
    },

    /// Set weights directly (no commit-reveal)
    Set {
        /// Wallet name
        #[arg(short, long)]
        wallet: String,
        /// Hotkey name
        #[arg(short = 'k', long)]
        hotkey: String,
        /// Subnet ID
        #[arg(short, long)]
        netuid: u16,
        /// Target UIDs (comma-separated, e.g., "1,2,3")
        #[arg(long)]
        uids: String,
        /// Weights (comma-separated, e.g., "0.3,0.5,0.2")
        #[arg(long)]
        weights: String,
    },

    /// Check current weight information
    Info {
        /// Subnet ID
        #[arg(short, long)]
        netuid: u16,
        /// Hotkey address to check (optional)
        #[arg(long)]
        hotkey: Option<String>,
    },

    /// Show pending commits
    Pending {
        /// Wallet name
        #[arg(short, long)]
        wallet: String,
        /// Hotkey name
        #[arg(short = 'k', long)]
        hotkey: String,
    },
}

/// Execute weight commands
pub async fn execute(cmd: WeightsCommand, cli: &Cli) -> anyhow::Result<()> {
    match cmd.command {
        WeightsCommands::Commit {
            wallet,
            hotkey,
            netuid,
            uids,
            weights,
        } => commit_weights(&wallet, &hotkey, netuid, &uids, &weights, cli).await,
        WeightsCommands::Reveal {
            wallet,
            hotkey,
            netuid,
        } => reveal_weights(&wallet, &hotkey, netuid, cli).await,
        WeightsCommands::Set {
            wallet,
            hotkey,
            netuid,
            uids,
            weights,
        } => set_weights(&wallet, &hotkey, netuid, &uids, &weights, cli).await,
        WeightsCommands::Info { netuid, hotkey } => weight_info(netuid, hotkey.as_deref(), cli).await,
        WeightsCommands::Pending { wallet, hotkey } => pending_commits(&wallet, &hotkey, cli).await,
    }
}

/// Commit weights (for commit-reveal protocol)
async fn commit_weights(
    wallet_name: &str,
    hotkey_name: &str,
    netuid: u16,
    uids_str: &str,
    weights_str: &str,
    cli: &Cli,
) -> anyhow::Result<()> {
    use crate::chain::{BittensorClient, ExtrinsicWait};
    use crate::utils::crypto::generate_subtensor_commit_hash;
    use crate::validator::weights::commit_weights as raw_commit_weights;

    let endpoint = resolve_endpoint(&cli.network, cli.endpoint.as_deref());

    // Parse UIDs and weights
    let uids = parse_u16_list(uids_str)?;
    let weight_values = parse_f64_list(weights_str)?;

    if uids.len() != weight_values.len() {
        print_error("Number of UIDs must match number of weights");
        return Err(anyhow::anyhow!("Mismatched UIDs and weights"));
    }

    // Normalize and convert weights to u16
    let sum: f64 = weight_values.iter().sum();
    if sum <= 0.0 {
        print_error("Weights must sum to a positive value");
        return Err(anyhow::anyhow!("Invalid weights"));
    }

    let normalized: Vec<u16> = weight_values
        .iter()
        .map(|w| ((w / sum) * 65535.0) as u16)
        .collect();

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

    print_info(&format!("Committing weights for subnet {}", netuid));
    print_info(&format!("Hotkey: {}", hotkey.ss58_address()));
    print_info(&format!("UIDs: {:?}", uids));
    print_info(&format!("Weights (normalized u16): {:?}", normalized));

    if !confirm("Proceed with weight commit?", cli.no_prompt) {
        print_info("Weight commit cancelled");
        return Ok(());
    }

    let sp = spinner(&format!("Connecting to {}...", endpoint));
    let client = BittensorClient::new(&endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e))?;
    sp.finish_and_clear();

    // Generate commit hash
    // Get the hotkey's public key bytes
    use sp_core::Pair;
    let hotkey_pubkey: [u8; 32] = hotkey.pair().public().0;
    
    // Generate random salt
    let salt: Vec<u16> = (0..8).map(|_| rand::random::<u16>()).collect();
    
    let commit_hash_bytes = generate_subtensor_commit_hash(
        &hotkey_pubkey,
        netuid,
        None, // mechanism_id
        &uids,
        &normalized,
        &salt,
        0, // version_key
    );
    let commit_hash = hex::encode(commit_hash_bytes);

    let sp = spinner("Submitting weight commit...");
    let result = raw_commit_weights(&client, &signer, netuid, &commit_hash, ExtrinsicWait::Finalized).await;
    sp.finish_and_clear();

    match result {
        Ok(tx_hash) => {
            print_success("Weights committed successfully!");
            print_info(&format!("Transaction hash: {}", tx_hash));
            print_warning("Remember to reveal your weights before the reveal period ends!");
        }
        Err(e) => {
            print_error(&format!("Failed to commit weights: {}", e));
            return Err(anyhow::anyhow!("Weight commit failed: {}", e));
        }
    }

    Ok(())
}

/// Reveal previously committed weights
async fn reveal_weights(
    wallet_name: &str,
    hotkey_name: &str,
    netuid: u16,
    cli: &Cli,
) -> anyhow::Result<()> {
    use crate::chain::BittensorClient;

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
    let _coldkey = wallet
        .coldkey_keypair(coldkey_password.as_deref())
        .map_err(|e| anyhow::anyhow!("Failed to unlock coldkey: {}", e))?;

    let hotkey_password = prompt_password_optional("Hotkey password (enter if unencrypted)");
    let hotkey = wallet
        .hotkey_keypair(hotkey_password.as_deref())
        .map_err(|e| anyhow::anyhow!("Failed to unlock hotkey: {}", e))?;

    print_info(&format!("Revealing weights for subnet {}", netuid));
    print_info(&format!("Hotkey: {}", hotkey.ss58_address()));

    if !confirm("Proceed with weight reveal?", cli.no_prompt) {
        print_info("Weight reveal cancelled");
        return Ok(());
    }

    let sp = spinner(&format!("Connecting to {}...", endpoint));
    let _client = BittensorClient::new(&endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e))?;
    sp.finish_and_clear();

    // Note: reveal_weights requires the original uids, weights, and salt that were committed
    // This information is typically stored locally when commit is performed
    print_warning("Weight reveal requires the original committed data (uids, weights, salt).");
    print_info("Use the high-level Subtensor API for automatic commit/reveal tracking.");
    print_info("Or use 'btcli weights set' for direct weight setting if commit-reveal is disabled.");

    Ok(())
}

/// Set weights directly (no commit-reveal)
async fn set_weights(
    wallet_name: &str,
    hotkey_name: &str,
    netuid: u16,
    uids_str: &str,
    weights_str: &str,
    cli: &Cli,
) -> anyhow::Result<()> {
    use crate::chain::{BittensorClient, ExtrinsicWait};
    use crate::validator::weights::set_weights as raw_set_weights;

    let endpoint = resolve_endpoint(&cli.network, cli.endpoint.as_deref());

    // Parse UIDs and weights
    let uids = parse_u16_list(uids_str)?;
    let weight_values = parse_f64_list(weights_str)?;

    if uids.len() != weight_values.len() {
        print_error("Number of UIDs must match number of weights");
        return Err(anyhow::anyhow!("Mismatched UIDs and weights"));
    }

    // Normalize and convert weights to f32 for the API
    let sum: f64 = weight_values.iter().sum();
    if sum <= 0.0 {
        print_error("Weights must sum to a positive value");
        return Err(anyhow::anyhow!("Invalid weights"));
    }

    let normalized_f32: Vec<f32> = weight_values
        .iter()
        .map(|w| (*w / sum) as f32)
        .collect();

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
    let _coldkey = wallet
        .coldkey_keypair(coldkey_password.as_deref())
        .map_err(|e| anyhow::anyhow!("Failed to unlock coldkey: {}", e))?;

    let hotkey_password = prompt_password_optional("Hotkey password (enter if unencrypted)");
    let hotkey = wallet
        .hotkey_keypair(hotkey_password.as_deref())
        .map_err(|e| anyhow::anyhow!("Failed to unlock hotkey: {}", e))?;
    let signer = keypair_to_signer(&hotkey);

    // Convert UIDs to u64
    let uids_u64: Vec<u64> = uids.iter().map(|u| *u as u64).collect();

    print_info(&format!("Setting weights for subnet {}", netuid));
    print_info(&format!("Hotkey: {}", hotkey.ss58_address()));
    print_info(&format!("UIDs: {:?}", uids));
    print_info(&format!("Weights (normalized): {:?}", normalized_f32));

    if !confirm("Proceed with setting weights?", cli.no_prompt) {
        print_info("Weight setting cancelled");
        return Ok(());
    }

    let sp = spinner(&format!("Connecting to {}...", endpoint));
    let client = BittensorClient::new(&endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e))?;
    sp.finish_and_clear();

    let sp = spinner("Submitting weights...");
    let result = raw_set_weights(
        &client, 
        &signer, 
        netuid, 
        &uids_u64, 
        &normalized_f32, 
        Some(0), // version_key
        ExtrinsicWait::Finalized
    ).await;
    sp.finish_and_clear();

    match result {
        Ok(tx_hash) => {
            print_success("Weights set successfully!");
            print_info(&format!("Transaction hash: {}", tx_hash));
        }
        Err(e) => {
            print_error(&format!("Failed to set weights: {}", e));
            return Err(anyhow::anyhow!("Set weights failed: {}", e));
        }
    }

    Ok(())
}

/// Show weight-related information for a subnet
async fn weight_info(netuid: u16, _hotkey: Option<&str>, cli: &Cli) -> anyhow::Result<()> {
    use crate::chain::BittensorClient;
    use crate::queries::subnets::{commit_reveal_enabled, tempo, weights_rate_limit};

    let endpoint = resolve_endpoint(&cli.network, cli.endpoint.as_deref());

    let sp = spinner(&format!("Connecting to {}...", endpoint));
    let client = BittensorClient::new(&endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e))?;
    sp.finish_and_clear();

    let sp = spinner(&format!("Fetching weight info for subnet {}...", netuid));

    // Fetch weight-related parameters
    let tempo_val = tempo(&client, netuid).await.ok().flatten().unwrap_or(0);
    let rate_limit = weights_rate_limit(&client, netuid).await.ok().flatten().unwrap_or(0);
    let cr_enabled = commit_reveal_enabled(&client, netuid).await.unwrap_or(false);

    sp.finish_and_clear();

    println!("\nWeight Information for Subnet {}", netuid);
    println!("═══════════════════════════════════════════════");

    let mut table = create_table_with_headers(&["Parameter", "Value"]);
    table.add_row(vec!["Tempo", &tempo_val.to_string()]);
    table.add_row(vec!["Weights Rate Limit", &rate_limit.to_string()]);
    table.add_row(vec![
        "Commit-Reveal Enabled",
        if cr_enabled { "Yes" } else { "No" },
    ]);

    println!("{table}");

    Ok(())
}

/// Show pending weight commits
async fn pending_commits(
    wallet_name: &str,
    hotkey_name: &str,
    cli: &Cli,
) -> anyhow::Result<()> {
    use crate::chain::BittensorClient;

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

    let hotkey_password = prompt_password_optional("Hotkey password (enter if unencrypted)");
    let hotkey = wallet
        .hotkey_keypair(hotkey_password.as_deref())
        .map_err(|e| anyhow::anyhow!("Failed to unlock hotkey: {}", e))?;

    let sp = spinner(&format!("Connecting to {}...", endpoint));
    let _client = BittensorClient::new(&endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e))?;
    sp.finish_and_clear();

    println!("\nPending Commits for {}", format_address(hotkey.ss58_address()));
    println!("═══════════════════════════════════════════════");

    // Note: Pending commits are typically stored locally by the application
    // since the chain only stores the hash. Display any local state if available.
    print_info("Pending commit tracking requires local state management.");
    print_info("Use the Subtensor high-level API for automatic commit tracking.");

    Ok(())
}
