//! Wallet management for Bittensor.
//!
//! This module provides the main `Wallet` struct for managing coldkeys and hotkeys,
//! compatible with the Python Bittensor SDK wallet structure.
//!
//! ## Wallet Structure
//!
//! A Bittensor wallet consists of:
//! - **Coldkey**: The main key that holds funds and controls the hotkey
//! - **Hotkey**: The key used for network operations (mining, validation)
//!
//! Wallets are stored in the filesystem with the following structure:
//! ```text
//! ~/.bittensor/wallets/
//!   └── <wallet_name>/
//!       ├── coldkey          # Encrypted coldkey
//!       ├── coldkeypub.txt   # Public coldkey SS58 address
//!       └── hotkeys/
//!           └── <hotkey_name> # Encrypted hotkey
//! ```

use crate::wallet::keyfile::{Keyfile, KeyfileError};
use crate::wallet::keypair::{Keypair, KeypairError};
use crate::wallet::mnemonic::{Mnemonic, MnemonicError};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Default wallet directory name under home
const WALLET_DIR_NAME: &str = ".bittensor/wallets";

/// Default coldkey filename
const COLDKEY_FILENAME: &str = "coldkey";

/// Coldkey public key filename
const COLDKEYPUB_FILENAME: &str = "coldkeypub.txt";

/// Hotkeys directory name
const HOTKEYS_DIR: &str = "hotkeys";

/// Default wallet name
#[allow(dead_code)]
const DEFAULT_WALLET_NAME: &str = "default";

/// Default hotkey name
const DEFAULT_HOTKEY_NAME: &str = "default";

/// Errors that can occur during wallet operations.
#[derive(Debug, Error)]
pub enum WalletError {
    #[error("Wallet directory not found: {0}")]
    DirectoryNotFound(PathBuf),

    #[error("Coldkey not found for wallet: {0}")]
    ColdkeyNotFound(String),

    #[error("Hotkey not found: {0}")]
    HotkeyNotFound(String),

    #[error("Keyfile error: {0}")]
    Keyfile(#[from] KeyfileError),

    #[error("Keypair error: {0}")]
    Keypair(#[from] KeypairError),

    #[error("Mnemonic error: {0}")]
    Mnemonic(#[from] MnemonicError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Wallet already exists: {0}")]
    AlreadyExists(String),

    #[error("Invalid wallet path: {0}")]
    InvalidPath(String),

    #[error("Home directory not found")]
    HomeNotFound,

    #[error("Invalid name: {0}")]
    InvalidName(String),
}

/// Sanitize a name to prevent path traversal attacks.
///
/// # Arguments
/// * `name` - The name to validate
///
/// # Returns
/// The validated name, or an error if the name contains invalid characters.
///
/// # Security
/// This function prevents directory traversal attacks (CWE-22) by rejecting:
/// - Path separators (`/` or `\`)
/// - Parent directory references (`..`)
/// - Empty or whitespace-only names
/// - Names starting with a dot (hidden files)
fn sanitize_name(name: &str) -> Result<&str, WalletError> {
    // Reject names with path separators or traversal sequences
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err(WalletError::InvalidName(format!(
            "Name '{}' contains invalid path characters",
            name
        )));
    }
    // Reject empty or whitespace-only names
    if name.trim().is_empty() {
        return Err(WalletError::InvalidName("Name cannot be empty".to_string()));
    }
    // Reject names starting with dots (hidden files)
    if name.starts_with('.') {
        return Err(WalletError::InvalidName(format!(
            "Name '{}' cannot start with a dot",
            name
        )));
    }
    Ok(name)
}

/// A Bittensor wallet containing coldkey and hotkey.
///
/// The wallet manages two keypairs:
/// - `coldkey`: Main key that holds funds
/// - `hotkey`: Key used for network operations
pub struct Wallet {
    /// Wallet name
    pub name: String,
    /// Base path for wallet storage
    pub path: PathBuf,
    /// Name of the hotkey to use
    pub hotkey_name: String,
    coldkey: Keyfile,
    hotkey: Keyfile,
}

impl std::fmt::Debug for Wallet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Wallet")
            .field("name", &self.name)
            .field("path", &self.path)
            .field("hotkey_name", &self.hotkey_name)
            .finish()
    }
}

