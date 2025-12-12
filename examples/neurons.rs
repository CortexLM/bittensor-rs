use anyhow::Result;
use rand::{rngs::StdRng, Rng, SeedableRng};
use subxt::dynamic::Value;

use bittensor_rs::chain::BittensorClient;

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

    let total = bittensor_rs::queries::subnets::total_subnets(&client)
        .await
        .unwrap_or(0);
    if total == 0 {
        println!("no subnets");
        return Ok(());
    }
    let netuid: u16 = rng.random_range(0..total);
    println!("netuid={}", netuid);

    let n_value = client
        .storage_with_keys(
            "SubtensorModule",
            "SubnetworkN",
            vec![Value::u128(netuid as u128)],
        )
        .await?;
    let n = n_value
        .and_then(|v| bittensor_rs::utils::decoders::decode_u64(&v).ok())
        .unwrap_or(0);
    if n == 0 {
        println!("empty subnet");
        return Ok(());
    }
    let uid: u64 = rng.random_range(0..n);

    let hotkey_val = client
        .storage_with_keys(
            "SubtensorModule",
            "Keys",
            vec![Value::u128(netuid as u128), Value::u128(uid as u128)],
        )
        .await?;
    let hotkey =
        hotkey_val.and_then(|v| bittensor_rs::utils::decoders::decode_account_id32(&v).ok());
    let owner_val = hotkey.as_ref().map(|h| {
        client.storage_with_keys(
            "SubtensorModule",
            "Owner",
            vec![Value::from_bytes(<sp_core::crypto::AccountId32 as AsRef<
                [u8],
            >>::as_ref(h))],
        )
    });
    let coldkey = match owner_val {
        Some(res) => res
            .await?
            .and_then(|v| bittensor_rs::utils::decoders::decode_account_id32(&v).ok()),
        None => None,
    };

    let total_stake = match (&hotkey, netuid) {
        (Some(h), n) => client
            .storage_with_keys(
                "SubtensorModule",
                "TotalHotkeyAlpha",
                vec![
                    Value::from_bytes(<sp_core::crypto::AccountId32 as AsRef<[u8]>>::as_ref(h)),
                    Value::u128(n as u128),
                ],
            )
            .await?
            .and_then(|v| bittensor_rs::utils::decoders::decode_u128(&v).ok())
            .unwrap_or(0),
        _ => 0,
    };

    let emis_vec = client
        .storage_with_keys(
            "SubtensorModule",
            "Emission",
            vec![Value::u128(netuid as u128)],
        )
        .await?
        .and_then(|v| bittensor_rs::utils::decoders::decode_vec_u128(&v).ok());
    let emission_uid = emis_vec.as_ref().and_then(|v| v.get(uid as usize).copied());

    println!(
        "uid={} hotkey={:?} coldkey={:?} total_stake_rao={} emission_uid_rao={:?}",
        uid,
        hotkey
            .as_ref()
            .map(|a| hex::encode(<sp_core::crypto::AccountId32 as AsRef<[u8]>>::as_ref(a))),
        coldkey
            .as_ref()
            .map(|a| hex::encode(<sp_core::crypto::AccountId32 as AsRef<[u8]>>::as_ref(a))),
        total_stake,
        emission_uid
    );
    Ok(())
}
