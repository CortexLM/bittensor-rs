//! SS58 address encoding/decoding utilities
//!
//! SS58 is the address format used by Substrate-based chains including Bittensor.

use crate::config::SS58_FORMAT;
use crate::error::{Error, Result};
use sp_core::crypto::{AccountId32, Ss58AddressFormat, Ss58Codec};

/// Encode a 32-byte public key to SS58 address
pub fn ss58_encode(public_key: &[u8; 32]) -> String {
    let account = AccountId32::from(*public_key);
    account.to_ss58check_with_version(Ss58AddressFormat::custom(SS58_FORMAT))
}

/// Decode an SS58 address to 32-byte public key
pub fn ss58_decode(address: &str) -> Result<[u8; 32]> {
    let account = AccountId32::from_ss58check(address)
        .map_err(|e| Error::invalid_address(format!("Invalid SS58 address: {}", e)))?;
    Ok(account.into())
}

/// Check if an address is a valid SS58 address
pub fn is_valid_ss58_address(address: &str) -> bool {
    AccountId32::from_ss58check(address).is_ok()
}

/// Convert AccountId32 to SS58 string
pub fn account_to_ss58(account: &AccountId32) -> String {
    account.to_ss58check_with_version(Ss58AddressFormat::custom(SS58_FORMAT))
}

/// Convert bytes to AccountId32
pub fn bytes_to_account(bytes: &[u8]) -> Result<AccountId32> {
    if bytes.len() != 32 {
        return Err(Error::invalid_address(format!(
            "Invalid public key length: expected 32, got {}",
            bytes.len()
        )));
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(bytes);
    Ok(AccountId32::from(arr))
}

/// Convert hex string (with or without 0x prefix) to AccountId32
pub fn hex_to_account(hex: &str) -> Result<AccountId32> {
    let hex = hex.trim_start_matches("0x");
    let bytes = hex::decode(hex)
        .map_err(|e| Error::invalid_address(format!("Invalid hex: {}", e)))?;
    bytes_to_account(&bytes)
}

/// Convert AccountId32 to hex string (without 0x prefix)
pub fn account_to_hex(account: &AccountId32) -> String {
    let bytes: &[u8; 32] = account.as_ref();
    hex::encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ss58_roundtrip() {
        let pubkey = [1u8; 32];
        let address = ss58_encode(&pubkey);
        let decoded = ss58_decode(&address).unwrap();
        assert_eq!(pubkey, decoded);
    }

    #[test]
    fn test_is_valid_ss58() {
        // Valid address (any Substrate address)
        assert!(is_valid_ss58_address(
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
        ));

        // Invalid address
        assert!(!is_valid_ss58_address("invalid"));
        assert!(!is_valid_ss58_address(""));
    }

    #[test]
    fn test_hex_conversion() {
        let hex = "0x0101010101010101010101010101010101010101010101010101010101010101";
        let account = hex_to_account(hex).unwrap();
        let back = account_to_hex(&account);
        assert_eq!(hex.trim_start_matches("0x"), back);
    }
}