impl Wallet {
    /// Create a new wallet handle without creating files on disk.
    ///
    /// # Arguments
    /// * `name` - Wallet name (directory name under wallets/)
    /// * `hotkey` - Hotkey name
    /// * `path` - Optional custom base path (defaults to ~/.bittensor/wallets)
    ///
    /// # Returns
    /// A new wallet handle, or an error if the name or hotkey contains invalid characters.
    ///
    /// # Security
    /// Both `name` and `hotkey` are sanitized to prevent path traversal attacks.
    /// Names containing `/`, `\`, `..`, or starting with `.` will be rejected.
    ///
    /// # Example
    /// ```
    /// use bittensor_rs::wallet::Wallet;
    /// let wallet = Wallet::new("my_wallet", "default", None).unwrap();
    /// ```
    pub fn new(name: &str, hotkey: &str, path: Option<&str>) -> Result<Self, WalletError> {
        // Sanitize inputs to prevent path traversal attacks
        let name = sanitize_name(name)?;
        let hotkey = sanitize_name(hotkey)?;

        let base_path = match path {
            Some(p) => PathBuf::from(p),
            None => default_wallet_path(),
        };

        let wallet_path = base_path.join(name);
        let coldkey_path = wallet_path.join(COLDKEY_FILENAME);
        let hotkey_path = wallet_path.join(HOTKEYS_DIR).join(hotkey);

        Ok(Self {
            name: name.to_string(),
            path: wallet_path,
            hotkey_name: hotkey.to_string(),
            coldkey: Keyfile::new(coldkey_path),
            hotkey: Keyfile::new(hotkey_path),
        })
    }

    /// Create a new wallet with both coldkey and hotkey.
    ///
    /// # Arguments
    /// * `name` - Wallet name
    /// * `hotkey` - Hotkey name
    /// * `password` - Optional password for encryption
    ///
    /// # Returns
    /// A new wallet with generated keys, or an error if creation fails.
    ///
    /// # Example
    /// ```no_run
    /// use bittensor_rs::wallet::Wallet;
    /// let wallet = Wallet::create("new_wallet", "default", Some("password")).unwrap();
    /// ```
    pub fn create(name: &str, hotkey: &str, password: Option<&str>) -> Result<Self, WalletError> {
        let mut wallet = Self::new(name, hotkey, None)?;

        // Create coldkey
        wallet.create_coldkey(password, None, false)?;

        // Create hotkey
        wallet.create_hotkey(password, None, false)?;

        Ok(wallet)
    }

    /// Create a new wallet with both coldkey and hotkey at a custom path.
    ///
    /// # Arguments
    /// * `name` - Wallet name
    /// * `hotkey` - Hotkey name
    /// * `path` - Custom base path for wallet storage
    /// * `password` - Optional password for encryption
    ///
    /// # Returns
    /// A new wallet with generated keys.
    pub fn create_at_path(
        name: &str,
        hotkey: &str,
        path: &str,
        password: Option<&str>,
    ) -> Result<Self, WalletError> {
        let mut wallet = Self::new(name, hotkey, Some(path))?;

        wallet.create_coldkey(password, None, false)?;
        wallet.create_hotkey(password, None, false)?;

        Ok(wallet)
    }

    /// Create or regenerate the coldkey.
    ///
    /// # Arguments
    /// * `password` - Optional password for encryption
    /// * `mnemonic` - Optional mnemonic for recovery (generates new if None)
    /// * `overwrite` - Whether to overwrite existing coldkey
    ///
    /// # Returns
    /// The mnemonic phrase used (save this for recovery!).
    pub fn create_coldkey(
        &mut self,
        password: Option<&str>,
        mnemonic: Option<&str>,
        overwrite: bool,
    ) -> Result<String, WalletError> {
        let (mnemonic_obj, provided_phrase) = match mnemonic {
            Some(phrase) => (Mnemonic::from_phrase(phrase)?, Some(phrase.to_string())),
            None => (Mnemonic::generate(), None),
        };

        let keypair = Keypair::from_mnemonic_obj(&mnemonic_obj, password)?;

        // Store the mnemonic phrase before potentially moving it
        let phrase = provided_phrase.unwrap_or_else(|| mnemonic_obj.phrase().to_string());

        // Ensure wallet directory exists
        fs::create_dir_all(&self.path)?;

        // Save coldkey
        self.coldkey
            .set_keypair(keypair.clone(), password, overwrite)?;

        // Save public key file
        self.save_coldkey_pub(&keypair)?;

        Ok(phrase)
    }

