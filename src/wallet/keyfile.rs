//! Keyfile encryption and storage for Bittensor wallets.
//!
//! This module provides functionality to securely store keypairs on disk,
//! compatible with the Python Bittensor SDK keyfile format.
//!
//! ## Keyfile Format
//!
//! The keyfile format uses JSON with the following structure:
//! ```json
//! {
//!     "crypto": {
//!         "cipher": "secretbox",
//!         "ciphertext": "<base64-encoded encrypted data>",
//!         "cipherparams": {"nonce": "<base64-encoded 24-byte nonce>"},
//!         "kdf": "argon2id",
//!         "kdfparams": {
//!             "salt": "<base64-encoded 16-byte salt>",
//!             "n": 65536,
//!             "r": 1,
//!             "p": 4
//!         }
//!     },
//!     "version": 4
//! }
//! ```

use crate::wallet::keypair::{Keypair, KeypairError};
use argon2::{Argon2, Params, Version};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
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

/// Current keyfile format version
pub const KEYFILE_VERSION: u32 = 4;

/// Default Argon2 parameters matching Python SDK
const ARGON2_TIME_COST: u32 = 1;
const ARGON2_MEMORY_COST: u32 = 65536; // 64 MiB
const ARGON2_PARALLELISM: u32 = 4;

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
}

/// Encryption parameters for a keyfile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KdfParams {
    pub salt: String,
    #[serde(rename = "n")]
    pub memory_cost: u32,
    #[serde(rename = "r")]
    pub time_cost: u32,
    #[serde(rename = "p")]
    pub parallelism: u32,
}

/// Cipher parameters for a keyfile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CipherParams {
    pub nonce: String,
}

/// Crypto section of the keyfile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoData {
    pub cipher: String,
    pub ciphertext: String,
    pub cipherparams: CipherParams,
    pub kdf: String,
    pub kdfparams: KdfParams,
}

/// The complete keyfile structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyfileJson {
    pub crypto: CryptoData,
    pub version: u32,
}

/// Data structure for encrypted key material.
#[derive(Debug, Clone)]
pub struct KeyfileData {
    /// Encrypted key bytes
    pub encrypted_key: Vec<u8>,
    /// 24-byte nonce for XSalsa20Poly1305
    pub nonce: [u8; 24],
    /// 16-byte salt for Argon2
    pub salt: [u8; 16],
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
            Ok(data) => {
                // Try to parse as encrypted JSON format
                serde_json::from_slice::<KeyfileJson>(&data).is_ok()
            }
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

        let raw_key = keypair.to_bytes();

        let content = match password {
            Some(pass) => {
                let keyfile_data = self.encrypt(&raw_key, pass)?;
                self.to_json(&keyfile_data)?
            }
            None => {
                // SECURITY WARNING: Storing key without encryption
                tracing::warn!(
                    "Storing keyfile without encryption at {:?}. \
                     This is insecure - consider using a password.",
                    self.path
                );
                // Store unencrypted (just the raw key bytes as hex)
                // This matches legacy unencrypted format
                hex::encode(&raw_key).into_bytes()
            }
        };

        // Write atomically by writing to temp file first
        // On Unix, set restrictive permissions (0o600) at creation time to avoid
        // a race condition where the file is briefly world-readable.
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

        // On non-Unix platforms, set permissions after rename (best effort)
        #[cfg(not(unix))]
        {
            // No race condition mitigation available on non-Unix
            // At least try to restrict permissions after creation
        }

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
        // Generate random salt and nonce
        let mut salt = [0u8; 16];
        let mut nonce = [0u8; 24];

        use rand::RngCore;
        let mut rng = rand::rng();
        rng.fill_bytes(&mut salt);
        rng.fill_bytes(&mut nonce);

        // Derive key using Argon2id
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
            encrypted_key,
            nonce,
            salt,
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

    /// Convert KeyfileData to JSON bytes.
    fn to_json(&self, data: &KeyfileData) -> Result<Vec<u8>, KeyfileError> {
        let json = KeyfileJson {
            crypto: CryptoData {
                cipher: "secretbox".to_string(),
                ciphertext: BASE64.encode(&data.encrypted_key),
                cipherparams: CipherParams {
                    nonce: BASE64.encode(data.nonce),
                },
                kdf: "argon2id".to_string(),
                kdfparams: KdfParams {
                    salt: BASE64.encode(data.salt),
                    memory_cost: ARGON2_MEMORY_COST,
                    time_cost: ARGON2_TIME_COST,
                    parallelism: ARGON2_PARALLELISM,
                },
            },
            version: KEYFILE_VERSION,
        };

        serde_json::to_vec_pretty(&json).map_err(KeyfileError::Json)
    }

