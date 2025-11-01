use crate::chain::{BittensorClient, BittensorSigner, ExtrinsicWait};
use anyhow::Result;
use subxt::dynamic::Value;

const SUBTENSOR_MODULE: &str = "SubtensorModule";

/// Commit mechanism weights
pub async fn commit_mechanism_weights(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    mechanism_id: u64,
    commit_hash: &str,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let hash_bytes =
        hex::decode(commit_hash).map_err(|e| anyhow::anyhow!("Invalid commit hash: {}", e))?;

    let args = vec![
        Value::u128(netuid as u128),
        Value::u128(mechanism_id as u128),
        Value::from_bytes(&hash_bytes),
    ];

    client
        .submit_extrinsic(
            SUBTENSOR_MODULE,
            "commit_mechanism_weights",
            args,
            signer,
            wait_for,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to commit mechanism weights: {}", e))
}

/// Reveal mechanism weights
pub async fn reveal_mechanism_weights(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    mechanism_id: u64,
    uids: &[u64],
    weights: &[u64],
    salt: &[u8],
    version_key: u64,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let uid_values: Vec<Value> = uids.iter().map(|uid| Value::u128(*uid as u128)).collect();
    let weight_values: Vec<Value> = weights.iter().map(|w| Value::u128(*w as u128)).collect();
    let salt_values: Vec<Value> = salt
        .iter()
        .map(|s| Value::u128((*s as u64) as u128))
        .collect();

    let args = vec![
        Value::u128(netuid as u128),
        Value::u128(mechanism_id as u128),
        Value::unnamed_composite(uid_values),
        Value::unnamed_composite(weight_values),
        Value::unnamed_composite(salt_values),
        Value::u128(version_key as u128),
    ];

    client
        .submit_extrinsic(
            SUBTENSOR_MODULE,
            "reveal_mechanism_weights",
            args,
            signer,
            wait_for,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to reveal mechanism weights: {}", e))
}

/// Set mechanism weights directly
pub async fn set_mechanism_weights(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    mechanism_id: u64,
    uids: &[u64],
    weights: &[f32],
    version_key: Option<u64>,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    // Normalize weights first
    let (weight_uids, weight_vals) = crate::utils::normalize_weights(uids, weights)?;

    let uid_values: Vec<Value> = weight_uids
        .iter()
        .map(|uid| Value::u128(*uid as u128))
        .collect();
    let weight_values: Vec<Value> = weight_vals
        .iter()
        .map(|w| Value::u128(*w as u128))
        .collect();

    let version = version_key
        .ok_or_else(|| anyhow::anyhow!("Version key is required for commit_mechanism_weights"))?;

    let args = vec![
        Value::u128(netuid as u128),
        Value::u128(mechanism_id as u128),
        Value::unnamed_composite(uid_values),
        Value::unnamed_composite(weight_values),
        Value::u128(version as u128),
    ];

    client
        .submit_extrinsic(
            SUBTENSOR_MODULE,
            "set_mechanism_weights",
            args,
            signer,
            wait_for,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to set mechanism weights: {}", e))
}
