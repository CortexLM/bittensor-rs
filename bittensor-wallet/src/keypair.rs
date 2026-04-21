//! Keypair wrapper around `subxt_signer::sr25519::Keypair` with seed tracking.

use std::str::FromStr;

use hmac::Hmac;
use pbkdf2::pbkdf2;
use schnorrkel::{
    ExpansionMode, MiniSecretKey,
    derive::{ChainCode, Derivation},
};
use sha2::Sha512;
use subxt_signer::sr25519::{self, Keypair as InnerKeypair, PublicKey, SecretKeyBytes, Signature};
use subxt_signer::{DeriveJunction, ExposeSecret, SecretUri, bip39};

use crate::keyfile;
use crate::ss58::{self, Ss58Error};

const BT_SS58_FORMAT: u8 = ss58::SS58_FORMAT_BYTE;
const SEED_LEN: usize = 32;

#[derive(Debug, thiserror::Error)]
pub enum KeypairError {
    #[error("Invalid mnemonic: {0}")]
    InvalidMnemonic(String),
    #[error("Invalid secret URI: {0}")]
    InvalidSecretUri(String),
    #[error("Invalid seed hex: {0}")]
    InvalidSeedHex(String),
    #[error("Seed derivation failed")]
    SeedDerivationFailed,
    #[error("Keyfile error: {0}")]
    Keyfile(#[from] keyfile::KeyfileError),
    #[error("SS58 error: {0}")]
    Ss58(#[from] Ss58Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Signer error: {0}")]
    Signer(#[from] sr25519::Error),
    #[error("Schnorrkel error: {0}")]
    Schnorrkel(String),
}

pub struct Keypair {
    inner: InnerKeypair,
    seed: SecretKeyBytes,
}

impl Clone for Keypair {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone(), seed: self.seed }
    }
}

impl std::fmt::Debug for Keypair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Keypair")
            .field("public_key", &hex::encode(self.inner.public_key().0))
            .finish_non_exhaustive()
    }
}

impl Keypair {
    /// Create a keypair from a [`SecretUri`], deriving the seed from the URI's
    /// phrase, password, and junctions.
    ///
    /// # Errors
    ///
    /// Returns [`KeypairError::InvalidMnemonic`], [`KeypairError::InvalidSeedHex`],
    /// or [`KeypairError::SeedDerivationFailed`] if the URI components are invalid.
    pub fn from_uri(uri: &SecretUri) -> Result<Self, KeypairError> {
        let inner = InnerKeypair::from_uri(uri)?;
        let phrase = uri.phrase.expose_secret();
        let base_seed = if let Some(hex_str) = phrase.strip_prefix("0x") {
            let bytes: SecretKeyBytes = hex::FromHex::from_hex(hex_str)
                .map_err(|e: hex::FromHexError| KeypairError::InvalidSeedHex(e.to_string()))?;
            bytes
        } else {
            let mnemonic = bip39::Mnemonic::from_str(phrase)
                .map_err(|e: bip39::Error| KeypairError::InvalidMnemonic(e.to_string()))?;
            let pass = uri.password.as_ref().map(|p| p.expose_secret());
            derive_seed_from_mnemonic(&mnemonic, pass)?
        };

        let seed = derive_seed_from_parent(&base_seed, &uri.junctions);
        Ok(Self { inner, seed })
    }

    /// Create a keypair from a BIP-39 mnemonic and optional password.
    ///
    /// The seed is derived using PBKDF2 with the mnemonic entropy, matching
    /// the Python SDK's key derivation.
    pub fn from_phrase(
        mnemonic: &bip39::Mnemonic,
        password: Option<&str>,
    ) -> Result<Self, KeypairError> {
        let seed = derive_seed_from_mnemonic(mnemonic, password)?;
        let inner = InnerKeypair::from_phrase(mnemonic, password)?;
        Ok(Self { inner, seed })
    }

    /// Create a keypair from raw 32-byte secret key bytes.
    pub fn from_secret_key(seed: SecretKeyBytes) -> Result<Self, KeypairError> {
        let inner = InnerKeypair::from_secret_key(seed)?;
        Ok(Self { inner, seed })
    }

    /// Create a keypair from a hex-encoded seed string (with or without `0x` prefix).
    pub fn from_seed_hex(hex_str: &str) -> Result<Self, KeypairError> {
        let stripped = hex_str.strip_prefix("0x").unwrap_or(hex_str);
        let seed: SecretKeyBytes = hex::FromHex::from_hex(stripped)
            .map_err(|e: hex::FromHexError| KeypairError::InvalidSeedHex(e.to_string()))?;
        Self::from_secret_key(seed)
    }

