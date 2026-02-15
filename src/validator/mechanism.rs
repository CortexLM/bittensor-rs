use crate::chain::{BittensorClient, BittensorSigner, ExtrinsicWait};
use crate::utils::{commit_hash_to_hex, generate_mechanism_commit_hash, generate_salt};
use anyhow::Result;
use subxt::dynamic::Value;

const SUBTENSOR_MODULE: &str = "SubtensorModule";

/// Helper struct for mechanism commit-reveal data
#[derive(Clone, Debug)]
pub struct MechanismCommitRevealData {
    pub commit_hash: String,
    pub mechanism_id: u8,
    pub uids: Vec<u16>,
    pub weights: Vec<u16>,
    pub salt: Vec<u16>,
    pub version_key: u64,
}

/// Prepare commit-reveal data for mechanism weights.
///
/// Generates hash matching subtensor's format and returns all data needed for commit/reveal.
pub fn prepare_mechanism_commit(
    account: &[u8; 32],
    netuid: u16,
    mechanism_id: u8,
    uids: &[u16],
    weights: &[u16],
    version_key: u64,
    salt_len: usize,
) -> MechanismCommitRevealData {
    let salt = generate_salt(salt_len);
    let hash = generate_mechanism_commit_hash(
        account,
        netuid,
        mechanism_id,
        uids,
        weights,
        &salt,
        version_key,
    );

    MechanismCommitRevealData {
        commit_hash: commit_hash_to_hex(&hash),
        mechanism_id,
        uids: uids.to_vec(),
        weights: weights.to_vec(),
        salt,
        version_key,
    }
}

/// Commit mechanism weights
/// Subtensor expects: (netuid, mecid: u8, commit_hash: H256)
pub async fn commit_mechanism_weights(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    mechanism_id: u8,
    commit_hash: &str,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let hash_bytes =
        hex::decode(commit_hash).map_err(|e| anyhow::anyhow!("Invalid commit hash: {}", e))?;

    if hash_bytes.len() != 32 {
        return Err(anyhow::anyhow!(
            "Commit hash must be 32 bytes, got {}",
            hash_bytes.len()
        ));
    }

    let args = vec![
        Value::from(netuid),
        Value::from(mechanism_id),
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
    mechanism_id: u8,
    uids: &[u16],
    weights: &[u16],
    _salt: &[u16],
    version_key: u64,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    if uids.len() != weights.len() {
        return Err(anyhow::anyhow!(
            "UIDS and weights must have the same length"
        ));
    }

    let args = vec![
        Value::u128(netuid as u128),
        Value::u128(mechanism_id as u128),
        Value::from(netuid),
        Value::from(mechanism_id),
        Value::from(version_key),
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
    mechanism_id: u8,
    uids: &[u16],
    weights: &[u16],
    version_key: u64,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    if uids.len() != weights.len() {
        return Err(anyhow::anyhow!(
            "UIDS and weights must have the same length"
        ));
    }

    let args = vec![
        Value::u128(netuid as u128),
        Value::u128(mechanism_id as u128),
        Value::from(netuid),
        Value::from(mechanism_id),
        Value::from(version_key),
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
