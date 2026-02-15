//! Wallet management for Bittensor.
//!
//! This module provides comprehensive wallet functionality compatible with the
//! Python Bittensor SDK, including:
//!
//! - **Mnemonic generation and recovery** (BIP39)
//! - **Keypair management** (SR25519)
//! - **Keyfile encryption and storage** (Argon2id + NaCl secretbox)
//! - **Wallet creation and management** (coldkey/hotkey)
//!
//! ## Quick Start
//!
//! ### Create a new wallet
//!
//! ```no_run
//! use bittensor_rs::wallet::Wallet;
//!
//! // Create a new wallet with encrypted keys
//! let wallet = Wallet::create("my_wallet", "default", Some("password")).unwrap();
//!
//! // Get the SS58 addresses
//! let coldkey_addr = wallet.coldkey_ss58(Some("password")).unwrap();
//! let hotkey_addr = wallet.hotkey_ss58(Some("password")).unwrap();
//!
//! println!("Coldkey: {}", coldkey_addr);
//! println!("Hotkey: {}", hotkey_addr);
//! ```
//!
//! ### Recover a wallet from mnemonic
//!
//! ```no_run
//! use bittensor_rs::wallet::Wallet;
//!
//! let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
//! let wallet = Wallet::regenerate_coldkey("recovered", mnemonic, Some("password")).unwrap();
//! ```
//!
//! ### Generate a mnemonic
//!
//! ```
//! use bittensor_rs::wallet::Mnemonic;
//!
//! // Generate 12-word mnemonic
//! let mnemonic = Mnemonic::generate();
//! println!("Save this: {}", mnemonic.phrase());
//!
//! // Generate 24-word mnemonic for extra security
//! let mnemonic24 = Mnemonic::generate_with_words(24).unwrap();
//! ```
//!
//! ### Sign and verify messages
//!
//! ```
//! use bittensor_rs::wallet::Keypair;
//!
//! let keypair = Keypair::generate();
//! let message = b"Hello, Bittensor!";
//!
//! let signature = keypair.sign(message);
//! assert!(keypair.verify(message, &signature));
//! ```
//!
//! ## Keyfile Format
//!
//! This module uses keyfile formats compatible with the Python Bittensor SDK:
//!
//! ### Encrypted Keyfile (Binary)
//! Uses NaCl binary format with `$NACL` header:
//! ```text
//! +--------+--------+---------+-----------+
//! |$NACL   |  Salt  |  Nonce  | Ciphertext|
//! |5 bytes |16 bytes| 24 bytes| variable  |
//! +--------+--------+---------+-----------+
//! ```
//! - Salt: 16 random bytes for Argon2id key derivation
//! - Nonce: 24 random bytes for XSalsa20-Poly1305 encryption
//! - Argon2id params: memory=64MiB, iterations=2, parallelism=1
//!
//! ### Unencrypted Keyfile (JSON)
//! ```json
//! {
//!     "ss58Address": "5EPCUjPxiHAcNooYipQFWr9NmmXJKpNG5RhcntXwbtUySrgH",
//!     "publicKey": "0x66933bd1f37070ef87bd1198af3dacceb095237f803f3d32b173e6b425ed7972",
//!     "privateKey": "0x2ec306fc1c5bc2f0e3a2c7a6ec6014ca4a0823a7d7d42ad5e9d7f376a1c36c0d14a2ddb1ef1df4adba49f3a4d8c0f6205117907265f09a53ccf07a4e8616dfd8",
//!     "secretSeed": "0x4ed8d4b17698ddeaa1f1559f152f87b5d472f725ca86d341bd0276f1b61197e2"
//! }
//! ```
//!
//! ## Security Notes
//!
//! - All sensitive data (seeds, private keys, mnemonics) is securely zeroed from
//!   memory when dropped using the `zeroize` crate.
//! - Keyfiles use Argon2id for key derivation (memory-hard, resistant to GPU attacks)
//! - Encryption uses XSalsa20-Poly1305 (NaCl secretbox)
//! - File permissions are set to 0600 on Unix systems

pub mod keyfile;
pub mod keypair;
pub mod mnemonic;
#[allow(clippy::module_inception)]
pub mod wallet;

