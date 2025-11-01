use anyhow::Result;
use rand::{rngs::StdRng, Rng, SeedableRng};
use sp_core::crypto::Ss58Codec;

use bittensor_rs::chain::BittensorClient;
use bittensor_rs::core::SS58_FORMAT;
use bittensor_rs::queries::neurons_bulk;

#[tokio::main]
async fn main() -> Result<()> {

    let seed: u64 = std::env::var("SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            (now & 0xFFFF_FFFF_FFFF_FFFF) as u64
        });
    let mut rng = StdRng::seed_from_u64(seed);
    println!("seed={}\n", seed);

    let client = BittensorClient::with_default().await?;

    let total = bittensor_rs::queries::subnets::total_subnets(&client)
        .await
        .unwrap_or(0);
    if total == 0 {
        println!("no subnets");
        return Ok(());
    }
    let netuid: u16 = rng.gen_range(0..total);
    println!("netuid={}\n", netuid);

    let neurons_list = neurons_bulk::neurons_bulk(&client, netuid, None).await?;

    // Display first 10 neurons with full information
    for (i, neuron) in neurons_list.iter().take(10).enumerate() {
        let hot_ss58 = neuron
            .hotkey
            .to_ss58check_with_version(sp_core::crypto::Ss58AddressFormat::custom(SS58_FORMAT));
        let cold_ss58 = neuron
            .coldkey
            .to_ss58check_with_version(sp_core::crypto::Ss58AddressFormat::custom(SS58_FORMAT));

        println!("═══ Neuron {} ═══", i);
        println!("UID:              {}", neuron.uid);
        println!("Hotkey:           {}", hot_ss58);
        println!("Coldkey:          {}", cold_ss58);
        println!("Active:           {}", neuron.active);
        println!("Stake:            {} RAO", neuron.stake);
        println!("Rank:             {:.6}", neuron.rank);
        println!("Trust:            {:.6}", neuron.trust);
        println!("Consensus:        {:.6}", neuron.consensus);
        println!("Incentive:        {:.6}", neuron.incentive);
        println!("Dividends:        {:.6}", neuron.dividends);
        println!("Emission:         {:.2} RAO", neuron.emission);
        println!("VTrust:           {:.6}", neuron.validator_trust);
        println!("VPermit:          {}", neuron.validator_permit);
        println!("Last Update:      {}", neuron.last_update);
        println!("Version:          {}", neuron.version);

        if let Some(ref axon) = neuron.axon_info {
            if axon.is_serving() {
                println!(
                    "Axon:             {}:{} (v{})",
                    axon.ip, axon.port, axon.version
                );
            } else {
                println!("Axon:             Not serving");
            }
        }

        if let Some(ref prom) = neuron.prometheus_info {
            if prom.is_serving() {
                println!(
                    "Prometheus:       {}:{} (v{})",
                    prom.ip, prom.port, prom.version
                );
            }
        }

        // Display weights if not empty
        if !neuron.weights.is_empty() && neuron.weights.len() <= 5 {
            println!(
                "Weights ({}):      {:?}",
                neuron.weights.len(),
                neuron.weights
            );
        } else if !neuron.weights.is_empty() {
            println!("Weights:          {} connections", neuron.weights.len());
        }

        println!();
    }

    Ok(())
}
