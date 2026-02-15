//! Keypair management for Bittensor wallets.
//!
//! This module provides SR25519 keypair functionality for signing and verification,
//! compatible with the Substrate ecosystem and the Python Bittensor SDK.

// Allow unused_assignments - the ZeroizeOnDrop derive macro generates code that clippy
// incorrectly flags as unused assignments when it reads/writes struct fields for zeroization
#![allow(unused_assignments)]

use crate::wallet::mnemonic::{Mnemonic, MnemonicError};
use sp_core::{
    crypto::{Ss58AddressFormat, Ss58Codec},
    sr25519, Pair,
};
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Bittensor SS58 address format (42 = "bt")
pub const BITTENSOR_SS58_FORMAT: u16 = 42;

/// Errors that can occur during keypair operations.
#[derive(Debug, Error)]
pub enum KeypairError {
    #[error("Invalid seed length: expected 32 bytes, got {0}")]
    InvalidSeedLength(usize),

    #[error("Invalid URI: {0}")]
    InvalidUri(String),

    #[error("Mnemonic error: {0}")]
    Mnemonic(#[from] MnemonicError),

    #[error("Invalid signature length: expected 64 bytes, got {0}")]
    InvalidSignatureLength(usize),

    #[error("Signature verification failed")]
    VerificationFailed,

    #[error("Key derivation error: {0}")]
    DerivationError(String),
}

/// An SR25519 keypair for signing transactions and messages.
///
/// This provides full keypair functionality including signing and verification.
///
/// # Security Note
///
/// The underlying `sr25519::Pair` type from sp_core does not implement `Zeroize`,
/// meaning the private key material may remain in memory after this struct is dropped.
/// For maximum security in sensitive applications, consider:
/// - Using short-lived Keypair instances
/// - Explicitly dropping Keypairs when no longer needed
/// - Using memory-safe practices at the application level
///
/// The `public_key` field IS properly zeroized on drop.
#[derive(ZeroizeOnDrop)]
pub struct Keypair {
    /// The underlying sr25519 pair. Note: This is NOT zeroized on drop as
    /// sp_core::sr25519::Pair does not implement Zeroize.
    #[zeroize(skip)]
    pair: sr25519::Pair,
    /// The 32-byte public key. This field IS zeroized on drop.
    public_key: [u8; 32],
    /// The SS58-encoded address. Skipped from zeroization as it's derived from public key.
    #[zeroize(skip)]
    ss58_address: String,
}

impl Clone for Keypair {
    fn clone(&self) -> Self {
        Self {
            pair: self.pair.clone(),
            public_key: self.public_key,
            ss58_address: self.ss58_address.clone(),
        }
    }
}

impl std::fmt::Debug for Keypair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Don't expose the private key in debug output
        f.debug_struct("Keypair")
            .field("ss58_address", &self.ss58_address)
            .finish()
    }
}

impl Keypair {
    /// Create a keypair from an sr25519 pair.
    fn from_pair(pair: sr25519::Pair) -> Self {
        let public = pair.public();
        let public_key: [u8; 32] = public.0;
        let ss58_address =
            public.to_ss58check_with_version(Ss58AddressFormat::custom(BITTENSOR_SS58_FORMAT));

        Self {
            pair,
            public_key,
            ss58_address,
        }
    }

    /// Generate a new random keypair.
    ///
    /// # Returns
    /// A new randomly generated keypair.
    ///
    /// # Example
    /// ```
    /// use bittensor_rs::wallet::Keypair;
    /// let keypair = Keypair::generate();
    /// println!("Address: {}", keypair.ss58_address());
    /// ```
    pub fn generate() -> Self {
        let (pair, _) = sr25519::Pair::generate();
        Self::from_pair(pair)
    }

    /// Create a keypair from a BIP39 mnemonic phrase.
    ///
    /// # Arguments
    /// * `mnemonic` - A valid BIP39 mnemonic phrase
    /// * `password` - Optional password for additional security
    ///
    /// # Returns
    /// The derived keypair or an error.
    ///
    /// # Example
    /// ```
    /// use bittensor_rs::wallet::Keypair;
    /// let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    /// let keypair = Keypair::from_mnemonic(phrase, None).unwrap();
    /// ```
    pub fn from_mnemonic(mnemonic: &str, password: Option<&str>) -> Result<Self, KeypairError> {
        let mnemonic_obj = Mnemonic::from_phrase(mnemonic)?;
        Self::from_mnemonic_obj(&mnemonic_obj, password)
    }

