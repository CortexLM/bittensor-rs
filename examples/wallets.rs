use anyhow::Result;
use rand::{rngs::StdRng, Rng, SeedableRng};
use subxt::dynamic::Value;

use bittensor_rs::chain::BittensorClient;
use bittensor_rs::queries::{subnets, wallets};

#[tokio::main]
async fn main() -> Result<()> {
    let rpc = std::env::var("BITTENSOR_RPC")
        .unwrap_or_else(|_| "wss://entrypoint-finney.opentensor.ai:443".to_string());
    let client = BittensorClient::new(rpc).await?;

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

    let total = subnets::total_subnets(&client).await.unwrap_or(0);
    if total == 0 {
        println!("no subnets");
        return Ok(());
    }
    let netuid: u16 = rng.gen_range(0..total);
    let n_val = client
        .storage_with_keys(
            "SubtensorModule",
            "SubnetworkN",
            vec![Value::u128(netuid as u128)],
        )
        .await?;
    let n = n_val
        .and_then(|v| bittensor_rs::utils::value_decode::decode_u64(&v).ok())
        .unwrap_or(0);
    if n == 0 {
        println!("empty subnet");
        return Ok(());
    }
    let uid: u64 = rng.gen_range(0..n);

    let hotkey_val = client
        .storage_with_keys(
            "SubtensorModule",
            "Keys",
            vec![Value::u128(netuid as u128), Value::u128(uid as u128)],
        )
        .await?;
    let Some(hotkey) =
        hotkey_val.and_then(|v| bittensor_rs::utils::value_decode::decode_account_id32(&v).ok())
    else {
        println!("no hotkey");
        return Ok(());
    };
    let coldkey_val = client
        .storage_with_keys(
            "SubtensorModule",
            "Owner",
            vec![Value::from_bytes(<sp_core::crypto::AccountId32 as AsRef<
                [u8],
            >>::as_ref(&hotkey))],
        )
        .await?;
    let coldkey =
        coldkey_val.and_then(|v| bittensor_rs::utils::value_decode::decode_account_id32(&v).ok());

    let exist = wallets::does_hotkey_exist(&client, &hotkey)
        .await
        .unwrap_or(false);
    let reg_any = wallets::is_hotkey_registered_any(&client, &hotkey)
        .await
        .unwrap_or(false);
    let owned = match &coldkey {
        Some(c) => wallets::get_owned_hotkeys(&client, c)
            .await
            .unwrap_or_default(),
        None => Vec::new(),
    };
    println!(
        "exists={} registered_any={} owned_hotkeys_count={}",
        exist,
        reg_any,
        owned.len()
    );
    Ok(())
}
