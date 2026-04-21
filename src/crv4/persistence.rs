//! CRv4 Persistence
//!
//! Handles persisting commit data to survive validator restarts.
//! When a CRv4 commit is made, the data is saved to disk so that
//! on restart, the validator knows it has already committed for
//! an epoch and doesn't need to commit again.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// CRv4 commit data for a single commit
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Crv4CommitData {
    /// Subnet ID
    pub netuid: u16,
    /// Mechanism ID (None for main mechanism)
    pub mechanism_id: Option<u8>,
    /// Hotkey public key bytes
    pub hotkey: Vec<u8>,
    /// UIDs committed
    pub uids: Vec<u16>,
    /// Weights committed
    pub weights: Vec<u16>,
    /// Version key used
    pub version_key: u64,
    /// DRAND reveal round
    pub reveal_round: u64,
    /// Commit-reveal version (e.g., 4)
    pub commit_reveal_version: u16,
    /// Encrypted payload (for verification)
    pub encrypted_payload: Vec<u8>,
    /// Transaction hash
    pub tx_hash: String,
    /// When the commit was made
    pub committed_at: DateTime<Utc>,
    /// Epoch when commit was made
    pub epoch: u64,
}

impl Crv4CommitData {
    /// Get storage key for this commit
    pub fn storage_key(&self) -> String {
        match self.mechanism_id {
            Some(mecid) => format!("{}_{}", self.netuid, mecid),
            None => format!("{}_main", self.netuid),
        }
    }
}

/// Persisted state for CRv4 commits
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Crv4PersistedState {
    /// Current epoch (updated from chain)
    pub current_epoch: u64,
    /// Pending commits by key (netuid_mecid -> commit data)
    pub pending_commits: HashMap<String, Crv4CommitData>,
}

impl Crv4PersistedState {
    /// Load state from file
    pub fn load(path: &PathBuf) -> Self {
        match std::fs::read_to_string(path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(state) => {
                    tracing::info!("Loaded CRv4 state from {:?}", path);
                    state
                }
                Err(e) => {
                    tracing::warn!("Failed to parse CRv4 state file: {}", e);
                    Self::default()
                }
            },
            Err(_) => {
                tracing::debug!("No existing CRv4 state file at {:?}", path);
                Self::default()
            }
        }
    }

    /// Save state to file
    pub fn save(&self, path: &PathBuf) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        tracing::debug!("Saved CRv4 state to {:?}", path);
        Ok(())
    }

    /// Add a new commit
    pub fn add_commit(&mut self, commit: Crv4CommitData) {
        let key = commit.storage_key();
        self.pending_commits.insert(key, commit);
    }

    /// Check if we have a pending commit for this netuid/mechanism
    pub fn has_pending_commit(&self, netuid: u16, mechanism_id: Option<u8>) -> bool {
        let key = match mechanism_id {
            Some(mecid) => format!("{}_{}", netuid, mecid),
            None => format!("{}_main", netuid),
        };
        self.pending_commits.contains_key(&key)
    }

    /// Check if we have a commit for the current epoch
    pub fn has_commit_for_epoch(&self, netuid: u16, mechanism_id: Option<u8>, epoch: u64) -> bool {
        let key = match mechanism_id {
            Some(mecid) => format!("{}_{}", netuid, mecid),
            None => format!("{}_main", netuid),
        };
        self.pending_commits
            .get(&key)
            .map(|c| c.epoch == epoch)
            .unwrap_or(false)
    }

    /// Mark a commit as revealed (remove from pending)
    pub fn mark_revealed(&mut self, netuid: u16, mechanism_id: Option<u8>) {
        let key = match mechanism_id {
            Some(mecid) => format!("{}_{}", netuid, mecid),
            None => format!("{}_main", netuid),
        };
        self.pending_commits.remove(&key);
    }

    /// Get pending commit for netuid/mechanism
    pub fn get_pending_commit(
        &self,
        netuid: u16,
        mechanism_id: Option<u8>,
    ) -> Option<&Crv4CommitData> {
        let key = match mechanism_id {
            Some(mecid) => format!("{}_{}", netuid, mecid),
            None => format!("{}_main", netuid),
        };
        self.pending_commits.get(&key)
    }

    /// Remove pending commit
    pub fn remove_pending_commit(
        &mut self,
        netuid: u16,
        mechanism_id: Option<u8>,
    ) -> Option<Crv4CommitData> {
        let key = match mechanism_id {
            Some(mecid) => format!("{}_{}", netuid, mecid),
            None => format!("{}_main", netuid),
        };
        self.pending_commits.remove(&key)
    }

    /// Update epoch and clean up old commits
    pub fn update_epoch(&mut self, new_epoch: u64) {
        if new_epoch <= self.current_epoch {
            return;
        }

        tracing::info!("CRv4 epoch update: {} -> {}", self.current_epoch, new_epoch);
        self.current_epoch = new_epoch;

        // Clean up commits from old epochs that weren't revealed
        let old_commits: Vec<String> = self
            .pending_commits
            .iter()
            .filter(|(_, c)| c.epoch < new_epoch.saturating_sub(1))
            .map(|(k, _)| k.clone())
            .collect();

        for key in old_commits {
            if let Some(commit) = self.pending_commits.remove(&key) {
                tracing::warn!(
                    "Removing stale CRv4 commit for {} from epoch {} (current: {})",
                    key,
                    commit.epoch,
                    new_epoch
                );
            }
        }
    }

    /// Get all pending commits
    pub fn all_pending_commits(&self) -> Vec<&Crv4CommitData> {
        self.pending_commits.values().collect()
    }

    /// Clear all pending commits
    pub fn clear_pending_commits(&mut self) {
        self.pending_commits.clear();
    }
}

