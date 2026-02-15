//! DRAND integration for CRv4
//!
//! DRAND (Distributed Randomness Beacon) provides the randomness used for
//! timelock encryption. We use the Quicknet network.
//!
//! IMPORTANT: The chain's DRAND pallet may be behind real-world time.
//! Always use `calculate_reveal_round` with the chain's `LastStoredRound`
//! to ensure reveal_rounds are within the chain's DRAND range.

use serde::{Deserialize, Serialize};

/// DRAND Quicknet public key (hex encoded, 96 bytes compressed G2 point)
pub const DRAND_QUICKNET_PK_HEX: &str = "83cf0f2896adee7eb8b5f01fcad3912212c437e0073e911fb90022d3e760183c8c4b450b6a0a6c3ac6a5776a2d1064510d1fec758c921cc22b0e17e63aaf4bcb5ed66304de9cf809bd274ca73bab4af5a6e9c76a4bc09e76eae8991ef5ece45a";

/// DRAND Quicknet genesis timestamp (Unix seconds)
/// Quicknet genesis: 2023-07-03 12:00:00 UTC
pub const DRAND_QUICKNET_GENESIS: u64 = 1688385600;

/// DRAND round interval in seconds (Quicknet = 3 seconds)
pub const DRAND_ROUND_INTERVAL_SECS: u64 = 3;

/// Information about DRAND beacon
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DrandInfo {
    /// Public key bytes (G2 point on BLS12-381)
    pub public_key: Vec<u8>,
    /// Genesis timestamp
    pub genesis_time: u64,
    /// Round interval in seconds
    pub period: u64,
}

impl Default for DrandInfo {
    fn default() -> Self {
        Self::quicknet()
    }
}

impl DrandInfo {
    /// Create DrandInfo with Quicknet parameters
    pub fn quicknet() -> Self {
        Self {
            public_key: hex::decode(DRAND_QUICKNET_PK_HEX).unwrap_or_default(),
            genesis_time: DRAND_QUICKNET_GENESIS,
            period: DRAND_ROUND_INTERVAL_SECS,
        }
    }

    /// Get the DRAND round number for a given Unix timestamp
    pub fn round_at_time(&self, timestamp: u64) -> u64 {
        if timestamp < self.genesis_time {
            return 1;
        }
        ((timestamp - self.genesis_time) / self.period) + 1
    }

    /// Get the Unix timestamp when a round becomes available
    pub fn time_for_round(&self, round: u64) -> u64 {
        if round <= 1 {
            return self.genesis_time;
        }
        self.genesis_time + (round - 1) * self.period
    }

    /// Get current DRAND round based on system time
    pub fn current_round(&self) -> u64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.round_at_time(now)
    }
}

/// Calculate the reveal round for CRv4 commits
///
/// IMPORTANT: This function calculates reveal_round relative to the chain's
/// LastStoredRound, NOT real-world time. The chain's DRAND pallet may be
/// significantly behind real time (e.g., 51 days), so using system time would
/// produce reveal_rounds that don't exist on chain yet.
///
/// # Arguments
/// * `tempo` - Number of blocks in one epoch
/// * `current_block` - Current block number
/// * `netuid` - Network/mechanism storage index (netuid or mechid * 4096 + netuid)
/// * `subnet_reveal_period_epochs` - Number of epochs until reveal
/// * `block_time` - Block time in seconds (default 12.0)
/// * `chain_last_drand_round` - The chain's Drand.LastStoredRound value
///
/// # Returns
/// The DRAND round number when the reveal should occur
pub fn calculate_reveal_round(
    tempo: u16,
    current_block: u64,
    netuid: u16,
    subnet_reveal_period_epochs: u64,
    block_time: f64,
    chain_last_drand_round: u64,
) -> u64 {
    let tempo = tempo as u64;
    let netuid = netuid as u64;

    // Calculate current epoch (same formula as subtensor)
    // epoch = (current_block + netuid + 1) / (tempo + 1)
    let tempo_plus_one = tempo.saturating_add(1);
    let netuid_plus_one = netuid.saturating_add(1);
    let current_epoch = current_block.saturating_add(netuid_plus_one) / tempo_plus_one;

    // Reveal epoch = current_epoch + reveal_period
    if subnet_reveal_period_epochs == 0 {
        return chain_last_drand_round.saturating_add(1);
    }

    let reveal_epoch = current_epoch.saturating_add(subnet_reveal_period_epochs);

    // First block of reveal epoch
    // first_block = reveal_epoch * (tempo + 1) - (netuid + 1)
    let first_reveal_block = reveal_epoch
        .saturating_mul(tempo_plus_one)
        .saturating_sub(netuid_plus_one);

    // Calculate time until reveal in seconds
    let blocks_until_reveal = first_reveal_block.saturating_sub(current_block);
    let secs_until_reveal = (blocks_until_reveal as f64 * block_time) as u64;

    // Calculate how many DRAND rounds until reveal
    // DRAND rounds occur every 3 seconds
    let drand_rounds_until_reveal = secs_until_reveal / DRAND_ROUND_INTERVAL_SECS;

    // Calculate reveal_round relative to chain's last stored round
    // Add +1 buffer for safety
    let reveal_round = chain_last_drand_round
        .saturating_add(drand_rounds_until_reveal)
        .saturating_add(1);

    tracing::info!(
        "CRv4 reveal round calculation: tempo={}, current_block={}, netuid={}, \
         reveal_period={}, current_epoch={}, reveal_epoch={}, \
         blocks_until_reveal={}, chain_last_drand_round={}, reveal_round={}",
        tempo,
        current_block,
        netuid,
        subnet_reveal_period_epochs,
        current_epoch,
        reveal_epoch,
        blocks_until_reveal,
        chain_last_drand_round,
        reveal_round
    );

    reveal_round
}

