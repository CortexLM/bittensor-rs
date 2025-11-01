use anyhow::Result;
use sp_core::crypto::{AccountId32, Ss58AddressFormat, Ss58Codec};
use std::str::FromStr;

/// SS58 format constant for Bittensor (42 = "bt")
pub const SS58_FORMAT: u16 = 42;

/// Encode AccountId32 to SS58 string
pub fn encode_ss58(account: &AccountId32) -> String {
    account.to_ss58check_with_version(Ss58AddressFormat::custom(SS58_FORMAT as u16))
}

/// Decode SS58 string to AccountId32
pub fn decode_ss58(ss58: &str) -> Result<AccountId32> {
    AccountId32::from_str(ss58)
        .or_else(|_| {
            let (account, _format) = AccountId32::from_ss58check_with_version(ss58)?;
            Ok(account)
        })
        .map_err(|e: sp_core::crypto::PublicError| {
            anyhow::anyhow!("Failed to decode SS58 address: {:?}", e)
        })
}

/// Validate SS58 address format
pub fn is_valid_ss58(ss58: &str) -> bool {
    decode_ss58(ss58).is_ok()
}
