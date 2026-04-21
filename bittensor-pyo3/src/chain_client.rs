//! Python bindings for bittensor-chain: SubtensorClient async methods.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyType;

use bittensor_chain::client::SubtensorClient as RustSubtensorClient;
use bittensor_chain::extrinsics::{TxSuccess, add_stake, burned_register, remove_stake, transfer};
use bittensor_chain::queries::{
    get_balance, get_metagraph, get_stake_info_for_coldkey, get_total_balance,
    get_total_network_stake,
};
use bittensor_wallet::keypair::Keypair as RustKeypair;
use bittensor_wallet::ss58;

use crate::core_types::{Balance, BittensorError, MetagraphInfo, NetworkConfig, StakeInfo};

// ---------------------------------------------------------------------------
// TxSuccessPy
// ---------------------------------------------------------------------------

/// Result of a successful on-chain transaction.
#[pyclass(frozen, name = "TxSuccess")]
#[derive(Clone)]
pub struct TxSuccessPy {
    inner: TxSuccess,
}

#[pymethods]
impl TxSuccessPy {
    #[getter]
    fn block_hash(&self) -> String {
        format!("0x{}", hex::encode(self.inner.block_hash.as_bytes()))
    }

    #[getter]
    fn extrinsic_hash(&self) -> String {
        format!("0x{}", hex::encode(self.inner.extrinsic_hash.as_bytes()))
    }

    fn __repr__(&self) -> String {
        format!(
            "TxSuccess(block_hash='{}', extrinsic_hash='{}')",
            self.block_hash(),
            self.extrinsic_hash()
        )
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_account_id(address: &str) -> PyResult<subxt::utils::AccountId32> {
    let (_format, pubkey) = ss58::decode_ss58(address)
        .map_err(|e| PyValueError::new_err(format!("Invalid SS58 address '{}': {}", address, e)))?;
    Ok(subxt::utils::AccountId32::from(pubkey))
}

fn parse_signer(signer_input: &str, password: Option<&str>) -> PyResult<RustKeypair> {
    let stripped = signer_input.strip_prefix("0x").unwrap_or(signer_input);
    if stripped.len() == 64 && stripped.chars().all(|c| c.is_ascii_hexdigit()) {
        RustKeypair::from_seed_hex(stripped)
            .map_err(|e| PyValueError::new_err(format!("Invalid signer seed hex: {}", e)))
    } else {
        let words: Vec<&str> = signer_input.split_whitespace().collect();
        if words.len() >= 12 {
            let mnemonic = subxt_signer::bip39::Mnemonic::parse(signer_input)
                .map_err(|e| PyValueError::new_err(format!("Invalid mnemonic: {}", e)))?;
            RustKeypair::from_phrase(&mnemonic, password).map_err(|e| {
                PyValueError::new_err(format!("Failed to create keypair from mnemonic: {}", e))
            })
        } else {
            Err(PyValueError::new_err(
                "Signer must be a 64-char hex seed or a valid BIP-39 mnemonic",
            ))
        }
    }
}

// ---------------------------------------------------------------------------
// SubtensorClient
// ---------------------------------------------------------------------------

/// Client for interacting with the Bittensor Subtensor blockchain.
#[pyclass]
pub struct SubtensorClient {
    inner: Option<RustSubtensorClient>,
}

impl SubtensorClient {
    fn require_client(&self) -> PyResult<RustSubtensorClient> {
        self.inner.clone().ok_or_else(|| PyValueError::new_err("SubtensorClient is not connected"))
    }
}

#[pymethods]
impl SubtensorClient {
    /// Create a disconnected client. Call connect() or from_url() to connect.
    #[new]
    #[pyo3(signature = ())]
    fn new() -> Self {
        Self { inner: None }
    }

    /// Connect to a Subtensor node using a NetworkConfig.
    #[classmethod]
    fn connect(cls: &Bound<'_, PyType>, network_config: &NetworkConfig) -> PyResult<PyObject> {
        let config = network_config.inner.clone();
        let py = cls.py();
        let coro = pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let client = RustSubtensorClient::from_config(config)
                .await
                .map_err(|e| BittensorError::new_err(e.to_string()))?;
            Ok(SubtensorClient { inner: Some(client) })
        })?;
        Ok(coro.into_any().unbind())
    }