    /// Create a keypair from a Mnemonic object.
    ///
    /// # Arguments
    /// * `mnemonic` - A Mnemonic object
    /// * `password` - Optional password for additional security
    ///
    /// # Returns
    /// The derived keypair.
    pub fn from_mnemonic_obj(
        mnemonic: &Mnemonic,
        password: Option<&str>,
    ) -> Result<Self, KeypairError> {
        // Use the mnemonic phrase directly with sp_core's from_phrase
        // This matches the Substrate/Polkadot standard derivation
        let pass = password.unwrap_or("");
        let (pair, _seed) = sr25519::Pair::from_phrase(mnemonic.phrase(), Some(pass))
            .map_err(|e| KeypairError::DerivationError(format!("{:?}", e)))?;

        Ok(Self::from_pair(pair))
    }

    /// Create a keypair from a 32-byte seed.
    ///
    /// # Arguments
    /// * `seed` - A 32-byte seed
    ///
    /// # Returns
    /// The derived keypair or an error if the seed is invalid.
    ///
    /// # Example
    /// ```
    /// use bittensor_rs::wallet::Keypair;
    /// let seed = [0u8; 32];
    /// let keypair = Keypair::from_seed(&seed).unwrap();
    /// ```
    pub fn from_seed(seed: &[u8]) -> Result<Self, KeypairError> {
        if seed.len() != 32 {
            return Err(KeypairError::InvalidSeedLength(seed.len()));
        }

        let mut seed_arr = [0u8; 32];
        seed_arr.copy_from_slice(seed);

        let pair = sr25519::Pair::from_seed(&seed_arr);

        // Zeroize the seed copy
        seed_arr.zeroize();

        Ok(Self::from_pair(pair))
    }

    /// Create a keypair from a Substrate URI (secret phrase with optional derivation path).
    ///
    /// # Arguments
    /// * `uri` - A secret URI (e.g., "//Alice" or "word word word//derive/path")
    ///
    /// # Returns
    /// The derived keypair or an error.
    ///
    /// # Example
    /// ```
    /// use bittensor_rs::wallet::Keypair;
    /// let keypair = Keypair::from_uri("//Alice").unwrap();
    /// ```
    pub fn from_uri(uri: &str) -> Result<Self, KeypairError> {
        let pair = sr25519::Pair::from_string(uri, None)
            .map_err(|e| KeypairError::InvalidUri(format!("{:?}", e)))?;
        Ok(Self::from_pair(pair))
    }

    /// Get the public key as raw bytes.
    ///
    /// # Returns
    /// A reference to the 32-byte public key.
    pub fn public_key(&self) -> &[u8; 32] {
        &self.public_key
    }

    /// Get the SS58 address with Bittensor format (prefix 42).
    ///
    /// # Returns
    /// The SS58-encoded address string.
    pub fn ss58_address(&self) -> &str {
        &self.ss58_address
    }

    /// Get the underlying sr25519 pair.
    ///
    /// This can be used for advanced operations or integration with other Substrate libraries.
    pub fn pair(&self) -> &sr25519::Pair {
        &self.pair
    }

    /// Sign a message and return the signature.
    ///
    /// # Arguments
    /// * `message` - The message to sign
    ///
    /// # Returns
    /// A 64-byte signature.
    ///
    /// # Example
    /// ```
    /// use bittensor_rs::wallet::Keypair;
    /// let keypair = Keypair::generate();
    /// let message = b"Hello, Bittensor!";
    /// let signature = keypair.sign(message);
    /// assert!(keypair.verify(message, &signature));
    /// ```
    pub fn sign(&self, message: &[u8]) -> [u8; 64] {
        let signature = self.pair.sign(message);
        signature.0
    }

    /// Verify a signature against a message using this keypair's public key.
    ///
    /// # Arguments
    /// * `message` - The original message
    /// * `signature` - The signature to verify (64 bytes)
    ///
    /// # Returns
    /// `true` if the signature is valid.
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> bool {
        if signature.len() != 64 {
            return false;
        }

        let mut sig_arr = [0u8; 64];
        sig_arr.copy_from_slice(signature);

        let sig = sr25519::Signature::from_raw(sig_arr);
        sr25519::Pair::verify(&sig, message, &self.pair.public())
    }

    /// Verify a signature against a message using a public key.
    ///
    /// # Arguments
    /// * `message` - The original message
    /// * `signature` - The signature to verify (64 bytes)
    /// * `public_key` - The public key (32 bytes)
    ///
    /// # Returns
    /// `true` if the signature is valid.
    pub fn verify_with_public(message: &[u8], signature: &[u8], public_key: &[u8; 32]) -> bool {
        if signature.len() != 64 {
            return false;
        }

        let mut sig_arr = [0u8; 64];
        sig_arr.copy_from_slice(signature);

        let sig = sr25519::Signature::from_raw(sig_arr);
        let public = sr25519::Public::from_raw(*public_key);

        sr25519::Pair::verify(&sig, message, &public)
    }

