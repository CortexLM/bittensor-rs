use crate::chain::BittensorClient;
use crate::types::delegate::DelegateInfoBase;
use crate::types::{DelegateInfo, DelegatedInfo};
use crate::utils::balance_newtypes::Rao;
use crate::utils::decoders::{decode_account_id32, decode_u16};
use anyhow::Result;
use parity_scale_codec::{Compact, Decode, Encode};
use sp_core::crypto::AccountId32;
use std::collections::{HashMap, HashSet};
use subxt::dynamic::Value;

const SUBTENSOR_MODULE: &str = "SubtensorModule";

// Type alias to simplify complex nominator stakes type
type NominatorStakes = Vec<(Compact<u16>, Compact<u64>)>;

/// DelegateInfo structure matching the on-chain SCALE encoding
/// This is the exact structure used in subtensor runtime
#[derive(Decode, Clone, Debug)]
struct DelegateInfoRaw {
    delegate_ss58: AccountId32,
    take: Compact<u16>,
    nominators: Vec<(AccountId32, NominatorStakes)>,
    owner_ss58: AccountId32,
    registrations: Vec<Compact<u16>>,
    validator_permits: Vec<Compact<u16>>,
    return_per_1000: Compact<u64>,
    total_daily_return: Compact<u64>,
}

/// Get delegate by hotkey - built from storage
pub async fn get_delegate_by_hotkey(
    client: &BittensorClient,
    hotkey: &AccountId32,
) -> Result<Option<DelegateInfo>> {
    // Owner[hotkey] -> coldkey
    let owner_val = client
        .storage_with_keys(
            SUBTENSOR_MODULE,
            "Owner",
            vec![Value::from_bytes(hotkey.encode())],
        )
        .await?;
    let owner = match owner_val {
        Some(v) => decode_account_id32(&v).ok(),
        None => None,
    };
    let Some(owner_ss58) = owner else {
        return Ok(None);
    };

    // Take from Delegates[hotkey] (u16 normalized)
    let take_val = client
        .storage_with_keys(
            SUBTENSOR_MODULE,
            "Delegates",
            vec![Value::from_bytes(hotkey.encode())],
        )
        .await?;
    let take = match take_val {
        Some(v) => decode_u16(&v)
            .map(|x| x as f64 / u16::MAX as f64)
            .unwrap_or(0.0),
        None => 0.0,
    };

    // Total stake per subnet from TotalHotkeyAlpha[(hotkey, netuid)] and nominators by Stake[(netuid, uid)]
    // Get total networks
    let total_networks_val = client
        .storage(SUBTENSOR_MODULE, "TotalNetworks", None)
        .await?;
    let total_networks: u16 = total_networks_val
        .and_then(|v| crate::utils::decoders::decode_u64(&v).ok())
        .and_then(|n| u16::try_from(n).ok())
        .unwrap_or(0);

    let mut total_stake: HashMap<u16, Rao> = HashMap::new();
    let mut nominators: HashMap<AccountId32, HashMap<u16, u128>> = HashMap::new();
    let mut validator_permits: Vec<u16> = Vec::new();
    let mut registrations: Vec<u16> = Vec::new();

    for netuid in 0u16..total_networks {
        // UID for this hotkey on subnet
        let uid_val = client
            .storage_with_keys(
                SUBTENSOR_MODULE,
                "Uids",
                vec![
                    Value::u128(netuid as u128),
                    Value::from_bytes(hotkey.encode()),
                ],
            )
            .await?;

        if let Some(uid_val) = uid_val {
            if let Ok(uid) = crate::utils::decoders::decode_u64(&uid_val) {
                registrations.push(netuid);

                // ValidatorPermit[(netuid, uid)] -> bool
                if let Some(vp_val) = client
                    .storage_with_keys(
                        SUBTENSOR_MODULE,
                        "ValidatorPermit",
                        vec![Value::u128(netuid as u128), Value::u128(uid as u128)],
                    )
                    .await?
                {
                    if crate::utils::decoders::decode_bool(&vp_val).unwrap_or(false) {
                        validator_permits.push(netuid);
                    }
                }

                // Nominators/stake dict: Stake[(netuid, uid)] -> Vec<(AccountId32, u64)>
                if let Some(stake_dict_val) = client
                    .storage_with_keys(
                        SUBTENSOR_MODULE,
                        "Stake",
                        vec![Value::u128(netuid as u128), Value::u128(uid as u128)],
                    )
                    .await?
                {
                    let entries =
                        crate::utils::decoders::decode_vec_account_u128_pairs(&stake_dict_val)
                            .unwrap_or_default();
                    for (ck, amt) in entries {
                        let e = nominators.entry(ck).or_default();
                        e.insert(netuid, amt);
                    }
                }

                // Total stake for this hotkey/subnet
                if let Some(alpha_val) = client
                    .storage_with_keys(
                        SUBTENSOR_MODULE,
                        "TotalHotkeyAlpha",
                        vec![
                            Value::from_bytes(hotkey.encode()),
                            Value::u128(netuid as u128),
                        ],
                    )
                    .await?
                {
                    if let Ok(ts) = crate::utils::decoders::decode_u128(&alpha_val) {
                        total_stake.insert(netuid, Rao::from(ts));
                    }
                }
            }
        }
    }

    let delegate = DelegateInfo {
        base: DelegateInfoBase {
            hotkey_ss58: hotkey.clone(),
            owner_ss58,
            take,
            validator_permits,
            registrations,
            // These fields are not stored on-chain explicitly; set to 0 if not derivable
            return_per_1000: Rao::ZERO,
            total_daily_return: Rao::ZERO,
        },
        total_stake,
        nominators,
    };

    Ok(Some(delegate))
}