    /// Create or regenerate the hotkey.
    ///
    /// # Arguments
    /// * `password` - Optional password for encryption
    /// * `mnemonic` - Optional mnemonic for recovery (generates new if None)
    /// * `overwrite` - Whether to overwrite existing hotkey
    ///
    /// # Returns
    /// The mnemonic phrase used (save this for recovery!).
    pub fn create_hotkey(
        &mut self,
        password: Option<&str>,
        mnemonic: Option<&str>,
        overwrite: bool,
    ) -> Result<String, WalletError> {
        let (mnemonic_obj, provided_phrase) = match mnemonic {
            Some(phrase) => (Mnemonic::from_phrase(phrase)?, Some(phrase.to_string())),
            None => (Mnemonic::generate(), None),
        };

        let keypair = Keypair::from_mnemonic_obj(&mnemonic_obj, password)?;
        let phrase = provided_phrase.unwrap_or_else(|| mnemonic_obj.phrase().to_string());

        // Ensure hotkeys directory exists
        let hotkeys_dir = self.path.join(HOTKEYS_DIR);
        fs::create_dir_all(&hotkeys_dir)?;

        // Save hotkey
        self.hotkey.set_keypair(keypair, password, overwrite)?;

        Ok(phrase)
    }

    /// Get a reference to the coldkey keyfile.
    pub fn coldkey(&self) -> &Keyfile {
        &self.coldkey
    }

    /// Get a reference to the hotkey keyfile.
    pub fn hotkey(&self) -> &Keyfile {
        &self.hotkey
    }

    /// Get the coldkey keypair.
    ///
    /// # Arguments
    /// * `password` - Password for decryption (if encrypted)
    pub fn coldkey_keypair(&self, password: Option<&str>) -> Result<Keypair, WalletError> {
        self.coldkey
            .get_keypair(password)
            .map_err(WalletError::Keyfile)
    }

    /// Get the hotkey keypair.
    ///
    /// # Arguments
    /// * `password` - Password for decryption (if encrypted)
    pub fn hotkey_keypair(&self, password: Option<&str>) -> Result<Keypair, WalletError> {
        self.hotkey
            .get_keypair(password)
            .map_err(WalletError::Keyfile)
    }

    /// Get the coldkey SS58 address.
    ///
    /// This reads from the coldkeypub.txt file if available, otherwise
    /// decrypts the coldkey to get the address.
    pub fn coldkey_ss58(&self, password: Option<&str>) -> Result<String, WalletError> {
        // Try to read from coldkeypub.txt first
        let pub_path = self.path.join(COLDKEYPUB_FILENAME);
        if pub_path.exists() {
            if let Ok(content) = fs::read_to_string(&pub_path) {
                let address = content.trim().to_string();
                if !address.is_empty() {
                    return Ok(address);
                }
            }
        }

        // Fall back to decrypting coldkey
        let keypair = self.coldkey_keypair(password)?;
        Ok(keypair.ss58_address().to_string())
    }

    /// Get the hotkey SS58 address.
    pub fn hotkey_ss58(&self, password: Option<&str>) -> Result<String, WalletError> {
        let keypair = self.hotkey_keypair(password)?;
        Ok(keypair.ss58_address().to_string())
    }

    /// Check if the coldkey exists on disk.
    pub fn coldkey_exists(&self) -> bool {
        self.coldkey.exists()
    }

    /// Check if the hotkey exists on disk.
    pub fn hotkey_exists(&self) -> bool {
        self.hotkey.exists()
    }

    /// Check if both coldkey and hotkey exist.
    pub fn exists(&self) -> bool {
        self.coldkey_exists() && self.hotkey_exists()
    }