    /// Derive a child keypair from this keypair using the given junctions.
    ///
    /// Supports both hard and soft derivation, matching subxt's derivation semantics.
    pub fn derive(&self, junctions: impl IntoIterator<Item = DeriveJunction>) -> Self {
        let junction_vec: Vec<DeriveJunction> = junctions.into_iter().collect();
        let inner = self.inner.derive(junction_vec.iter().copied());
        let seed = derive_seed_from_parent(&self.seed, &junction_vec);
        Self { inner, seed }
    }

    /// Return the SR25519 public key.
    pub fn public_key(&self) -> PublicKey {
        self.inner.public_key()
    }

    /// Sign a message with this keypair's secret key.
    pub fn sign(&self, message: &[u8]) -> Signature {
        self.inner.sign(message)
    }

    /// Return the SS58-encoded address for Bittensor's chain format (prefix 42).
    pub fn ss58_address(&self) -> String {
        ss58::encode_ss58(&self.public_key().0, BT_SS58_FORMAT)
    }

    /// Return the hex-encoded seed with a `0x` prefix.
    pub fn seed_hex(&self) -> String {
        format!("0x{}", hex::encode(self.seed))
    }

    /// Return a reference to the raw secret key bytes.
    pub fn seed(&self) -> &SecretKeyBytes {
        &self.seed
    }

    /// Returns a reference to the inner `subxt_signer::sr25519::Keypair`
    /// for use with subxt transaction signing.
    pub fn signer(&self) -> &InnerKeypair {
        &self.inner
    }

    /// Consumes self and returns the inner `subxt_signer::sr25519::Keypair`,
    /// useful when you need an owned keypair for moving into async closures.
    pub fn into_signer(self) -> InnerKeypair {
        self.inner
    }

    /// Load a keypair from a NaCl-encrypted coldkey JSON file.
    ///
    /// The file is decrypted using the provided password, then the `secretSeed`
    /// field is extracted to reconstruct the keypair.
    pub fn from_encrypted_coldkey(
        path: &std::path::Path,
        password: &str,
    ) -> Result<Self, KeypairError> {
        let encrypted_data = std::fs::read(path)?;
        let decrypted = keyfile::decrypt(&encrypted_data, password.as_bytes())?;
        let json_str = String::from_utf8(decrypted).map_err(|e| {
            KeypairError::Keyfile(keyfile::KeyfileError::InvalidEncryption(format!(
                "Decrypted data is not valid UTF-8: {e}"
            )))
        })?;
        let json: serde_json::Value = serde_json::from_str(&json_str).map_err(|e| {
            KeypairError::Keyfile(keyfile::KeyfileError::InvalidEncryption(format!(
                "Decrypted data is not valid JSON: {e}"
            )))
        })?;
        let seed_hex = json["secretSeed"].as_str().ok_or_else(|| {
            KeypairError::Keyfile(keyfile::KeyfileError::InvalidEncryption(
                "No secretSeed field in coldkey JSON".into(),
            ))
        })?;
        Self::from_seed_hex(seed_hex)
    }

    /// Load a keypair from a plaintext hotkey file containing a hex-encoded seed.
    pub fn from_hotkey_file(path: &std::path::Path) -> Result<Self, KeypairError> {
        let data = std::fs::read_to_string(path)?;
        Self::from_seed_hex(data.trim())
    }
}

/// Verify an SR25519 signature against a message and public key.
///
/// Returns `true` if the signature is valid for the given message and public key.
pub fn verify(signature: &Signature, message: &[u8], public_key: &PublicKey) -> bool {
    sr25519::verify(signature, message, public_key)
}

impl From<schnorrkel::SignatureError> for KeypairError {
    fn from(e: schnorrkel::SignatureError) -> Self {
        KeypairError::Schnorrkel(e.to_string())
    }
}

fn derive_seed_from_mnemonic(
    mnemonic: &bip39::Mnemonic,
    password: Option<&str>,
) -> Result<SecretKeyBytes, KeypairError> {
    let (arr, len) = mnemonic.to_entropy_array();
    let big_seed = seed_from_entropy(&arr[..len], password.unwrap_or(""))
        .ok_or(KeypairError::SeedDerivationFailed)?;
    let seed: SecretKeyBytes = big_seed[..SEED_LEN].try_into().expect("seed length is correct");
    Ok(seed)
}

fn seed_from_entropy(entropy: &[u8], password: &str) -> Option<[u8; 64]> {
    if entropy.len() < 16 || entropy.len() > 32 || entropy.len() % 4 != 0 {
        return None;
    }
    let mut salt = String::with_capacity(8 + password.len());
    salt.push_str("mnemonic");
    salt.push_str(password);
    let mut seed = [0u8; 64];
    pbkdf2::<Hmac<Sha512>>(entropy, salt.as_bytes(), 2048, &mut seed).ok()?;
    Some(seed)
}

