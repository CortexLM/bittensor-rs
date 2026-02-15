//! Root network commands for managing the root subnet (netuid 0).

use crate::cli::utils::{
    confirm, create_table_with_headers, format_address, format_tao, keypair_to_signer,
    parse_f64_list, parse_u16_list, print_error, print_info, print_success, print_warning,
    prompt_password_optional, resolve_endpoint, spinner,
};
use crate::cli::Cli;
use crate::wallet::Wallet;
use clap::{Args, Subcommand};
use std::str::FromStr;

/// Root network command container
#[derive(Args, Clone)]
pub struct RootCommand {
    #[command(subcommand)]
    pub command: RootCommands,
}

/// Available root network operations
#[derive(Subcommand, Clone)]
pub enum RootCommands {
    /// Register on the root network (netuid 0)
    Register {
        /// Wallet name
        #[arg(short, long)]
        wallet: String,
    },

    /// List all root network validators
    List,

    /// Set root network weights
    SetWeights {
        /// Wallet name
        #[arg(short, long)]
        wallet: String,
        /// Hotkey name
        #[arg(short = 'k', long, default_value = "default")]
        hotkey: String,
        /// Network UIDs (comma-separated, e.g., "1,2,3")
        #[arg(long)]
        netuids: String,
        /// Weights (comma-separated, e.g., "0.3,0.5,0.2")
        #[arg(long)]
        weights: String,
    },

    /// Get root network weights for a validator
    GetWeights {
        /// Hotkey address (SS58 format)
        #[arg(long)]
        hotkey: String,
    },

    /// Show root network information
    Info,

    /// Show root network delegates
    Delegates,
}

/// Execute root network commands
pub async fn execute(cmd: RootCommand, cli: &Cli) -> anyhow::Result<()> {
    match cmd.command {
        RootCommands::Register { wallet } => register(&wallet, cli).await,
        RootCommands::List => list_root_validators(cli).await,
        RootCommands::SetWeights {
            wallet,
            hotkey,
            netuids,
            weights,
        } => set_weights(&wallet, &hotkey, &netuids, &weights, cli).await,
        RootCommands::GetWeights { hotkey } => get_weights(&hotkey, cli).await,
        RootCommands::Info => show_info(cli).await,
        RootCommands::Delegates => show_delegates(cli).await,
    }
}

/// Register on the root network
async fn register(wallet_name: &str, cli: &Cli) -> anyhow::Result<()> {
    use crate::chain::{BittensorClient, ExtrinsicWait};
    use crate::validator::root::root_register;

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
    let coldkey = wallet
        .coldkey_keypair(coldkey_password.as_deref())
        .map_err(|e| anyhow::anyhow!("Failed to unlock coldkey: {}", e))?;
    let signer = keypair_to_signer(&coldkey);

    let hotkey_password = prompt_password_optional("Hotkey password (enter if unencrypted)");
    let hotkey = wallet
        .hotkey_keypair(hotkey_password.as_deref())
        .map_err(|e| anyhow::anyhow!("Failed to unlock hotkey: {}", e))?;

    print_info("Registering on root network (subnet 0)");
    print_info(&format!("Coldkey: {}", coldkey.ss58_address()));
    print_info(&format!("Hotkey: {}", hotkey.ss58_address()));

    if !confirm("Proceed with root registration?", cli.no_prompt) {
        print_info("Registration cancelled");
        return Ok(());
    }

    let sp = spinner(&format!("Connecting to {}...", endpoint));
    let client = BittensorClient::new(&endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e))?;
    sp.finish_and_clear();

    let sp = spinner("Submitting root registration...");
    let hotkey_account = sp_core::crypto::AccountId32::from_str(hotkey.ss58_address())
        .map_err(|e| anyhow::anyhow!("Invalid hotkey address: {:?}", e))?;
    let result = root_register(&client, &signer, &hotkey_account, ExtrinsicWait::Finalized).await;
    sp.finish_and_clear();

    match result {
        Ok(tx_hash) => {
            print_success("Root registration successful!");
            print_info(&format!("Transaction hash: {}", tx_hash));
        }
        Err(e) => {
            print_error(&format!("Root registration failed: {}", e));
            return Err(anyhow::anyhow!("Registration failed: {}", e));
        }
    }

    Ok(())
}

