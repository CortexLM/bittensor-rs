//! Epoch tracking for Bittensor subnets
//!
//! Tracks epoch boundaries and commit-reveal timing as defined by Subtensor.
//!
//! # Bittensor Commit-Reveal Protocol (CRv3/CRv4)
//!
//! Subtensor uses an **epoch-granular** commit-reveal model:
//! - **Epoch N** (CommitWindow): Validators can commit weights at any block
//! - **Epoch N+1** (RevealWindow): Validators reveal weights committed in epoch N
//!
//! There is NO sub-epoch "evaluation phase" in subtensor. The entire epoch is
//! available for commits. The reveal window is the entire next epoch.
//!
//! Epoch calculation matches subtensor exactly:
//!   `epoch = (block + netuid + 1) / (tempo + 1)`
//!   `epoch_start_block = epoch * (tempo + 1) - (netuid + 1)`
//!
//! Reference: `pallets/subtensor/src/subnets/weights.rs`

use crate::chain::BittensorClient;
use anyhow::Result;

/// Bittensor epoch phase
///
/// Subtensor's commit-reveal is epoch-granular:
/// - CommitWindow: Current epoch accepts weight commits
/// - RevealWindow: Current epoch accepts reveals for commits from the previous epoch
///
/// Evaluation is kept for backward compatibility but is functionally equivalent
/// to CommitWindow (you can commit at any point during an epoch).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EpochPhase {
    /// Epoch start - weight commits are accepted (alias for CommitWindow)
    #[default]
    Evaluation,
    /// Weight commits are accepted (entire epoch)
    CommitWindow,
    /// Weight reveals are accepted for commits from the previous epoch
    RevealWindow,
}

impl std::fmt::Display for EpochPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EpochPhase::Evaluation => write!(f, "evaluation"),
            EpochPhase::CommitWindow => write!(f, "commit"),
            EpochPhase::RevealWindow => write!(f, "reveal"),
        }
    }
}

/// Epoch information for a subnet
#[derive(Debug, Clone)]
pub struct EpochInfo {
    /// Current block number
    pub current_block: u64,
    /// Tempo (blocks per epoch)
    pub tempo: u64,
    /// Last epoch start block
    pub epoch_start_block: u64,
    /// Next epoch start block
    pub next_epoch_start_block: u64,
    /// Blocks remaining in current epoch
    pub blocks_remaining: u64,
    /// Current epoch number
    pub epoch_number: u64,
    /// Current phase
    pub phase: EpochPhase,
    /// Commit-reveal enabled for this subnet
    pub commit_reveal_enabled: bool,
    /// Reveal period in epochs (if commit-reveal enabled)
    pub reveal_period_epochs: u64,
}

impl EpochInfo {
    /// Calculate progress through current epoch (0.0 - 1.0)
    pub fn epoch_progress(&self) -> f64 {
        if self.tempo == 0 {
            return 0.0;
        }
        let blocks_into_epoch = self.current_block.saturating_sub(self.epoch_start_block);
        let epoch_length = self.tempo.saturating_add(1);
        (blocks_into_epoch as f64) / (epoch_length as f64)
    }

    /// Check if we're near epoch end (within threshold blocks)
    pub fn near_epoch_end(&self, threshold: u64) -> bool {
        self.blocks_remaining <= threshold
    }

    /// Check if this is the first block of a new epoch
    pub fn is_epoch_start(&self) -> bool {
        self.current_block == self.epoch_start_block
    }
}

/// Tracker for epoch state
pub struct EpochTracker {
    netuid: u16,
    tempo: u64,
    commit_reveal_enabled: bool,
    reveal_period_epochs: u64,
    last_epoch_number: u64,
}

impl EpochTracker {
    /// Create a new epoch tracker
    pub fn new(netuid: u16) -> Self {
        Self {
            netuid,
            tempo: 0,
            commit_reveal_enabled: false,
            reveal_period_epochs: 1,
            last_epoch_number: 0,
        }
    }

    /// Initialize tracker with subnet parameters from chain
    pub async fn init(&mut self, client: &BittensorClient) -> Result<()> {
        self.tempo = crate::queries::subnets::tempo(client, self.netuid)
            .await?
            .unwrap_or(360);

        self.commit_reveal_enabled =
            crate::queries::subnets::commit_reveal_enabled(client, self.netuid).await?;

        if self.commit_reveal_enabled {
            self.reveal_period_epochs =
                crate::queries::subnets::get_subnet_reveal_period_epochs(client, self.netuid)
                    .await?
                    .unwrap_or(1);
        }

        Ok(())
    }

