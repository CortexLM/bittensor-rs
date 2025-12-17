//! Delegate information types

use serde::{Deserialize, Serialize};

use crate::utils::balance::Balance;

/// Full delegate information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegateInfo {
    /// Delegate hotkey SS58 address
    pub hotkey: String,
    /// Total stake delegated to this delegate
    pub total_stake: Balance,
    /// Number of nominators
    pub nominator_count: u64,
    /// Owner coldkey SS58 address
    pub owner: String,
    /// Take percentage (0-65535, representing 0-100%)
    pub take: u16,
    /// Validator permits by subnet
    pub validator_permits: Vec<u16>,
    /// Registrations by subnet
    pub registrations: Vec<u16>,
    /// Return per 1000 TAO (daily)
    pub return_per_1000: Balance,
    /// Total daily return
    pub total_daily_return: Balance,
}

impl DelegateInfo {
    /// Get take as percentage (0.0 - 1.0)
    pub fn take_percentage(&self) -> f64 {
        self.take as f64 / u16::MAX as f64
    }

    /// Check if delegate is registered on a subnet
    pub fn is_registered(&self, netuid: u16) -> bool {
        self.registrations.contains(&netuid)
    }

    /// Check if delegate has validator permit on a subnet
    pub fn has_validator_permit(&self, netuid: u16) -> bool {
        self.validator_permits.contains(&netuid)
    }
}

impl std::fmt::Display for DelegateInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DelegateInfo( hotkey={}, stake={}, nominators={}, take={:.2}% )",
            self.hotkey,
            self.total_stake,
            self.nominator_count,
            self.take_percentage() * 100.0
        )
    }
}

/// Lite version of delegate info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegateInfoLite {
    /// Delegate hotkey SS58 address
    pub hotkey: String,
    /// Total stake delegated to this delegate
    pub total_stake: Balance,
    /// Owner coldkey SS58 address
    pub owner: String,
    /// Take percentage (0-65535)
    pub take: u16,
}

impl DelegateInfoLite {
    /// Get take as percentage (0.0 - 1.0)
    pub fn take_percentage(&self) -> f64 {
        self.take as f64 / u16::MAX as f64
    }
}

impl std::fmt::Display for DelegateInfoLite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DelegateInfoLite( hotkey={}, stake={}, take={:.2}% )",
            self.hotkey,
            self.total_stake,
            self.take_percentage() * 100.0
        )
    }
}

/// Delegated stake info (stake from a nominator to a delegate)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegatedInfo {
    /// Delegate hotkey
    pub delegate: String,
    /// Amount staked
    pub stake: Balance,
}