    /// Regenerate a wallet from a coldkey mnemonic.
    ///
    /// # Arguments
    /// * `name` - Wallet name
    /// * `mnemonic` - The coldkey mnemonic phrase
    /// * `password` - Optional password for derivation and encryption
    ///
    /// # Returns
    /// A wallet with the regenerated coldkey (hotkey must be created separately).
    pub fn regenerate_coldkey(
        name: &str,
        mnemonic: &str,
        password: Option<&str>,
    ) -> Result<Self, WalletError> {
        let mut wallet = Self::new(name, DEFAULT_HOTKEY_NAME, None)?;
        wallet.create_coldkey(password, Some(mnemonic), true)?;
        Ok(wallet)
    }

    /// Regenerate a hotkey from a mnemonic.
    ///
    /// # Arguments
    /// * `name` - Wallet name
    /// * `hotkey_name` - Hotkey name
    /// * `mnemonic` - The hotkey mnemonic phrase
    /// * `password` - Optional password for derivation and encryption
    ///
    /// # Returns
    /// A wallet handle with the regenerated hotkey.
    pub fn regenerate_hotkey(
        name: &str,
        hotkey_name: &str,
        mnemonic: &str,
        password: Option<&str>,
    ) -> Result<Self, WalletError> {
        let mut wallet = Self::new(name, hotkey_name, None)?;
        wallet.create_hotkey(password, Some(mnemonic), true)?;
        Ok(wallet)
    }

    /// List all hotkeys for this wallet.
    ///
    /// # Returns
    /// A list of hotkey names.
    pub fn list_hotkeys(&self) -> Result<Vec<String>, WalletError> {
        let hotkeys_dir = self.path.join(HOTKEYS_DIR);
        if !hotkeys_dir.exists() {
            return Ok(Vec::new());
        }

        let mut hotkeys = Vec::new();
        for entry in fs::read_dir(&hotkeys_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                if let Some(name) = entry.file_name().to_str() {
                    hotkeys.push(name.to_string());
                }
            }
        }

        hotkeys.sort();
        Ok(hotkeys)
    }

    /// Switch to a different hotkey.
    ///
    /// # Arguments
    /// * `hotkey_name` - Name of the hotkey to switch to
    ///
    /// # Returns
    /// Ok(()) on success, or an error if the hotkey name is invalid.
    ///
    /// # Security
    /// The hotkey name is sanitized to prevent path traversal attacks.
    pub fn use_hotkey(&mut self, hotkey_name: &str) -> Result<(), WalletError> {
        let hotkey_name = sanitize_name(hotkey_name)?;
        self.hotkey_name = hotkey_name.to_string();
        let hotkey_path = self.path.join(HOTKEYS_DIR).join(hotkey_name);
        self.hotkey = Keyfile::new(hotkey_path);
        Ok(())
    }

    /// Save the coldkey public address to coldkeypub.txt.
    ///
    /// # Security
    /// The file is created with restrictive permissions (0o600 on Unix)
    /// to prevent unauthorized access to the public key.
    fn save_coldkey_pub(&self, keypair: &Keypair) -> Result<(), WalletError> {
        let pub_path = self.path.join(COLDKEYPUB_FILENAME);
        let mut file = fs::File::create(&pub_path)?;
        writeln!(file, "{}", keypair.ss58_address())?;

        // Set restrictive permissions on Unix (readable by owner only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = fs::Permissions::from_mode(0o600);
            fs::set_permissions(&pub_path, permissions)?;
        }

        Ok(())
    }
}

/// Get the default wallet path (~/.bittensor/wallets).
///
/// # Returns
/// The default wallet directory path.
///
/// # Panics
/// Panics if the home directory cannot be determined.
pub fn default_wallet_path() -> PathBuf {
    dirs::home_dir()
        .map(|home| home.join(WALLET_DIR_NAME))
        .unwrap_or_else(|| PathBuf::from(WALLET_DIR_NAME))
}

/// Get the full path to a specific wallet.
///
/// # Arguments
/// * `name` - Wallet name
///
/// # Returns
/// The full path to the wallet directory.
pub fn wallet_path(name: &str) -> PathBuf {
    default_wallet_path().join(name)
}

/// List all wallets in the default wallet directory.
///
/// # Returns
/// A list of wallet names.
pub fn list_wallets() -> Result<Vec<String>, WalletError> {
    list_wallets_at(&default_wallet_path())
}

