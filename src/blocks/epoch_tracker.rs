//! Epoch tracking for Bittensor subnets
//!
//! Tracks epoch boundaries and phases for weight commit-reveal.

use crate::chain::BittensorClient;
use anyhow::Result;

/// Epoch phase in Bittensor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpochPhase {
    /// Normal operation / evaluation phase
    Evaluation,
    /// Weight commit window is open
    CommitWindow,
    /// Weight reveal window is open  
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
        (blocks_into_epoch as f64) / (self.tempo as f64)
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
        // Get tempo
        self.tempo = crate::queries::subnets::tempo(client, self.netuid)
            .await?
            .unwrap_or(360); // Default tempo

        // Check if commit-reveal is enabled
        self.commit_reveal_enabled =
            crate::queries::subnets::commit_reveal_enabled(client, self.netuid).await?;

        // Get reveal period if commit-reveal enabled
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

    /// Get current epoch info for a block
    pub fn get_epoch_info(&self, current_block: u64) -> EpochInfo {
        let epoch_number = if self.tempo > 0 {
            current_block / self.tempo
        } else {
            0
        };

        let epoch_start_block = epoch_number * self.tempo;
        let next_epoch_start_block = (epoch_number + 1) * self.tempo;
        let blocks_remaining = next_epoch_start_block.saturating_sub(current_block);

        // Determine phase based on position in epoch
        // Typically: last ~10% is reveal window, before that is commit window
        let phase = self.determine_phase(current_block, epoch_start_block);

        EpochInfo {
            current_block,
            tempo: self.tempo,
            epoch_start_block,
            next_epoch_start_block,
            blocks_remaining,
            epoch_number,
            phase,
            commit_reveal_enabled: self.commit_reveal_enabled,
            reveal_period_epochs: self.reveal_period_epochs,
        }
    }

    /// Determine the current phase based on block position
    fn determine_phase(&self, current_block: u64, epoch_start_block: u64) -> EpochPhase {
        if !self.commit_reveal_enabled {
            return EpochPhase::Evaluation;
        }

        let blocks_into_epoch = current_block.saturating_sub(epoch_start_block);

        // Bittensor commit-reveal windows:
        // - Commit window: blocks before reveal window
        // - Reveal window: last portion of epoch
        // The exact timing depends on subnet parameters

        // Default: last 25% of epoch is commit/reveal
        // Last 12.5% is reveal, 12.5% before that is commit
        let reveal_start = (self.tempo * 7) / 8; // 87.5%
        let commit_start = (self.tempo * 3) / 4; // 75%

        if blocks_into_epoch >= reveal_start {
            EpochPhase::RevealWindow
        } else if blocks_into_epoch >= commit_start {
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

    #[test]
    fn test_epoch_info() {
        let tracker = EpochTracker {
            netuid: 1,
            tempo: 360,
            commit_reveal_enabled: true,
            reveal_period_epochs: 1,
            last_epoch_number: 0,
        };

        // Block 0 - start of epoch 0
        let info = tracker.get_epoch_info(0);
        assert_eq!(info.epoch_number, 0);
        assert_eq!(info.blocks_remaining, 360);
        assert_eq!(info.phase, EpochPhase::Evaluation);

        // Block 270 - in commit window (75%)
        let info = tracker.get_epoch_info(270);
        assert_eq!(info.epoch_number, 0);
        assert_eq!(info.phase, EpochPhase::CommitWindow);

        // Block 315 - in reveal window (87.5%)
        let info = tracker.get_epoch_info(315);
        assert_eq!(info.epoch_number, 0);
        assert_eq!(info.phase, EpochPhase::RevealWindow);

        // Block 360 - start of epoch 1
        let info = tracker.get_epoch_info(360);
        assert_eq!(info.epoch_number, 1);
        assert_eq!(info.blocks_remaining, 360);
    }

    #[test]
    fn test_epoch_transition() {
        let mut tracker = EpochTracker {
            netuid: 1,
            tempo: 360,
            commit_reveal_enabled: false,
            reveal_period_epochs: 1,
            last_epoch_number: 0,
        };

        // No transition at block 100
        assert!(tracker.check_epoch_transition(100).is_none());

        // Transition at block 360
        let transition = tracker.check_epoch_transition(360);
        assert!(matches!(
            transition,
            Some(EpochTransition::NewEpoch { new_epoch: 1, .. })
        ));

        // No transition at block 400 (same epoch)
        assert!(tracker.check_epoch_transition(400).is_none());
    }
}
