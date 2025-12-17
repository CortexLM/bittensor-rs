//! Neuron-related queries

use crate::error::Result;
use crate::queries::chain_info::{
    decode_bool, decode_u16, decode_u64, extract_account_bytes, query_storage_value,
};
use crate::utils::ss58::account_to_ss58;
use scale_value::{Primitive, ValueDef};
use sp_core::crypto::AccountId32;
use subxt::dynamic::Value;
use subxt::OnlineClient;
use subxt::PolkadotConfig;

const SUBTENSOR_MODULE: &str = "SubtensorModule";

/// Get UID for a hotkey on a subnet
pub async fn get_uid_for_hotkey(
    client: &OnlineClient<PolkadotConfig>,
    netuid: u16,
    hotkey: &AccountId32,
) -> Result<Option<u16>> {
    let hotkey_bytes: &[u8; 32] = hotkey.as_ref();
    let val = query_storage_value(
        client,
        SUBTENSOR_MODULE,
        "Uids",
        vec![
            Value::u128(netuid as u128),
            Value::from_bytes(hotkey_bytes),
        ],
    )
    .await?;
    Ok(val.and_then(|v| decode_u16(&v)))
}

/// Check if a hotkey is registered on a subnet
pub async fn is_hotkey_registered(
    client: &OnlineClient<PolkadotConfig>,
    netuid: u16,
    hotkey: &AccountId32,
) -> Result<bool> {
    Ok(get_uid_for_hotkey(client, netuid, hotkey).await?.is_some())
}

/// Get stake for a hotkey from a coldkey
pub async fn get_stake(
    client: &OnlineClient<PolkadotConfig>,
    hotkey: &AccountId32,
    coldkey: &AccountId32,
) -> Result<u64> {
    let hotkey_bytes: &[u8; 32] = hotkey.as_ref();
    let coldkey_bytes: &[u8; 32] = coldkey.as_ref();
    let val = query_storage_value(
        client,
        SUBTENSOR_MODULE,
        "Stake",
        vec![
            Value::from_bytes(hotkey_bytes),
            Value::from_bytes(coldkey_bytes),
        ],
    )
    .await?;
    Ok(val.and_then(|v| decode_u64(&v)).unwrap_or(0))
}

/// Get total stake for a hotkey
pub async fn get_total_stake_for_hotkey(
    client: &OnlineClient<PolkadotConfig>,
    hotkey: &AccountId32,
) -> Result<u64> {
    let hotkey_bytes: &[u8; 32] = hotkey.as_ref();
    let val = query_storage_value(
        client,
        SUBTENSOR_MODULE,
        "TotalHotkeyStake",
        vec![Value::from_bytes(hotkey_bytes)],
    )
    .await?;
    Ok(val.and_then(|v| decode_u64(&v)).unwrap_or(0))
}

/// Get last update block for a neuron
pub async fn get_last_update(
    client: &OnlineClient<PolkadotConfig>,
    netuid: u16,
    uid: u16,
) -> Result<u64> {
    let val = query_storage_value(
        client,
        SUBTENSOR_MODULE,
        "LastUpdate",
        vec![Value::u128(netuid as u128), Value::u128(uid as u128)],
    )
    .await?;
    Ok(val.and_then(|v| decode_u64(&v)).unwrap_or(0))
}

/// Get rank for a neuron
pub async fn get_rank(
    client: &OnlineClient<PolkadotConfig>,
    netuid: u16,
    uid: u16,
) -> Result<u16> {
    let val = query_storage_value(
        client,
        SUBTENSOR_MODULE,
        "Rank",
        vec![Value::u128(netuid as u128), Value::u128(uid as u128)],
    )
    .await?;
    Ok(val.and_then(|v| decode_u16(&v)).unwrap_or(0))
}

/// Get trust for a neuron
pub async fn get_trust(
    client: &OnlineClient<PolkadotConfig>,
    netuid: u16,
    uid: u16,
) -> Result<u16> {
    let val = query_storage_value(
        client,
        SUBTENSOR_MODULE,
        "Trust",
        vec![Value::u128(netuid as u128), Value::u128(uid as u128)],
    )
    .await?;
    Ok(val.and_then(|v| decode_u16(&v)).unwrap_or(0))
}

/// Get consensus for a neuron
pub async fn get_consensus(
    client: &OnlineClient<PolkadotConfig>,
    netuid: u16,
    uid: u16,
) -> Result<u16> {
    let val = query_storage_value(
        client,
        SUBTENSOR_MODULE,
        "Consensus",
        vec![Value::u128(netuid as u128), Value::u128(uid as u128)],
    )
    .await?;
    Ok(val.and_then(|v| decode_u16(&v)).unwrap_or(0))
}

