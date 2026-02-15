//! CRv4 (Commit-Reveal v4) - Timelock encrypted weight commitments
//!
//! This module implements the CRv4 protocol for Bittensor which uses DRAND timelock
//! encryption to submit encrypted weights that are automatically revealed by the chain.
//!
//! ## How it works:
//! 1. Client builds a `WeightsTlockPayload` with hotkey, uids, weights, and version_key
//! 2. Payload is SCALE-encoded
//! 3. Payload is encrypted using TLE (Timelock Encryption) for a future DRAND round
//! 4. Encrypted ciphertext is submitted via `commit_timelocked_mechanism_weights` extrinsic
//! 5. Chain automatically decrypts and applies weights when the DRAND pulse is available
//!
//! ## Usage:
//! ```ignore
//! use bittensor_rs::crv4::{prepare_crv4_commit, Crv4CommitData};
//!
//! let commit_data = prepare_crv4_commit(
//!     &hotkey_bytes,
//!     &uids,
//!     &weights,
//!     version_key,
//!     tempo,
//!     current_block,
//!     netuid,
//!     reveal_period_epochs,
//!     block_time,
//! )?;
//!
//! // Submit to chain
//! commit_timelocked_mechanism_weights(client, signer, netuid, mecid, &commit_data).await?;
//! ```

mod drand;
mod encryption;
mod payload;
mod persistence;

pub use drand::*;
pub use encryption::*;
pub use payload::*;
pub use persistence::*;

use crate::chain::{BittensorClient, BittensorSigner, ExtrinsicWait};
use anyhow::Result;
use subxt::dynamic::Value;

const SUBTENSOR_MODULE: &str = "SubtensorModule";
const COMMIT_TIMELOCKED_WEIGHTS: &str = "commit_timelocked_weights";
const COMMIT_TIMELOCKED_MECHANISM_WEIGHTS: &str = "commit_timelocked_mechanism_weights";

/// Default commit-reveal version (CRv4)
pub const DEFAULT_COMMIT_REVEAL_VERSION: u16 = 4;

