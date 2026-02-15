//! Keyfile encryption and storage for Bittensor wallets.
//!
//! This module provides functionality to securely store keypairs on disk,
//! compatible with the Python Bittensor SDK keyfile format.
//!
//! ## Python-Compatible Keyfile Format
//!
//! Python Bittensor SDK (bittensor-wallet) uses two formats:
//!
//! ### Encrypted Format (Binary with $NACL header)
//! ```
//! +--------+--------+---------+-----------+
//! | Header |  Salt  |  Nonce  | Ciphertext|
//! | 5 bytes|16 bytes| 24 bytes| variable  |
//! +--------+--------+---------+-----------+
//! ```
//!
//! - **Header**: `$NACL` (5 bytes)
//! - **Salt**: 16 random bytes for Argon2id key derivation
//! - **Nonce**: 24 random bytes for XSalsa20-Poly1305 encryption
//! - **Ciphertext**: The encrypted keypair JSON data
//!
//! ### Unencrypted Format (JSON)
//! ```json
//! {
//!     "ss58Address": "5EPCUjPxiHAcNooYipQFWr9NmmXJKpNG5RhcntXwbtUySrgH",
//!     "publicKey": "0x66933bd1f37070ef87bd1198af3dacceb095237f803f3d32b173e6b425ed7972",
//!     "privateKey": "0x2ec306fc1c5bc2f0e3a2c7a6ec6014ca4a0823a7d7d42ad5e9d7f376a1c36c0d...",
//!     "secretSeed": "0x4ed8d4b17698ddeaa1f1559f152f87b5d472f725ca86d341bd0276f1b61197e2",
//!     "secretPhrase": "abandon abandon abandon ...",
//!     "accountId": "0x66933bd1f37070ef87bd1198af3dacceb095237f803f3d32b173e6b425ed7972"
//! }
//! ```
//!
//! ### Argon2id Parameters (PyNaCl Compatible)
//! - Memory: 64 MiB (67108864 bytes = 65536 KiB blocks)
//! - Iterations: 2 (OPSLIMIT_INTERACTIVE)
//! - Parallelism: 1
//! - Salt length: 16 bytes
//! - Key length: 32 bytes
//! - Algorithm: Argon2id v1.3

use crate::wallet::keypair::{Keypair, KeypairError};
use argon2::{Argon2, Params, Version};
use crypto_secretbox::{
    aead::{Aead, KeyInit},
    XSalsa20Poly1305,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;
use zeroize::Zeroize;

/// NaCl header for encrypted keyfiles (matches Python bittensor-wallet)
pub const NACL_HEADER: &[u8] = b"$NACL";

/// Argon2id parameters matching Python bittensor-wallet (PyNaCl interactive preset)
/// - Memory: 64 MiB (67108864 bytes = 65536 KiB blocks)
/// - Iterations: 2 (matches PyNaCl's OPSLIMIT_INTERACTIVE)
/// - Parallelism: 1
const ARGON2_MEMORY_COST: u32 = 65536; // 64 MiB in KiB blocks
const ARGON2_TIME_COST: u32 = 2; // Iterations
const ARGON2_PARALLELISM: u32 = 1; // Parallelism

/// Errors that can occur during keyfile operations.
#[derive(Debug, Error)]
pub enum KeyfileError {
    #[error("Keyfile not found: {0}")]
    NotFound(PathBuf),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid keyfile format: {0}")]
    InvalidFormat(String),

    #[error("Decryption failed: wrong password or corrupted keyfile")]
    DecryptionFailed,

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),

    #[error("Keyfile already exists and overwrite is not enabled")]
    AlreadyExists,

    #[error("Keypair error: {0}")]
    Keypair(#[from] KeypairError),

    #[error("Base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("Unsupported keyfile version: {0}")]
    UnsupportedVersion(u32),

    #[error("Keyfile is not encrypted")]
    NotEncrypted,

    #[error("Password required for encrypted keyfile")]
    PasswordRequired,

    #[error("Legacy format detected: {0}")]
    LegacyFormat(String),

    #[error("Invalid NACL header")]
    InvalidNaclHeader,
}

/// Data structure for unencrypted keyfile (JSON format) matching Python SDK.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyfileJsonData {
    #[serde(rename = "ss58Address")]
    pub ss58_address: String,
    #[serde(rename = "publicKey")]
    pub public_key: String,
    #[serde(rename = "privateKey")]
    pub private_key: String,
    #[serde(rename = "secretSeed", skip_serializing_if = "Option::is_none")]
    pub secret_seed: Option<String>,
    #[serde(rename = "secretPhrase", skip_serializing_if = "Option::is_none")]
    pub secret_phrase: Option<String>,
    #[serde(rename = "accountId", skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
}

