//! High-level Subtensor API
//!
//! This module provides a `Subtensor` struct that mirrors the Python SDK's interface,
//! with intelligent `set_weights` that automatically handles commit-reveal.
//!
//! # Usage
//! ```ignore
//! use bittensor_rs::subtensor::Subtensor;
//!
//! let subtensor = Subtensor::new("wss://entrypoint-finney.opentensor.ai:443").await?;
//!
//! // Automatically uses commit-reveal if enabled on subnet
//! let response = subtensor.set_weights(
//!     &wallet,
//!     netuid,
//!     &uids,
//!     &weights,
//!     version_key,
//! ).await?;
//! ```

use crate::chain::{BittensorClient, BittensorSigner, ExtrinsicWait};
use crate::crv4::{
    calculate_reveal_round, commit_timelocked_mechanism_weights, commit_timelocked_weights,
    get_mechid_storage_index, prepare_crv4_commit, DEFAULT_COMMIT_REVEAL_VERSION,
};
use crate::queries::subnets::{commit_reveal_enabled, tempo, weights_rate_limit};
use crate::utils::weights::normalize_weights;
use crate::validator::weights::{
    commit_weights as raw_commit_weights, reveal_weights as raw_reveal_weights,
    set_weights as raw_set_weights,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

const SUBTENSOR_MODULE: &str = "SubtensorModule";

/// Response from weight submission operations
#[derive(Clone, Debug)]
pub struct WeightResponse {
    /// Whether the operation succeeded
    pub success: bool,
    /// Transaction hash if successful
    pub tx_hash: Option<String>,
    /// Error message if failed
    pub message: String,
    /// Additional data (e.g., reveal round for CRv4)
    pub data: Option<WeightResponseData>,
}

impl WeightResponse {
    pub fn success(tx_hash: String, message: &str) -> Self {
        Self {
            success: true,
            tx_hash: Some(tx_hash),
            message: message.to_string(),
            data: None,
        }
    }

    pub fn failure(message: &str) -> Self {
        Self {
            success: false,
            tx_hash: None,
            message: message.to_string(),
            data: None,
        }
    }

    pub fn with_data(mut self, data: WeightResponseData) -> Self {
        self.data = Some(data);
        self
    }
}

/// Additional data from weight operations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WeightResponseData {
    /// CRv4 commit data
    Crv4 {
        reveal_round: u64,
        encrypted_payload: Vec<u8>,
    },
    /// Legacy commit-reveal data
    CommitReveal { commit_hash: String, salt: Vec<u16> },
}

/// Salt type for commit-reveal (Vec<u16>)
pub type Salt = Vec<u16>;

/// Pending commit for legacy commit-reveal (V2)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PendingCommit {
    pub netuid: u16,
    pub mechanism_id: Option<u8>,
    pub commit_hash: String,
    pub uids: Vec<u16>,
    pub weights: Vec<u16>,
    pub salt: Vec<u16>,
    pub version_key: u64,
    pub epoch: u64,
    pub committed_at: chrono::DateTime<chrono::Utc>,
}

/// Persisted state for pending commits
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SubtensorState {
    /// Pending commits by (netuid, mechanism_id)
    pub pending_commits: HashMap<(u16, Option<u8>), PendingCommit>,
    /// Last revealed epoch per (netuid, mechanism_id)
    pub last_revealed: HashMap<(u16, Option<u8>), u64>,
}