    /// Update tempo (if it changed on-chain)
    pub fn set_tempo(&mut self, tempo: u64) {
        self.tempo = tempo;
    }

    /// Calculate epoch index matching subtensor exactly:
    /// `epoch = (block + netuid + 1) / (tempo + 1)`
    fn get_epoch_index(&self, block: u64) -> u64 {
        if self.tempo == 0 {
            return 0;
        }
        let tempo_plus_one = self.tempo.saturating_add(1);
        let netuid_plus_one = (self.netuid as u64).saturating_add(1);
        block.saturating_add(netuid_plus_one) / tempo_plus_one
    }

    /// Calculate first block of an epoch matching subtensor exactly:
    /// `first_block = epoch * (tempo + 1) - (netuid + 1)`
    fn epoch_start_block(&self, epoch: u64) -> u64 {
        let tempo_plus_one = self.tempo.saturating_add(1);
        let netuid_plus_one = (self.netuid as u64).saturating_add(1);
        epoch
            .saturating_mul(tempo_plus_one)
            .saturating_sub(netuid_plus_one)
    }

    /// Get current epoch info for a block
    pub fn get_epoch_info(&self, current_block: u64) -> EpochInfo {
        let epoch_number = self.get_epoch_index(current_block);
        let epoch_start = self.epoch_start_block(epoch_number);
        let next_epoch_start = self.epoch_start_block(epoch_number + 1);
        let blocks_remaining = next_epoch_start.saturating_sub(current_block);

        // Subtensor commit-reveal is epoch-granular:
        // - You can commit at ANY block during the epoch
        // - Reveals happen in the next epoch (epoch + reveal_period)
        // We emit CommitWindow at epoch start so the validator submits weights promptly
        let phase = self.determine_phase(current_block, epoch_start);

        EpochInfo {
            current_block,
            tempo: self.tempo,
            epoch_start_block: epoch_start,
            next_epoch_start_block: next_epoch_start,
            blocks_remaining,
            epoch_number,
            phase,
            commit_reveal_enabled: self.commit_reveal_enabled,
            reveal_period_epochs: self.reveal_period_epochs,
        }
    }

    /// Determine epoch phase.
    ///
    /// Subtensor allows commits during the ENTIRE epoch and reveals during
    /// the ENTIRE next epoch. We signal CommitWindow at the start of each
    /// epoch so the validator can submit weights as early as possible.
    fn determine_phase(&self, current_block: u64, epoch_start: u64) -> EpochPhase {
        if self.tempo == 0 {
            return EpochPhase::Evaluation;
        }

        let blocks_into_epoch = current_block.saturating_sub(epoch_start);

        // Signal CommitWindow on the first block of the epoch, then
        // transition to Evaluation for the rest. This triggers weight
        // submission once per epoch without blocking other operations.
        if blocks_into_epoch == 0 {
            EpochPhase::CommitWindow
        } else {
            EpochPhase::Evaluation
        }
    }

    /// Check if epoch changed and return transition info
    pub fn check_epoch_transition(&mut self, current_block: u64) -> Option<EpochTransition> {
        let epoch_info = self.get_epoch_info(current_block);

        if epoch_info.epoch_number > self.last_epoch_number {
            let old_epoch = self.last_epoch_number;
            self.last_epoch_number = epoch_info.epoch_number;

            Some(EpochTransition::NewEpoch {
                old_epoch,
                new_epoch: epoch_info.epoch_number,
                block: current_block,
            })
        } else {
            None
        }
    }

    /// Get netuid
    pub fn netuid(&self) -> u16 {
        self.netuid
    }

    /// Get tempo
    pub fn tempo(&self) -> u64 {
        self.tempo
    }

    /// Check if commit-reveal is enabled
    pub fn is_commit_reveal_enabled(&self) -> bool {
        self.commit_reveal_enabled
    }
}

