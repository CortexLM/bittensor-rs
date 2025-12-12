use crate::chain::{BittensorClient, BittensorSigner, ExtrinsicWait};
use anyhow::Result;
use subxt::dynamic::Value;

const SUBTENSOR_MODULE: &str = "SubtensorModule";

/// Commit mechanism weights
/// Subtensor expects: (netuid, mecid: u8, commit_hash: H256)
pub async fn commit_mechanism_weights(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    mechanism_id: u8, // Changed from u64 to u8 (MechId)
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
/// Subtensor expects: (netuid, mecid: u8, uids: Vec<u16>, values: Vec<u16>, salt: Vec<u16>, version_key: u64)
#[allow(clippy::too_many_arguments)]
pub async fn reveal_mechanism_weights(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    mechanism_id: u8, // Changed from u64 to u8 (MechId)
    uids: &[u64],
    weights: &[u16], // Changed from u64 to u16 to match Subtensor format
    salt: &[u8],
    version_key: u64,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    // Convert uids from u64 to u16 (Subtensor expects Vec<u16>)
    let uid_u16: Vec<u16> = uids.iter().map(|uid| *uid as u16).collect();
    // Convert salt from u8 to u16 (Subtensor expects Vec<u16>)
    let salt_u16: Vec<u16> = salt.iter().map(|b| *b as u16).collect();

    let uid_values: Vec<Value> = uid_u16
        .iter()
        .map(|uid| Value::u128(*uid as u128))
        .collect();
    let weight_values: Vec<Value> = weights.iter().map(|w| Value::u128(*w as u128)).collect();
    let salt_values: Vec<Value> = salt_u16.iter().map(|s| Value::u128(*s as u128)).collect();

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
/// Subtensor expects: (netuid, mecid: u8, dests: Vec<u16>, weights: Vec<u16>, version_key: u64)
#[allow(clippy::too_many_arguments)]
pub async fn set_mechanism_weights(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    mechanism_id: u8, // Changed from u64 to u8 (MechId)
    uids: &[u64],
    weights: &[f32],
    version_key: Option<u64>,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    // Normalize weights first (returns Vec<u16>)
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
        .ok_or_else(|| anyhow::anyhow!("Version key is required for set_mechanism_weights"))?;

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