impl SubtensorState {
    pub fn load(path: &PathBuf) -> Self {
        match std::fs::read_to_string(path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

/// High-level Subtensor interface similar to Python SDK
///
/// Provides intelligent `set_weights` that automatically:
/// 1. Checks if commit-reveal is enabled on the subnet
/// 2. Uses CRv4 (timelock encryption) if version >= 4
/// 3. Falls back to legacy commit-reveal if needed
/// 4. Uses direct set_weights if commit-reveal is disabled
pub struct Subtensor {
    client: Arc<BittensorClient>,
    /// Cached commit-reveal version
    crv_version: RwLock<Option<u16>>,
    /// Persisted state for pending commits
    state: RwLock<SubtensorState>,
    /// Path for state persistence
    state_path: Option<PathBuf>,
    /// Default block time (seconds)
    block_time: f64,
}

impl Subtensor {
    /// Create a new Subtensor connection
    pub async fn new(endpoint: &str) -> Result<Self> {
        let client = BittensorClient::new(endpoint)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e))?;
        Ok(Self {
            client: Arc::new(client),
            crv_version: RwLock::new(None),
            state: RwLock::new(SubtensorState::default()),
            state_path: None,
            block_time: 12.0,
        })
    }

    /// Create with persistence for pending commits
    pub async fn with_persistence(endpoint: &str, state_path: PathBuf) -> Result<Self> {
        let client = BittensorClient::new(endpoint)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e))?;
        let state = SubtensorState::load(&state_path);
        Ok(Self {
            client: Arc::new(client),
            crv_version: RwLock::new(None),
            state: RwLock::new(state),
            state_path: Some(state_path),
            block_time: 12.0,
        })
    }

    /// Create from existing client
    pub fn from_client(client: Arc<BittensorClient>) -> Self {
        Self {
            client,
            crv_version: RwLock::new(None),
            state: RwLock::new(SubtensorState::default()),
            state_path: None,
            block_time: 12.0,
        }
    }

    /// Get the underlying client
    pub fn client(&self) -> &BittensorClient {
        &self.client
    }

    /// Get current block number
    pub async fn get_current_block(&self) -> Result<u64> {
        self.client
            .block_number()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get block number: {}", e))
    }

    /// Check if commit-reveal is enabled for a subnet
    pub async fn commit_reveal_enabled(&self, netuid: u16) -> Result<bool> {
        commit_reveal_enabled(&self.client, netuid).await
    }

    /// Get commit-reveal version from chain (cached)
    pub async fn get_commit_reveal_version(&self) -> Result<u16> {
        {
            let cached = self.crv_version.read().await;
            if let Some(v) = *cached {
                return Ok(v);
            }
        }

        let version = crate::crv4::get_commit_reveal_version(&self.client)
            .await
            .unwrap_or(DEFAULT_COMMIT_REVEAL_VERSION);

        let mut cached = self.crv_version.write().await;
        *cached = Some(version);

        info!("Commit-reveal version from chain: {}", version);
        Ok(version)
    }

    /// Check if CRv4 (timelock encryption) is enabled
    pub async fn is_crv4_enabled(&self) -> bool {
        self.get_commit_reveal_version().await.unwrap_or(0) >= 4
    }

    /// Get tempo for a subnet
    pub async fn tempo(&self, netuid: u16) -> Result<u16> {
        let t = tempo(&self.client, netuid).await?.unwrap_or(360);
        Ok(t as u16)
    }

    /// Get weights rate limit for a subnet
    pub async fn weights_rate_limit(&self, netuid: u16) -> Result<u64> {
        weights_rate_limit(&self.client, netuid)
            .await
            .map(|v| v.unwrap_or(0))
    }

    /// Get blocks since last update for a neuron
    pub async fn blocks_since_last_update(&self, netuid: u16, uid: u16) -> Result<u64> {
        use crate::utils::decoders::decode_u64;
        use subxt::dynamic::Value;

        let storage_index = get_mechid_storage_index(netuid, 0);
        if let Some(val) = self
            .client
            .storage_with_keys(
                SUBTENSOR_MODULE,
                "LastUpdate",
                vec![Value::u128(storage_index as u128), Value::u128(uid as u128)],
            )
            .await?
        {
            let last_update = decode_u64(&val).unwrap_or(0);
            let current_block = self.get_current_block().await?;
            return Ok(current_block.saturating_sub(last_update));
        }
        Ok(0)
    }

    /// Check if rate limit allows setting weights
    pub async fn can_set_weights(&self, netuid: u16, uid: u16) -> Result<bool> {
        let bslu = self.blocks_since_last_update(netuid, uid).await?;
        let wrl = self.weights_rate_limit(netuid).await?;
        Ok(bslu > wrl)
    }

    /// Get reveal period epochs
    pub async fn get_reveal_period(&self, netuid: u16) -> Result<u64> {
        crate::crv4::get_reveal_period(&self.client, netuid).await
    }

    /// Get UID for hotkey on subnet
    pub async fn get_uid_for_hotkey(&self, netuid: u16, hotkey: &[u8; 32]) -> Result<Option<u16>> {
        use crate::utils::decoders::decode_u16;
        use subxt::dynamic::Value;

        if let Some(val) = self
            .client
            .storage_with_keys(
                SUBTENSOR_MODULE,
                "Uids",
                vec![
                    Value::u128(netuid as u128),
                    Value::from_bytes(hotkey.as_slice()),
                ],
            )
            .await?
        {
            return Ok(decode_u16(&val).ok());
        }
        Ok(None)
    }

    /// Get current epoch number for a subnet
    pub async fn get_current_epoch(&self, netuid: u16) -> Result<u64> {
        let block = self.get_current_block().await?;
        let tempo = self.tempo(netuid).await? as u64;
        Ok(block / (tempo + 1))
    }

    /// Get mechanism count for a subnet
    pub async fn get_mechanism_count(&self, netuid: u16) -> Result<u8> {
        crate::get_mechanism_count(&self.client, netuid).await
    }

    /// Get current epoch phase for a subnet
    /// Returns: "evaluation", "commit", or "reveal"
    pub async fn get_current_phase(&self, netuid: u16) -> Result<String> {
        let block = self.get_current_block().await?;
        let tempo = self.tempo(netuid).await? as u64;
        let block_in_epoch = block % (tempo + 1);

        // Standard phase distribution: 75% eval, 15% commit, 10% reveal
        let eval_end = (tempo * 75) / 100;
        let commit_end = eval_end + (tempo * 15) / 100;

        if block_in_epoch < eval_end {
            Ok("evaluation".to_string())
        } else if block_in_epoch < commit_end {
            Ok("commit".to_string())
        } else {
            Ok("reveal".to_string())
        }
    }

    /// Check if currently in reveal phase
    pub async fn is_in_reveal_phase(&self, netuid: u16) -> Result<bool> {
        let phase = self.get_current_phase(netuid).await?;
        Ok(phase == "reveal")
    }

    /// Check if currently in commit phase
    pub async fn is_in_commit_phase(&self, netuid: u16) -> Result<bool> {
        let phase = self.get_current_phase(netuid).await?;
        Ok(phase == "commit")
    }

    /// Get pending commits info string (for logging)
    pub async fn pending_commits_info(&self) -> String {
        let state = self.state.read().await;
        if state.pending_commits.is_empty() {
            "none".to_string()
        } else {
            let keys: Vec<_> = state.pending_commits.keys().collect();
            format!("{} pending: {:?}", keys.len(), keys)
        }
    }

    // ==========================================================================
    // MAIN SET_WEIGHTS - Intelligent routing like Python SDK
    // ==========================================================================

    /// Set weights for a subnet - automatically uses commit-reveal if enabled
    ///
    /// This is the main entry point, similar to Python SDK's `subtensor.set_weights()`.
    ///
    /// # Behavior
    /// 1. If commit-reveal is enabled and CRv4 version >= 4:
    ///    - Uses timelock encryption (auto-revealed by chain)
    /// 2. If commit-reveal is enabled but version < 4:
    ///    - Uses legacy commit-reveal (needs manual reveal)
    /// 3. If commit-reveal is disabled:
    ///    - Uses direct set_weights extrinsic
    ///
    /// # Arguments
    /// * `signer` - Wallet signer (hotkey)
    /// * `netuid` - Subnet ID
    /// * `uids` - Neuron UIDs to set weights for
    /// * `weights` - Weight values (f32 0.0-1.0 or raw u16 0-65535)
    /// * `version_key` - Network version key
    #[allow(clippy::too_many_arguments)]
    pub async fn set_weights(
        &self,
        signer: &BittensorSigner,
        netuid: u16,
        uids: &[u16],
        weights: &[u16],
        version_key: u64,
        wait_for: ExtrinsicWait,
    ) -> Result<WeightResponse> {
        self.set_mechanism_weights(signer, netuid, 0, uids, weights, version_key, wait_for)
            .await
    }

    /// Set mechanism weights with full control
    #[allow(clippy::too_many_arguments)]
    pub async fn set_mechanism_weights(
        &self,
        signer: &BittensorSigner,
        netuid: u16,
        mechanism_id: u8,
        uids: &[u16],
        weights: &[u16],
        version_key: u64,
        wait_for: ExtrinsicWait,
    ) -> Result<WeightResponse> {
        // Check if commit-reveal is enabled
        let cr_enabled = self.commit_reveal_enabled(netuid).await?;

        if cr_enabled {
            let crv_version = self.get_commit_reveal_version().await?;

            if crv_version >= 4 {
                // CRv4: Timelock encryption - chain auto-reveals
                info!(
                    "Using CRv4 (timelock encryption) for netuid={}, mechanism={}",
                    netuid, mechanism_id
                );
                self.set_weights_crv4(
                    signer,
                    netuid,
                    mechanism_id,
                    uids,
                    weights,
                    version_key,
                    wait_for,
                )
                .await
            } else {
                // Legacy commit-reveal
                info!(
                    "Using legacy commit-reveal for netuid={}, mechanism={}",
                    netuid, mechanism_id
                );
                self.set_weights_commit_reveal(
                    signer,
                    netuid,
                    mechanism_id,
                    uids,
                    weights,
                    version_key,
                    wait_for,
                )
                .await
            }
        } else {
            // Direct set_weights
            info!(
                "Using direct set_weights for netuid={}, mechanism={}",
                netuid, mechanism_id
            );
            self.set_weights_direct(
                signer,
                netuid,
                mechanism_id,
                uids,
                weights,
                version_key,
                wait_for,
            )
            .await
        }
    }

    // ==========================================================================
    // CRv4 (Timelock Encryption)
    // ==========================================================================

    /// Get the chain's last stored DRAND round
    pub async fn get_last_drand_round(&self) -> Result<u64> {
        crate::queries::chain_info::last_drand_round(&self.client)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to get Drand.LastStoredRound from chain"))
    }

    /// Set weights using CRv4 timelock encryption
    /// Chain automatically decrypts when DRAND pulse arrives
    #[allow(clippy::too_many_arguments)]
    async fn set_weights_crv4(
        &self,
        signer: &BittensorSigner,
        netuid: u16,
        mechanism_id: u8,
        uids: &[u16],
        weights: &[u16],
        version_key: u64,
        wait_for: ExtrinsicWait,
    ) -> Result<WeightResponse> {
        let hotkey_bytes = signer.account_id().0.to_vec();

        // Get chain parameters
        let current_block = self.get_current_block().await?;
        let tempo = self.tempo(netuid).await?;
        let reveal_period = self.get_reveal_period(netuid).await?;
        let crv_version = self.get_commit_reveal_version().await?;

        // Get chain's last DRAND round (CRITICAL: must use chain state, not system time)
        let chain_last_drand_round = self.get_last_drand_round().await?;

        // Calculate reveal round relative to chain's DRAND state
        let storage_index = get_mechid_storage_index(netuid, mechanism_id);
        let reveal_round = calculate_reveal_round(
            tempo,
            current_block,
            storage_index,
            reveal_period,
            self.block_time,
            chain_last_drand_round,
        );

        // Encrypt payload
        let encrypted =
            prepare_crv4_commit(&hotkey_bytes, uids, weights, version_key, reveal_round)?;

        info!(
            "CRv4 commit: netuid={}, mechanism={}, uids={}, chain_last_drand={}, reveal_round={}",
            netuid,
            mechanism_id,
            uids.len(),
            chain_last_drand_round,
            reveal_round
        );

        // Submit to chain
        let tx_hash = if mechanism_id == 0 {
            commit_timelocked_weights(
                &self.client,
                signer,
                netuid,
                &encrypted,
                reveal_round,
                crv_version,
                wait_for,
            )
            .await?
        } else {
            commit_timelocked_mechanism_weights(
                &self.client,
                signer,
                netuid,
                mechanism_id,
                &encrypted,
                reveal_round,
                crv_version,
                wait_for,
            )
            .await?
        };

        info!(
            "CRv4 commit submitted: tx={}, reveal_round={}, chain_last_drand={} (no manual reveal needed)",
            tx_hash, reveal_round, chain_last_drand_round
        );

        Ok(
            WeightResponse::success(tx_hash, "CRv4 weights committed (auto-reveal)").with_data(
                WeightResponseData::Crv4 {
                    reveal_round,
                    encrypted_payload: encrypted,
                },
            ),
        )
    }

    // ==========================================================================
    // Legacy Commit-Reveal (V2)
    // ==========================================================================

    /// Set weights using legacy commit-reveal pattern
    /// Requires manual reveal after commit
    #[allow(clippy::too_many_arguments)]
    async fn set_weights_commit_reveal(
        &self,
        signer: &BittensorSigner,
        netuid: u16,
        mechanism_id: u8,
        uids: &[u16],
        weights: &[u16],
        version_key: u64,
        wait_for: ExtrinsicWait,
    ) -> Result<WeightResponse> {
        let key = (netuid, Some(mechanism_id).filter(|&m| m != 0));

        // Check if we have a pending commit to reveal
        let pending_to_reveal = {
            let state = self.state.read().await;
            state.pending_commits.get(&key).cloned()
        };

        if let Some(pending) = pending_to_reveal {
            return self.reveal_pending_commit(signer, pending, wait_for).await;
        }

        // Create new commit
        let account = signer.account_id().0;
        let commit_data = if mechanism_id == 0 {
            crate::validator::weights::prepare_commit_reveal(
                &account,
                netuid,
                uids,
                weights,
                version_key,
                8,
            )
        } else {
            crate::validator::weights::prepare_mechanism_commit_reveal(
                &account,
                netuid,
                mechanism_id,
                uids,
                weights,
                version_key,
                8,
            )
        };

        info!(
            "Committing weights hash: {} (netuid={}, mechanism={})",
            &commit_data.commit_hash[..16],
            netuid,
            mechanism_id
        );

        // Submit commit
        let tx_hash = if mechanism_id == 0 {
            raw_commit_weights(
                &self.client,
                signer,
                netuid,
                &commit_data.commit_hash,
                wait_for,
            )
            .await?
        } else {
            crate::commit_mechanism_weights(
                &self.client,
                signer,
                netuid,
                mechanism_id,
                &commit_data.commit_hash,
                wait_for,
            )
            .await?
        };

        // Store pending commit
        let current_block = self.get_current_block().await?;
        let tempo = self.tempo(netuid).await? as u64;
        let epoch = current_block / (tempo + 1);

        let pending = PendingCommit {
            netuid,
            mechanism_id: Some(mechanism_id).filter(|&m| m != 0),
            commit_hash: commit_data.commit_hash.clone(),
            uids: commit_data.uids,
            weights: commit_data.weights,
            salt: commit_data.salt.clone(),
            version_key: commit_data.version_key,
            epoch,
            committed_at: chrono::Utc::now(),
        };

        {
            let mut state = self.state.write().await;
            state.pending_commits.insert(key, pending);
            if let Some(ref path) = self.state_path {
                if let Err(e) = state.save(path) {
                    tracing::warn!("Failed to save state: {}", e);
                }
            }
        }

        info!(
            "Weights committed: {} (reveal pending, epoch={})",
            tx_hash, epoch
        );

        Ok(
            WeightResponse::success(tx_hash, "Weights committed - call again to reveal").with_data(
                WeightResponseData::CommitReveal {
                    commit_hash: commit_data.commit_hash,
                    salt: commit_data.salt,
                },
            ),
        )
    }

    /// Reveal a pending commit
    async fn reveal_pending_commit(
        &self,
        signer: &BittensorSigner,
        pending: PendingCommit,
        wait_for: ExtrinsicWait,
    ) -> Result<WeightResponse> {
        info!(
            "Revealing weights for commit: {} (netuid={}, mechanism={:?})",
            &pending.commit_hash[..16],
            pending.netuid,
            pending.mechanism_id
        );

        let tx_hash = match pending.mechanism_id {
            None | Some(0) => {
                raw_reveal_weights(
                    &self.client,
                    signer,
                    pending.netuid,
                    &pending.uids,
                    &pending.weights,
                    &pending.salt,
                    pending.version_key,
                    wait_for,
                )
                .await?
            }
            Some(mechanism_id) => {
                crate::reveal_mechanism_weights(
                    &self.client,
                    signer,
                    pending.netuid,
                    mechanism_id,
                    &pending.uids,
                    &pending.weights,
                    &pending.salt,
                    pending.version_key,
                    wait_for,
                )
                .await?
            }
        };

        // Remove pending commit
        let key = (pending.netuid, pending.mechanism_id);
        {
            let mut state = self.state.write().await;
            state.pending_commits.remove(&key);
            state.last_revealed.insert(key, pending.epoch);
            if let Some(ref path) = self.state_path {
                if let Err(e) = state.save(path) {
                    tracing::warn!("Failed to save state: {}", e);
                }
            }
        }

        info!("Weights revealed: {}", tx_hash);

        Ok(WeightResponse::success(
            tx_hash,
            "Weights revealed successfully",
        ))
    }

    // ==========================================================================
    // Direct Set Weights (No Commit-Reveal)
    // ==========================================================================

    /// Set weights directly (no commit-reveal)
    #[allow(clippy::too_many_arguments)]
    async fn set_weights_direct(
        &self,
        signer: &BittensorSigner,
        netuid: u16,
        mechanism_id: u8,
        uids: &[u16],
        weights: &[u16],
        version_key: u64,
        wait_for: ExtrinsicWait,
    ) -> Result<WeightResponse> {
        let tx_hash = if mechanism_id == 0 {
            raw_set_weights(
                &self.client,
                signer,
                netuid,
                uids,
                weights,
                version_key,
                wait_for,
            )
            .await?
        } else {
            crate::set_mechanism_weights(
                &self.client,
                signer,
                netuid,
                mechanism_id,
                uids,
                weights,
                version_key,
                wait_for,
            )
            .await?
        };

        info!("Weights set directly: {}", tx_hash);

        Ok(WeightResponse::success(tx_hash, "Weights set successfully"))
    }

    // ==========================================================================
    // Utility Methods
    // ==========================================================================

    /// Check if there are pending commits to reveal
    pub async fn has_pending_commits(&self) -> bool {
        let state = self.state.read().await;
        !state.pending_commits.is_empty()
    }

    /// Get all pending commits
    pub async fn pending_commits(&self) -> Vec<PendingCommit> {
        let state = self.state.read().await;
        state.pending_commits.values().cloned().collect()
    }

    /// Force reveal all pending commits
    pub async fn reveal_all_pending(
        &self,
        signer: &BittensorSigner,
        wait_for: ExtrinsicWait,
    ) -> Result<Vec<WeightResponse>> {
        let pending: Vec<PendingCommit> = {
            let state = self.state.read().await;
            state.pending_commits.values().cloned().collect()
        };

        let mut results = Vec::new();
        for commit in pending {
            let result = self.reveal_pending_commit(signer, commit, wait_for).await;
            match result {
                Ok(r) => results.push(r),
                Err(e) => {
                    error!("Failed to reveal commit: {}", e);
                    results.push(WeightResponse::failure(&e.to_string()));
                }
            }
        }

        Ok(results)
    }

    /// Clear expired pending commits
    pub async fn cleanup_old_commits(&self, current_epoch: u64, max_age_epochs: u64) {
        let mut state = self.state.write().await;
        let cutoff = current_epoch.saturating_sub(max_age_epochs);

        state
            .pending_commits
            .retain(|_, commit| commit.epoch >= cutoff);

        if let Some(ref path) = self.state_path {
            if let Err(e) = state.save(path) {
                tracing::warn!("Failed to save state: {}", e);
            }
        }
    }

    /// Normalize weights from f32 to u16
    pub fn normalize_weights_f32(uids: &[u64], weights: &[f32]) -> Result<(Vec<u16>, Vec<u16>)> {
        normalize_weights(uids, weights)
    }

    /// Persist current state to disk
    pub async fn persist_state(&self) -> Result<()> {
        if let Some(ref path) = self.state_path {
            let state = self.state.read().await;
            state.save(path)?;
        }
        Ok(())
    }
}