/// Data structure for encrypted key material.
#[derive(Debug, Clone)]
pub struct KeyfileData {
    /// 16-byte salt for Argon2
    pub salt: [u8; 16],
    /// 24-byte nonce for XSalsa20Poly1305
    pub nonce: [u8; 24],
    /// Encrypted key bytes (ciphertext)
    pub encrypted_key: Vec<u8>,
}

/// A keyfile represents a keypair stored on disk.
///
/// The keyfile can be encrypted (password-protected) or unencrypted.
pub struct Keyfile {
    path: PathBuf,
    keypair: Option<Keypair>,
}

impl std::fmt::Debug for Keyfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Keyfile")
            .field("path", &self.path)
            .field("loaded", &self.keypair.is_some())
            .finish()
    }
}

impl Keyfile {
    /// Create a new keyfile handle for the given path.
    ///
    /// This does not load or create the keyfile on disk.
    ///
    /// # Arguments
    /// * `path` - Path where the keyfile is or will be stored
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            keypair: None,
        }
    }

    /// Get the path to this keyfile.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Check if the keyfile exists on disk.
    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    /// Check if the keyfile is encrypted.
    ///
    /// Returns `false` if the file doesn't exist or can't be read.
    pub fn is_encrypted(&self) -> bool {
        if !self.exists() {
            return false;
        }

        match self.read_raw() {
            Ok(data) => data.starts_with(NACL_HEADER),
            Err(_) => false,
        }
    }

    /// Get the keypair, decrypting if necessary.
    ///
    /// # Arguments
    /// * `password` - Password for decryption (required if encrypted)
    ///
    /// # Returns
    /// The keypair or an error.
    pub fn get_keypair(&self, password: Option<&str>) -> Result<Keypair, KeyfileError> {
        if let Some(ref kp) = self.keypair {
            return Ok(kp.clone());
        }

        if !self.exists() {
            return Err(KeyfileError::NotFound(self.path.clone()));
        }

        let data = self.read_raw()?;
        self.decrypt_keypair(&data, password)
    }

    /// Store a keypair in this keyfile.
    ///
    /// # Arguments
    /// * `keypair` - The keypair to store
    /// * `password` - Optional password for encryption (if None, stores unencrypted)
    /// * `overwrite` - Whether to overwrite an existing keyfile
    ///
    /// # Returns
    /// Ok(()) on success, or an error.
    pub fn set_keypair(
        &mut self,
        keypair: Keypair,
        password: Option<&str>,
        overwrite: bool,
    ) -> Result<(), KeyfileError> {
        if self.exists() && !overwrite {
            return Err(KeyfileError::AlreadyExists);
        }

        // Ensure parent directory exists
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let raw_key = keypair.to_full_bytes();

        let content = match password {
            Some(pass) => {
                // Encrypt and create binary format with $NACL header
                let encrypted_data = self.encrypt(&raw_key, pass)?;
                self.to_binary_format(&encrypted_data)?
            }
            None => {
                // SECURITY WARNING: Storing key without encryption
                tracing::warn!(
                    "Storing keyfile without encryption at {:?}. \
                     This is insecure - consider using a password.",
                    self.path
                );
                // Store as JSON format matching Python SDK
                let json_data = KeyfileJsonData {
                    ss58_address: keypair.ss58_address().to_string(),
                    public_key: format!("0x{}", hex::encode(keypair.public_key())),
                    private_key: format!("0x{}", hex::encode(&raw_key)),
                    secret_seed: None,
                    secret_phrase: None,
                    account_id: Some(format!("0x{}", hex::encode(keypair.public_key()))),
                };
                serde_json::to_vec_pretty(&json_data)?
            }
        };

        // Write atomically by writing to temp file first
        let temp_path = self.path.with_extension("tmp");
        {
            #[cfg(unix)]
            let mut file = {
                use std::os::unix::fs::OpenOptionsExt;
                fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .mode(0o600)
                    .open(&temp_path)?
            };
            #[cfg(not(unix))]
            let mut file = fs::File::create(&temp_path)?;

            file.write_all(&content)?;
            file.sync_all()?;
        }
        fs::rename(&temp_path, &self.path)?;

        self.keypair = Some(keypair);
        Ok(())
    }

    /// Encrypt data using Argon2id + XSalsa20Poly1305.
    ///
    /// # Arguments
    /// * `data` - The data to encrypt
    /// * `password` - The encryption password
    ///
    /// # Returns
    /// The encrypted data with salt and nonce.
    pub fn encrypt(&self, data: &[u8], password: &str) -> Result<KeyfileData, KeyfileError> {
        // Generate random salt (16 bytes) and nonce (24 bytes)
        let mut salt = [0u8; 16];
        let mut nonce = [0u8; 24];

        use rand::RngCore;
        let mut rng = rand::rng();
        rng.fill_bytes(&mut salt);
        rng.fill_bytes(&mut nonce);

        // Derive key using Argon2id with PyNaCl-compatible parameters
        let mut key = derive_key(password, &salt)?;

        // Encrypt using XSalsa20Poly1305
        let cipher = XSalsa20Poly1305::new_from_slice(&key)
            .map_err(|e| KeyfileError::EncryptionFailed(e.to_string()))?;

        let encrypted_key = cipher
            .encrypt(nonce.as_ref().into(), data)
            .map_err(|e| KeyfileError::EncryptionFailed(e.to_string()))?;

        // Zeroize the derived key
        key.zeroize();

        Ok(KeyfileData {
            salt,
            nonce,
            encrypted_key,
        })
    }

    /// Decrypt data using Argon2id + XSalsa20Poly1305.
    ///
    /// # Arguments
    /// * `data` - The encrypted data with salt and nonce
    /// * `password` - The decryption password
    ///
    /// # Returns
    /// The decrypted data.
    pub fn decrypt(&self, data: &KeyfileData, password: &str) -> Result<Vec<u8>, KeyfileError> {
        // Derive key using Argon2id
        let mut key = derive_key(password, &data.salt)?;

        // Decrypt using XSalsa20Poly1305
        let cipher = XSalsa20Poly1305::new_from_slice(&key).map_err(|e| {
            KeyfileError::EncryptionFailed(format!("Failed to create cipher: {}", e))
        })?;

        let decrypted = cipher
            .decrypt(data.nonce.as_ref().into(), data.encrypted_key.as_ref())
            .map_err(|_| KeyfileError::DecryptionFailed)?;

        // Zeroize the derived key
        key.zeroize();

        Ok(decrypted)
    }

    /// Convert encrypted data to binary format with $NACL header.
    fn to_binary_format(&self, data: &KeyfileData) -> Result<Vec<u8>, KeyfileError> {
        // Binary format: $NACL + salt (16) + nonce (24) + ciphertext
        let mut result = Vec::with_capacity(NACL_HEADER.len() + 16 + 24 + data.encrypted_key.len());

        result.extend_from_slice(NACL_HEADER);
        result.extend_from_slice(&data.salt);
        result.extend_from_slice(&data.nonce);
        result.extend_from_slice(&data.encrypted_key);

        Ok(result)
    }

    /// Parse binary format with $NACL header.
    fn parse_nacl_format(data: &[u8]) -> Option<KeyfileData> {
        if data.len() < 5 + 16 + 24 {
            return None;
        }

        if !data.starts_with(NACL_HEADER) {
            return None;
        }

        let salt_slice = &data[5..21];
        let nonce_slice = &data[21..45];
        let ciphertext = &data[45..];

        let mut salt = [0u8; 16];
        let mut nonce = [0u8; 24];
        salt.copy_from_slice(salt_slice);
        nonce.copy_from_slice(nonce_slice);

        Some(KeyfileData {
            salt,
            nonce,
            encrypted_key: ciphertext.to_vec(),
        })
    }

    /// Re-encrypt the keyfile with a new password or update encryption parameters.
    ///
    /// # Arguments
    /// * `old_password` - Current password (or None if unencrypted)
    /// * `new_password` - New password for encryption
    ///
    /// # Returns
    /// Ok(()) on success.
    pub fn check_and_update_encryption(
        &mut self,
        old_password: Option<&str>,
        new_password: &str,
    ) -> Result<(), KeyfileError> {
        let keypair = self.get_keypair(old_password)?;
        self.set_keypair(keypair, Some(new_password), true)
    }

    /// Read raw bytes from the keyfile.
    fn read_raw(&self) -> Result<Vec<u8>, KeyfileError> {
        let mut file = fs::File::open(&self.path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        Ok(data)
    }

    /// Parse keyfile data and decrypt to keypair.
    fn decrypt_keypair(
        &self,
        data: &[u8],
        password: Option<&str>,
    ) -> Result<Keypair, KeyfileError> {
        // Check for NACL format (encrypted)
        if data.starts_with(NACL_HEADER) {
            return self.decrypt_nacl(data, password);
        }

        // Try as unencrypted JSON format
        if let Ok(json) = serde_json::from_slice::<KeyfileJsonData>(data) {
            // Extract private key from JSON and create keypair
            let private_key_hex = json.private_key.trim_start_matches("0x");
            if let Ok(key_bytes) = hex::decode(private_key_hex) {
                return Keypair::from_bytes(&key_bytes).map_err(KeyfileError::Keypair);
            }
        }

        // Try legacy hex format
        if let Ok(s) = std::str::from_utf8(data) {
            let trimmed = s.trim();
            if let Ok(bytes) = hex::decode(trimmed) {
                if let Ok(kp) = Keypair::from_bytes(&bytes) {
                    return Ok(kp);
                }
            }
        }

        // Try raw keypair bytes
        if data.len() >= 64 {
            if let Ok(kp) = Keypair::from_bytes(data) {
                return Ok(kp);
            }
        }

        // Check for legacy formats
        if is_legacy_format(data) {
            return Err(KeyfileError::LegacyFormat(
                "Please migrate this keyfile using migrate_legacy_keyfile()".to_string(),
            ));
        }

        Err(KeyfileError::InvalidFormat(
            "Could not parse keyfile data".to_string(),
        ))
    }

    /// Decrypt keypair from NACL binary format.
    fn decrypt_nacl(&self, data: &[u8], password: Option<&str>) -> Result<Keypair, KeyfileError> {
        let password = password.ok_or(KeyfileError::PasswordRequired)?;

        let keyfile_data = Self::parse_nacl_format(data).ok_or(KeyfileError::InvalidNaclHeader)?;

        let key_bytes = self.decrypt(&keyfile_data, password)?;
        Keypair::from_bytes(&key_bytes).map_err(KeyfileError::Keypair)
    }
}

