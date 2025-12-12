use crate::chain::{BittensorClient, BittensorSigner, ExtrinsicWait};
use crate::utils::decoders::{decode_vec_account_id32, decode_vec_tuple_u64_account};
use anyhow::Result;
use parity_scale_codec::Encode;
use sp_core::crypto::AccountId32;
use subxt::dynamic::Value;

const SUBTENSOR_MODULE: &str = "SubtensorModule";

/// Set children hotkeys with proportions
pub async fn set_children(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    hotkey: &AccountId32,
    children: Vec<(u64, AccountId32)>, // (proportion, child_hotkey)
    wait_for: ExtrinsicWait,
) -> Result<String> {
    // Build children list as composite values
    let children_values: Vec<Value> = children
        .iter()
        .map(|(proportion, child_key)| {
            Value::unnamed_composite(vec![
                Value::u128(*proportion as u128),
                Value::from_bytes(&child_key.encode()),
            ])
        })
        .collect();

    let args = vec![
        Value::from_bytes(&hotkey.encode()),
        Value::u128(netuid as u128),
        Value::unnamed_composite(children_values),
    ];

    client
        .submit_extrinsic(SUBTENSOR_MODULE, "set_children", args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to set children: {}", e))
}

/// Get parent hotkeys for a neuron
pub async fn get_parents(
    client: &BittensorClient,
    netuid: u16,
    hotkey: &AccountId32,
) -> Result<Vec<AccountId32>> {
    let keys = vec![
        Value::u128(netuid as u128),
        Value::from_bytes(&hotkey.encode()),
    ];

    if let Some(parents_val) = client
        .storage_with_keys(SUBTENSOR_MODULE, "ParentKeys", keys)
        .await?
    {
        // Decode the list of parent AccountId32s
        return decode_vec_account_id32(&parents_val);
    }

    Ok(vec![])
}

/// Get children hotkeys for a neuron
pub async fn get_children(
    client: &BittensorClient,
    netuid: u16,
    hotkey: &AccountId32,
) -> Result<Vec<(AccountId32, u64)>> {
    let keys = vec![
        Value::u128(netuid as u128),
        Value::from_bytes(&hotkey.encode()),
    ];

    if let Some(children_val) = client
        .storage_with_keys(SUBTENSOR_MODULE, "ChildKeys", keys)
        .await?
    {
        // Decode the list of children with proportions
        let children_with_proportions = decode_vec_tuple_u64_account(&children_val)?;

        // Convert to the expected format: (AccountId32, proportion)
        let result: Vec<(AccountId32, u64)> = children_with_proportions
            .into_iter()
            .map(|(proportion, account)| (account, proportion))
            .collect();

        return Ok(result);
    }

    Ok(vec![])
}

/// Get pending children hotkeys
pub async fn get_children_pending(
    client: &BittensorClient,
    netuid: u16,
    hotkey: &AccountId32,
) -> Result<Vec<AccountId32>> {
    let keys = vec![
        Value::u128(netuid as u128),
        Value::from_bytes(&hotkey.encode()),
    ];

    if let Some(pending_val) = client
        .storage_with_keys(SUBTENSOR_MODULE, "ChildrenPending", keys)
        .await?
    {
        // Decode the list of pending AccountId32s
        return decode_vec_account_id32(&pending_val);
    }

    Ok(vec![])
}