/// List all root network validators
async fn list_root_validators(cli: &Cli) -> anyhow::Result<()> {
    use crate::chain::BittensorClient;
    use crate::metagraph::sync_metagraph;

    const ROOT_NETUID: u16 = 0;

    let endpoint = resolve_endpoint(&cli.network, cli.endpoint.as_deref());

    let sp = spinner(&format!("Connecting to {}...", endpoint));
    let client = BittensorClient::new(&endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e))?;
    sp.finish_and_clear();

    let sp = spinner("Fetching root network metagraph...");
    let metagraph = sync_metagraph(&client, ROOT_NETUID)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to sync root metagraph: {}", e))?;
    sp.finish_and_clear();

    println!("\nRoot Network Validators (Subnet 0)");
    println!("═══════════════════════════════════════════════════════════════");

    let mut table = create_table_with_headers(&[
        "UID",
        "Hotkey",
        "Coldkey",
        "Stake",
        "Trust",
        "Consensus",
        "Incentive",
    ]);

    let n = metagraph.n as usize;
    for uid in 0..n as u64 {
        if let Some(neuron) = metagraph.neurons.get(&uid) {
            table.add_row(vec![
                uid.to_string(),
                format_address(&neuron.hotkey.to_string()),
                format_address(&neuron.coldkey.to_string()),
                format_tao(neuron.total_stake),
                format!("{:.4}", neuron.trust),
                format!("{:.4}", neuron.consensus),
                format!("{:.4}", neuron.incentive),
            ]);
        }
    }

    println!("{table}");
    println!("\nTotal root validators: {}", n);

    Ok(())
}

/// Set root network weights
async fn set_weights(
    wallet_name: &str,
    hotkey_name: &str,
    netuids_str: &str,
    weights_str: &str,
    cli: &Cli,
) -> anyhow::Result<()> {
    use crate::chain::{BittensorClient, ExtrinsicWait};
    use crate::validator::root::root_set_weights;

    let endpoint = resolve_endpoint(&cli.network, cli.endpoint.as_deref());

    // Parse netuids
    let netuids = parse_u16_list(netuids_str)?;

    // Parse weights
    let weight_values = parse_f64_list(weights_str)?;

    if netuids.len() != weight_values.len() {
        print_error("Number of netuids must match number of weights");
        return Err(anyhow::anyhow!("Mismatched netuids and weights"));
    }

    // Normalize weights to u16 for the API
    let sum: f64 = weight_values.iter().sum();
    if sum <= 0.0 {
        print_error("Weights must sum to a positive value");
        return Err(anyhow::anyhow!("Invalid weights"));
    }

    let normalized_f32: Vec<f32> = weight_values.iter().map(|w| (*w / sum) as f32).collect();
    let normalized_weights: Vec<u16> = normalized_f32
        .iter()
        .map(|w| crate::utils::weights::float_to_u16(*w as f64))
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
    let _hotkey = wallet
        .hotkey_keypair(hotkey_password.as_deref())
        .map_err(|e| anyhow::anyhow!("Failed to unlock hotkey: {}", e))?;

    print_info("Setting root network weights");
    print_info(&format!("Coldkey: {}", coldkey.ss58_address()));
    print_info(&format!("Netuids: {:?}", netuids));
    print_info(&format!("Weights (normalized): {:?}", normalized_weights));

    if !confirm("Proceed with setting weights?", cli.no_prompt) {
        print_info("Weights setting cancelled");
        return Ok(());
    }

    let sp = spinner(&format!("Connecting to {}...", endpoint));
    let client = BittensorClient::new(&endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e))?;
    sp.finish_and_clear();

    let result = root_set_weights(
        &client,
        &signer,
        &netuids,
        &normalized_weights,
        0,
        ExtrinsicWait::Finalized,
    )
    .await;
    sp.finish_and_clear();

    match result {
        Ok(tx_hash) => {
            print_success("Root weights set successfully!");
            print_info(&format!("Transaction hash: {}", tx_hash));
        }
        Err(e) => {
            print_error(&format!("Failed to set weights: {}", e));
            return Err(anyhow::anyhow!("Set weights failed: {}", e));
        }
    }

    Ok(())
}

