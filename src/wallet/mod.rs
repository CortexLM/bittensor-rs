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
//! This module uses a keyfile format compatible with the Python SDK:
//!
//! ```json
//! {
//!     "crypto": {
//!         "cipher": "secretbox",
//!         "ciphertext": "<base64>",
//!         "cipherparams": {"nonce": "<base64>"},
//!         "kdf": "argon2id",
//!         "kdfparams": {"salt": "<base64>", "n": 65536, "r": 1, "p": 4}
//!     },
//!     "version": 4
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
    is_legacy_format, migrate_legacy_keyfile, Keyfile, KeyfileData, KeyfileError, KeyfileJson,
    KEYFILE_VERSION,
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
        // This test verifies the JSON format matches Python SDK expectations
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_keyfile");

        let keypair = Keypair::generate();
        let mut keyfile = Keyfile::new(&path);
        keyfile
            .set_keypair(keypair.clone(), Some("password"), false)
            .unwrap();

        // Read and parse the JSON
        let content = std::fs::read_to_string(&path).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();

        // Verify structure
        assert_eq!(json["version"], 4);
        assert_eq!(json["crypto"]["cipher"], "secretbox");
        assert_eq!(json["crypto"]["kdf"], "argon2id");
        assert!(json["crypto"]["ciphertext"].as_str().is_some());
        assert!(json["crypto"]["cipherparams"]["nonce"].as_str().is_some());
        assert!(json["crypto"]["kdfparams"]["salt"].as_str().is_some());
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
