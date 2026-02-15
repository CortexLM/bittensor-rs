//! Subnet commands for viewing subnet information and registration.

use crate::cli::utils::{
    confirm, create_table_with_headers, format_address, format_tao, keypair_to_signer, print_error,
    print_info, print_success, print_warning, prompt_password_optional, resolve_endpoint, spinner,
};
use crate::cli::Cli;
use crate::wallet::Wallet;
use clap::{Args, Subcommand};
use std::str::FromStr;

/// Subnet command container
#[derive(Args, Clone)]
pub struct SubnetCommand {
    #[command(subcommand)]
    pub command: SubnetCommands,
}

/// Available subnet operations
#[derive(Subcommand, Clone)]
pub enum SubnetCommands {
    /// List all subnets
    List,

    /// Show detailed subnet information
    Show {
        /// Subnet ID
        #[arg(short, long)]
        netuid: u16,
    },

    /// Show subnet metagraph
    Metagraph {
        /// Subnet ID
        #[arg(short, long)]
        netuid: u16,
    },

    /// Register on a subnet
    Register {
        /// Wallet name
        #[arg(short, long)]
        wallet: String,
        /// Hotkey name
        #[arg(short = 'k', long)]
        hotkey: String,
        /// Subnet ID
        #[arg(short, long)]
        netuid: u16,
        /// Use burned (paid) registration
        #[arg(long)]
        burned: bool,
    },

    /// Show subnet hyperparameters
    Hyperparams {
        /// Subnet ID
        #[arg(short, long)]
        netuid: u16,
    },

    /// Create a new subnet
    Create {
        /// Wallet name
        #[arg(short, long)]
        wallet: String,
    },
}

/// Execute subnet commands
pub async fn execute(cmd: SubnetCommand, cli: &Cli) -> anyhow::Result<()> {
    match cmd.command {
        SubnetCommands::List => list_subnets(cli).await,
        SubnetCommands::Show { netuid } => show_subnet(netuid, cli).await,
        SubnetCommands::Metagraph { netuid } => show_metagraph(netuid, cli).await,
        SubnetCommands::Register {
            wallet,
            hotkey,
            netuid,
            burned,
        } => register(&wallet, &hotkey, netuid, burned, cli).await,
        SubnetCommands::Hyperparams { netuid } => show_hyperparams(netuid, cli).await,
        SubnetCommands::Create { wallet } => create_subnet(&wallet, cli).await,
    }
}

/// List all subnets
async fn list_subnets(cli: &Cli) -> anyhow::Result<()> {
    use crate::chain::BittensorClient;
    use crate::queries::subnets::all_subnets;

    let endpoint = resolve_endpoint(&cli.network, cli.endpoint.as_deref());

    let sp = spinner(&format!("Connecting to {}...", endpoint));
    let client = BittensorClient::new(&endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e))?;
    sp.finish_and_clear();

    let sp = spinner("Fetching subnet list...");
    let subnets = all_subnets(&client)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch subnets: {}", e))?;
    sp.finish_and_clear();

    if subnets.is_empty() {
        print_info("No subnets found");
        return Ok(());
    }

    let mut table = create_table_with_headers(&["NetUID", "Name", "Neurons", "Emission"]);

    for info in &subnets {
        table.add_row(vec![
            info.netuid.to_string(),
            info.name.clone().unwrap_or_else(|| "N/A".to_string()),
            info.neuron_count.to_string(),
            format_tao(info.emission),
        ]);
    }

    println!("\n{table}");
    println!("\nTotal subnets: {}", subnets.len());

    Ok(())
}

