use crate::chain::{BittensorClient, BittensorSigner, ExtrinsicWait};
use crate::utils::{
    commit_hash_to_hex, generate_salt, generate_subtensor_commit_hash, normalize_weights,
    salt_u8_to_u16,
};
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
#[allow(clippy::too_many_arguments)]
pub async fn reveal_weights(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    uids: &[u64],
    weights: &[u16],
    salt: &[u16], // Changed from &[u8] to &[u16] to match Subtensor format
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

    let uid_values: Vec<Value> = uid_u16
        .iter()
        .map(|uid| Value::u128(*uid as u128))
        .collect();
    let weight_values: Vec<Value> = weights.iter().map(|w| Value::u128(*w as u128)).collect();
    let salt_values: Vec<Value> = salt.iter().map(|s| Value::u128(*s as u128)).collect();

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

/// Generate commit hash from weights for commit-reveal pattern (LEGACY)
///
/// NOTE: This function uses a legacy hash format. For proper subtensor compatibility,
/// use `generate_commit_hash_v2` which matches subtensor's exact hash format.
pub fn generate_commit_hash(uids: &[u64], weights: &[u16], salt: &[u8]) -> Result<String> {
    // Convert to u16 format
    let uid_u16: Vec<u16> = uids.iter().map(|u| *u as u16).collect();
    let salt_u16 = salt_u8_to_u16(salt);

    // Use a dummy account for legacy compatibility - this won't match subtensor!
    let dummy_account = [0u8; 32];
    let hash = generate_subtensor_commit_hash(
        &dummy_account,
        0, // netuid unknown in legacy call
        None,
        &uid_u16,
        weights,
        &salt_u16,
        0, // version_key unknown in legacy call
    );
    Ok(hex::encode(hash))
}

/// Generate commit hash matching subtensor's exact format.
///
/// This is the correct function to use for commit-reveal with subtensor.
///
/// # Arguments
/// * `account` - The hotkey's public key (32 bytes)
/// * `netuid` - The subnet ID
/// * `uids` - Neuron UIDs (will be converted to u16)
/// * `weights` - Weight values (u16, 0-65535 scale)
/// * `salt` - Random salt (Vec<u16>)
/// * `version_key` - Network version key
///
/// # Returns
/// Hex-encoded 32-byte Blake2b-256 hash
pub fn generate_commit_hash_v2(
    account: &[u8; 32],
    netuid: u16,
    uids: &[u16],
    weights: &[u16],
    salt: &[u16],
    version_key: u64,
) -> String {
    let hash = generate_subtensor_commit_hash(
        account,
        netuid,
        None, // main mechanism
        uids,
        weights,
        salt,
        version_key,
    );
    commit_hash_to_hex(&hash)
}

/// Generate commit hash for mechanism weights.
///
/// Same as `generate_commit_hash_v2` but for sub-subnet mechanisms.
pub fn generate_mechanism_commit_hash_v2(
    account: &[u8; 32],
    netuid: u16,
    mechanism_id: u8,
    uids: &[u16],
    weights: &[u16],
    salt: &[u16],
    version_key: u64,
) -> String {
    let hash = generate_subtensor_commit_hash(
        account,
        netuid,
        Some(mechanism_id),
        uids,
        weights,
        salt,
        version_key,
    );
    commit_hash_to_hex(&hash)
}

/// Helper struct for commit-reveal data
#[derive(Clone, Debug)]
pub struct CommitRevealData {
    pub commit_hash: String,
    pub uids: Vec<u16>,
    pub weights: Vec<u16>,
    pub salt: Vec<u16>,
    pub version_key: u64,
}

/// Prepare commit-reveal data for weight submission.
///
/// Generates the hash and returns all data needed for commit and later reveal.
pub fn prepare_commit_reveal(
    account: &[u8; 32],
    netuid: u16,
    uids: &[u16],
    weights: &[u16],
    version_key: u64,
    salt_len: usize,
) -> CommitRevealData {
    let salt = generate_salt(salt_len);
    let commit_hash = generate_commit_hash_v2(account, netuid, uids, weights, &salt, version_key);

    CommitRevealData {
        commit_hash,
        uids: uids.to_vec(),
        weights: weights.to_vec(),
        salt,
        version_key,
    }
}

/// Prepare commit-reveal data for mechanism weight submission.
pub fn prepare_mechanism_commit_reveal(
    account: &[u8; 32],
    netuid: u16,
    mechanism_id: u8,
    uids: &[u16],
    weights: &[u16],
    version_key: u64,
    salt_len: usize,
) -> CommitRevealData {
    let salt = generate_salt(salt_len);
    let commit_hash = generate_mechanism_commit_hash_v2(
        account,
        netuid,
        mechanism_id,
        uids,
        weights,
        &salt,
        version_key,
    );

    CommitRevealData {
        commit_hash,
        uids: uids.to_vec(),
        weights: weights.to_vec(),
        salt,
        version_key,
    }
}
