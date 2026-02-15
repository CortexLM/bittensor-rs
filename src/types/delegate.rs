use serde::{Deserialize, Serialize};
use sp_core::crypto::AccountId32;

/// Base delegate information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegateInfoBase {
    /// Hotkey of delegate (SS58)
    #[serde(with = "crate::utils::ss58::serde_account")]
    pub hotkey_ss58: AccountId32,
    /// Coldkey of owner (SS58)
    #[serde(with = "crate::utils::ss58::serde_account")]
    pub owner_ss58: AccountId32,
    /// Take of the delegate as a percentage (normalized)
    pub take: f64,
    /// List of subnets that the delegate is allowed to validate on
    pub validator_permits: Vec<u16>,
    /// List of subnets that the delegate is registered on
    pub registrations: Vec<u16>,
    /// Return per 1000 TAO of the delegate over a day (RAO)
    pub return_per_1000: u128,
    /// Total daily return of the delegate
    pub total_daily_return: u128,
}

/// Complete delegate information with stake details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegateInfo {
    /// Base delegate information
    #[serde(flatten)]
    pub base: DelegateInfoBase,
    /// Total stake of the delegate mapped by netuid
    pub total_stake: std::collections::HashMap<u16, u128>,
    /// Mapping of nominator SS58 addresses to their stakes per subnet
    #[serde(with = "crate::utils::ss58::serde_account_map")]
    pub nominators: std::collections::HashMap<AccountId32, std::collections::HashMap<u16, u128>>,
}

/// Delegated information specific to a subnet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegatedInfo {
    /// Base delegate information
    #[serde(flatten)]
    pub base: DelegateInfoBase,
    /// Network ID of the subnet
    pub netuid: u16,
    /// Stake amount for this specific delegation
    pub stake: u128,
}

impl DelegateInfo {
    pub fn new(hotkey: AccountId32, owner: AccountId32, take: f64) -> Self {
        Self {
            base: DelegateInfoBase {
                hotkey_ss58: hotkey,
                owner_ss58: owner,
                take,
                validator_permits: vec![],
                registrations: vec![],
                return_per_1000: 0,
                total_daily_return: 0,
            },
            total_stake: std::collections::HashMap::new(),
            nominators: std::collections::HashMap::new(),
        }
    }
}
