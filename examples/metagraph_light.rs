use anyhow::Result;
use futures::{stream, StreamExt};
use rand::{rngs::StdRng, Rng, SeedableRng};
use sp_core::crypto::Ss58Codec;
use subxt::dynamic::Value;

use bittensor_rs::chain::BittensorClient;
use bittensor_rs::core::SS58_FORMAT;

#[tokio::main]
async fn main() -> Result<()> {

    let seed: u64 = std::env::var("SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64
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
    println!("netuid={} (selected)\n", netuid);

    // Get neuron count
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
    println!("neurons_count={} (netuid={})\n", n, netuid);

    // Fetch all UIDs and hotkeys
    let storage = client.api.storage();
    let base =
        subxt::storage::dynamic("SubtensorModule", "Keys", vec![Value::u128(netuid as u128)]);
    let mut iter = storage.at_latest().await?.iter(base).await?;
    let mut uid_to_hotkey: Vec<(u64, sp_core::crypto::AccountId32)> =
        Vec::with_capacity(n as usize);
    while let Some(item) = iter.next().await {
        let kv = item?;
        let value = kv.value.to_value()?.remove_context();
        if let Ok(hot) = bittensor_rs::utils::value_decode::decode_account_id32(&value) {
            let key_str = format!("{:?}", kv.keys);
            let uid = extract_last_u64(&key_str).unwrap_or(0);
            uid_to_hotkey.push((uid, hot));
        }
    }
    uid_to_hotkey.sort_by_key(|(u, _)| *u);
    println!("loaded_hotkeys={}", uid_to_hotkey.len());

    // Fetch normalized values
    let emission_vec = fetch_normalized_vec(&client, "Emission", netuid).await?;
    let incentive_vec = fetch_normalized_vec(&client, "Incentive", netuid).await?;
    let trust_vec = fetch_normalized_vec(&client, "Trust", netuid).await?;
    let consensus_vec = fetch_normalized_vec(&client, "Consensus", netuid).await?;
    let vtrust_vec = fetch_normalized_vec(&client, "ValidatorTrust", netuid).await?;
    let dividends_vec = fetch_normalized_vec(&client, "Dividends", netuid).await?;

    // Concurrently fetch additional per-neuron data
    let concurrency = std::env::var("CONC")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(32usize);

    let results: Vec<_> = stream::iter(uid_to_hotkey.into_iter())
        .map(|(uid, hotkey)| {
            let client = &client;
            let emis = emission_vec.get(uid as usize).copied().unwrap_or(0);
            let incentive = incentive_vec.get(uid as usize).copied().unwrap_or(0);
            let trust = trust_vec.get(uid as usize).copied().unwrap_or(0);
            let consensus = consensus_vec.get(uid as usize).copied().unwrap_or(0);
            let vtrust = vtrust_vec.get(uid as usize).copied().unwrap_or(0);
            let dividends = dividends_vec.get(uid as usize).copied().unwrap_or(0);

            async move {
                let hot_bytes: &[u8] =
                    <sp_core::crypto::AccountId32 as AsRef<[u8]>>::as_ref(&hotkey);

                // Get owner (coldkey)
                let owner = client
                    .storage_with_keys(
                        "SubtensorModule",
                        "Owner",
                        vec![Value::from_bytes(hot_bytes)],
                    )
                    .await
                    .ok()
                    .flatten()
                    .and_then(|v| bittensor_rs::utils::value_decode::decode_account_id32(&v).ok());

                // Get stake
                let stake = client
                    .storage_with_keys(
                        "SubtensorModule",
                        "TotalHotkeyAlpha",
                        vec![Value::from_bytes(hot_bytes), Value::u128(netuid as u128)],
                    )
                    .await
                    .ok()
                    .flatten()
                    .and_then(|v| bittensor_rs::utils::value_decode::decode_u128(&v).ok())
                    .unwrap_or(0);

                (
                    uid, hotkey, owner, stake, emis, incentive, trust, consensus, vtrust, dividends,
                )
            }
        })
        .buffer_unordered(concurrency)
        .collect()
        .await;

    println!("\nmetagraph_entries={}", results.len());
    println!(
        "\n{:<5} {:<56} {:<56} {:>14} {:>14} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "UID",
        "Hotkey",
        "Coldkey",
        "Stake(RAO)",
        "Emission",
        "Incentive",
        "Trust",
        "Consensus",
        "VTrust",
        "Dividends"
    );
    println!("{}", "─".repeat(200));

    for (uid, hot, cold, stake, emis, incentive, trust, consensus, vtrust, dividends) in
        results.iter().take(15)
    {
        let hot_ss58 =
            hot.to_ss58check_with_version(sp_core::crypto::Ss58AddressFormat::custom(SS58_FORMAT));
        let cold_ss58 = cold
            .as_ref()
            .map(|c| {
                c.to_ss58check_with_version(sp_core::crypto::Ss58AddressFormat::custom(SS58_FORMAT))
            })
            .unwrap_or_else(|| "None".to_string());

        // Convert normalized u16 values to float percentages
        let incentive_f = (*incentive as f64) / 65535.0;
        let trust_f = (*trust as f64) / 65535.0;
        let consensus_f = (*consensus as f64) / 65535.0;
        let vtrust_f = (*vtrust as f64) / 65535.0;
        let dividends_f = (*dividends as f64) / 65535.0;

        println!(
            "{:<5} {:<56} {:<56} {:>14} {:>14} {:>10.6} {:>10.6} {:>10.6} {:>10.6} {:>10.6}",
            uid,
            hot_ss58,
            cold_ss58,
            stake,
            emis,
            incentive_f,
            trust_f,
            consensus_f,
            vtrust_f,
            dividends_f
        );
    }

    Ok(())
}

async fn fetch_normalized_vec(
    client: &BittensorClient,
    storage_name: &str,
    netuid: u16,
) -> Result<Vec<u16>> {
    client
        .storage_with_keys(
            "SubtensorModule",
            storage_name,
            vec![Value::u128(netuid as u128)],
        )
        .await?
        .and_then(|v| bittensor_rs::utils::value_decode::decode_vec_u16(&v).ok())
        .ok_or_else(|| anyhow::anyhow!("{} not found", storage_name))
}

fn extract_last_u64(s: &str) -> Option<u64> {
    let mut last: Option<u64> = None;
    let bytes = s.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            let mut num: u64 = 0;
            let mut j = i;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                num = num * 10 + (bytes[j] - b'0') as u64;
                j += 1;
            }
            last = Some(num);
            i = j;
        } else {
            i += 1;
        }
    }
    last
}
