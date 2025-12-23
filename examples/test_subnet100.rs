use anyhow::Result;
use subxt::dynamic::Value;

use bittensor_rs::chain::BittensorClient;
use bittensor_rs::queries::subnets;
use bittensor_rs::metagraph::sync_metagraph;

#[tokio::main]
async fn main() -> Result<()> {
    let netuid: u16 = 100;
    println!("Testing subnet {}", netuid);
    
    let client = BittensorClient::with_default().await?;
    println!("Connected to Bittensor");

    // Check if subnet exists
    let n_value = client
        .storage_with_keys(
            "SubtensorModule",
            "SubnetworkN",
            vec![Value::u128(netuid as u128)],
        )
        .await?;
    
    let n = n_value
        .as_ref()
        .and_then(|v| bittensor_rs::utils::decoders::decode_u64(v).ok())
        .unwrap_or(0);
    
    println!("Subnet {} has {} neurons", netuid, n);
    
    if n == 0 {
        println!("Subnet {} does not exist or has no neurons", netuid);
        return Ok(());
    }

    // Try to get owner
    let owner = subnets::subnet_owner_hotkey(&client, netuid)
        .await
        .ok()
        .flatten();
    println!("Owner: {:?}", owner.map(|a| hex::encode(<sp_core::crypto::AccountId32 as AsRef<[u8]>>::as_ref(&a))));

    // Try metagraph sync
    println!("\nTrying metagraph sync...");
    match sync_metagraph(&client, netuid).await {
        Ok(metagraph) => {
            println!("Metagraph synced successfully!");
            println!("  Block: {}", metagraph.block);
            println!("  N: {}", metagraph.n);
            println!("  Neurons: {}", metagraph.neurons.len());
            
            // Show first 3 neurons
            for (uid, neuron) in metagraph.neurons.iter().take(3) {
                println!("\n  Neuron UID {}:", uid);
                println!("    Hotkey: {}", hex::encode(<sp_core::crypto::AccountId32 as AsRef<[u8]>>::as_ref(&neuron.hotkey)));
                println!("    Stake: {} RAO ({:.2} TAO)", neuron.stake, neuron.stake as f64 / 1e9);
                println!("    Active: {}", neuron.active);
                println!("    ValidatorPermit: {}", neuron.validator_permit);
            }
        }
        Err(e) => {
            println!("Metagraph sync failed: {}", e);
            println!("Error chain: {:?}", e);
        }
    }

    Ok(())
}