fn derive_seed_from_parent(
    parent_seed: &SecretKeyBytes,
    junctions: &[DeriveJunction],
) -> SecretKeyBytes {
    let mut current_mini = MiniSecretKey::from_bytes(parent_seed).expect("parent seed is valid");
    let mut current_secret = current_mini.expand(ExpansionMode::Ed25519);

    for junction in junctions {
        match junction {
            DeriveJunction::Hard(cc) => {
                let chain_code = ChainCode(*cc);
                current_mini = current_secret.hard_derive_mini_secret_key(Some(chain_code), b"").0;
                current_secret = current_mini.expand(ExpansionMode::Ed25519);
            }
            DeriveJunction::Soft(cc) => {
                let chain_code = ChainCode(*cc);
                current_secret = current_secret.derived_key_simple(chain_code, []).0;
                let bytes = current_secret.to_bytes();
                let mut new_seed = [0u8; SEED_LEN];
                new_seed.copy_from_slice(&bytes[..SEED_LEN]);
                current_mini = MiniSecretKey::from_bytes(&new_seed).expect("derived seed is valid");
            }
        }
    }

    current_mini.to_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_alice_address() {
        let alice = subxt_signer::sr25519::dev::alice();
        let pk = alice.public_key();
        eprintln!("Alice pubkey: 0x{}", hex::encode(pk.0));
        let addr = ss58::encode_ss58(&pk.0, BT_SS58_FORMAT);
        eprintln!("Alice SS58 address: {addr}");
    }

    #[test]
    fn alice_from_uri() {
        let alice_dev = subxt_signer::sr25519::dev::alice();
        let uri = SecretUri::from_str("//Alice").expect("parse uri");
        let kp = Keypair::from_uri(&uri).expect("keypair from uri");
        assert_eq!(kp.public_key().0, alice_dev.public_key().0);
    }

    #[test]
    fn sign_and_verify_roundtrip() {
        let uri = SecretUri::from_str("//Alice").expect("parse uri");
        let kp = Keypair::from_uri(&uri).expect("keypair");
        let msg = b"hello bittensor";
        let sig = kp.sign(msg);
        assert!(verify(&sig, msg, &kp.public_key()));
    }

    #[test]
    fn sign_verify_wrong_message_fails() {
        let uri = SecretUri::from_str("//Alice").expect("parse uri");
        let kp = Keypair::from_uri(&uri).expect("keypair");
        let sig = kp.sign(b"correct message");
        assert!(!verify(&sig, b"wrong message", &kp.public_key()));
    }

    #[test]
    fn from_known_mnemonic() {
        let phrase = "bottom drive obey lake curtain smoke basket hold race lonely fit walk";
        let mnemonic = bip39::Mnemonic::parse(phrase).expect("parse mnemonic");
        let kp = Keypair::from_phrase(&mnemonic, None).expect("from phrase");
        let alice_uri = SecretUri::from_str("//Alice").expect("parse uri");
        let alice_kp = Keypair::from_uri(&alice_uri).expect("alice keypair");
        assert_ne!(kp.public_key().0, alice_kp.public_key().0);
        assert_eq!(kp.ss58_address(), "5DfhGyQdFobKM8NsWvEeAKk5EQQgYe9AydgJ7rMB6E1EqRzV");
        assert_eq!(alice_kp.ss58_address(), "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY");
    }

    #[test]
    fn derive_hard_junction() {
        let uri = SecretUri::from_str("//Alice").expect("parse uri");
        let kp = Keypair::from_uri(&uri).expect("keypair");
        let derived = kp.derive([DeriveJunction::hard("stash")]);
        let stash_uri = SecretUri::from_str("//Alice//stash").expect("parse uri");
        let stash_kp = Keypair::from_uri(&stash_uri).expect("stash keypair");
        assert_eq!(derived.public_key().0, stash_kp.public_key().0);
    }

    #[test]
    fn seed_hex_roundtrip() {
        let uri = SecretUri::from_str("//Alice").expect("parse uri");
        let kp = Keypair::from_uri(&uri).expect("keypair");
        let seed = kp.seed_hex();
        let kp2 = Keypair::from_seed_hex(&seed).expect("from seed hex");
        assert_eq!(kp.public_key().0, kp2.public_key().0);
        assert_eq!(kp.seed, kp2.seed);
    }

    #[test]
    fn from_secret_key() {
        let seed = [42u8; 32];
        let kp = Keypair::from_secret_key(seed).expect("from secret key");
        assert_eq!(kp.seed, seed);
        assert!(!kp.ss58_address().is_empty());
    }
}
