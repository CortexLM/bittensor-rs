use anyhow::Result;
use hex;
use sp_core::blake2_256;

/// Generate a commit hash for weights using Blake2b
pub fn commit_weights_hash(uids: &[u64], weights: &[u64], salt: &[u8]) -> Vec<u8> {
    // Serialize UIDs, weights, and salt
    let mut data = Vec::new();
    for uid in uids {
        data.extend_from_slice(&uid.to_le_bytes());
    }
    for weight in weights {
        data.extend_from_slice(&weight.to_le_bytes());
    }
    data.extend_from_slice(salt);

    // Hash with Blake2b
    blake2_256(&data).to_vec()
}

/// Convert commit hash to hex string
pub fn commit_hash_to_hex(hash: &[u8]) -> String {
    hex::encode(hash)
}

/// Parse hex string to commit hash
pub fn hex_to_commit_hash(hex_str: &str) -> Result<Vec<u8>> {
    hex::decode(hex_str).map_err(|e| anyhow::anyhow!("Invalid hex string: {}", e))
}
