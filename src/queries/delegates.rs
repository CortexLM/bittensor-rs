use crate::chain::BittensorClient;
use crate::types::delegate::DelegateInfoBase;
use crate::types::{DelegateInfo, DelegatedInfo};
use crate::utils::value_decode::{decode_account_id32, decode_u16};
use anyhow::Result;
use parity_scale_codec::Encode;
use sp_core::crypto::AccountId32;
use std::collections::{HashMap, HashSet};
use subxt::dynamic::Value;

const SUBTENSOR_MODULE: &str = "SubtensorModule";

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
            vec![Value::from_bytes(&hotkey.encode())],
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
            vec![Value::from_bytes(&hotkey.encode())],
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
        .and_then(|v| crate::utils::value_decode::decode_u64(&v).ok())
        .and_then(|n| u16::try_from(n).ok())
        .unwrap_or(0);

    let mut total_stake: HashMap<u16, u128> = HashMap::new();
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
                    Value::from_bytes(&hotkey.encode()),
                ],
            )
            .await?;

        if let Some(uid_val) = uid_val {
            if let Ok(uid) = crate::utils::value_decode::decode_u64(&uid_val) {
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
                    if crate::utils::value_decode::decode_bool(&vp_val).unwrap_or(false) {
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
                        crate::utils::value_decode::decode_vec_account_u128_pairs(&stake_dict_val)
                            .unwrap_or_default();
                    for (ck, amt) in entries {
                        let e = nominators.entry(ck).or_insert_with(HashMap::new);
                        e.insert(netuid, amt);
                    }
                }

                // Total stake for this hotkey/subnet
                if let Some(alpha_val) = client
                    .storage_with_keys(
                        SUBTENSOR_MODULE,
                        "TotalHotkeyAlpha",
                        vec![
                            Value::from_bytes(&hotkey.encode()),
                            Value::u128(netuid as u128),
                        ],
                    )
                    .await?
                {
                    if let Ok(ts) = crate::utils::value_decode::decode_u128(&alpha_val) {
                        total_stake.insert(netuid, ts);
                    }
                }
            }
        }
    }

    let delegate = DelegateInfo {
        base: DelegateInfoBase {
            hotkey_ss58: hotkey.clone(),
            owner_ss58: owner_ss58,
            take,
            validator_permits,
            registrations,
            // These fields are not stored on-chain explicitly; set to 0 if not derivable
            return_per_1000: 0,
            total_daily_return: 0,
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
        .and_then(|v| crate::utils::value_decode::decode_u64(&v).ok())
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
            .and_then(|v| crate::utils::value_decode::decode_u64(&v).ok())
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
                let stake_sum: u128 = nets.values().cloned().sum();
                out.push(DelegatedInfo {
                    base: delegate.base.clone(),
                    netuid: 0,
                    stake: stake_sum,
                });
            }
        }
    }
    Ok(out)
}

/// Get all delegates by building from storage
pub async fn get_delegates(client: &BittensorClient) -> Result<Vec<DelegateInfo>> {
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
    let keys = vec![Value::from_bytes(&hotkey.encode())];

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

/// Check if hotkey is a delegate
pub async fn is_hotkey_delegate(client: &BittensorClient, hotkey: &AccountId32) -> Result<bool> {
    Ok(get_delegate_take(client, hotkey).await.unwrap_or(0.0) > 0.0)
}
