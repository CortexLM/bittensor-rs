//! SHA3-256 body hashing matching Python's `hashlib.sha3_256`.

use sha3::{Digest, Sha3_256};

/// Computes the FIPS 202 SHA3-256 hash of the input bytes and returns
/// the hexadecimal digest string.
///
/// This is the same algorithm as Python's `hashlib.sha3_256(data).hexdigest()`.
/// It uses the FIPS 202 variant, NOT Keccak-256 (which is Ethereum's variant).
pub fn sha3_256_hex(data: &[u8]) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes.as_ref().iter().map(|b| format!("{b:02x}")).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string_sha3_256() {
        // NIST test vector: SHA3-256("") = a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a
        let hash = sha3_256_hex(b"");
        assert_eq!(hash, "a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a");
    }

    #[test]
    fn abc_sha3_256() {
        // NIST test vector: SHA3-256("abc")
        let hash = sha3_256_hex(b"abc");
        assert_eq!(hash, "3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532");
    }

    #[test]
    fn json_body_sha3_256() {
        let body = br#"{"prompt":"hello","max_tokens":100}"#;
        let hash = sha3_256_hex(body);
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn consistent_and_different_inputs() {
        let hash1 = sha3_256_hex(b"test data");
        let hash2 = sha3_256_hex(b"test data");
        assert_eq!(hash1, hash2);
        let hash3 = sha3_256_hex(b"test datb");
        assert_ne!(hash1, hash3);
    }
}
