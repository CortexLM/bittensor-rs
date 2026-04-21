//! Python bindings for bittensor-wallet: Wallet class with create, load, sign, verify.

use std::path::PathBuf;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyType;

use bittensor_wallet::wallet::Wallet as RustWallet;

// NOTE: Cannot impl From<WalletError> for PyErr due to orphan rules.
// Use `PyRuntimeError::new_err(format!("WalletError: {err}"))` explicitly.

/// Bittensor wallet with coldkey/hotkey management, compatible with Python SDK file layout.
///
/// Example:
///     w = Wallet.create("my-wallet", "/tmp/wallets")
///     addr = w.ss58_address
#[pyclass]
pub struct Wallet {
    inner: RustWallet,
}

#[pymethods]
impl Wallet {
    /// Create a new coldkey (encrypted) and hotkey, returning a Wallet.
    ///
    /// Args:
    ///     name: Wallet name (directory name under path).
    ///     path: Parent directory for wallet storage.
    ///     password: Password to encrypt the coldkey.
    ///
    /// Returns:
    ///     Wallet with keys written to disk.
    #[classmethod]
    #[pyo3(signature = (name, path, password=""))]
    fn create(
        _cls: &Bound<'_, PyType>,
        name: String,
        path: String,
        password: &str,
    ) -> PyResult<Self> {
        let path_buf = PathBuf::from(&path);
        let mut wallet = RustWallet::with_path(&name, path_buf);

        wallet
            .create_coldkey(password)
            .map_err(|e| PyRuntimeError::new_err(format!("create_coldkey failed: {e}")))?;

        std::fs::create_dir_all(wallet.path.join("hotkeys"))
            .map_err(|e| PyRuntimeError::new_err(format!("mkdir hotkeys failed: {e}")))?;

        wallet
            .create_hotkey()
            .map_err(|e| PyRuntimeError::new_err(format!("create_hotkey failed: {e}")))?;

        Ok(Self { inner: wallet })
    }

    /// Load an existing wallet from disk.
    ///
    /// Args:
    ///     name: Wallet name.
    ///     path: Parent directory for wallet storage.
    ///     hotkey_name: Hotkey name (defaults to "default").
    ///
    /// Returns:
    ///     Wallet loaded from the specified directory.
    #[classmethod]
    #[pyo3(signature = (name, path, hotkey_name="default"))]
    fn load(
        _cls: &Bound<'_, PyType>,
        name: String,
        path: String,
        hotkey_name: &str,
    ) -> PyResult<Self> {
        let path_buf = PathBuf::from(&path);
        let mut wallet = RustWallet::with_path(&name, path_buf);
        wallet.set_hotkey_name(hotkey_name);
        Ok(Self { inner: wallet })
    }

    /// SS58 address of the coldkeypub (read from file, no password needed).
    #[getter]
    fn ss58_address(&mut self) -> PyResult<String> {
        self.inner
            .get_coldkeypub()
            .map_err(|e| PyRuntimeError::new_err(format!("get_coldkeypub failed: {e}")))
    }

    /// Get the coldkeypub SS58 address. Requires password to decrypt coldkey if not already loaded.
    fn get_coldkeypub(&mut self) -> PyResult<String> {
        self.inner
            .get_coldkeypub()
            .map_err(|e| PyRuntimeError::new_err(format!("get_coldkeypub failed: {e}")))
    }

    /// Get the coldkey keypair SS58 address (requires password).
    fn get_coldkey_pair(&mut self, password: &str) -> PyResult<String> {
        let kp = self
            .inner
            .get_coldkey_pair(password)
            .map_err(|e| PyRuntimeError::new_err(format!("get_coldkey_pair failed: {e}")))?;
        Ok(kp.ss58_address())
    }

    /// Get the hotkey keypair SS58 address.
    fn get_hotkey_pair(&mut self) -> PyResult<String> {
        let kp = self
            .inner
            .get_hotkey_pair()
            .map_err(|e| PyRuntimeError::new_err(format!("get_hotkey_pair failed: {e}")))?;
        Ok(kp.ss58_address())
    }

    /// Sign a message with the hotkey. Returns hex-encoded signature.
    fn sign(&mut self, message: &[u8]) -> PyResult<String> {
        let sig = self
            .inner
            .sign(message)
            .map_err(|e| PyRuntimeError::new_err(format!("sign failed: {e}")))?;
        Ok(hex::encode(sig.0))
    }

    /// Sign a message with the coldkey (requires password). Returns hex-encoded signature.
    fn sign_coldkey(&mut self, message: &[u8], password: &str) -> PyResult<String> {
        let sig = self
            .inner
            .sign_coldkey(message, password)
            .map_err(|e| PyRuntimeError::new_err(format!("sign_coldkey failed: {e}")))?;
        Ok(hex::encode(sig.0))
    }