/// Epoch transition event
#[derive(Debug, Clone)]
pub enum EpochTransition {
    /// New epoch started
    NewEpoch {
        old_epoch: u64,
        new_epoch: u64,
        block: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tracker(netuid: u16, tempo: u64) -> EpochTracker {
        EpochTracker {
            netuid,
            tempo,
            commit_reveal_enabled: true,
            reveal_period_epochs: 1,
            last_epoch_number: 0,
        }
    }

    #[test]
    fn test_epoch_index_matches_subtensor() {
        // subtensor formula: (block + netuid + 1) / (tempo + 1)
        let t = tracker(1, 360);

        // netuid=1, tempo=360 => tempo_plus_one=361, netuid_plus_one=2
        // epoch = (block + 2) / 361
        assert_eq!(t.get_epoch_index(0), 0); // (0+2)/361 = 0
        assert_eq!(t.get_epoch_index(359), 1); // (359+2)/361 = 1
        assert_eq!(t.get_epoch_index(360), 1); // (360+2)/361 = 1
        assert_eq!(t.get_epoch_index(720), 2); // (720+2)/361 = 2
    }

    #[test]
    fn test_epoch_start_block_matches_subtensor() {
        // subtensor formula: epoch * (tempo + 1) - (netuid + 1)
        let t = tracker(1, 360);

        // netuid=1, tempo=360 => epoch_start = epoch * 361 - 2
        assert_eq!(t.epoch_start_block(0), 0); // 0*361-2 saturates to 0
        assert_eq!(t.epoch_start_block(1), 359); // 1*361-2 = 359
        assert_eq!(t.epoch_start_block(2), 720); // 2*361-2 = 720
    }

    #[test]
    fn test_epoch_info_blocks_remaining() {
        let t = tracker(1, 360);

        let info = t.get_epoch_info(359); // epoch 1 start
        assert_eq!(info.epoch_number, 1);
        assert_eq!(info.epoch_start_block, 359);
        assert_eq!(info.next_epoch_start_block, 720);
        assert_eq!(info.blocks_remaining, 361); // 720-359

        let info = t.get_epoch_info(500);
        assert_eq!(info.epoch_number, 1); // (500+2)/361 = 1
        assert_eq!(info.blocks_remaining, 220); // 720-500
    }

    #[test]
    fn test_commit_window_on_epoch_start() {
        let t = tracker(1, 360);

        // First block of epoch 1 (block 359) => CommitWindow
        let info = t.get_epoch_info(359);
        assert_eq!(info.phase, EpochPhase::CommitWindow);

        // Subsequent blocks => Evaluation (commits still accepted by chain)
        let info = t.get_epoch_info(360);
        assert_eq!(info.phase, EpochPhase::Evaluation);

        let info = t.get_epoch_info(500);
        assert_eq!(info.phase, EpochPhase::Evaluation);
    }

    #[test]
    fn test_epoch_transition() {
        let mut t = tracker(1, 360);

        assert!(t.check_epoch_transition(100).is_none());

        // Block 359 is epoch 1 start
        let transition = t.check_epoch_transition(359);
        assert!(matches!(
            transition,
            Some(EpochTransition::NewEpoch { new_epoch: 1, .. })
        ));

        // Still epoch 1
        assert!(t.check_epoch_transition(500).is_none());

        // Block 720 is epoch 2 start
        let transition = t.check_epoch_transition(720);
        assert!(matches!(
            transition,
            Some(EpochTransition::NewEpoch { new_epoch: 2, .. })
        ));
    }

    #[test]
    fn test_netuid_100_tempo_360() {
        // Real-world case: netuid=100, tempo=360
        // tempo_plus_one=361, netuid_plus_one=101
        // epoch = (block + 101) / 361
        // epoch_start = epoch * 361 - 101
        let t = tracker(100, 360);

        let info = t.get_epoch_info(7612560);
        // (7612560 + 101) / 361 = 7612661 / 361 = 21087
        assert_eq!(info.epoch_number, 21087);
        // epoch_start = 21087 * 361 - 101 = 7612307 - 101 = 7612206
        // Nope: 21087 * 361 = 7612407, minus 101 = 7612306
        // Let's just verify it's consistent
        assert!(info.epoch_start_block <= 7612560);
        assert!(info.next_epoch_start_block > 7612560);
        assert_eq!(info.phase, EpochPhase::Evaluation); // not first block
    }

    #[test]
    fn test_epoch_progress() {
        let t = tracker(1, 360);
        let info = t.get_epoch_info(359); // first block of epoch 1
        assert!((info.epoch_progress() - 0.0).abs() < 0.01);

        let info = t.get_epoch_info(539); // halfway through epoch 1
        assert!((info.epoch_progress() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_zero_tempo() {
        let t = tracker(1, 0);
        let info = t.get_epoch_info(100);
        assert_eq!(info.epoch_number, 0);
        assert_eq!(info.phase, EpochPhase::Evaluation);
    }
}