/// Builder for Subtensor with options
pub struct SubtensorBuilder {
    endpoint: String,
    state_path: Option<PathBuf>,
    block_time: f64,
}

impl SubtensorBuilder {
    pub fn new(endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            state_path: None,
            block_time: 12.0,
        }
    }

    pub fn with_persistence(mut self, path: PathBuf) -> Self {
        self.state_path = Some(path);
        self
    }

    pub fn with_block_time(mut self, block_time: f64) -> Self {
        self.block_time = block_time;
        self
    }

    pub async fn build(self) -> Result<Subtensor> {
        let client = BittensorClient::new(&self.endpoint)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e))?;
        let state = self
            .state_path
            .as_ref()
            .map(SubtensorState::load)
            .unwrap_or_default();

        Ok(Subtensor {
            client: Arc::new(client),
            crv_version: RwLock::new(None),
            state: RwLock::new(state),
            state_path: self.state_path,
            block_time: self.block_time,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weight_response() {
        let response = WeightResponse::success("0x123".to_string(), "Test");
        assert!(response.success);
        assert_eq!(response.tx_hash, Some("0x123".to_string()));
    }

    #[test]
    fn test_pending_commit_serialization() {
        let commit = PendingCommit {
            netuid: 1,
            mechanism_id: Some(0),
            commit_hash: "0xabc".to_string(),
            uids: vec![1, 2, 3],
            weights: vec![10000, 20000, 35535],
            salt: vec![1, 2, 3, 4, 5, 6, 7, 8],
            version_key: 1,
            epoch: 100,
            committed_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&commit).unwrap();
        let decoded: PendingCommit = serde_json::from_str(&json).unwrap();

        assert_eq!(commit.commit_hash, decoded.commit_hash);
        assert_eq!(commit.uids, decoded.uids);
    }
}
