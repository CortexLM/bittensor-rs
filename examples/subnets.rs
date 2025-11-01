use anyhow::Result;
use rand::{rngs::StdRng, Rng, SeedableRng};
use subxt::dynamic::Value;

use bittensor_rs::chain::BittensorClient;
use bittensor_rs::queries::subnets;

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
    println!("seed={}", seed);
    let client = BittensorClient::with_default().await?;

    let total = subnets::total_subnets(&client).await.unwrap_or(0);
    println!("total_subnets={}", total);
    if total == 0 {
        return Ok(());
    }

    let netuid: u16 = rng.gen_range(0..total);
    println!("netuid={}", netuid);

    let n_value = client
        .storage_with_keys(
            "SubtensorModule",
            "SubnetworkN",
            vec![Value::u128(netuid as u128)],
        )
        .await?;
    let n = n_value
        .and_then(|v| bittensor_rs::utils::value_decode::decode_u64(&v).ok())
        .unwrap_or(0);
    println!("neurons_count={} (netuid={})", n, netuid);

    let owner = subnets::subnet_owner_hotkey(&client, netuid)
        .await
        .ok()
        .flatten();
    println!(
        "owner_hotkey={:?}",
        owner
            .as_ref()
            .map(|a| hex::encode(<sp_core::crypto::AccountId32 as AsRef<[u8]>>::as_ref(a)))
    );

    let price_rao = subnets::get_subnet_price(&client, netuid)
        .await
        .unwrap_or(0);
    println!("subnet_price_rao={}", price_rao);

    let next_epoch = subnets::get_next_epoch_start_block(&client, netuid, None).await?;
    println!("next_epoch_start_block={:?}", next_epoch);

    match subnets::subnet_emission_percent(&client, netuid).await {
        Ok(Some(p)) => println!("subnet_emission_percent={:.6}", p),
        _ => println!("subnet_emission_percent=None"),
    }

    Ok(())
}
