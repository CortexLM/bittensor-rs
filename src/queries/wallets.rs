use crate::chain::BittensorClient;
use anyhow::Result;
use parity_scale_codec::Encode;
use sp_core::crypto::AccountId32;
use subxt::dynamic::Value;

const SUBTENSOR_MODULE: &str = "SubtensorModule";

/// Returns true if the hotkey is known by the chain (Owner[hotkey] not zero)
pub async fn does_hotkey_exist(client: &BittensorClient, hotkey: &AccountId32) -> Result<bool> {
    if let Some(owner_val) = client
        .storage_with_keys(
            SUBTENSOR_MODULE,
            "Owner",
            vec![Value::from_bytes(&hotkey.encode())],
        )
        .await?
    {
        // Check if decoded account is not all-zero
        let s = format!("{:?}", owner_val);
        if let Some(pos) = s.find("0x") {
            let hex = &s[pos + 2..]
                .chars()
                .take_while(|c| c.is_ascii_hexdigit())
                .collect::<String>();
            if hex.len() >= 64 {
                let bytes = hex::decode(&hex[..64]).unwrap_or_default();
                let is_zero = bytes.iter().all(|&b| b == 0);
                return Ok(!is_zero);
            }
        }
    }
    Ok(false)
}

/// Check if hotkey is registered on a given subnet (Uids[(netuid, hotkey)] exists)
pub async fn is_hotkey_registered(
    client: &BittensorClient,
    hotkey: &AccountId32,
    netuid: u16,
) -> Result<bool> {
    let val = client
        .storage_with_keys(
            SUBTENSOR_MODULE,
            "Uids",
            vec![
                Value::u128(netuid as u128),
                Value::from_bytes(&hotkey.encode()),
            ],
        )
        .await?;
    Ok(val.is_some())
}

/// Check if hotkey is registered on any subnet
pub async fn is_hotkey_registered_any(
    client: &BittensorClient,
    hotkey: &AccountId32,
) -> Result<bool> {
    let netuids = get_netuids_for_hotkey(client, hotkey).await?;
    Ok(!netuids.is_empty())
}

/// Get all netuids where the hotkey is registered
pub async fn get_netuids_for_hotkey(
    client: &BittensorClient,
    hotkey: &AccountId32,
) -> Result<Vec<u16>> {
    let total_val = client
        .storage(SUBTENSOR_MODULE, "TotalNetworks", None)
        .await?;
    let total: u16 = total_val
        .and_then(|v| crate::utils::value_decode::decode_u64(&v).ok())
        .and_then(|n| u16::try_from(n).ok())
        .unwrap_or(0);
    let mut nets = Vec::new();
    for netuid in 0u16..total {
        if is_hotkey_registered(client, hotkey, netuid).await? {
            nets.push(netuid);
        }
    }
    Ok(nets)
}

/// Get hotkeys owned by a coldkey (OwnedHotkeys[coldkey])
pub async fn get_owned_hotkeys(
    client: &BittensorClient,
    coldkey: &AccountId32,
) -> Result<Vec<AccountId32>> {
    if let Some(val) = client
        .storage_with_keys(
            SUBTENSOR_MODULE,
            "OwnedHotkeys",
            vec![Value::from_bytes(&coldkey.encode())],
        )
        .await?
    {
        return Ok(crate::utils::value_decode::decode_vec_account_id32(&val).unwrap_or_default());
    }
    Ok(Vec::new())
}

/// Get owner (coldkey) of hotkey (Owner[hotkey])
pub async fn get_hotkey_owner(
    client: &BittensorClient,
    hotkey: &AccountId32,
) -> Result<Option<AccountId32>> {
    if let Some(val) = client
        .storage_with_keys(
            SUBTENSOR_MODULE,
            "Owner",
            vec![Value::from_bytes(&hotkey.encode())],
        )
        .await?
    {
        return Ok(crate::utils::value_decode::decode_account_id32(&val).ok());
    }
    Ok(None)
}

/// Filter subnets by hotkeys that are registered on them
pub async fn filter_netuids_by_registered_hotkeys(
    client: &BittensorClient,
    hotkeys: &[AccountId32],
) -> Result<Vec<u16>> {
    let total_val = client
        .storage(SUBTENSOR_MODULE, "TotalNetworks", None)
        .await?;
    let total: u16 = total_val
        .and_then(|v| crate::utils::value_decode::decode_u64(&v).ok())
        .and_then(|n| u16::try_from(n).ok())
        .unwrap_or(0);
    let mut nets = Vec::new();
    'outer: for netuid in 0u16..total {
        for hk in hotkeys.iter() {
            if is_hotkey_registered(client, hk, netuid).await? {
                nets.push(netuid);
                continue 'outer;
            }
        }
    }
    Ok(nets)
}

/// Estimate transfer fee by reading fee-related storage (FeeRate). Returns raw fee rate (u128)
pub async fn get_transfer_fee(client: &BittensorClient) -> Result<u128> {
    if let Some(val) = client.storage(SUBTENSOR_MODULE, "FeeRate", None).await? {
        return crate::utils::value_decode::decode_u128(&val).map_err(|e| anyhow::anyhow!("{}", e));
    }
    Ok(0)
}