    /// Export the full keypair as bytes (64 bytes for SR25519).
    ///
    /// This returns the full keypair suitable for storage in Python-compatible
    /// keyfile format. The private key is 64 bytes (32 bytes seed + 32 bytes public key).
    ///
    /// WARNING: This exposes the private key. Handle with care.
    /// Get the 32-byte secret seed for this keypair.
    ///
    /// This derives the seed from the underlying keypair bytes.
    pub fn secret_seed(&self) -> Result<[u8; 32], KeypairError> {
        let mut raw = self.pair.to_raw_vec();
        if raw.len() < 32 {
            return Err(KeypairError::InvalidSeedLength(raw.len()));
        }
        let mut seed = [0u8; 32];
        seed.copy_from_slice(&raw[..32]);
        raw.zeroize();
        Ok(seed)
    }
    pub fn to_full_bytes(&self) -> Vec<u8> {
        // SR25519 stores private key as 64 bytes: 32-byte seed + 32-byte public key
        // The to_raw_vec() returns this format
        self.pair.to_raw_vec()
    }

    /// Alias for to_full_bytes() for backward compatibility.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.to_full_bytes()
    }

    /// Create a keypair from exported bytes.
    ///
    /// # Arguments
    /// * `bytes` - The raw keypair bytes
    ///
    /// # Returns
    /// The restored keypair or an error.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, KeypairError> {
        // Try to restore from raw vec
        let pair = sr25519::Pair::from_seed_slice(bytes).map_err(|e| {
            KeypairError::DerivationError(format!("Failed to restore keypair: {:?}", e))
        })?;
        Ok(Self::from_pair(pair))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate() {
        let keypair = Keypair::generate();
        assert_eq!(keypair.public_key().len(), 32);
        assert!(keypair.ss58_address().starts_with('5')); // SS58 prefix for substrate
    }

    #[test]
    fn test_from_mnemonic() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let keypair = Keypair::from_mnemonic(phrase, None).unwrap();

        // Should be deterministic
        let keypair2 = Keypair::from_mnemonic(phrase, None).unwrap();
        assert_eq!(keypair.public_key(), keypair2.public_key());
        assert_eq!(keypair.ss58_address(), keypair2.ss58_address());
    }

    #[test]
    fn test_from_mnemonic_with_password() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let keypair_no_pass = Keypair::from_mnemonic(phrase, None).unwrap();
        let keypair_with_pass = Keypair::from_mnemonic(phrase, Some("password")).unwrap();

        // Different passwords should produce different keys
        assert_ne!(keypair_no_pass.public_key(), keypair_with_pass.public_key());
    }

    #[test]
    fn test_from_seed() {
        let seed = [42u8; 32];
        let keypair = Keypair::from_seed(&seed).unwrap();

        // Should be deterministic
        let keypair2 = Keypair::from_seed(&seed).unwrap();
        assert_eq!(keypair.public_key(), keypair2.public_key());
    }

    #[test]
    fn test_from_seed_invalid_length() {
        let seed = [0u8; 16];
        assert!(Keypair::from_seed(&seed).is_err());
    }

    #[test]
    fn test_from_uri() {
        let keypair = Keypair::from_uri("//Alice").unwrap();
        assert!(!keypair.ss58_address().is_empty());

        // Should be deterministic
        let keypair2 = Keypair::from_uri("//Alice").unwrap();
        assert_eq!(keypair.public_key(), keypair2.public_key());
    }

    #[test]
    fn test_sign_and_verify() {
        let keypair = Keypair::generate();
        let message = b"Hello, Bittensor!";

        let signature = keypair.sign(message);
        assert_eq!(signature.len(), 64);

        assert!(keypair.verify(message, &signature));

        // Wrong message should fail
        assert!(!keypair.verify(b"Wrong message", &signature));
    }

    #[test]
    fn test_verify_with_public() {
        let keypair = Keypair::generate();
        let message = b"Test message";
        let signature = keypair.sign(message);

        assert!(Keypair::verify_with_public(
            message,
            &signature,
            keypair.public_key()
        ));
    }

    #[test]
    fn test_to_and_from_bytes() {
        let original = Keypair::generate();
        let bytes = original.to_bytes();

        let restored = Keypair::from_bytes(&bytes).unwrap();
        assert_eq!(original.public_key(), restored.public_key());

        // Verify signing still works
        let message = b"Test";
        let sig = original.sign(message);
        assert!(restored.verify(message, &sig));
    }

    #[test]
    fn test_invalid_signature_length() {
        let keypair = Keypair::generate();
        let message = b"Test";

        // Too short
        assert!(!keypair.verify(message, &[0u8; 32]));

        // Too long
        assert!(!keypair.verify(message, &[0u8; 128]));
    }
}