/// CRv4 State Manager
///
/// Manages CRv4 commit persistence with automatic saving.
pub struct Crv4StateManager {
    state: Crv4PersistedState,
    path: PathBuf,
}

impl Crv4StateManager {
    /// Create new state manager
    pub fn new(data_dir: Option<PathBuf>) -> Self {
        let path = data_dir
            .unwrap_or_else(|| PathBuf::from("."))
            .join("crv4_commits.json");

        let state = Crv4PersistedState::load(&path);

        if !state.pending_commits.is_empty() {
            tracing::info!(
                "Loaded {} pending CRv4 commits from previous session",
                state.pending_commits.len()
            );
        }

        Self { state, path }
    }

    /// Get mutable state
    pub fn state_mut(&mut self) -> &mut Crv4PersistedState {
        &mut self.state
    }

    /// Get state
    pub fn state(&self) -> &Crv4PersistedState {
        &self.state
    }

    /// Save state to disk
    pub fn save(&self) -> anyhow::Result<()> {
        self.state.save(&self.path)
    }

    /// Add commit and save
    pub fn add_and_save(&mut self, commit: Crv4CommitData) -> anyhow::Result<()> {
        self.state.add_commit(commit);
        self.save()
    }

    /// Mark revealed and save
    pub fn mark_revealed_and_save(
        &mut self,
        netuid: u16,
        mechanism_id: Option<u8>,
    ) -> anyhow::Result<()> {
        self.state.mark_revealed(netuid, mechanism_id);
        self.save()
    }

    /// Update epoch and save
    pub fn update_epoch_and_save(&mut self, new_epoch: u64) -> anyhow::Result<()> {
        self.state.update_epoch(new_epoch);
        self.save()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_storage_key() {
        let commit = Crv4CommitData {
            netuid: 1,
            mechanism_id: Some(2),
            hotkey: vec![],
            uids: vec![],
            weights: vec![],
            version_key: 0,
            reveal_round: 0,
            commit_reveal_version: 4,
            encrypted_payload: vec![],
            tx_hash: String::new(),
            committed_at: Utc::now(),
            epoch: 0,
        };

        assert_eq!(commit.storage_key(), "1_2");

        let commit_main = Crv4CommitData {
            mechanism_id: None,
            ..commit
        };

        assert_eq!(commit_main.storage_key(), "1_main");
    }

    #[test]
    fn test_state_operations() {
        let mut state = Crv4PersistedState::default();

        let commit = Crv4CommitData {
            netuid: 1,
            mechanism_id: Some(0),
            hotkey: vec![1; 32],
            uids: vec![0, 1],
            weights: vec![100, 200],
            version_key: 1,
            reveal_round: 1000,
            commit_reveal_version: 4,
            encrypted_payload: vec![1, 2, 3],
            tx_hash: "0x123".to_string(),
            committed_at: Utc::now(),
            epoch: 5,
        };

        state.add_commit(commit.clone());

        assert!(state.has_pending_commit(1, Some(0)));
        assert!(state.has_commit_for_epoch(1, Some(0), 5));
        assert!(!state.has_commit_for_epoch(1, Some(0), 6));

        state.mark_revealed(1, Some(0));

        assert!(!state.has_pending_commit(1, Some(0)));
    }
}
