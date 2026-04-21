//! Python bindings for bittensor-chain MEV Shield: MevShield class.
//!
//! Feature-gated: `#[cfg(feature = "mev-shield")]`

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

use bittensor_chain::mev_shield::{EncryptedPayload, MevShieldEncrypt, MevShieldSubmit};

/// MEV Shield — post-quantum encrypted extrinsic submission using ML-KEM-768.
///
/// Encrypts extrinsic payloads with an on-chain ML-KEM-768 public key
/// and formats them for submission via `submit_encrypted_extrinsic`.
///
/// Example:
///     shield = MevShield()
///     encrypted = shield.encrypt_extrinsic("0x...", "password")
///     result = await shield.submit_encrypted(encrypted)
#[cfg(feature = "mev-shield")]
#[pyclass]
pub struct MevShield;

#[cfg(feature = "mev-shield")]
#[pymethods]
impl MevShield {
    /// Create a new MEV Shield instance.
    #[new]
    #[pyo3(signature = ())]
    fn new() -> Self {
        Self
    }

    /// Encrypt an extrinsic payload using an ML-KEM-768 public key.
    ///
    /// Args:
    ///     extrinsic_hex: Hex-encoded extrinsic bytes (with or without 0x prefix).
    ///     password: Unused placeholder (reserved for future key derivation).
    ///
    /// Returns:
    ///     Dict with keys: kem_ciphertext (hex), encrypted_extrinsic (hex)
    fn encrypt_extrinsic(&self, extrinsic_hex: &str, _password: &str) -> PyResult<PyObject> {
        let extrinsic_bytes = hex_decode(extrinsic_hex)
            .map_err(|e| PyRuntimeError::new_err(format!("hex decode failed: {e}")))?;

        // For encryption we need the on-chain NextKey, which is fetched dynamically.
        // This method creates a standalone encrypted payload if a public key is available.
        // For now, return an error indicating the key must be fetched first.
        Err(PyRuntimeError::new_err(
            "encrypt_extrinsic requires an on-chain NextKey — use the chain client's submit_encrypted_extrinsic method instead",
        ))
    }

    /// Submit an already-encrypted payload to the chain.
    ///
    /// Args:
    ///     encrypted_hex: Hex-encoded encrypted payload.
    ///
    /// This is a placeholder — actual submission requires a chain client connection.
    fn submit_encrypted(&self, py: Python<'_>, _encrypted_hex: &str) -> PyResult<PyObject> {
        let coro = pyo3_async_runtimes::tokio::future_into_py(py, async move {
            Err::<(), _>(PyRuntimeError::new_err(
                "submit_encrypted requires a connected SubtensorClient — use SubtensorClient methods instead",
            ))
        })?;
        Ok(coro.into_any().unbind())
    }

    fn __repr__(&self) -> String {
        "MevShield()".to_string()
    }
}

/// Hex decode helper that strips optional 0x prefix.
fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    if s.len() % 2 != 0 {
        return Err(format!("odd length hex string: {s}"));
    }
    let mut buf = Vec::with_capacity(s.len() / 2);
    for i in (0..s.len()).step_by(2) {
        let byte = u8::from_str_radix(&s[i..i + 2], 16)
            .map_err(|e| format!("hex decode at offset {i}: {e}"))?;
        buf.push(byte);
    }
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_decode_with_prefix() {
        let result = hex_decode("0xdeadbeef").unwrap();
        assert_eq!(result, vec![0xde, 0xad, 0xbe, 0xef]);
    }

    #[test]
    fn test_hex_decode_without_prefix() {
        let result = hex_decode("deadbeef").unwrap();
        assert_eq!(result, vec![0xde, 0xad, 0xbe, 0xef]);
    }

    #[test]
    fn test_hex_decode_odd_length_fails() {
        assert!(hex_decode("abc").is_err());
    }

    #[test]
    fn test_hex_decode_invalid_chars_fails() {
        assert!(hex_decode("xyz0").is_err());
    }

    #[test]
    fn test_hex_decode_empty_string() {
        let result = hex_decode("").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_hex_decode_single_byte() {
        let result = hex_decode("ff").unwrap();
        assert_eq!(result, vec![0xff]);
    }

    #[test]
    fn test_hex_decode_with_prefix_empty() {
        let result = hex_decode("0x").unwrap();
        assert!(result.is_empty());
    }

    #[cfg(feature = "mev-shield")]
    #[test]
    fn test_mev_shield_repr() {
        let shield = MevShield::new();
        assert_eq!(shield.__repr__(), "MevShield()");
    }

    #[cfg(feature = "mev-shield")]
    #[test]
    fn test_mev_shield_new() {
        let _shield = MevShield::new();
    }

    #[cfg(feature = "mev-shield")]
    #[test]
    fn test_mev_shield_encrypt_returns_error() {
        let shield = MevShield::new();
        let result = shield.encrypt_extrinsic("0xdeadbeef", "password");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("NextKey"));
    }
}