/// Get root network weights for a validator
async fn get_weights(hotkey_addr: &str, cli: &Cli) -> anyhow::Result<()> {
    use crate::chain::BittensorClient;
    use crate::metagraph::sync_metagraph;

    const ROOT_NETUID: u16 = 0;

    let endpoint = resolve_endpoint(&cli.network, cli.endpoint.as_deref());

    let sp = spinner(&format!("Connecting to {}...", endpoint));
    let client = BittensorClient::new(&endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e))?;
    sp.finish_and_clear();

    let sp = spinner("Fetching root metagraph...");
    let metagraph = sync_metagraph(&client, ROOT_NETUID)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch metagraph: {}", e))?;
    sp.finish_and_clear();

    println!("\nRoot Network Weights for {}", format_address(hotkey_addr));
    println!("═══════════════════════════════════════════════════");

    // Find the UID for this hotkey
    let mut found_uid: Option<u64> = None;
    for (uid, neuron) in &metagraph.neurons {
        if neuron.hotkey.to_string() == hotkey_addr {
            found_uid = Some(*uid);
            break;
        }
    }

    match found_uid {
        Some(uid) => {
            print_info(&format!("Hotkey found at UID {}", uid));
            // Display root network neuron info
            let mut table = create_table_with_headers(&["UID", "Incentive", "Consensus"]);
            for uid in 0..metagraph.n {
                if let Some(neuron) = metagraph.neurons.get(&uid) {
                    table.add_row(vec![
                        uid.to_string(),
                        format!("{:.4}", neuron.incentive),
                        format!("{:.4}", neuron.consensus),
                    ]);
                }
            }
            println!("{table}");
        }
        None => {
            print_warning(&format!("Hotkey {} not found in root network", hotkey_addr));
        }
    }

    Ok(())
}

/// Show root network information
async fn show_info(cli: &Cli) -> anyhow::Result<()> {
    use crate::chain::BittensorClient;
    use crate::queries::subnets::{difficulty, immunity_period, subnet_info, tempo};

    const ROOT_NETUID: u16 = 0;

    let endpoint = resolve_endpoint(&cli.network, cli.endpoint.as_deref());

    let sp = spinner(&format!("Connecting to {}...", endpoint));
    let client = BittensorClient::new(&endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e))?;
    sp.finish_and_clear();

    let sp = spinner("Fetching root network info...");
    let info = subnet_info(&client, ROOT_NETUID)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch root info: {}", e))?;

    let tempo_val = tempo(&client, ROOT_NETUID)
        .await
        .ok()
        .flatten()
        .unwrap_or(0);
    let diff_val = difficulty(&client, ROOT_NETUID)
        .await
        .ok()
        .flatten()
        .unwrap_or(0);
    let immunity_val = immunity_period(&client, ROOT_NETUID)
        .await
        .ok()
        .flatten()
        .unwrap_or(0);
    sp.finish_and_clear();

    match info {
        Some(info) => {
            println!("\nRoot Network (Subnet 0)");
            println!("═══════════════════════════════════════════════");
            println!(
                "Name:             {}",
                info.name.unwrap_or_else(|| "Root Network".to_string())
            );
            println!("Validators:       {}", info.neuron_count);
            println!("Tempo:            {} blocks", tempo_val);
            println!("Difficulty:       {}", diff_val);
            println!("Immunity Period:  {} blocks", immunity_val);
            println!("Total Stake:      {}", format_tao(info.total_stake));
        }
        None => {
            print_warning("Could not fetch root network info");
        }
    }

    Ok(())
}

/// Show root network delegates
async fn show_delegates(cli: &Cli) -> anyhow::Result<()> {
    use crate::chain::BittensorClient;
    use crate::queries::delegates::get_delegates;

    let endpoint = resolve_endpoint(&cli.network, cli.endpoint.as_deref());

    let sp = spinner(&format!("Connecting to {}...", endpoint));
    let client = BittensorClient::new(&endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e))?;
    sp.finish_and_clear();

    let sp = spinner("Fetching delegates...");
    let delegates = get_delegates(&client)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch delegates: {}", e))?;
    sp.finish_and_clear();

    if delegates.is_empty() {
        print_info("No delegates found");
        return Ok(());
    }

    println!("\nRoot Network Delegates");
    println!("═══════════════════════════════════════════════════════════════");

    let mut table = create_table_with_headers(&["Hotkey", "Total Stake", "Take", "Owner"]);

    for delegate in &delegates {
        // Calculate total stake across all subnets
        let total_stake: u128 = delegate.total_stake.values().sum();
        table.add_row(vec![
            format_address(&delegate.base.hotkey_ss58.to_string()),
            format_tao(total_stake),
            format!("{:.2}%", delegate.base.take * 100.0),
            format_address(&delegate.base.owner_ss58.to_string()),
        ]);
    }

    println!("{table}");
    println!("\nTotal delegates: {}", delegates.len());

    Ok(())
}