/// Submit a timelocked weight commitment (CRv4) for main mechanism
pub async fn commit_timelocked_weights(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    commit: &[u8],
    reveal_round: u64,
    commit_reveal_version: u16,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    if commit.is_empty() {
        return Err(anyhow::anyhow!(
            "Encrypted commit payload must not be empty"
        ));
    }

    let args = vec![
        Value::u128(netuid as u128),
        Value::from_bytes(commit),
        Value::u128(reveal_round as u128),
        Value::u128(commit_reveal_version as u128),
    ];

    let tx_hash = client
        .submit_extrinsic(
            SUBTENSOR_MODULE,
            COMMIT_TIMELOCKED_WEIGHTS,
            args,
            signer,
            wait_for,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to commit timelocked weights: {}", e))?;

    Ok(tx_hash)
}

/// Submit a timelocked mechanism weight commitment (CRv4)
#[allow(clippy::too_many_arguments)]
pub async fn commit_timelocked_mechanism_weights(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    mechanism_id: u8,
    commit: &[u8],
    reveal_round: u64,
    commit_reveal_version: u16,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    if commit.is_empty() {
        return Err(anyhow::anyhow!(
            "Encrypted commit payload must not be empty"
        ));
    }

    let args = vec![
        Value::u128(netuid as u128),
        Value::u128(mechanism_id as u128),
        Value::from_bytes(commit),
        Value::u128(reveal_round as u128),
        Value::u128(commit_reveal_version as u128),
    ];

    let tx_hash = client
        .submit_extrinsic(
            SUBTENSOR_MODULE,
            COMMIT_TIMELOCKED_MECHANISM_WEIGHTS,
            args,
            signer,
            wait_for,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to commit timelocked mechanism weights: {}", e))?;

    Ok(tx_hash)
}

/// Get the chain's last stored DRAND round
pub async fn get_last_drand_round(client: &BittensorClient) -> Result<u64> {
    if let Some(val) = client.storage("Drand", "LastStoredRound", None).await? {
        if let Ok(round) = crate::utils::decoders::decode_u64(&val) {
            return Ok(round);
        }
    }
    Err(anyhow::anyhow!(
        "Failed to get Drand.LastStoredRound from chain"
    ))
}

/// High-level function: Prepare and submit CRv4 weights
///
/// This handles the entire flow:
/// 1. Calculate reveal round based on tempo/epoch and chain's DRAND state
/// 2. Encrypt payload with TLE
/// 3. Submit to chain
/// 4. Return commit data for persistence
pub async fn prepare_and_commit_crv4_weights(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    uids: &[u16],
    weights: &[u16],
    version_key: u64,
    wait_for: ExtrinsicWait,
) -> Result<Crv4CommitData> {
    let hotkey_bytes = signer.account_id().0.to_vec();

    // Get chain parameters
    let current_block = client.block_number().await?;
    let tempo = get_tempo(client, netuid).await.unwrap_or(360);
    let reveal_period = get_reveal_period(client, netuid).await.unwrap_or(1);
    let block_time = 12.0; // Standard Bittensor block time

    // Get chain's last DRAND round (CRITICAL: must use chain state, not system time)
    let chain_last_drand_round = get_last_drand_round(client).await?;

    // Calculate reveal round relative to chain's DRAND state
    let storage_index = get_mechid_storage_index(netuid, 0); // Main mechanism
    let reveal_round = calculate_reveal_round(
        tempo,
        current_block,
        storage_index,
        reveal_period,
        block_time,
        chain_last_drand_round,
    );

    // Prepare and encrypt payload
    let encrypted = prepare_crv4_commit(&hotkey_bytes, uids, weights, version_key, reveal_round)?;

    // Get commit reveal version from chain
    let crv_version = get_commit_reveal_version(client)
        .await
        .unwrap_or(DEFAULT_COMMIT_REVEAL_VERSION);

    // Submit to chain
    let tx_hash = commit_timelocked_weights(
        client,
        signer,
        netuid,
        &encrypted,
        reveal_round,
        crv_version,
        wait_for,
    )
    .await?;

    tracing::info!(
        "CRv4 commit submitted: tx={}, netuid={}, reveal_round={}, chain_last_drand={}, version={}",
        tx_hash,
        netuid,
        reveal_round,
        chain_last_drand_round,
        crv_version
    );

    Ok(Crv4CommitData {
        netuid,
        mechanism_id: None,
        hotkey: hotkey_bytes,
        uids: uids.to_vec(),
        weights: weights.to_vec(),
        version_key,
        reveal_round,
        commit_reveal_version: crv_version,
        encrypted_payload: encrypted,
        tx_hash,
        committed_at: chrono::Utc::now(),
        epoch: current_block / (tempo as u64 + 1),
    })
}

/// High-level function: Prepare and submit CRv4 mechanism weights
#[allow(clippy::too_many_arguments)]
pub async fn prepare_and_commit_crv4_mechanism_weights(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    mechanism_id: u8,
    uids: &[u16],
    weights: &[u16],
    version_key: u64,
    wait_for: ExtrinsicWait,
) -> Result<Crv4CommitData> {
    let hotkey_bytes = signer.account_id().0.to_vec();

    let current_block = client.block_number().await?;
    let tempo = get_tempo(client, netuid).await.unwrap_or(360);
    let reveal_period = get_reveal_period(client, netuid).await.unwrap_or(1);
    let block_time = 12.0;

    // Get chain's last DRAND round (CRITICAL: must use chain state, not system time)
    let chain_last_drand_round = get_last_drand_round(client).await?;

    let storage_index = get_mechid_storage_index(netuid, mechanism_id);
    let reveal_round = calculate_reveal_round(
        tempo,
        current_block,
        storage_index,
        reveal_period,
        block_time,
        chain_last_drand_round,
    );

    let encrypted = prepare_crv4_commit(&hotkey_bytes, uids, weights, version_key, reveal_round)?;

    let crv_version = get_commit_reveal_version(client)
        .await
        .unwrap_or(DEFAULT_COMMIT_REVEAL_VERSION);

    let tx_hash = commit_timelocked_mechanism_weights(
        client,
        signer,
        netuid,
        mechanism_id,
        &encrypted,
        reveal_round,
        crv_version,
        wait_for,
    )
    .await?;

    tracing::info!(
        "CRv4 mechanism commit submitted: tx={}, netuid={}, mecid={}, reveal_round={}, chain_last_drand={}",
        tx_hash,
        netuid,
        mechanism_id,
        reveal_round,
        chain_last_drand_round
    );

    Ok(Crv4CommitData {
        netuid,
        mechanism_id: Some(mechanism_id),
        hotkey: hotkey_bytes,
        uids: uids.to_vec(),
        weights: weights.to_vec(),
        version_key,
        reveal_round,
        commit_reveal_version: crv_version,
        encrypted_payload: encrypted,
        tx_hash,
        committed_at: chrono::Utc::now(),
        epoch: current_block / (tempo as u64 + 1),
    })
}

/// Get commit-reveal version from chain
pub async fn get_commit_reveal_version(client: &BittensorClient) -> Result<u16> {
    if let Some(val) = client
        .storage(SUBTENSOR_MODULE, "CommitRevealWeightsVersion", None)
        .await?
    {
        if let Ok(version) = crate::utils::decoders::decode_u16(&val) {
            return Ok(version);
        }
    }
    Ok(DEFAULT_COMMIT_REVEAL_VERSION)
}

/// Get tempo for a subnet
pub async fn get_tempo(client: &BittensorClient, netuid: u16) -> Result<u16> {
    let key = vec![Value::u128(netuid as u128)];
    if let Some(val) = client.storage(SUBTENSOR_MODULE, "Tempo", Some(key)).await? {
        return crate::utils::decoders::decode_u16(&val)
            .map_err(|e| anyhow::anyhow!("Failed to decode tempo: {}", e));
    }
    Ok(360)
}

/// Get reveal period in epochs
pub async fn get_reveal_period(client: &BittensorClient, netuid: u16) -> Result<u64> {
    let key = vec![Value::u128(netuid as u128)];
    if let Some(val) = client
        .storage(SUBTENSOR_MODULE, "RevealPeriodEpochs", Some(key))
        .await?
    {
        return crate::utils::decoders::decode_u64(&val)
            .map_err(|e| anyhow::anyhow!("Failed to decode reveal period: {}", e));
    }
    Ok(1)
}

/// Calculate mechanism storage index (same as subtensor)
/// Formula: mechid * 4096 + netuid
pub fn get_mechid_storage_index(netuid: u16, mechid: u8) -> u16 {
    (mechid as u16).saturating_mul(4096).saturating_add(netuid)
}
