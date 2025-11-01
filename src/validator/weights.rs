use crate::chain::{BittensorClient, BittensorSigner, ExtrinsicWait};
use crate::utils::{commit_weights_hash, normalize_weights};
use anyhow::Result;
use subxt::dynamic::Value;

const SUBTENSOR_MODULE: &str = "SubtensorModule";
const SET_WEIGHTS_FUNCTION: &str = "set_weights";
const COMMIT_WEIGHTS_FUNCTION: &str = "commit_weights";
const REVEAL_WEIGHTS_FUNCTION: &str = "reveal_weights";

/// Set weights for specified UIDs on the subnet
pub async fn set_weights(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    uids: &[u64],
    weights: &[f32],
    version_key: Option<u64>,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    // Normalize weights
    let (weight_uids, weight_vals) = normalize_weights(uids, weights)?;

    if weight_uids.is_empty() {
        return Err(anyhow::anyhow!("No valid weights to set"));
    }

    let version =
        version_key.ok_or_else(|| anyhow::anyhow!("Version key is required for set_weights"))?;

    // Build call arguments - set_weights takes (netuid, dests: Vec<u16>, weights: Vec<u16>, version_key)
    // Subtensor expects Vec<u16> for both uids (dests) and weights
    let uid_values: Vec<Value> = weight_uids
        .iter()
        .map(|uid| Value::u128(*uid as u128))
        .collect();
    let weight_values: Vec<Value> = weight_vals
        .iter()
        .map(|w| Value::u128(*w as u128))
        .collect();

    let args = vec![
        Value::u128(netuid as u128),
        Value::unnamed_composite(uid_values),
        Value::unnamed_composite(weight_values),
        Value::u128(version as u128),
    ];

    let tx_hash = client
        .submit_extrinsic(
            SUBTENSOR_MODULE,
            SET_WEIGHTS_FUNCTION,
            args,
            signer,
            wait_for,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to set weights: {}", e))?;

    Ok(tx_hash)
}

/// Commit weights hash for reveal pattern
pub async fn commit_weights(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    commit_hash: &str,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    // Decode hex commit hash
    let hash_bytes = hex::decode(commit_hash)
        .map_err(|e| anyhow::anyhow!("Invalid commit hash format: {}", e))?;

    let args = vec![Value::u128(netuid as u128), Value::from_bytes(&hash_bytes)];

    let tx_hash = client
        .submit_extrinsic(
            SUBTENSOR_MODULE,
            COMMIT_WEIGHTS_FUNCTION,
            args,
            signer,
            wait_for,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to commit weights: {}", e))?;

    Ok(tx_hash)
}

/// Reveal committed weights
/// Subtensor expects: uids: Vec<u16>, values: Vec<u16>, salt: Vec<u16>
pub async fn reveal_weights(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    uids: &[u64],
    weights: &[u16],  // Changed from u64 to u16 to match Subtensor format
    salt: &[u8],
    version_key: u64,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    if uids.len() != weights.len() {
        return Err(anyhow::anyhow!(
            "UIDS and weights must have the same length"
        ));
    }

    // Convert uids from u64 to u16 (Subtensor expects Vec<u16>)
    let uid_u16: Vec<u16> = uids.iter().map(|uid| *uid as u16).collect();
    
    // Convert salt from u8 to u16 (Subtensor expects Vec<u16>)
    let salt_u16: Vec<u16> = salt.iter().map(|b| *b as u16).collect();

    let uid_values: Vec<Value> = uid_u16.iter().map(|uid| Value::u128(*uid as u128)).collect();
    let weight_values: Vec<Value> = weights.iter().map(|w| Value::u128(*w as u128)).collect();
    let salt_values: Vec<Value> = salt_u16.iter().map(|s| Value::u128(*s as u128)).collect();

    let args = vec![
        Value::u128(netuid as u128),
        Value::unnamed_composite(uid_values),
        Value::unnamed_composite(weight_values),
        Value::unnamed_composite(salt_values),
        Value::u128(version_key as u128),
    ];

    let tx_hash = client
        .submit_extrinsic(
            SUBTENSOR_MODULE,
            REVEAL_WEIGHTS_FUNCTION,
            args,
            signer,
            wait_for,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to reveal weights: {}", e))?;

    Ok(tx_hash)
}

/// Generate commit hash from weights for commit-reveal pattern
/// Uses u16 format for weights to match Subtensor's internal representation
pub fn generate_commit_hash(uids: &[u64], weights: &[u16], salt: &[u8]) -> Result<String> {
    let hash = commit_weights_hash(uids, weights, salt);
    Ok(hex::encode(hash))
}
