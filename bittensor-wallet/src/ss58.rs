//! SS58 address encoding and decoding for Substrate-based chains.
//!
//! SS58 is a base58-encoded address format used in the Substrate ecosystem.
//! Bittensor uses format 42 (Substrate default).

use blake2::{Blake2b512, Digest};

/// The SS58 address format byte for Bittensor/Substrate default.
pub const SS58_FORMAT_BYTE: u8 = 42;

/// The length of the checksum appended to SS58 addresses.
const CHECKSUM_LEN: usize = 2;

/// Error type for SS58 encoding/decoding operations.
#[derive(Debug, thiserror::Error)]
pub enum Ss58Error {
    #[error("Invalid SS58 address: {0}")]
    InvalidAddress(String),
    #[error("Bad prefix byte: {0}")]
    BadPrefix(u8),
    #[error("Decoded length mismatch: expected {expected}, got {actual}")]
    LengthMismatch { expected: usize, actual: usize },
    #[error("Checksum verification failed")]
    InvalidChecksum,
    #[error("Base58 decode error: {0}")]
    Base58(#[from] bs58::decode::Error),
}

/// Encode a 32-byte public key as an SS58 address string with the given format byte.
///
/// The SS58 format is: `\[format_byte\]\[public_key_bytes\]\[checksum\]`
/// where checksum = blake2b512(\[format_byte\]\[public_key_bytes\])\[0..2\]
pub fn encode_ss58(public_key: &[u8; 32], format: u8) -> String {
    let mut data = Vec::with_capacity(1 + 32 + CHECKSUM_LEN);
    data.push(format);
    data.extend_from_slice(public_key);

    let checksum = blake2b_checksum(&data);
    data.extend_from_slice(&checksum[..CHECKSUM_LEN]);

    bs58::encode(&data).into_string()
}

/// Encode a 32-byte public key as an SS58 address with Bittensor default format (42).
pub fn encode_ss58_address(public_key: &[u8; 32]) -> String {
    encode_ss58(public_key, SS58_FORMAT_BYTE)
}

/// Decode an SS58 address string into its format byte and 32-byte public key.
pub fn decode_ss58(address: &str) -> Result<(u8, [u8; 32]), Ss58Error> {
    let decoded = bs58::decode(address).into_vec().map_err(Ss58Error::Base58)?;

    if decoded.len() < 1 + 32 + CHECKSUM_LEN {
        return Err(Ss58Error::LengthMismatch {
            expected: 1 + 32 + CHECKSUM_LEN,
            actual: decoded.len(),
        });
    }

    // Handle simple 1-byte prefix (format 0-63)
    let prefix_len = if decoded[0] < 64 { 1 } else { 2 };

    if decoded.len() < prefix_len + 32 + CHECKSUM_LEN {
        return Err(Ss58Error::LengthMismatch {
            expected: prefix_len + 32 + CHECKSUM_LEN,
            actual: decoded.len(),
        });
    }

    let format_byte = decoded[0];
    let public_key_bytes = &decoded[prefix_len..prefix_len + 32];
    let checksum_offset = prefix_len + 32;

    // Verify checksum: blake2b512(prefix + pubkey)[0..2]
    let mut checksum_input = decoded[..prefix_len].to_vec();
    checksum_input.extend_from_slice(public_key_bytes);
    let expected_checksum = blake2b_checksum(&checksum_input);

    let actual_checksum = &decoded[checksum_offset..checksum_offset + CHECKSUM_LEN];
    if &expected_checksum[..CHECKSUM_LEN] != actual_checksum {
        return Err(Ss58Error::InvalidChecksum);
    }

    let mut public_key = [0u8; 32];
    public_key.copy_from_slice(public_key_bytes);

    Ok((format_byte, public_key))
}

/// Compute the blake2b-512 checksum of the given data with the SS58 pre-image.
/// The SS58 spec requires: blake2b_512(b"SS58PRE" + data)[0..2]
fn blake2b_checksum(data: &[u8]) -> [u8; 64] {
    let mut hasher = Blake2b512::new();
    hasher.update(b"SS58PRE");
    hasher.update(data);
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_alice_polkadot_format_0() {
        let alice_pubkey: [u8; 32] = [
            212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214, 130, 44, 133,
            88, 133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162, 125,
        ];
        let address = encode_ss58(&alice_pubkey, 0);
        assert_eq!(address, "15oF4uVJwmo4TdGW7VfQxNLavjCXviqxT9S1MgbjMNHr6Sp5");
    }

    #[test]
    fn encode_alice_address_format_42() {
        let alice_pubkey: [u8; 32] = [
            0xd4, 0x35, 0x93, 0xc7, 0x15, 0xfd, 0xd3, 0x1c, 0x61, 0x14, 0x1a, 0xbd, 0x04, 0xa9,
            0x9f, 0xd6, 0x82, 0x2c, 0x85, 0x58, 0x85, 0x4c, 0xcd, 0xe3, 0x9a, 0x56, 0x84, 0xe7,
            0xa5, 0x6d, 0xa2, 0x7d,
        ];
        let address = encode_ss58_address(&alice_pubkey);
        assert_eq!(address, "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY");
    }

    #[test]
    fn round_trip_encode_decode() {
        let key = [42u8; 32];
        let address = encode_ss58_address(&key);
        let (format, decoded_key) = decode_ss58(&address).expect("decode failed");
        assert_eq!(format, SS58_FORMAT_BYTE);
        assert_eq!(decoded_key, key);
    }

    #[test]
    fn decode_invalid_checksum() {
        let mut data = vec![SS58_FORMAT_BYTE];
        data.extend_from_slice(&[0u8; 32]);
        data.extend_from_slice(&[0xFF, 0xFF]);
        let address = bs58::encode(&data).into_string();
        assert!(decode_ss58(&address).is_err());
    }

    #[test]
    fn encode_with_custom_format() {
        let key = [1u8; 32];
        let addr_0 = encode_ss58(&key, 0);
        let addr_42 = encode_ss58(&key, 42);
        assert_ne!(addr_0, addr_42);

        let (fmt, decoded) = decode_ss58(&addr_42).expect("decode failed");
        assert_eq!(fmt, 42);
        assert_eq!(decoded, key);
    }

    #[test]
    fn decode_alice_address_format_42() {
        let address = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
        let (format, pubkey) = decode_ss58(address).expect("decode failed");
        assert_eq!(format, 42);
        let alice_expected: [u8; 32] = [
            0xd4, 0x35, 0x93, 0xc7, 0x15, 0xfd, 0xd3, 0x1c, 0x61, 0x14, 0x1a, 0xbd, 0x04, 0xa9,
            0x9f, 0xd6, 0x82, 0x2c, 0x85, 0x58, 0x85, 0x4c, 0xcd, 0xe3, 0x9a, 0x56, 0x84, 0xe7,
            0xa5, 0x6d, 0xa2, 0x7d,
        ];
        assert_eq!(pubkey, alice_expected);
    }
}