/// List all wallets at a specific path.
///
/// # Arguments
/// * `path` - The wallet directory path
///
/// # Returns
/// A list of wallet names.
pub fn list_wallets_at(path: &Path) -> Result<Vec<String>, WalletError> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let mut wallets = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            // Check if it has a coldkey (makes it a valid wallet)
            let coldkey_path = entry.path().join(COLDKEY_FILENAME);
            if coldkey_path.exists() {
                if let Some(name) = entry.file_name().to_str() {
                    wallets.push(name.to_string());
                }
            }
        }
    }

    wallets.sort();
    Ok(wallets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_wallet_new() {
        let wallet = Wallet::new("test_wallet", "test_hotkey", None).unwrap();
        assert_eq!(wallet.name, "test_wallet");
        assert_eq!(wallet.hotkey_name, "test_hotkey");
    }

    #[test]
    fn test_wallet_create() {
        let dir = tempdir().unwrap();
        let base_path = dir.path().to_str().unwrap();

        let wallet = Wallet::create_at_path("test_wallet", "default", base_path, None).unwrap();

        assert!(wallet.coldkey_exists());
        assert!(wallet.hotkey_exists());
        assert!(wallet.exists());
    }

    #[test]
    fn test_wallet_create_with_password() {
        let dir = tempdir().unwrap();
        let base_path = dir.path().to_str().unwrap();
        let password = "test_password";

        // Create wallet with password
        let wallet =
            Wallet::create_at_path("test_wallet", "default", base_path, Some(password)).unwrap();

        // Should be able to get keypairs with password (from cached version)
        let coldkey = wallet.coldkey_keypair(Some(password)).unwrap();
        let hotkey = wallet.hotkey_keypair(Some(password)).unwrap();

        assert!(!coldkey.ss58_address().is_empty());
        assert!(!hotkey.ss58_address().is_empty());

        // Create a fresh wallet instance pointing to the same files
        // This tests that reading from disk requires password
        let wallet2 = Wallet::new("test_wallet", "default", Some(base_path)).unwrap();

        // Should fail without password when reading from disk
        assert!(wallet2.coldkey_keypair(None).is_err());
        assert!(wallet2.hotkey_keypair(None).is_err());

        // Should succeed with correct password
        assert!(wallet2.coldkey_keypair(Some(password)).is_ok());
        assert!(wallet2.hotkey_keypair(Some(password)).is_ok());
    }

    #[test]
    fn test_wallet_regenerate_coldkey() {
        let dir = tempdir().unwrap();
        let base_path = dir.path().to_str().unwrap();
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

        let mut wallet = Wallet::new("test_wallet", "default", Some(base_path)).unwrap();
        let returned_mnemonic = wallet.create_coldkey(None, Some(mnemonic), false).unwrap();

        assert_eq!(returned_mnemonic, mnemonic);

        // Should be deterministic
        let keypair1 = wallet.coldkey_keypair(None).unwrap();

        // Create another wallet with same mnemonic
        let mut wallet2 = Wallet::new("test_wallet2", "default", Some(base_path)).unwrap();
        wallet2.create_coldkey(None, Some(mnemonic), false).unwrap();
        let keypair2 = wallet2.coldkey_keypair(None).unwrap();

        assert_eq!(keypair1.ss58_address(), keypair2.ss58_address());
    }

    #[test]
    fn test_wallet_list_hotkeys() {
        let dir = tempdir().unwrap();
        let base_path = dir.path().to_str().unwrap();

        let mut wallet = Wallet::new("test_wallet", "hotkey1", Some(base_path)).unwrap();
        wallet.create_coldkey(None, None, false).unwrap();
        wallet.create_hotkey(None, None, false).unwrap();

        // Create second hotkey
        wallet.use_hotkey("hotkey2").unwrap();
        wallet.create_hotkey(None, None, false).unwrap();

        let hotkeys = wallet.list_hotkeys().unwrap();
        assert_eq!(hotkeys.len(), 2);
        assert!(hotkeys.contains(&"hotkey1".to_string()));
        assert!(hotkeys.contains(&"hotkey2".to_string()));
    }

    #[test]
    fn test_list_wallets() {
        let dir = tempdir().unwrap();
        let base_path = dir.path().to_str().unwrap();

        // Create multiple wallets
        Wallet::create_at_path("wallet1", "default", base_path, None).unwrap();
        Wallet::create_at_path("wallet2", "default", base_path, None).unwrap();

        let wallets = list_wallets_at(dir.path()).unwrap();
        assert_eq!(wallets.len(), 2);
        assert!(wallets.contains(&"wallet1".to_string()));
        assert!(wallets.contains(&"wallet2".to_string()));
    }

    #[test]
    fn test_coldkey_ss58() {
        let dir = tempdir().unwrap();
        let base_path = dir.path().to_str().unwrap();

        let wallet = Wallet::create_at_path("test_wallet", "default", base_path, None).unwrap();

        let ss58 = wallet.coldkey_ss58(None).unwrap();
        assert!(!ss58.is_empty());
        assert!(ss58.starts_with('5')); // Substrate SS58 format
    }

    #[test]
    fn test_wallet_use_hotkey() {
        let dir = tempdir().unwrap();
        let base_path = dir.path().to_str().unwrap();

        let mut wallet = Wallet::create_at_path("test_wallet", "hotkey1", base_path, None).unwrap();

        assert_eq!(wallet.hotkey_name, "hotkey1");

        wallet.use_hotkey("hotkey2").unwrap();
        assert_eq!(wallet.hotkey_name, "hotkey2");
        assert!(!wallet.hotkey_exists()); // hotkey2 doesn't exist yet
    }

    #[test]
    fn test_wallet_path_functions() {
        let default_path = default_wallet_path();
        assert!(default_path.ends_with(".bittensor/wallets"));

        let specific_path = wallet_path("my_wallet");
        assert!(specific_path.ends_with("my_wallet"));
    }

    #[test]
    fn test_coldkeypub_txt() {
        let dir = tempdir().unwrap();
        let base_path = dir.path().to_str().unwrap();

        let wallet = Wallet::create_at_path("test_wallet", "default", base_path, None).unwrap();

        // Check coldkeypub.txt was created
        let pub_path = dir.path().join("test_wallet").join("coldkeypub.txt");
        assert!(pub_path.exists());

        // Content should match SS58 address
        let content = fs::read_to_string(&pub_path).unwrap();
        let ss58 = wallet.coldkey_ss58(None).unwrap();
        assert_eq!(content.trim(), ss58);
    }

    #[test]
    fn test_path_traversal_prevention() {
        // These should all fail due to path traversal protection
        assert!(sanitize_name("../evil").is_err());
        assert!(sanitize_name("foo/../bar").is_err());
        assert!(sanitize_name("foo/bar").is_err());
        assert!(sanitize_name("foo\\bar").is_err());
        assert!(sanitize_name(".hidden").is_err());
        assert!(sanitize_name("").is_err());
        assert!(sanitize_name("   ").is_err());

        // These should succeed
        assert!(sanitize_name("valid_name").is_ok());
        assert!(sanitize_name("wallet-1").is_ok());
        assert!(sanitize_name("MyWallet").is_ok());
    }

    #[test]
    fn test_wallet_new_rejects_path_traversal() {
        // Wallet::new should reject path traversal attempts
        assert!(Wallet::new("../evil", "default", None).is_err());
        assert!(Wallet::new("good", "../evil", None).is_err());
        assert!(Wallet::new(".hidden", "default", None).is_err());
        assert!(Wallet::new("good", ".hidden", None).is_err());
        assert!(Wallet::new("foo/bar", "default", None).is_err());
        assert!(Wallet::new("good", "foo/bar", None).is_err());

        // Valid names should work
        assert!(Wallet::new("valid_wallet", "valid_hotkey", None).is_ok());
    }

    #[test]
    fn test_use_hotkey_rejects_path_traversal() {
        let dir = tempdir().unwrap();
        let base_path = dir.path().to_str().unwrap();

        let mut wallet = Wallet::create_at_path("test_wallet", "default", base_path, None).unwrap();

        // Should reject path traversal in use_hotkey
        assert!(wallet.use_hotkey("../evil").is_err());
        assert!(wallet.use_hotkey(".hidden").is_err());
        assert!(wallet.use_hotkey("foo/bar").is_err());

        // Valid name should work
        assert!(wallet.use_hotkey("valid_hotkey").is_ok());
    }
}