/// Calculate reveal round with explicit epoch information
///
/// This is useful when you already know the epoch boundaries.
///
/// # Arguments
/// * `reveal_epoch` - The epoch when weights should be revealed
/// * `tempo` - Number of blocks in one epoch
/// * `netuid` - Network/mechanism storage index
/// * `current_block` - Current block number
/// * `block_time` - Block time in seconds
/// * `chain_last_drand_round` - The chain's Drand.LastStoredRound value
pub fn calculate_reveal_round_for_epoch(
    reveal_epoch: u64,
    tempo: u16,
    netuid: u16,
    current_block: u64,
    block_time: f64,
    chain_last_drand_round: u64,
) -> u64 {
    let tempo = tempo as u64;
    let netuid = netuid as u64;

    let tempo_plus_one = tempo.saturating_add(1);
    let netuid_plus_one = netuid.saturating_add(1);

    // First block of reveal epoch
    let first_reveal_block = reveal_epoch
        .saturating_mul(tempo_plus_one)
        .saturating_sub(netuid_plus_one);

    // Calculate blocks until reveal
    let blocks_until_reveal = first_reveal_block.saturating_sub(current_block);
    let secs_until_reveal = (blocks_until_reveal as f64 * block_time) as u64;

    // Calculate DRAND rounds until reveal
    let drand_rounds_until_reveal = secs_until_reveal / DRAND_ROUND_INTERVAL_SECS;

    // Return reveal_round relative to chain's last stored round
    chain_last_drand_round
        .saturating_add(drand_rounds_until_reveal)
        .saturating_add(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drand_round_calculation() {
        let info = DrandInfo::quicknet();

        // Test round at genesis
        assert_eq!(info.round_at_time(info.genesis_time), 1);

        // Test round 3 seconds after genesis
        assert_eq!(info.round_at_time(info.genesis_time + 3), 2);

        // Test round 6 seconds after genesis
        assert_eq!(info.round_at_time(info.genesis_time + 6), 3);
    }

    #[test]
    fn test_time_for_round() {
        let info = DrandInfo::quicknet();

        assert_eq!(info.time_for_round(1), info.genesis_time);
        assert_eq!(info.time_for_round(2), info.genesis_time + 3);
        assert_eq!(info.time_for_round(1000), info.genesis_time + 999 * 3);
    }

    #[test]
    fn test_quicknet_public_key() {
        let pk_bytes = hex::decode(DRAND_QUICKNET_PK_HEX).expect("Valid hex");
        assert_eq!(pk_bytes.len(), 96); // G2 compressed is 96 bytes
    }

    #[test]
    fn test_reveal_round_calculation() {
        let tempo = 360u16;
        let current_block = 1000u64;
        let netuid = 1u16;
        let reveal_period = 1u64;
        let block_time = 12.0;
        let chain_last_drand_round = 24_405_700u64; // Example chain state

        let reveal_round = calculate_reveal_round(
            tempo,
            current_block,
            netuid,
            reveal_period,
            block_time,
            chain_last_drand_round,
        );

        // Should be greater than chain's last stored round
        assert!(reveal_round > chain_last_drand_round);

        // Should be within reasonable range (not millions of rounds ahead)
        // With tempo=360, reveal_period=1, blocks_until_reveal ~= 360 blocks
        // 360 blocks * 12 sec/block = 4320 secs, / 3 sec/round = 1440 rounds
        let expected_rounds_ahead = (360 * 12 / 3) + 1; // ~1441
        assert!(reveal_round < chain_last_drand_round + expected_rounds_ahead + 100);
    }

    #[test]
    fn test_reveal_round_relative_to_chain() {
        // Test that reveal_round is calculated relative to chain state
        let tempo = 360u16;
        let current_block = 5000u64;
        let netuid = 1u16;
        let reveal_period = 1u64;
        let block_time = 12.0;

        // Two different chain states
        let chain_round_1 = 24_000_000u64;
        let chain_round_2 = 25_000_000u64;

        let reveal_1 = calculate_reveal_round(
            tempo,
            current_block,
            netuid,
            reveal_period,
            block_time,
            chain_round_1,
        );
        let reveal_2 = calculate_reveal_round(
            tempo,
            current_block,
            netuid,
            reveal_period,
            block_time,
            chain_round_2,
        );

        // The difference should be exactly 1_000_000 (the difference in chain state)
        assert_eq!(reveal_2 - reveal_1, chain_round_2 - chain_round_1);
    }
}
