use serde::{Deserialize, Serialize};
use sp_core::crypto::AccountId32;

/// Weight commitment information for commit-reveal pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightCommitInfo {
    /// Hotkey that made the commitment
    pub hotkey: AccountId32,
    /// Block number when commitment was made
    pub block: u64,
    /// Commit hash
    pub commit_hash: Vec<u8>,
    /// Number of blocks since commitment
    pub blocks_since_commit: u64,
    /// Whether this commitment has been revealed
    pub revealed: bool,
}

impl WeightCommitInfo {
    /// Create WeightCommitInfo from chain data - all fields must be provided
    /// 
    /// # Arguments
    /// * `hotkey` - AccountId32 of the hotkey that made the commitment
    /// * `block` - Block number when commitment was made
    /// * `commit_hash` - Commit hash bytes
    /// * `current_block` - Current block number to calculate blocks_since_commit
    /// * `revealed` - Whether this commitment has been revealed
    pub fn from_chain_data(
        hotkey: AccountId32,
        block: u64,
        commit_hash: Vec<u8>,
        current_block: u64,
        revealed: bool,
    ) -> Self {
        let blocks_since_commit = current_block.saturating_sub(block);
        Self {
            hotkey,
            block,
            commit_hash,
            blocks_since_commit,
            revealed,
        }
    }
}

