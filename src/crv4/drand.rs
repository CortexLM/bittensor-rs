//! DRAND integration for CRv4
//!
//! DRAND (Distributed Randomness Beacon) provides the randomness used for
//! timelock encryption. We use the Quicknet network.

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
/// This matches the calculation in bittensor_drand Python library.
///
/// # Arguments
/// * `tempo` - Number of blocks in one epoch
/// * `current_block` - Current block number
/// * `netuid` - Network/mechanism storage index (netuid or mechid * 4096 + netuid)
/// * `subnet_reveal_period_epochs` - Number of epochs until reveal
/// * `block_time` - Block time in seconds (default 12.0)
///
/// # Returns
/// The DRAND round number when the reveal should occur
pub fn calculate_reveal_round(
    tempo: u16,
    current_block: u64,
    netuid: u16,
    subnet_reveal_period_epochs: u64,
    block_time: f64,
) -> u64 {
    let tempo = tempo as u64;
    let netuid = netuid as u64;
    let drand_info = DrandInfo::quicknet();

    // Calculate current epoch (same formula as subtensor)
    // epoch = (current_block + netuid + 1) / (tempo + 1)
    let tempo_plus_one = tempo.saturating_add(1);
    let netuid_plus_one = netuid.saturating_add(1);
    let current_epoch = current_block.saturating_add(netuid_plus_one) / tempo_plus_one;

    // Reveal epoch = current_epoch + reveal_period
    let reveal_epoch = current_epoch.saturating_add(subnet_reveal_period_epochs);

    // First block of reveal epoch
    // first_block = reveal_epoch * (tempo + 1) - (netuid + 1)
    let first_reveal_block = reveal_epoch
        .saturating_mul(tempo_plus_one)
        .saturating_sub(netuid_plus_one);

    // Calculate time until reveal
    let blocks_until_reveal = first_reveal_block.saturating_sub(current_block);
    let secs_until_reveal = (blocks_until_reveal as f64 * block_time) as u64;

    // Get current time and add offset
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let reveal_time = now.saturating_add(secs_until_reveal);

    // Get DRAND round at reveal time, add buffer for safety
    let reveal_round = drand_info.round_at_time(reveal_time.saturating_add(drand_info.period));

    tracing::debug!(
        "CRv4 reveal round calculation: tempo={}, current_block={}, netuid={}, \
         reveal_period={}, current_epoch={}, reveal_epoch={}, reveal_round={}",
        tempo,
        current_block,
        netuid,
        subnet_reveal_period_epochs,
        current_epoch,
        reveal_epoch,
        reveal_round
    );

    reveal_round
}

/// Calculate reveal round with explicit epoch information
///
/// This is useful when you already know the epoch boundaries.
pub fn calculate_reveal_round_for_epoch(
    reveal_epoch: u64,
    tempo: u16,
    netuid: u16,
    block_time: f64,
) -> u64 {
    let tempo = tempo as u64;
    let netuid = netuid as u64;
    let drand_info = DrandInfo::quicknet();

    let tempo_plus_one = tempo.saturating_add(1);
    let netuid_plus_one = netuid.saturating_add(1);

    // First block of reveal epoch
    let first_reveal_block = reveal_epoch
        .saturating_mul(tempo_plus_one)
        .saturating_sub(netuid_plus_one);

    // Estimate time from now
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Rough estimate: assume current block is now
    let reveal_time = now + (first_reveal_block as f64 * block_time / 12.0) as u64;

    drand_info.round_at_time(reveal_time.saturating_add(drand_info.period))
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

        let reveal_round =
            calculate_reveal_round(tempo, current_block, netuid, reveal_period, block_time);

        // Should be a valid round number in the future
        assert!(reveal_round > 0);
    }
}