/// Get all delegate identities (delegate hotkeys) by scanning subnets
pub async fn get_delegate_identities(client: &BittensorClient) -> Result<Vec<AccountId32>> {
    // Collect unique hotkeys across all subnets, then filter those with a non-zero delegate take
    let total_networks_val = client
        .storage(SUBTENSOR_MODULE, "TotalNetworks", None)
        .await?;
    let total_networks: u16 = total_networks_val
        .and_then(|v| crate::utils::decoders::decode_u64(&v).ok())
        .and_then(|n| u16::try_from(n).ok())
        .unwrap_or(0);

    let mut hotkeys: HashSet<AccountId32> = HashSet::new();
    for netuid in 0u16..total_networks {
        let n_val = client
            .storage_with_keys(
                SUBTENSOR_MODULE,
                "SubnetworkN",
                vec![Value::u128(netuid as u128)],
            )
            .await?;
        let n: u64 = n_val
            .and_then(|v| crate::utils::decoders::decode_u64(&v).ok())
            .unwrap_or(0);
        for uid in 0..n {
            if let Some(hk_val) = client
                .storage_with_keys(
                    SUBTENSOR_MODULE,
                    "Keys",
                    vec![Value::u128(netuid as u128), Value::u128(uid as u128)],
                )
                .await?
            {
                if let Ok(hk) = decode_account_id32(&hk_val) {
                    hotkeys.insert(hk);
                }
            }
        }
    }

    // Filter by Delegates[hotkey] > 0
    let mut delegates = Vec::new();
    for hk in hotkeys.into_iter() {
        if get_delegate_take(client, &hk).await.unwrap_or(0.0) > 0.0 {
            delegates.push(hk);
        }
    }
    Ok(delegates)
}

/// Get delegated stake information (per coldkey)
pub async fn get_delegated(
    client: &BittensorClient,
    coldkey: &AccountId32,
) -> Result<Vec<DelegatedInfo>> {
    // For each delegate hotkey, check if this coldkey appears in their nominators and collect per-netuid stakes
    let hotkeys = get_delegate_identities(client).await?;
    let mut out: Vec<DelegatedInfo> = Vec::new();
    for hotkey in hotkeys.iter() {
        if let Some(delegate) = get_delegate_by_hotkey(client, hotkey).await? {
            if let Some(nets) = delegate.nominators.get(coldkey) {
                let stake_sum: u128 = nets.values().copied().sum();
                out.push(DelegatedInfo {
                    base: delegate.base.clone(),
                    netuid: 0,
                    stake: Rao::from(stake_sum),
                });
            }
        }
    }
    Ok(out)
}

