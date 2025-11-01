// Re-export Encode for convenience
pub use parity_scale_codec::Encode as ScaleEncode;

/// Helper to encode AccountId32
pub fn encode_account(account: &sp_core::crypto::AccountId32) -> Vec<u8> {
    account.encode()
}