/// Derive an encryption key using Argon2id with PyNaCl-compatible parameters.
fn derive_key(password: &str, salt: &[u8; 16]) -> Result<[u8; 32], KeyfileError> {
    let params = Params::new(
        ARGON2_MEMORY_COST,
        ARGON2_TIME_COST,
        ARGON2_PARALLELISM,
        Some(32),
    )
    .map_err(|e| KeyfileError::KeyDerivationFailed(e.to_string()))?;

    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params);

    let mut key = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt.as_slice(), &mut key)
        .map_err(|e| KeyfileError::KeyDerivationFailed(e.to_string()))?;

    Ok(key)
}

/// Check if data is in a legacy (pre-v4) format.
///
/// # Arguments
/// * `data` - The raw keyfile data
///
/// # Returns
/// `true` if the data appears to be in a legacy format.
pub fn is_legacy_format(data: &[u8]) -> bool {
    // Check for old JSON formats with different structure
    if let Ok(value) = serde_json::from_slice::<serde_json::Value>(data) {
        if let Some(obj) = value.as_object() {
            // Check for pre-v4 format markers
            if obj.contains_key("secretPhrase") && !obj.contains_key("privateKey") {
                return true;
            }
            if obj.contains_key("data") && !obj.contains_key("privateKey") {
                return true;
            }
        }
    }
    false
}