    /// Parse JSON and decrypt to keypair.
    fn decrypt_keypair(
        &self,
        data: &[u8],
        password: Option<&str>,
    ) -> Result<Keypair, KeyfileError> {
        // Try to parse as JSON (encrypted format)
        if let Ok(json) = serde_json::from_slice::<KeyfileJson>(data) {
            return self.decrypt_from_json(&json, password);
        }

        // Try as unencrypted hex
        if let Ok(hex_str) = std::str::from_utf8(data) {
            let hex_str = hex_str.trim();
            if let Ok(key_bytes) = hex::decode(hex_str) {
                return Keypair::from_bytes(&key_bytes).map_err(KeyfileError::Keypair);
            }
        }

        // Try as raw bytes (legacy unencrypted)
        if data.len() >= 32 {
            if let Ok(keypair) = Keypair::from_bytes(data) {
                return Ok(keypair);
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

    /// Decrypt keypair from parsed JSON.
    fn decrypt_from_json(
        &self,
        json: &KeyfileJson,
        password: Option<&str>,
    ) -> Result<Keypair, KeyfileError> {
        if json.version > KEYFILE_VERSION {
            return Err(KeyfileError::UnsupportedVersion(json.version));
        }

        let password = password.ok_or(KeyfileError::PasswordRequired)?;

        // Decode base64 fields
        let ciphertext = BASE64.decode(&json.crypto.ciphertext)?;
        let nonce_bytes = BASE64.decode(&json.crypto.cipherparams.nonce)?;
        let salt_bytes = BASE64.decode(&json.crypto.kdfparams.salt)?;

        if nonce_bytes.len() != 24 {
            return Err(KeyfileError::InvalidFormat(format!(
                "Invalid nonce length: expected 24, got {}",
                nonce_bytes.len()
            )));
        }

        if salt_bytes.len() != 16 {
            return Err(KeyfileError::InvalidFormat(format!(
                "Invalid salt length: expected 16, got {}",
                salt_bytes.len()
            )));
        }

        let mut nonce = [0u8; 24];
        let mut salt = [0u8; 16];
        nonce.copy_from_slice(&nonce_bytes);
        salt.copy_from_slice(&salt_bytes);

        let keyfile_data = KeyfileData {
            encrypted_key: ciphertext,
            nonce,
            salt,
        };

        let key_bytes = self.decrypt(&keyfile_data, password)?;
        Keypair::from_bytes(&key_bytes).map_err(KeyfileError::Keypair)
    }
}

/// Derive an encryption key using Argon2id.
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
        .hash_password_into(password.as_bytes(), salt, &mut key)
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
        // Legacy formats might have different fields
        if let Some(obj) = value.as_object() {
            // Check for pre-v4 format markers
            if obj.contains_key("secretPhrase") {
                return true;
            }
            if obj.contains_key("data") && !obj.contains_key("crypto") {
                return true;
            }
            // Version check
            if let Some(version) = obj.get("version") {
                if let Some(v) = version.as_u64() {
                    if v < KEYFILE_VERSION as u64 {
                        return true;
                    }
                }
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
        // Legacy secretPhrase format
        let legacy1 = br#"{"secretPhrase": "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"}"#;
        assert!(is_legacy_format(legacy1));

        // Legacy data format
        let legacy2 = br#"{"data": "0123456789abcdef", "version": 2}"#;
        assert!(is_legacy_format(legacy2));

        // Current format should not be detected as legacy
        let current = br#"{"crypto": {"cipher": "secretbox"}, "version": 4}"#;
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
        // This simulates loading from disk like a real application would
        let keyfile2 = Keyfile::new(&path);

        // Should require password when loading from encrypted file
        let result = keyfile2.get_keypair(None);
        if let Err(ref e) = result {
            eprintln!("Got error: {:?}", e);
        }
        assert!(matches!(result, Err(KeyfileError::PasswordRequired)));
    }
}