/// Get incentive for a neuron
pub async fn get_incentive(
    client: &OnlineClient<PolkadotConfig>,
    netuid: u16,
    uid: u16,
) -> Result<u16> {
    let val = query_storage_value(
        client,
        SUBTENSOR_MODULE,
        "Incentive",
        vec![Value::u128(netuid as u128), Value::u128(uid as u128)],
    )
    .await?;
    Ok(val.and_then(|v| decode_u16(&v)).unwrap_or(0))
}

/// Get dividends for a neuron
pub async fn get_dividends(
    client: &OnlineClient<PolkadotConfig>,
    netuid: u16,
    uid: u16,
) -> Result<u16> {
    let val = query_storage_value(
        client,
        SUBTENSOR_MODULE,
        "Dividends",
        vec![Value::u128(netuid as u128), Value::u128(uid as u128)],
    )
    .await?;
    Ok(val.and_then(|v| decode_u16(&v)).unwrap_or(0))
}

/// Get emission for a neuron
pub async fn get_emission(
    client: &OnlineClient<PolkadotConfig>,
    netuid: u16,
    uid: u16,
) -> Result<u64> {
    let val = query_storage_value(
        client,
        SUBTENSOR_MODULE,
        "Emission",
        vec![Value::u128(netuid as u128), Value::u128(uid as u128)],
    )
    .await?;
    Ok(val.and_then(|v| decode_u64(&v)).unwrap_or(0))
}

/// Get validator trust for a neuron
pub async fn get_validator_trust(
    client: &OnlineClient<PolkadotConfig>,
    netuid: u16,
    uid: u16,
) -> Result<u16> {
    let val = query_storage_value(
        client,
        SUBTENSOR_MODULE,
        "ValidatorTrust",
        vec![Value::u128(netuid as u128), Value::u128(uid as u128)],
    )
    .await?;
    Ok(val.and_then(|v| decode_u16(&v)).unwrap_or(0))
}

/// Get validator permit for a neuron
pub async fn get_validator_permit(
    client: &OnlineClient<PolkadotConfig>,
    netuid: u16,
    uid: u16,
) -> Result<bool> {
    let val = query_storage_value(
        client,
        SUBTENSOR_MODULE,
        "ValidatorPermit",
        vec![Value::u128(netuid as u128), Value::u128(uid as u128)],
    )
    .await?;
    Ok(val.and_then(|v| decode_bool(&v)).unwrap_or(false))
}

/// Get active status for a neuron
pub async fn get_active(
    client: &OnlineClient<PolkadotConfig>,
    netuid: u16,
    uid: u16,
) -> Result<bool> {
    let val = query_storage_value(
        client,
        SUBTENSOR_MODULE,
        "Active",
        vec![Value::u128(netuid as u128), Value::u128(uid as u128)],
    )
    .await?;
    Ok(val.and_then(|v| decode_bool(&v)).unwrap_or(false))
}

/// Get pruning score for a neuron
pub async fn get_pruning_score(
    client: &OnlineClient<PolkadotConfig>,
    netuid: u16,
    uid: u16,
) -> Result<u16> {
    let val = query_storage_value(
        client,
        SUBTENSOR_MODULE,
        "PruningScores",
        vec![Value::u128(netuid as u128), Value::u128(uid as u128)],
    )
    .await?;
    Ok(val.and_then(|v| decode_u16(&v)).unwrap_or(0))
}

/// Get all hotkeys for a subnet
pub async fn get_all_hotkeys(
    client: &OnlineClient<PolkadotConfig>,
    netuid: u16,
) -> Result<Vec<(u16, String)>> {
    use subxt::dynamic::storage;

    let storage_query = storage(
        SUBTENSOR_MODULE,
        "Keys",
        vec![Value::u128(netuid as u128)],
    );
    let mut results = client
        .storage()
        .at_latest()
        .await?
        .iter(storage_query)
        .await?;

    let mut hotkeys = Vec::new();
    while let Some(Ok(kv)) = results.next().await {
        let key = &kv.key_bytes;
        // Extract UID from key
        if key.len() >= 2 {
            let uid_bytes = &key[key.len() - 2..];
            let uid = u16::from_le_bytes([uid_bytes[0], uid_bytes[1]]);
            
            // Decode hotkey from value
            if let Ok(val) = kv.value.to_value() {
                if let Some(bytes) = extract_account_bytes(&val) {
                    let account = AccountId32::from(bytes);
                    hotkeys.push((uid, account_to_ss58(&account)));
                } else if let ValueDef::Primitive(Primitive::U256(bytes)) = &val.value {
                    let account = AccountId32::from(*bytes);
                    hotkeys.push((uid, account_to_ss58(&account)));
                }
            }
        }
    }

    hotkeys.sort_by_key(|(uid, _)| *uid);
    Ok(hotkeys)
}