/// Get all delegates using runtime API (single RPC call like Python SDK)
/// Optimized version using direct SCALE decoding for maximum performance
pub async fn get_delegates(client: &BittensorClient) -> Result<Vec<DelegateInfo>> {
    // Use runtime_api_call which returns raw bytes for direct SCALE decoding
    // This is much faster than going through Value parsing
    let raw_bytes = client
        .runtime_api_call("DelegateInfoRuntimeApi", "get_delegates", None)
        .await?;

    if raw_bytes.is_empty() {
        return Ok(Vec::new());
    }

    // Decode Vec<DelegateInfoRaw> directly from SCALE bytes
    let raw_delegates: Vec<DelegateInfoRaw> =
        Vec::<DelegateInfoRaw>::decode(&mut &raw_bytes[..])
            .map_err(|e| anyhow::anyhow!("Failed to decode delegates from runtime API: {}", e))?;

    // Convert to our DelegateInfo type
    let delegates: Vec<DelegateInfo> = raw_delegates
        .into_iter()
        .map(|raw| {
            // Convert nominators: Vec<(AccountId32, Vec<(netuid, stake)>)> to HashMap
            let mut nominators: HashMap<AccountId32, HashMap<u16, u128>> = HashMap::new();
            let mut total_stake: HashMap<u16, Rao> = HashMap::new();

            for (nominator, stakes) in raw.nominators {
                let mut stake_map: HashMap<u16, u128> = HashMap::new();
                for (netuid, stake) in stakes {
                    let netuid_val = netuid.0;
                    let stake_val = stake.0 as u128;
                    stake_map.insert(netuid_val, stake_val);

                    let entry = total_stake.entry(netuid_val).or_insert(Rao::ZERO);
                    *entry = entry.saturating_add(Rao::from(stake_val));
                }
                nominators.insert(nominator, stake_map);
            }

            DelegateInfo {
                base: DelegateInfoBase {
                    hotkey_ss58: raw.delegate_ss58,
                    owner_ss58: raw.owner_ss58,
                    take: raw.take.0 as f64 / u16::MAX as f64,
                    validator_permits: raw.validator_permits.iter().map(|c| c.0).collect(),
                    registrations: raw.registrations.iter().map(|c| c.0).collect(),
                    return_per_1000: Rao::from(raw.return_per_1000.0 as u128),
                    total_daily_return: Rao::from(raw.total_daily_return.0 as u128),
                },
                total_stake,
                nominators,
            }
        })
        .collect();

    Ok(delegates)
}

/// Get all delegates by building from storage (fallback method, slower but complete)
pub async fn get_delegates_from_storage(client: &BittensorClient) -> Result<Vec<DelegateInfo>> {
    let ids = get_delegate_identities(client).await?;
    let mut delegates = Vec::new();
    for hk in ids.iter() {
        if let Some(d) = get_delegate_by_hotkey(client, hk).await? {
            delegates.push(d);
        }
    }
    Ok(delegates)
}

/// Get delegate take (commission)
pub async fn get_delegate_take(client: &BittensorClient, hotkey: &AccountId32) -> Result<f64> {
    let keys = vec![Value::from_bytes(hotkey.encode())];

    if let Some(take_val) = client
        .storage_with_keys(SUBTENSOR_MODULE, "Delegates", keys)
        .await?
    {
        // Decode u16 and normalize to f64
        if let Ok(take) = decode_u16(&take_val) {
            return Ok(take as f64 / u16::MAX as f64);
        }
        return Ok(0.0);
    }

    Ok(0.0)
}

