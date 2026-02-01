//! BIP39 mnemonic generation and recovery for wallet creation.
//!
//! This module provides functionality to generate and validate BIP39 mnemonics,
//! which are used to create and recover wallet keypairs.

// Allow unused_assignments - the ZeroizeOnDrop derive macro generates code that clippy
// incorrectly flags as unused assignments when it reads/writes struct fields for zeroization
#![allow(unused_assignments)]

use bip39::Mnemonic as Bip39Mnemonic;
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Errors that can occur during mnemonic operations.
#[derive(Debug, Error)]
pub enum MnemonicError {
    #[error("Invalid word count: {0}. Must be 12, 15, 18, 21, or 24")]
    InvalidWordCount(usize),

    #[error("Invalid mnemonic phrase: {0}")]
    InvalidPhrase(String),

    #[error("Entropy generation failed: {0}")]
    EntropyError(String),
}

/// A BIP39 mnemonic phrase for wallet generation and recovery.
///
/// The mnemonic is securely zeroed from memory when dropped.
#[derive(Clone, ZeroizeOnDrop)]
pub struct Mnemonic {
    #[zeroize(skip)]
    inner: Bip39Mnemonic,
    phrase: String,
    words: Vec<String>,
}

impl std::fmt::Debug for Mnemonic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Don't expose the actual phrase in debug output
        f.debug_struct("Mnemonic")
            .field("word_count", &self.words.len())
            .finish()
    }
}

impl Mnemonic {
    /// Generate a new 12-word mnemonic phrase.
    ///
    /// # Returns
    /// A new randomly generated mnemonic.
    ///
    /// # Example
    /// ```
    /// use bittensor_rs::wallet::Mnemonic;
    /// let mnemonic = Mnemonic::generate();
    /// assert_eq!(mnemonic.word_count(), 12);
    /// ```
    pub fn generate() -> Self {
        Self::generate_with_words(12).expect("12 words is always valid")
    }

    /// Generate a new mnemonic with the specified number of words.
    ///
    /// # Arguments
    /// * `word_count` - Number of words (12, 15, 18, 21, or 24)
    ///
    /// # Returns
    /// A new mnemonic or an error if the word count is invalid.
    ///
    /// # Example
    /// ```
    /// use bittensor_rs::wallet::Mnemonic;
    /// let mnemonic = Mnemonic::generate_with_words(24).unwrap();
    /// assert_eq!(mnemonic.word_count(), 24);
    /// ```
    pub fn generate_with_words(word_count: usize) -> Result<Self, MnemonicError> {
        let entropy_bits = match word_count {
            12 => 128,
            15 => 160,
            18 => 192,
            21 => 224,
            24 => 256,
            _ => return Err(MnemonicError::InvalidWordCount(word_count)),
        };

        let entropy_bytes = entropy_bits / 8;
        let mut entropy = vec![0u8; entropy_bytes];
        getrandom(&mut entropy).map_err(|e| MnemonicError::EntropyError(e.to_string()))?;

        let inner = Bip39Mnemonic::from_entropy(&entropy)
            .map_err(|e| MnemonicError::EntropyError(e.to_string()))?;

        // Zeroize entropy after use
        entropy.zeroize();

        let phrase = inner.to_string();
        let words: Vec<String> = phrase.split_whitespace().map(String::from).collect();

        Ok(Self {
            inner,
            phrase,
            words,
        })
    }

    /// Create a mnemonic from an existing phrase.
    ///
    /// # Arguments
    /// * `phrase` - A valid BIP39 mnemonic phrase
    ///
    /// # Returns
    /// The parsed mnemonic or an error if the phrase is invalid.
    ///
    /// # Example
    /// ```
    /// use bittensor_rs::wallet::Mnemonic;
    /// let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    /// let mnemonic = Mnemonic::from_phrase(phrase).unwrap();
    /// ```
    pub fn from_phrase(phrase: &str) -> Result<Self, MnemonicError> {
        let normalized = phrase.trim().to_lowercase();
        let inner = Bip39Mnemonic::parse_normalized(&normalized)
            .map_err(|e| MnemonicError::InvalidPhrase(e.to_string()))?;

        let phrase = inner.to_string();
        let words: Vec<String> = phrase.split_whitespace().map(String::from).collect();

        Ok(Self {
            inner,
            phrase,
            words,
        })
    }