/// Migrate a legacy keyfile to the current format.
///
/// # Arguments
/// * `path` - Path to the legacy keyfile
/// * `password` - Password for encryption (may be needed for old encrypted formats)
/// * `new_password` - Password for the new format
///
/// # Returns
/// Ok(()) on success.
pub fn migrate_legacy_keyfile(
    path: &Path,
    password: Option<&str>,
    new_password: &str,
) -> Result<(), KeyfileError> {
    let mut data = Vec::new();
    fs::File::open(path)?.read_to_end(&mut data)?;

    if !is_legacy_format(&data) {
        return Err(KeyfileError::InvalidFormat(
            "Not a legacy format keyfile".to_string(),
        ));
    }

    // Try to extract keypair from legacy format
    let keypair = parse_legacy_keyfile(&data, password)?;

    // Create new keyfile with current format
    let mut keyfile = Keyfile::new(path);
    keyfile.set_keypair(keypair, Some(new_password), true)?;

    Ok(())
}

/// Parse a legacy keyfile to extract the keypair.
fn parse_legacy_keyfile(data: &[u8], password: Option<&str>) -> Result<Keypair, KeyfileError> {
    if let Ok(value) = serde_json::from_slice::<serde_json::Value>(data) {
        if let Some(obj) = value.as_object() {
            // Handle secretPhrase format
            if let Some(phrase) = obj.get("secretPhrase").and_then(|v| v.as_str()) {
                return Keypair::from_mnemonic(phrase, password).map_err(KeyfileError::Keypair);
            }

            // Handle old encrypted format with "data" field
            if let Some(data_field) = obj.get("data").and_then(|v| v.as_str()) {
                let key_bytes = hex::decode(data_field)
                    .map_err(|e| KeyfileError::InvalidFormat(e.to_string()))?;
                return Keypair::from_bytes(&key_bytes).map_err(KeyfileError::Keypair);
            }
        }
    }

    Err(KeyfileError::InvalidFormat(
        "Could not parse legacy keyfile".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_encrypt_decrypt() {
        let keyfile = Keyfile::new("/tmp/test");
        let data = b"secret data";
        let password = "test_password";

        let encrypted = keyfile.encrypt(data, password).unwrap();
        let decrypted = keyfile.decrypt(&encrypted, password).unwrap();

        assert_eq!(data.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_encrypt_decrypt_wrong_password() {
        let keyfile = Keyfile::new("/tmp/test");
        let data = b"secret data";

        let encrypted = keyfile.encrypt(data, "correct_password").unwrap();
        let result = keyfile.decrypt(&encrypted, "wrong_password");

        assert!(result.is_err());
    }

    #[test]
    fn test_keyfile_roundtrip_encrypted() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_key");

        let original = Keypair::generate();
        let password = "test_password";

        {
            let mut keyfile = Keyfile::new(&path);
            keyfile
                .set_keypair(original.clone(), Some(password), false)
                .unwrap();
        }

        {
            let keyfile = Keyfile::new(&path);
            assert!(keyfile.exists());
            assert!(keyfile.is_encrypted());

            let loaded = keyfile.get_keypair(Some(password)).unwrap();
            assert_eq!(original.public_key(), loaded.public_key());
        }
    }

    #[test]
    fn test_keyfile_roundtrip_unencrypted() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_key_unenc");

        let original = Keypair::generate();

        {
            let mut keyfile = Keyfile::new(&path);
            keyfile.set_keypair(original.clone(), None, false).unwrap();
        }

        {
            let keyfile = Keyfile::new(&path);
            assert!(keyfile.exists());
            assert!(!keyfile.is_encrypted());

            let loaded = keyfile.get_keypair(None).unwrap();
            assert_eq!(original.public_key(), loaded.public_key());
        }
    }

    #[test]
    fn test_keyfile_no_overwrite() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_key_no_ow");

        let keypair = Keypair::generate();

        let mut keyfile = Keyfile::new(&path);
        keyfile.set_keypair(keypair.clone(), None, false).unwrap();

        // Should fail without overwrite
        let result = keyfile.set_keypair(keypair.clone(), None, false);
        assert!(matches!(result, Err(KeyfileError::AlreadyExists)));

        // Should succeed with overwrite
        keyfile.set_keypair(keypair, None, true).unwrap();
    }

    #[test]
    fn test_keyfile_not_found() {
        let keyfile = Keyfile::new("/nonexistent/path/key");
        let result = keyfile.get_keypair(None);
        assert!(matches!(result, Err(KeyfileError::NotFound(_))));
    }

    #[test]
    fn test_is_legacy_format() {
        // Legacy secretPhrase format without privateKey
        let legacy1 = br#"{"secretPhrase": "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"}"#;
        assert!(is_legacy_format(legacy1));

        // Legacy data format
        let legacy2 = br#"{"data": "0123456789abcdef"}"#;
        assert!(is_legacy_format(legacy2));

        // Current format should not be detected as legacy
        let current = br#"{"privateKey": "0x1234", "publicKey": "0x5678", "ss58Address": "5xxx"}"#;
        assert!(!is_legacy_format(current));
    }

    #[test]
    fn test_keyfile_password_required() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_enc_key");

        let keypair = Keypair::generate();
        let mut keyfile = Keyfile::new(&path);
        keyfile
            .set_keypair(keypair, Some("password"), false)
            .unwrap();

        // Create a fresh Keyfile instance to avoid cached keypair
        let keyfile2 = Keyfile::new(&path);

        // Should require password when loading from encrypted file
        let result = keyfile2.get_keypair(None);
        if let Err(ref e) = result {
            eprintln!("Got error: {:?}", e);
        }
        assert!(matches!(result, Err(KeyfileError::PasswordRequired)));
    }

    #[test]
    fn test_nacl_binary_format() {
        let keyfile = Keyfile::new("/tmp/test");
        let data = b"test data for encryption";
        let password = "test_password";

        // Encrypt
        let encrypted = keyfile.encrypt(data, password).unwrap();
        let binary = keyfile.to_binary_format(&encrypted).unwrap();

        // Verify format
        assert!(binary.starts_with(NACL_HEADER));
        assert_eq!(binary.len(), 5 + 16 + 24 + encrypted.encrypted_key.len());

        // Parse back
        let parsed = Keyfile::parse_nacl_format(&binary).unwrap();
        assert_eq!(parsed.salt, encrypted.salt);
        assert_eq!(parsed.nonce, encrypted.nonce);
        assert_eq!(parsed.encrypted_key, encrypted.encrypted_key);

        // Decrypt
        let decrypted = keyfile.decrypt(&parsed, password).unwrap();
        assert_eq!(data.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_json_format_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_json_key");

        let original = Keypair::generate();

        {
            let mut keyfile = Keyfile::new(&path);
            keyfile.set_keypair(original.clone(), None, false).unwrap();
        }

        // Read and verify JSON format
        let content = fs::read_to_string(&path).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();

        assert!(json.get("ss58Address").is_some());
        assert!(json.get("publicKey").is_some());
        assert!(json.get("privateKey").is_some());

        // Verify we can load it back
        let keyfile = Keyfile::new(&path);
        let loaded = keyfile.get_keypair(None).unwrap();
        assert_eq!(original.public_key(), loaded.public_key());
    }
}