    /// Connect to a Subtensor node using a WebSocket URL string.
    #[staticmethod]
    fn from_url(py: Python<'_>, url: String) -> PyResult<PyObject> {
        let coro = pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let client = RustSubtensorClient::from_url(&url)
                .await
                .map_err(|e| BittensorError::new_err(e.to_string()))?;
            Ok(SubtensorClient { inner: Some(client) })
        })?;
        Ok(coro.into_any().unbind())
    }

    /// Get the free balance of an account (SS58 address).
    #[pyo3(signature = (address))]
    fn get_balance(&self, py: Python<'_>, address: String) -> PyResult<PyObject> {
        let client = self.require_client()?;
        let account_id = parse_account_id(&address)?;
        let coro = pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let balance = get_balance(client.rpc(), &account_id)
                .await
                .map_err(|e| BittensorError::new_err(e.to_string()))?;
            Ok(Balance::from(balance))
        })?;
        Ok(coro.into_any().unbind())
    }

    /// Get the total balance (free + reserved) of an account.
    #[pyo3(signature = (address))]
    fn get_total_balance(&self, py: Python<'_>, address: String) -> PyResult<PyObject> {
        let client = self.require_client()?;
        let account_id = parse_account_id(&address)?;
        let coro = pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let balance = get_total_balance(client.rpc(), &account_id)
                .await
                .map_err(|e| BittensorError::new_err(e.to_string()))?;
            Ok(Balance::from(balance))
        })?;
        Ok(coro.into_any().unbind())
    }

    /// Get the total network stake across all subnets.
    fn get_total_stake(&self, py: Python<'_>) -> PyResult<PyObject> {
        let client = self.require_client()?;
        let coro = pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let balance = get_total_network_stake(client.rpc())
                .await
                .map_err(|e| BittensorError::new_err(e.to_string()))?;
            Ok(Balance::from(balance))
        })?;
        Ok(coro.into_any().unbind())
    }

    /// Get stake info for a coldkey (SS58 address).
    #[pyo3(signature = (coldkey_address))]
    fn get_stake_info(&self, py: Python<'_>, coldkey_address: String) -> PyResult<PyObject> {
        let client = self.require_client()?;
        let coldkey = parse_account_id(&coldkey_address)?;
        let coro = pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let stakes = get_stake_info_for_coldkey(client.rpc(), &coldkey)
                .await
                .map_err(|e| BittensorError::new_err(e.to_string()))?;
            let py_stakes: Vec<StakeInfo> = stakes.into_iter().map(StakeInfo::from).collect();
            Ok(py_stakes)
        })?;
        Ok(coro.into_any().unbind())
    }

    /// Get metagraph information for a subnet.
    #[pyo3(signature = (netuid))]
    fn get_metagraph(&self, py: Python<'_>, netuid: u16) -> PyResult<PyObject> {
        let client = self.require_client()?;
        let coro = pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let meta = get_metagraph(client.rpc(), netuid)
                .await
                .map_err(|e| BittensorError::new_err(e.to_string()))?;
            Ok(MetagraphInfo::from(meta))
        })?;
        Ok(coro.into_any().unbind())
    }

    /// Add stake to a hotkey on a subnet.
    #[pyo3(signature = (hotkey, netuid, amount, signer, password=None))]
    fn add_stake(
        &self,
        py: Python<'_>,
        hotkey: String,
        netuid: u16,
        amount: u64,
        signer: String,
        password: Option<String>,
    ) -> PyResult<PyObject> {
        let client = self.require_client()?;
        let hotkey_id = parse_account_id(&hotkey)?;
        let kp = parse_signer(&signer, password.as_deref())?;
        let inner_kp = kp.into_signer();
        let coro = pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let result = add_stake(client.rpc(), &inner_kp, hotkey_id, netuid, amount)
                .await
                .map_err(|e| BittensorError::new_err(e.to_string()))?;
            Ok(TxSuccessPy { inner: result })
        })?;
        Ok(coro.into_any().unbind())
    }

    /// Remove stake from a hotkey on a subnet.
    #[pyo3(signature = (hotkey, netuid, amount, signer, password=None))]
    fn remove_stake(
        &self,
        py: Python<'_>,
        hotkey: String,
        netuid: u16,
        amount: u64,
        signer: String,
        password: Option<String>,
    ) -> PyResult<PyObject> {
        let client = self.require_client()?;
        let hotkey_id = parse_account_id(&hotkey)?;
        let kp = parse_signer(&signer, password.as_deref())?;
        let inner_kp = kp.into_signer();
        let coro = pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let result = remove_stake(client.rpc(), &inner_kp, hotkey_id, netuid, amount)
                .await
                .map_err(|e| BittensorError::new_err(e.to_string()))?;
            Ok(TxSuccessPy { inner: result })
        })?;
        Ok(coro.into_any().unbind())
    }

    /// Transfer TAO to a destination address.
    #[pyo3(signature = (dest, amount, signer, password=None))]
    fn transfer(
        &self,
        py: Python<'_>,
        dest: String,
        amount: u64,
        signer: String,
        password: Option<String>,
    ) -> PyResult<PyObject> {
        let client = self.require_client()?;
        let dest_id = parse_account_id(&dest)?;
        let kp = parse_signer(&signer, password.as_deref())?;
        let inner_kp = kp.into_signer();
        let coro = pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let result = transfer(client.rpc(), &inner_kp, dest_id, amount)
                .await
                .map_err(|e| BittensorError::new_err(e.to_string()))?;
            Ok(TxSuccessPy { inner: result })
        })?;
        Ok(coro.into_any().unbind())
    }

    /// Register a neuron on a subnet by burning TAO.
    #[pyo3(signature = (netuid, hotkey, signer, password=None))]
    fn register(
        &self,
        py: Python<'_>,
        netuid: u16,
        hotkey: String,
        signer: String,
        password: Option<String>,
    ) -> PyResult<PyObject> {
        let client = self.require_client()?;
        let hotkey_id = parse_account_id(&hotkey)?;
        let kp = parse_signer(&signer, password.as_deref())?;
        let inner_kp = kp.into_signer();
        let coro = pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let result = burned_register(client.rpc(), &inner_kp, netuid, hotkey_id)
                .await
                .map_err(|e| BittensorError::new_err(e.to_string()))?;
            Ok(TxSuccessPy { inner: result })
        })?;
        Ok(coro.into_any().unbind())
    }

    fn __repr__(&self) -> String {
        match &self.inner {
            Some(_) => "SubtensorClient(connected)".into(),
            None => "SubtensorClient(disconnected)".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bittensor_wallet::prelude::Keypair;

    #[test]
    fn client_new_is_disconnected() {
        let client = SubtensorClient::new();
        assert!(client.inner.is_none());
    }

    #[test]
    fn client_repr_disconnected() {
        let client = SubtensorClient::new();
        assert_eq!(client.__repr__(), "SubtensorClient(disconnected)");
    }

    #[test]
    fn client_require_client_fails_when_disconnected() {
        let client = SubtensorClient::new();
        let result = client.require_client();
        assert!(result.is_err());
    }

    #[test]
    fn tx_success_py_block_hash_format() {
        let inner = bittensor_chain::extrinsics::TxSuccess {
            block_hash: Default::default(),
            extrinsic_hash: Default::default(),
        };
        let tx = TxSuccessPy { inner };
        let bh = tx.block_hash();
        assert!(bh.starts_with("0x"));
        assert_eq!(bh.len(), 66);
    }

    #[test]
    fn tx_success_py_extrinsic_hash_format() {
        let inner = bittensor_chain::extrinsics::TxSuccess {
            block_hash: Default::default(),
            extrinsic_hash: Default::default(),
        };
        let tx = TxSuccessPy { inner };
        let eh = tx.extrinsic_hash();
        assert!(eh.starts_with("0x"));
        assert_eq!(eh.len(), 66);
    }

    #[test]
    fn tx_success_py_repr() {
        let inner = bittensor_chain::extrinsics::TxSuccess {
            block_hash: Default::default(),
            extrinsic_hash: Default::default(),
        };
        let tx = TxSuccessPy { inner };
        let repr = tx.__repr__();
        assert!(repr.contains("TxSuccess"));
        assert!(repr.contains("block_hash="));
        assert!(repr.contains("extrinsic_hash="));
    }

    #[test]
    fn tx_success_py_clone() {
        let inner = bittensor_chain::extrinsics::TxSuccess {
            block_hash: Default::default(),
            extrinsic_hash: Default::default(),
        };
        let tx = TxSuccessPy { inner };
        let tx2 = tx.clone();
        assert_eq!(tx.block_hash(), tx2.block_hash());
        assert_eq!(tx.extrinsic_hash(), tx2.extrinsic_hash());
    }

    #[test]
    fn parse_account_id_valid_ss58() {
        let kp = Keypair::from_seed_hex(
            "0x0000000000000000000000000000000000000000000000000000000000000001",
        )
        .unwrap();
        let ss58 = kp.ss58_address();
        let result = parse_account_id(&ss58);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_account_id_invalid_string() {
        let result = parse_account_id("not_a_valid_ss58_address");
        assert!(result.is_err());
    }

    #[test]
    fn parse_account_id_empty_string() {
        let result = parse_account_id("");
        assert!(result.is_err());
    }

    #[test]
    fn parse_signer_hex_seed_without_prefix() {
        let seed = "0000000000000000000000000000000000000000000000000000000000000001";
        let result = parse_signer(seed, None);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_signer_hex_seed_with_prefix() {
        let seed = "0x0000000000000000000000000000000000000000000000000000000000000001";
        let result = parse_signer(seed, None);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_signer_invalid_hex() {
        let result = parse_signer("zzzzzzzz", None);
        assert!(result.is_err());
    }

    #[test]
    fn parse_signer_short_string_not_mnemonic() {
        let result = parse_signer("short", None);
        assert!(result.is_err());
    }

    #[test]
    fn parse_signer_valid_mnemonic() {
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let result = parse_signer(mnemonic, None);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_signer_invalid_mnemonic_words() {
        let result =
            parse_signer("not valid words but enough of them to pass length check maybe", None);
        assert!(result.is_err());
    }

    #[test]
    fn parse_signer_mnemonic_with_password() {
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let result = parse_signer(mnemonic, Some("mypassword"));
        assert!(result.is_ok());
    }

    #[test]
    fn parse_signer_24_word_mnemonic() {
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
        let result = parse_signer(mnemonic, None);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_signer_very_short_input() {
        let result = parse_signer("a", None);
        assert!(result.is_err());
    }

    #[test]
    fn parse_account_id_polkadot_format_0() {
        let address = "15oF4uVJwmo4TdGW7VfQxNLavjCXviqxT9S1MgbjMNHr6Sp5";
        let result = parse_account_id(address);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_account_id_truncated_ss58() {
        let result = parse_account_id("5GrwvaE");
        assert!(result.is_err());
    }

    #[test]
    fn get_balance_disconnected() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let client = SubtensorClient { inner: None };
            let result = client
                .get_balance(py, "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".to_string());
            assert!(result.is_err());
        });
    }

    #[test]
    fn get_total_balance_disconnected() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let client = SubtensorClient { inner: None };
            let result = client.get_total_balance(
                py,
                "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".to_string(),
            );
            assert!(result.is_err());
        });
    }

    #[test]
    fn get_total_stake_disconnected() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let client = SubtensorClient { inner: None };
            let result = client.get_total_stake(py);
            assert!(result.is_err());
        });
    }

    #[test]
    fn get_stake_info_disconnected() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let client = SubtensorClient { inner: None };
            let result = client
                .get_stake_info(py, "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".to_string());
            assert!(result.is_err());
        });
    }

    #[test]
    fn get_metagraph_disconnected() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let client = SubtensorClient { inner: None };
            let result = client.get_metagraph(py, 1);
            assert!(result.is_err());
        });
    }

    #[test]
    fn add_stake_disconnected() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let client = SubtensorClient { inner: None };
            let result = client.add_stake(
                py,
                "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".to_string(),
                1,
                1000,
                "0x0000000000000000000000000000000000000000000000000000000000000001".to_string(),
                None,
            );
            assert!(result.is_err());
        });
    }

    #[test]
    fn remove_stake_disconnected() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let client = SubtensorClient { inner: None };
            let result = client.remove_stake(
                py,
                "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".to_string(),
                1,
                1000,
                "0x0000000000000000000000000000000000000000000000000000000000000001".to_string(),
                None,
            );
            assert!(result.is_err());
        });
    }

    #[test]
    fn transfer_disconnected() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let client = SubtensorClient { inner: None };
            let result = client.transfer(
                py,
                "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".to_string(),
                1000,
                "0x0000000000000000000000000000000000000000000000000000000000000001".to_string(),
                None,
            );
            assert!(result.is_err());
        });
    }

    #[test]
    fn register_disconnected() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let client = SubtensorClient { inner: None };
            let result = client.register(
                py,
                1,
                "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".to_string(),
                "0x0000000000000000000000000000000000000000000000000000000000000001".to_string(),
                None,
            );
            assert!(result.is_err());
        });
    }

    // --- TxSuccessPy non-zero hash ---
    #[test]
    fn tx_success_py_nonzero_hash_encoding() {
        let block_bytes: [u8; 32] = [
            0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        let ext_bytes: [u8; 32] = [
            0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa, 0x99, 0x88, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        let inner = bittensor_chain::extrinsics::TxSuccess {
            block_hash: subxt::utils::H256::from(block_bytes),
            extrinsic_hash: subxt::utils::H256::from(ext_bytes),
        };
        let tx = TxSuccessPy { inner };
        let bh = tx.block_hash();
        assert!(bh.starts_with("0x"));
        assert_eq!(bh.len(), 66);
        assert!(bh.contains("abcdef01234567"));

        let eh = tx.extrinsic_hash();
        assert!(eh.starts_with("0x"));
        assert_eq!(eh.len(), 66);
        assert!(eh.contains("ffeeddccbbaa9988"));
    }
}