    /// Validate a mnemonic phrase without creating a Mnemonic object.
    ///
    /// # Arguments
    /// * `phrase` - The mnemonic phrase to validate
    ///
    /// # Returns
    /// `true` if the phrase is a valid BIP39 mnemonic.
    pub fn validate(phrase: &str) -> bool {
        let normalized = phrase.trim().to_lowercase();
        Bip39Mnemonic::parse_normalized(&normalized).is_ok()
    }

    /// Get the mnemonic phrase as a string.
    ///
    /// # Returns
    /// The mnemonic phrase.
    pub fn phrase(&self) -> &str {
        &self.phrase
    }

    /// Get the individual words of the mnemonic.
    ///
    /// # Returns
    /// A slice of the mnemonic words.
    pub fn words(&self) -> &[String] {
        &self.words
    }

    /// Get the number of words in the mnemonic.
    ///
    /// # Returns
    /// The word count (12, 15, 18, 21, or 24).
    pub fn word_count(&self) -> usize {
        self.words.len()
    }

    /// Convert the mnemonic to a seed for key derivation.
    ///
    /// # Arguments
    /// * `password` - Optional password for additional security (BIP39 passphrase)
    ///
    /// # Returns
    /// A 64-byte seed suitable for key derivation.
    ///
    /// # Example
    /// ```
    /// use bittensor_rs::wallet::Mnemonic;
    /// let mnemonic = Mnemonic::generate();
    /// let seed = mnemonic.to_seed(None);
    /// assert_eq!(seed.len(), 64);
    /// ```
    pub fn to_seed(&self, password: Option<&str>) -> [u8; 64] {
        let passphrase = password.unwrap_or("");
        self.inner.to_seed(passphrase)
    }

    /// Convert the mnemonic to entropy bytes.
    ///
    /// # Returns
    /// The underlying entropy as bytes.
    pub fn to_entropy(&self) -> Vec<u8> {
        self.inner.to_entropy()
    }
}

/// Generate random bytes using the system's secure random number generator.
fn getrandom(buf: &mut [u8]) -> Result<(), MnemonicError> {
    use rand::RngCore;
    let mut rng = rand::rng();
    rng.fill_bytes(buf);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_12_words() {
        let mnemonic = Mnemonic::generate();
        assert_eq!(mnemonic.word_count(), 12);
        assert!(Mnemonic::validate(mnemonic.phrase()));
    }

    #[test]
    fn test_generate_24_words() {
        let mnemonic = Mnemonic::generate_with_words(24).unwrap();
        assert_eq!(mnemonic.word_count(), 24);
        assert!(Mnemonic::validate(mnemonic.phrase()));
    }

    #[test]
    fn test_invalid_word_count() {
        assert!(Mnemonic::generate_with_words(13).is_err());
        assert!(Mnemonic::generate_with_words(10).is_err());
    }

    #[test]
    fn test_from_phrase() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let mnemonic = Mnemonic::from_phrase(phrase).unwrap();
        assert_eq!(mnemonic.word_count(), 12);
        assert_eq!(mnemonic.phrase(), phrase);
    }

    #[test]
    fn test_from_phrase_with_extra_whitespace() {
        let phrase = "  abandon  abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about  ";
        let mnemonic = Mnemonic::from_phrase(phrase).unwrap();
        assert_eq!(mnemonic.word_count(), 12);
    }

    #[test]
    fn test_invalid_phrase() {
        let result = Mnemonic::from_phrase("invalid mnemonic phrase");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate() {
        let valid = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        assert!(Mnemonic::validate(valid));

        let invalid = "invalid mnemonic phrase that is not valid";
        assert!(!Mnemonic::validate(invalid));
    }

    #[test]
    fn test_to_seed() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let mnemonic = Mnemonic::from_phrase(phrase).unwrap();
        
        let seed_no_pass = mnemonic.to_seed(None);
        let seed_with_pass = mnemonic.to_seed(Some("password"));
        
        assert_eq!(seed_no_pass.len(), 64);
        assert_eq!(seed_with_pass.len(), 64);
        assert_ne!(seed_no_pass, seed_with_pass);
    }

    #[test]
    fn test_deterministic_seed() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let m1 = Mnemonic::from_phrase(phrase).unwrap();
        let m2 = Mnemonic::from_phrase(phrase).unwrap();
        
        assert_eq!(m1.to_seed(None), m2.to_seed(None));
        assert_eq!(m1.to_seed(Some("test")), m2.to_seed(Some("test")));
    }
}