/// Show detailed subnet information
async fn show_subnet(netuid: u16, cli: &Cli) -> anyhow::Result<()> {
    use crate::chain::BittensorClient;
    use crate::queries::subnets::{difficulty, immunity_period, subnet_info, tempo};

    let endpoint = resolve_endpoint(&cli.network, cli.endpoint.as_deref());

    let sp = spinner(&format!("Connecting to {}...", endpoint));
    let client = BittensorClient::new(&endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e))?;
    sp.finish_and_clear();

    let sp = spinner(&format!("Fetching subnet {} info...", netuid));
    let info = subnet_info(&client, netuid)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch subnet info: {}", e))?;

    // Fetch additional params
    let tempo_val = tempo(&client, netuid).await.unwrap_or(Some(0)).unwrap_or(0);
    let diff_val = difficulty(&client, netuid)
        .await
        .unwrap_or(Some(0))
        .unwrap_or(0);
    let immunity_val = immunity_period(&client, netuid)
        .await
        .unwrap_or(Some(0))
        .unwrap_or(0);
    sp.finish_and_clear();

    match info {
        Some(info) => {
            println!("\nSubnet {}", netuid);
            println!("═════════════════════════════════════════");
            println!(
                "Name:             {}",
                info.name.unwrap_or_else(|| "N/A".to_string())
            );
            println!("Neurons:          {}", info.neuron_count);
            println!("Emission:         {}", format_tao(info.emission));
            println!("Total Stake:      {}", format_tao(info.total_stake));
            println!("Tempo:            {} blocks", tempo_val);
            println!("Difficulty:       {}", diff_val);
            println!("Immunity Period:  {} blocks", immunity_val);
        }
        None => {
            print_error(&format!("Subnet {} not found", netuid));
            return Err(anyhow::anyhow!("Subnet not found"));
        }
    }

    Ok(())
}

/// Show subnet metagraph
async fn show_metagraph(netuid: u16, cli: &Cli) -> anyhow::Result<()> {
    use crate::chain::BittensorClient;
    use crate::metagraph::sync_metagraph;

    let endpoint = resolve_endpoint(&cli.network, cli.endpoint.as_deref());

    let sp = spinner(&format!("Connecting to {}...", endpoint));
    let client = BittensorClient::new(&endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e))?;
    sp.finish_and_clear();

    let sp = spinner(&format!("Syncing metagraph for subnet {}...", netuid));
    let metagraph = sync_metagraph(&client, netuid)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to sync metagraph: {}", e))?;
    sp.finish_and_clear();

    println!("\nMetagraph for Subnet {}", netuid);
    println!("═════════════════════════════════════════════════════════════════");

    let mut table = create_table_with_headers(&[
        "UID",
        "Hotkey",
        "Coldkey",
        "Stake",
        "Trust",
        "Consensus",
        "Incentive",
        "Active",
    ]);

    let n = metagraph.n;
    let display_count = n.min(50);
    for uid in 0..display_count {
        if let Some(neuron) = metagraph.neurons.get(&uid) {
            table.add_row(vec![
                uid.to_string(),
                format_address(&neuron.hotkey.to_string()),
                format_address(&neuron.coldkey.to_string()),
                format_tao(neuron.total_stake),
                format!("{:.4}", neuron.trust),
                format!("{:.4}", neuron.consensus),
                format!("{:.4}", neuron.incentive),
                if neuron.active { "✓" } else { "✗" }.to_string(),
            ]);
        }
    }

    println!("{table}");

    if n > 50 {
        print_info(&format!("Showing first 50 of {} neurons", n));
    }

    println!("\nTotal neurons: {}", n);
    println!("Block: {}", metagraph.block);

    Ok(())
}

/// Register on a subnet
async fn register(
    wallet_name: &str,
    hotkey_name: &str,
    netuid: u16,
    burned: bool,
    cli: &Cli,
) -> anyhow::Result<()> {
    use crate::chain::{BittensorClient, ExtrinsicWait};
    use crate::validator::registration::{burned_register, register as pow_register};

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

    print_info(&format!("Registering on subnet {}", netuid));
    print_info(&format!("Coldkey: {}", coldkey.ss58_address()));
    print_info(&format!("Hotkey: {}", hotkey.ss58_address()));
    print_info(&format!(
        "Method: {}",
        if burned { "Burned (paid)" } else { "PoW" }
    ));

    if !confirm("Proceed with registration?", cli.no_prompt) {
        print_info("Registration cancelled");
        return Ok(());
    }

    let sp = spinner(&format!("Connecting to {}...", endpoint));
    let client = BittensorClient::new(&endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e))?;
    sp.finish_and_clear();

    let result = if burned {
        let hotkey_account = sp_core::crypto::AccountId32::from_str(hotkey.ss58_address())
            .map_err(|e| anyhow::anyhow!("Invalid hotkey address: {:?}", e))?;
        let r = burned_register(
            &client,
            &signer,
            netuid,
            &hotkey_account,
            ExtrinsicWait::Finalized,
        )
        .await;
        sp.finish_and_clear();
        r
    } else {
        let sp = spinner("Performing PoW registration (this may take a while)...");
        // Standard register
        let hotkey_account = sp_core::crypto::AccountId32::from_str(hotkey.ss58_address())
            .map_err(|e| anyhow::anyhow!("Invalid hotkey address: {:?}", e))?;
        let r = pow_register(
            &client,
            &signer,
            netuid,
            &hotkey_account,
            ExtrinsicWait::Finalized,
        )
        .await;
        sp.finish_and_clear();
        r
    };

    match result {
        Ok(tx_hash) => {
            print_success("Registration successful!");
            print_info(&format!("Transaction hash: {}", tx_hash));
        }
        Err(e) => {
            print_error(&format!("Registration failed: {}", e));
            return Err(anyhow::anyhow!("Registration failed: {}", e));
        }
    }

    Ok(())
}