    /// Verify a signature.
    ///
    /// Args:
    ///     message: Original message bytes.
    ///     signature_hex: Hex-encoded 64-byte signature.
    ///     public_key_hex: Hex-encoded 32-byte public key.
    ///
    /// Returns:
    ///     True if the signature is valid.
    #[staticmethod]
    fn verify(message: &[u8], signature_hex: &str, public_key_hex: &str) -> PyResult<bool> {
        let sig_bytes = hex::decode(signature_hex)
            .map_err(|e| PyRuntimeError::new_err(format!("invalid signature hex: {e}")))?;
        let sig_arr: [u8; 64] = sig_bytes.try_into().map_err(|v: Vec<u8>| {
            PyRuntimeError::new_err(format!("signature must be 64 bytes, got {}", v.len()))
        })?;
        let sig = subxt_signer::sr25519::Signature(sig_arr);

        let pk_bytes = hex::decode(public_key_hex)
            .map_err(|e| PyRuntimeError::new_err(format!("invalid public key hex: {e}")))?;
        let pk_arr: [u8; 32] = pk_bytes.try_into().map_err(|v: Vec<u8>| {
            PyRuntimeError::new_err(format!("public key must be 32 bytes, got {}", v.len()))
        })?;
        let pk = subxt_signer::sr25519::PublicKey(pk_arr);

        Ok(bittensor_wallet::keypair::verify(&sig, message, &pk))
    }

    /// Wallet name.
    #[getter]
    fn name(&self) -> &str {
        &self.inner.name
    }

    /// Wallet path on disk.
    #[getter]
    fn path(&self) -> String {
        self.inner.path.to_string_lossy().to_string()
    }

    /// Hotkey name.
    #[getter]
    fn hotkey_name(&self) -> &str {
        &self.inner.hotkey_name
    }

    fn __repr__(&self) -> String {
        format!("Wallet(name='{}', path='{}')", self.inner.name, self.inner.path.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bittensor_wallet::prelude::Keypair;
    use std::path::PathBuf;

    #[test]
    fn wallet_with_path_fields() {
        let inner = RustWallet::with_path("mywallet", PathBuf::from("/tmp/test_wallets"));
        let w = Wallet { inner };
        assert_eq!(w.name(), "mywallet");
        assert_eq!(w.hotkey_name(), "default");
        assert!(w.path().contains("test_wallets"));
    }

    #[test]
    fn wallet_name_getter() {
        let inner = RustWallet::with_path("alice", PathBuf::from("/tmp/w"));
        let w = Wallet { inner };
        assert_eq!(w.name(), "alice");
    }

    #[test]
    fn wallet_hotkey_name_getter() {
        let inner = RustWallet::with_path("bob", PathBuf::from("/tmp/w2"));
        let w = Wallet { inner };
        assert_eq!(w.hotkey_name(), "default");
    }

    #[test]
    fn wallet_path_getter() {
        let inner = RustWallet::with_path("carol", PathBuf::from("/custom/path"));
        let w = Wallet { inner };
        let path_str = w.path();
        assert!(path_str.contains("custom") || path_str.contains("path"));
    }

    #[test]
    fn wallet_repr() {
        let inner = RustWallet::with_path("test", PathBuf::from("/tmp/w"));
        let w = Wallet { inner };
        let repr = w.__repr__();
        assert!(repr.contains("Wallet(name='test'"));
        assert!(repr.contains("path="));
    }

    #[test]
    fn wallet_verify_valid_signature() {
        let kp = Keypair::from_seed_hex(
            "0x0000000000000000000000000000000000000000000000000000000000000001",
        )
        .unwrap();
        let message = b"hello bittensor";
        let sig = kp.sign(message);
        let sig_hex = hex::encode(sig.0);
        let pk_hex = hex::encode(kp.public_key().0);
        let result = Wallet::verify(message, &sig_hex, &pk_hex).unwrap();
        assert!(result);
    }

    #[test]
    fn wallet_verify_tampered_message() {
        let kp = Keypair::from_seed_hex(
            "0x0000000000000000000000000000000000000000000000000000000000000001",
        )
        .unwrap();
        let message = b"original message";
        let sig = kp.sign(message);
        let sig_hex = hex::encode(sig.0);
        let pk_hex = hex::encode(kp.public_key().0);
        let result = Wallet::verify(b"tampered message", &sig_hex, &pk_hex).unwrap();
        assert!(!result);
    }

    #[test]
    fn wallet_verify_wrong_public_key() {
        let kp = Keypair::from_seed_hex(
            "0x0000000000000000000000000000000000000000000000000000000000000001",
        )
        .unwrap();
        let message = b"test";
        let sig = kp.sign(message);
        let sig_hex = hex::encode(sig.0);
        let wrong_pk = hex::encode([0u8; 32]);
        let result = Wallet::verify(message, &sig_hex, &wrong_pk).unwrap();
        assert!(!result);
    }

    #[test]
    fn wallet_verify_bad_hex_signature() {
        let result = Wallet::verify(b"msg", "zzzz", &"00".repeat(32));
        assert!(result.is_err());
    }

    #[test]
    fn wallet_verify_bad_hex_public_key() {
        let result = Wallet::verify(b"msg", &"00".repeat(64), "zzzz");
        assert!(result.is_err());
    }

    #[test]
    fn wallet_verify_short_signature() {
        let result = Wallet::verify(b"msg", "aabb", &"00".repeat(32));
        assert!(result.is_err());
    }

    #[test]
    fn wallet_verify_short_public_key() {
        let result = Wallet::verify(b"msg", &"00".repeat(64), "aabb");
        assert!(result.is_err());
    }
}
