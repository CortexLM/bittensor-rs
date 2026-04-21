//! BIP39 mnemonic generation and parsing for wallet key derivation.

use rand::RngCore;

use crate::keypair::{Keypair, KeypairError};

#[derive(Debug, thiserror::Error)]
pub enum MnemonicError {
    #[error("Mnemonic parse error: {0}")]
    Parse(#[from] bip39::Error),
    #[error("Keypair error: {0}")]
    Keypair(#[from] KeypairError),
}

/// Supported mnemonic word counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordCount {
    Words12,
    Words24,
}

impl WordCount {
    fn to_entropy_bits(self) -> usize {
        match self {
            WordCount::Words12 => 128,
            WordCount::Words24 => 256,
        }
    }

    fn to_entropy_bytes(self) -> usize {
        self.to_entropy_bits() / 8
    }
}

/// Generate a random BIP39 mnemonic of the given word count.
pub fn generate_mnemonic(word_count: WordCount) -> Result<bip39::Mnemonic, MnemonicError> {
    let mut entropy = vec![0u8; word_count.to_entropy_bytes()];
    rand::thread_rng().fill_bytes(&mut entropy);
    let mnemonic = bip39::Mnemonic::from_entropy(&entropy)?;
    Ok(mnemonic)
}

/// Parse a mnemonic phrase string into a Mnemonic.
pub fn parse_mnemonic(phrase: &str) -> Result<bip39::Mnemonic, MnemonicError> {
    bip39::Mnemonic::parse(phrase).map_err(MnemonicError::Parse)
}

/// Create a coldkey Keypair from a mnemonic phrase with optional password.
pub fn keypair_from_mnemonic(
    mnemonic: &bip39::Mnemonic,
    password: Option<&str>,
) -> Result<Keypair, MnemonicError> {
    let keypair = Keypair::from_phrase(mnemonic, password)?;
    Ok(keypair)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_12_word_mnemonic() {
        let mnemonic = generate_mnemonic(WordCount::Words12).expect("generate 12-word");
        let words: Vec<&str> = mnemonic.words().collect();
        assert_eq!(words.len(), 12);
    }

    #[test]
    fn generate_24_word_mnemonic() {
        let mnemonic = generate_mnemonic(WordCount::Words24).expect("generate 24-word");
        let words: Vec<&str> = mnemonic.words().collect();
        assert_eq!(words.len(), 24);
    }

    #[test]
    fn parse_valid_mnemonic() {
        let phrase = "bottom drive obey lake curtain smoke basket hold race lonely fit walk";
        let mnemonic = parse_mnemonic(phrase).expect("parse mnemonic");
        let keypair = keypair_from_mnemonic(&mnemonic, None).expect("keypair");
        assert_eq!(keypair.ss58_address(), "5DfhGyQdFobKM8NsWvEeAKk5EQQgYe9AydgJ7rMB6E1EqRzV");
    }

    #[test]
    fn parse_invalid_mnemonic_fails() {
        let result = parse_mnemonic("not valid mnemonic words here sorry");
        assert!(result.is_err());
    }

    #[test]
    fn generated_mnemonic_produces_valid_keypair() {
        let mnemonic = generate_mnemonic(WordCount::Words12).expect("generate");
        let keypair = keypair_from_mnemonic(&mnemonic, None).expect("keypair");
        let addr = keypair.ss58_address();
        assert!(addr.starts_with('5'));
        assert!(!addr.is_empty());
    }

    #[test]
    fn mnemonic_with_password_produces_different_key() {
        let mnemonic = generate_mnemonic(WordCount::Words12).expect("generate");
        let kp_no_pass = keypair_from_mnemonic(&mnemonic, None).expect("keypair no pass");
        let kp_with_pass =
            keypair_from_mnemonic(&mnemonic, Some("password")).expect("keypair pass");
        assert_ne!(kp_no_pass.public_key().0, kp_with_pass.public_key().0);
    }
}