/// Show subnet hyperparameters
async fn show_hyperparams(netuid: u16, cli: &Cli) -> anyhow::Result<()> {
    use crate::chain::BittensorClient;
    use crate::queries::subnets::{
        difficulty, immunity_period, max_weight_limit, min_allowed_weights, tempo,
        weights_rate_limit,
    };

    let endpoint = resolve_endpoint(&cli.network, cli.endpoint.as_deref());

    let sp = spinner(&format!("Connecting to {}...", endpoint));
    let client = BittensorClient::new(&endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e))?;
    sp.finish_and_clear();

    let sp = spinner(&format!(
        "Fetching hyperparameters for subnet {}...",
        netuid
    ));

    // Fetch available hyperparameters
    let tempo_val = tempo(&client, netuid).await.ok().flatten().unwrap_or(0);
    let difficulty_val = difficulty(&client, netuid)
        .await
        .ok()
        .flatten()
        .unwrap_or(0);
    let immunity_val = immunity_period(&client, netuid)
        .await
        .ok()
        .flatten()
        .unwrap_or(0);
    let max_weights = max_weight_limit(&client, netuid)
        .await
        .ok()
        .flatten()
        .unwrap_or(0);
    let max_weights_ratio = crate::utils::weights::u16_normalized_float(max_weights);
    let min_weights = min_allowed_weights(&client, netuid)
        .await
        .ok()
        .flatten()
        .unwrap_or(0);
    let weights_rate = weights_rate_limit(&client, netuid)
        .await
        .ok()
        .flatten()
        .unwrap_or(0);

    sp.finish_and_clear();

    println!("\nHyperparameters for Subnet {}", netuid);
    println!("═══════════════════════════════════════════════");

    let mut table = create_table_with_headers(&["Parameter", "Value"]);
    table.add_row(vec!["Tempo", &tempo_val.to_string()]);
    table.add_row(vec!["Difficulty", &difficulty_val.to_string()]);
    table.add_row(vec!["Immunity Period", &immunity_val.to_string()]);
    table.add_row(vec![
        "Max Weight Limit",
        &format!("{:.4}", max_weights_ratio),
    ]);
    table.add_row(vec!["Min Allowed Weights", &min_weights.to_string()]);
    table.add_row(vec!["Weights Rate Limit", &weights_rate.to_string()]);

    println!("{table}");

    Ok(())
}

/// Create a new subnet
async fn create_subnet(wallet_name: &str, cli: &Cli) -> anyhow::Result<()> {
    use crate::chain::BittensorClient;

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

    print_info("Creating new subnet");
    print_info(&format!("Coldkey: {}", coldkey.ss58_address()));
    print_warning("This will cost TAO to register a new subnet");

    if !confirm("Proceed with subnet creation?", cli.no_prompt) {
        print_info("Subnet creation cancelled");
        return Ok(());
    }

    let sp = spinner(&format!("Connecting to {}...", endpoint));
    let _client = BittensorClient::new(&endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e))?;
    sp.finish_and_clear();

    // Note: register_network is not currently available in the validator module
    // This would require adding the extrinsic call for RegisterNetwork
    print_warning("Subnet registration requires a specialized extrinsic call.");
    print_info("Please use the Python btcli or submit the RegisterNetwork extrinsic directly.");

    Ok(())
}