// Re-export main types at module level
pub use keyfile::{
    is_legacy_format, migrate_legacy_keyfile, Keyfile, KeyfileData, KeyfileError, KeyfileJsonData,
    NACL_HEADER,
};
pub use keypair::{Keypair, KeypairError, BITTENSOR_SS58_FORMAT};
pub use mnemonic::{Mnemonic, MnemonicError};
pub use wallet::{
    default_wallet_path, list_wallets, list_wallets_at, wallet_path, Wallet, WalletError,
};

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_full_wallet_workflow() {
        let dir = tempdir().unwrap();
        let base_path = dir.path().to_str().unwrap();

        // Generate mnemonic
        let coldkey_mnemonic = Mnemonic::generate();
        let hotkey_mnemonic = Mnemonic::generate();

        // Create wallet with mnemonics
        let mut wallet = Wallet::new("test_wallet", "default", Some(base_path)).unwrap();
        wallet
            .create_coldkey(Some("password"), Some(coldkey_mnemonic.phrase()), false)
            .unwrap();
        wallet
            .create_hotkey(Some("password"), Some(hotkey_mnemonic.phrase()), false)
            .unwrap();

        // Verify wallet exists
        assert!(wallet.exists());

        // Get keypairs
        let coldkey = wallet.coldkey_keypair(Some("password")).unwrap();
        let hotkey = wallet.hotkey_keypair(Some("password")).unwrap();

        // Verify addresses
        assert!(!coldkey.ss58_address().is_empty());
        assert!(!hotkey.ss58_address().is_empty());

        // Sign and verify
        let message = b"test message";
        let signature = coldkey.sign(message);
        assert!(coldkey.verify(message, &signature));

        // Recover wallet with same mnemonic
        let mut recovered = Wallet::new("recovered", "default", Some(base_path)).unwrap();
        recovered
            .create_coldkey(Some("password"), Some(coldkey_mnemonic.phrase()), false)
            .unwrap();

        let recovered_coldkey = recovered.coldkey_keypair(Some("password")).unwrap();
        assert_eq!(coldkey.ss58_address(), recovered_coldkey.ss58_address());
    }

    #[test]
    fn test_keyfile_python_compatibility() {
        // This test verifies the keyfile format matches Python SDK (bittensor-wallet)
        // Python SDK uses binary format with $NACL header for encrypted files
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_keyfile");

        let keypair = Keypair::generate();
        let mut keyfile = Keyfile::new(&path);
        keyfile
            .set_keypair(keypair.clone(), Some("password"), false)
            .unwrap();

        // Read the raw bytes
        let content = std::fs::read(&path).unwrap();

        // Verify binary format with $NACL header (Python SDK compatible)
        assert!(
            content.starts_with(b"$NACL"),
            "Keyfile should start with $NACL header"
        );

        // Verify structure: $NACL (5) + salt (16) + nonce (24) + ciphertext
        assert!(content.len() >= 5 + 16 + 24, "Keyfile too short");

        // Verify we can decrypt it back
        let keyfile2 = Keyfile::new(&path);
        let loaded = keyfile2.get_keypair(Some("password")).unwrap();
        assert_eq!(keypair.public_key(), loaded.public_key());

        // Also test unencrypted JSON format
        let json_path = dir.path().join("test_keyfile_json");
        let mut json_keyfile = Keyfile::new(&json_path);
        json_keyfile
            .set_keypair(keypair.clone(), None, false)
            .unwrap();

        let json_content = std::fs::read_to_string(&json_path).unwrap();
        let json: serde_json::Value = serde_json::from_str(&json_content).unwrap();

        // Verify JSON structure matches Python SDK
        assert!(json["ss58Address"].as_str().is_some());
        assert!(json["publicKey"].as_str().is_some());
        assert!(json["privateKey"].as_str().is_some());
    }

    #[test]
    fn test_keypair_from_uri() {
        // Test well-known development accounts
        let alice = Keypair::from_uri("//Alice").unwrap();
        let bob = Keypair::from_uri("//Bob").unwrap();

        assert_ne!(alice.ss58_address(), bob.ss58_address());

        // Verify deterministic
        let alice2 = Keypair::from_uri("//Alice").unwrap();
        assert_eq!(alice.ss58_address(), alice2.ss58_address());
    }

    #[test]
    fn test_mnemonic_validation() {
        let valid = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        assert!(Mnemonic::validate(valid));

        let invalid = "not a valid mnemonic phrase at all";
        assert!(!Mnemonic::validate(invalid));
    }
}