/// Get delegate take as raw u16 value (0-65535)
/// Direct storage read without normalization
pub async fn get_delegate_take_raw(client: &BittensorClient, hotkey: &AccountId32) -> Result<u16> {
    let keys = vec![Value::from_bytes(hotkey.encode())];

    if let Some(take_val) = client
        .storage_with_keys(SUBTENSOR_MODULE, "Delegates", keys)
        .await?
    {
        if let Ok(take) = decode_u16(&take_val) {
            return Ok(take);
        }
    }
    Ok(0)
}

/// Get total hotkey stake across all subnets
/// Direct storage read from SubtensorModule::TotalHotkeyStake
pub async fn get_total_hotkey_stake(
    client: &BittensorClient,
    hotkey: &AccountId32,
) -> Result<u128> {
    let keys = vec![Value::from_bytes(hotkey.encode())];

    if let Some(val) = client
        .storage_with_keys(SUBTENSOR_MODULE, "TotalHotkeyStake", keys)
        .await?
    {
        if let Ok(stake) = crate::utils::decoders::decode_u128(&val) {
            return Ok(stake);
        }
    }
    Ok(0)
}

/// Optimized: Get delegate info using direct storage reads (O(D) instead of O(N*M))
/// Reads Delegates map for take, TotalHotkeyStake for total stake
pub async fn get_delegate_info_optimized(
    client: &BittensorClient,
    hotkey: &AccountId32,
) -> Result<Option<DelegateInfo>> {
    let take_raw = get_delegate_take_raw(client, hotkey).await?;
    if take_raw == 0 {
        return Ok(None);
    }

    let owner_val = client
        .storage_with_keys(
            SUBTENSOR_MODULE,
            "Owner",
            vec![Value::from_bytes(hotkey.encode())],
        )
        .await?;
    let owner = match owner_val {
        Some(v) => decode_account_id32(&v).ok(),
        None => None,
    };
    let Some(owner_ss58) = owner else {
        return Ok(None);
    };

    let total_stake_all = get_total_hotkey_stake(client, hotkey).await.unwrap_or(0);

    let total_networks_val = client
        .storage(SUBTENSOR_MODULE, "TotalNetworks", None)
        .await?;
    let total_networks: u16 = total_networks_val
        .and_then(|v| crate::utils::decoders::decode_u64(&v).ok())
        .and_then(|n| u16::try_from(n).ok())
        .unwrap_or(0);

    let mut registrations: Vec<u16> = Vec::new();
    let validator_permits: Vec<u16> = Vec::new();
    let mut total_stake: HashMap<u16, Rao> = HashMap::new();

    for netuid in 0u16..total_networks {
        let uid_val = client
            .storage_with_keys(
                SUBTENSOR_MODULE,
                "Uids",
                vec![
                    Value::u128(netuid as u128),
                    Value::from_bytes(hotkey.encode()),
                ],
            )
            .await?;

        if let Some(uid_val) = uid_val {
            if let Ok(_uid) = crate::utils::decoders::decode_u64(&uid_val) {
                registrations.push(netuid);

                if let Some(alpha_val) = client
                    .storage_with_keys(
                        SUBTENSOR_MODULE,
                        "TotalHotkeyAlpha",
                        vec![
                            Value::from_bytes(hotkey.encode()),
                            Value::u128(netuid as u128),
                        ],
                    )
                    .await?
                {
                    if let Ok(ts) = crate::utils::decoders::decode_u128(&alpha_val) {
                        total_stake.insert(netuid, Rao::from(ts));
                    }
                }
            }
        }
    }

    let _ = total_stake_all;

    let delegate = DelegateInfo {
        base: DelegateInfoBase {
            hotkey_ss58: hotkey.clone(),
            owner_ss58,
            take: take_raw as f64 / u16::MAX as f64,
            validator_permits,
            registrations,
            return_per_1000: Rao::ZERO,
            total_daily_return: Rao::ZERO,
        },
        total_stake,
        nominators: HashMap::new(),
    };

    Ok(Some(delegate))
}

/// Check if hotkey is a delegate
pub async fn is_hotkey_delegate(client: &BittensorClient, hotkey: &AccountId32) -> Result<bool> {
    Ok(get_delegate_take(client, hotkey).await.unwrap_or(0.0) > 0.0)
}
