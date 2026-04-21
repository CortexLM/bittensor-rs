//! Wallet management for Bittensor, compatible with the Python SDK's file layout.

use std::path::PathBuf;

use crate::keyfile;
use crate::keypair::{Keypair, KeypairError};
use crate::mnemonic::{self, MnemonicError, WordCount};
use crate::ss58::Ss58Error;
use subxt_signer::bip39;

#[derive(Debug, thiserror::Error)]
pub enum WalletError {
    #[error("Keypair error: {0}")]
    Keypair(#[from] KeypairError),
    #[error("Mnemonic error: {0}")]
    Mnemonic(#[from] MnemonicError),
    #[error("SS58 error: {0}")]
    Ss58(#[from] Ss58Error),
    #[error("Keyfile error: {0}")]
    Keyfile(#[from] keyfile::KeyfileError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Wallet not found: {0}")]
    NotFound(String),
    #[error("Coldkey required but not loaded: {0}")]
    ColdkeyRequired(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Default wallet directory name under the home directory.
const WALLET_DIR_NAME: &str = ".bittensor";
const WALLETS_DIR_NAME: &str = "wallets";

/// Full wallet state including lazy-loaded coldkey pair.
pub struct Wallet {
    pub name: String,
    pub path: PathBuf,
    pub hotkey_name: String,
    coldkey: Option<Keypair>,
    coldkeypub: Option<Keypair>,
    hotkey: Option<Keypair>,
}

impl Wallet {
    /// Create a new Wallet pointing at the standard Bittensor directory layout.
    ///
    /// Directory structure (matches Python SDK):
    /// ```text
    /// ~/.bittensor/wallets/<name>/
    ///   coldkey        (encrypted NaCl)
    ///   coldkeypub     (plaintext SS58 address)
    ///   hotkeys/
    ///     <hotkey_name>  (raw hex seed, unencrypted)
    /// ```
    pub fn new(name: &str) -> Self {
        let path = default_wallet_path(name);
        Self {
            name: name.to_string(),
            path,
            hotkey_name: "default".to_string(),
            coldkey: None,
            coldkeypub: None,
            hotkey: None,
        }
    }

    /// Create a Wallet with a custom path (useful for testing).
    pub fn with_path(name: &str, path: PathBuf) -> Self {
        Self {
            name: name.to_string(),
            path,
            hotkey_name: "default".to_string(),
            coldkey: None,
            coldkeypub: None,
            hotkey: None,
        }
    }

    /// Set the hotkey name (defaults to "default").
    pub fn set_hotkey_name(&mut self, hotkey_name: &str) {
        self.hotkey_name = hotkey_name.to_string();
    }

    /// Create a new coldkey from a generated mnemonic, encrypt it, and write to disk.
    ///
    /// Returns the mnemonic so the user can back it up.
    pub fn create_coldkey(&mut self, password: &str) -> Result<bip39::Mnemonic, WalletError> {
        let mnemonic = mnemonic::generate_mnemonic(WordCount::Words12)?;
        self.create_coldkey_from_mnemonic(&mnemonic, password)?;
        Ok(mnemonic)
    }

    /// Create a coldkey from an existing mnemonic, encrypt it, and write to disk.
    pub fn create_coldkey_from_mnemonic(
        &mut self,
        mnemonic: &bip39::Mnemonic,
        password: &str,
    ) -> Result<(), WalletError> {
        let keypair = mnemonic::keypair_from_mnemonic(mnemonic, Some(password))?;
        self.coldkey = Some(keypair.clone());
        self.coldkeypub = Some(keypair.clone());

        std::fs::create_dir_all(&self.path)?;

        let coldkey_path = self.coldkey_path();
        let coldkeypub_path = self.coldkeypub_path();

        let coldkey_json = coldkey_json(&keypair, mnemonic);
        let encrypted = keyfile::encrypt(coldkey_json.as_bytes(), password.as_bytes())?;
        std::fs::write(&coldkey_path, &encrypted)?;

        let ss58_addr = keypair.ss58_address();
        std::fs::write(&coldkeypub_path, &ss58_addr)?;

        Ok(())
    }

    /// Create a new hotkey and write it to disk as raw hex seed (unencrypted, matching Python).
    pub fn create_hotkey(&mut self) -> Result<Keypair, WalletError> {
        let mnemonic = mnemonic::generate_mnemonic(WordCount::Words12)?;
        let keypair = mnemonic::keypair_from_mnemonic(&mnemonic, None)?;
        self.hotkey = Some(keypair.clone());

        let hotkeys_dir = self.path.join("hotkeys");
        std::fs::create_dir_all(&hotkeys_dir)?;

        let hotkey_path = self.hotkey_path();
        std::fs::write(&hotkey_path, keypair.seed_hex())?;

        Ok(keypair)
    }

    /// Create a hotkey derived from the coldkey using a hard derivation path.
    pub fn create_hotkey_from_coldkey(&mut self, password: &str) -> Result<Keypair, WalletError> {
        let coldkey = self.get_coldkey_pair(password)?;
        let derived = coldkey.derive([subxt_signer::DeriveJunction::hard(&self.hotkey_name)]);
        self.hotkey = Some(derived.clone());

        let hotkeys_dir = self.path.join("hotkeys");
        std::fs::create_dir_all(&hotkeys_dir)?;

        let hotkey_path = self.hotkey_path();
        std::fs::write(&hotkey_path, derived.seed_hex())?;

        Ok(derived)
    }

    /// Load the coldkey pair by decrypting the coldkey file.
    pub fn get_coldkey_pair(&mut self, password: &str) -> Result<Keypair, WalletError> {
        if let Some(ref kp) = self.coldkey {
            return Ok(kp.clone());
        }
        let path = self.coldkey_path();
        if !path.exists() {
            return Err(WalletError::NotFound(format!(
                "Coldkey file not found: {}",
                path.display()
            )));
        }
        let kp = Keypair::from_encrypted_coldkey(&path, password)?;
        self.coldkey = Some(kp.clone());
        Ok(kp)
    }

    /// Get the coldkey public key as an SS58 address (reading from coldkeypub file if available).
    pub fn get_coldkeypub(&mut self) -> Result<String, WalletError> {
        if let Some(ref kp) = self.coldkeypub {
            return Ok(kp.ss58_address());
        }

        let coldkeypub_path = self.coldkeypub_path();
        if coldkeypub_path.exists() {
            let addr = std::fs::read_to_string(&coldkeypub_path)?;
            return Ok(addr.trim().to_string());
        }

        // If no coldkeypub file, try loading coldkey (needs password)
        Err(WalletError::ColdkeyRequired("Password required to load coldkey for public key".into()))
    }

    /// Load the coldkeypub keypair (coldkey pair but only using public key).
    /// Requires password to decrypt the coldkey file.
    pub fn get_coldkeypub_pair(&mut self, password: &str) -> Result<Keypair, WalletError> {
        if let Some(ref kp) = self.coldkeypub {
            return Ok(kp.clone());
        }
        let kp = self.get_coldkey_pair(password)?;
        self.coldkeypub = Some(kp.clone());
        Ok(kp)
    }

    /// Load the hotkey pair from the hotkey file.
    pub fn get_hotkey_pair(&mut self) -> Result<Keypair, WalletError> {
        if let Some(ref kp) = self.hotkey {
            return Ok(kp.clone());
        }
        let path = self.hotkey_path();
        if !path.exists() {
            return Err(WalletError::NotFound(format!(
                "Hotkey file not found: {}",
                path.display()
            )));
        }
        let kp = Keypair::from_hotkey_file(&path)?;
        self.hotkey = Some(kp.clone());
        Ok(kp)
    }

    /// Sign a message with the hotkey.
    pub fn sign(
        &mut self,
        message: &[u8],
    ) -> Result<subxt_signer::sr25519::Signature, WalletError> {
        let kp = self.get_hotkey_pair()?;
        Ok(kp.sign(message))
    }

    /// Sign a message with the coldkey (requires password).
    pub fn sign_coldkey(
        &mut self,
        message: &[u8],
        password: &str,
    ) -> Result<subxt_signer::sr25519::Signature, WalletError> {
        let kp = self.get_coldkey_pair(password)?;
        Ok(kp.sign(message))
    }

    /// Verify a signature.
    pub fn verify(
        signature: &subxt_signer::sr25519::Signature,
        message: &[u8],
        public_key: &subxt_signer::sr25519::PublicKey,
    ) -> bool {
        crate::keypair::verify(signature, message, public_key)
    }

    /// Get the SS58 address of the hotkey.
    pub fn hotkey_ss58_address(&mut self) -> Result<String, WalletError> {
        let kp = self.get_hotkey_pair()?;
        Ok(kp.ss58_address())
    }

    /// Get the SS58 address of the coldkeypub.
    pub fn coldkey_ss58_address(&mut self, password: &str) -> Result<String, WalletError> {
        let kp = self.get_coldkey_pair(password)?;
        Ok(kp.ss58_address())
    }

    /// Path to the encrypted coldkey file (`<wallet_path>/coldkey`).
    pub fn coldkey_path(&self) -> PathBuf {
        self.path.join("coldkey")
    }

    /// Path to the plaintext coldkey public key file (`<wallet_path>/coldkeypub`).
    pub fn coldkeypub_path(&self) -> PathBuf {
        self.path.join("coldkeypub")
    }

    /// Path to the hotkey file (`<wallet_path>/hotkeys/<hotkey_name>`).
    pub fn hotkey_path(&self) -> PathBuf {
        self.path.join("hotkeys").join(&self.hotkey_name)
    }
}

fn default_wallet_path(name: &str) -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(WALLET_DIR_NAME).join(WALLETS_DIR_NAME).join(name)
}

/// Build the JSON payload stored in an encrypted coldkey file, matching the Python SDK format.
fn coldkey_json(keypair: &Keypair, mnemonic: &bip39::Mnemonic) -> String {
    let ss58 = keypair.ss58_address();
    let pubkey_hex = hex::encode(keypair.public_key().0);
    let seed_hex = keypair.seed_hex();
    let phrase = mnemonic.to_string();

    serde_json::json!({
        "accountId": format!("0x{pubkey_hex}"),
        "publicKey": format!("0x{pubkey_hex}"),
        "secretPhrase": phrase,
        "secretSeed": seed_hex,
        "ss58Address": ss58
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::TempDir;

    fn make_test_wallet(dir: &Path) -> Wallet {
        Wallet::with_path("test", dir.to_path_buf())
    }

    #[test]
    fn create_coldkey_and_read_back() {
        let dir = TempDir::new().expect("tempdir");
        let mut wallet = make_test_wallet(dir.path());
        let _mnemonic = wallet.create_coldkey("testpass").expect("create coldkey");

        assert!(wallet.coldkey_path().exists());
        assert!(wallet.coldkeypub_path().exists());

        let kp = wallet.get_coldkey_pair("testpass").expect("load coldkey");
        let addr = kp.ss58_address();
        assert!(addr.starts_with('5'));

        let pub_addr = wallet.get_coldkeypub().expect("coldkeypub");
        assert_eq!(addr, pub_addr);
    }

    #[test]
    fn create_hotkey_and_read_back() {
        let dir = TempDir::new().expect("tempdir");
        let mut wallet = make_test_wallet(dir.path());
        std::fs::create_dir_all(&wallet.path).expect("mkdir");
        let kp = wallet.create_hotkey().expect("create hotkey");
        assert!(wallet.hotkey_path().exists());

        let loaded = wallet.get_hotkey_pair().expect("load hotkey");
        assert_eq!(kp.public_key().0, loaded.public_key().0);
    }

    #[test]
    fn coldkeypub_file_contains_ss58() {
        let dir = TempDir::new().expect("tempdir");
        let mut wallet = make_test_wallet(dir.path());
        wallet.create_coldkey("pass").expect("create coldkey");

        let content = std::fs::read_to_string(wallet.coldkeypub_path()).expect("read coldkeypub");
        let addr = wallet.get_coldkeypub().expect("coldkeypub");
        assert_eq!(content.trim(), addr);
    }

    #[test]
    fn sign_and_verify_with_hotkey() {
        let dir = TempDir::new().expect("tempdir");
        let mut wallet = make_test_wallet(dir.path());
        std::fs::create_dir_all(&wallet.path).expect("mkdir");
        let kp = wallet.create_hotkey().expect("create hotkey");
        let msg = b"test message";
        let sig = wallet.sign(msg).expect("sign");
        assert!(Wallet::verify(&sig, msg, &kp.public_key()));
    }

    #[test]
    fn sign_and_verify_with_coldkey() {
        let dir = TempDir::new().expect("tempdir");
        let mut wallet = make_test_wallet(dir.path());
        wallet.create_coldkey("pass123").expect("create coldkey");
        let msg = b"coldkey sign test";
        let sig = wallet.sign_coldkey(msg, "pass123").expect("sign coldkey");
        let kp = wallet.get_coldkey_pair("pass123").expect("coldkey");
        assert!(Wallet::verify(&sig, msg, &kp.public_key()));
    }

    #[test]
    fn wrong_password_fails() {
        let dir = TempDir::new().expect("tempdir");
        let mut wallet = make_test_wallet(dir.path());
        wallet.create_coldkey("right_pass").expect("create coldkey");

        // Reset cached coldkey
        wallet.coldkey = None;
        let result = wallet.get_coldkey_pair("wrong_pass");
        assert!(result.is_err());
    }

    #[test]
    fn encryption_roundtrip_with_keyfile() {
        let dir = TempDir::new().expect("tempdir");
        let mut wallet = make_test_wallet(dir.path());
        let mnemonic = wallet.create_coldkey("mypassword").expect("create");

        let encrypted = std::fs::read(&wallet.coldkey_path()).expect("read coldkey");
        assert!(keyfile::is_encrypted_nacl(&encrypted));

        let decrypted = keyfile::decrypt(&encrypted, b"mypassword").expect("decrypt");
        let json: serde_json::Value = serde_json::from_slice(&decrypted).expect("parse json");
        assert_eq!(json["secretPhrase"], mnemonic.to_string());
        assert!(json["ss58Address"].as_str().unwrap().starts_with('5'));
    }

    #[test]
    fn custom_hotkey_name() {
        let dir = TempDir::new().expect("tempdir");
        let mut wallet = make_test_wallet(dir.path());
        wallet.set_hotkey_name("validator");
        std::fs::create_dir_all(&wallet.path).expect("mkdir");
        wallet.create_hotkey().expect("create hotkey");
        assert!(wallet.hotkey_path().to_string_lossy().contains("validator"));
    }
}
