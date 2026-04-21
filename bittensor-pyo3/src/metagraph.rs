//! Python bindings for bittensor-metagraph: Metagraph class with sync, save, load, neuron access.

use std::path::PathBuf;

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;

use bittensor_metagraph::Metagraph as RustMetagraph;

/// Bittensor subnet metagraph — columnar neural graph state.
///
/// Provides chain sync, serialization, iteration, and index access.
///
/// Example:
///     mg = Metagraph("finney", 1)
///     await mg.sync()
///     print(f"Neurons: {len(mg)}")
///     for n in mg.neurons():
///         print(n)
#[pyclass(subclass)]
pub struct Metagraph {
    inner: Option<RustMetagraph>,
    network: String,
    netuid: u16,
}

#[pymethods]
impl Metagraph {
    /// Create a new Metagraph for the given network and subnet.
    ///
    /// Args:
    ///     network: Network name ("finney", "test", "local"). Defaults to "finney".
    ///     netuid: Subnet identifier.
    #[new]
    #[pyo3(signature = (network="finney", netuid=1))]
    fn new(network: &str, netuid: u16) -> Self {
        Self { inner: None, network: network.to_string(), netuid }
    }

    /// Sync the metagraph from the chain.
    ///
    /// This fetches all neuron data for the subnet and populates
    /// all columnar fields (stake, ranks, weights, etc.).
    fn sync(&self, py: Python<'_>) -> PyResult<PyObject> {
        let network_name = self.network.clone();
        let netuid = self.netuid;
        let coro = pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let config = resolve_network_config(&network_name)?;
            let client = bittensor_chain::client::SubtensorClient::from_config(config)
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("failed to connect: {e}")))?;
            let metagraph = bittensor_metagraph::sync(&client, netuid)
                .await
                .map_err(|e| PyRuntimeError::new_err(format!("sync failed: {e}")))?;
            Ok(Metagraph { inner: Some(metagraph), network: network_name, netuid })
        })?;
        Ok(coro.into_any().unbind())
    }

    /// Save the metagraph to a JSON file.
    ///
    /// Args:
    ///     path: File path for the JSON output.
    fn save(&self, path: &str) -> PyResult<()> {
        let mg = self.require_inner()?;
        let path = PathBuf::from(path);
        bittensor_metagraph::save(mg, &path)
            .map_err(|e| PyRuntimeError::new_err(format!("save failed: {e}")))?;
        Ok(())
    }

    /// Load a metagraph from a JSON file.
    ///
    /// Args:
    ///     path: File path to the JSON file.
    ///
    /// Returns:
    ///     Metagraph loaded from disk.
    #[staticmethod]
    fn load(path: &str) -> PyResult<Self> {
        let path = PathBuf::from(path);
        let mg = bittensor_metagraph::load(&path)
            .map_err(|e| PyRuntimeError::new_err(format!("load failed: {e}")))?;
        Ok(Self { inner: Some(mg), network: String::new(), netuid: 0 })
    }

    /// Return a list of neuron info dicts for all neurons in the subnet.
    ///
    /// Each dict contains: uid, netuid, active, hotkey, coldkey, stake, rank,
    /// trust, consensus, incentive, dividend, emission, validator_trust.
    fn neurons(&self, py: Python<'_>) -> PyResult<Vec<PyObject>> {
        let mg = self.require_inner()?;
        let mut result = Vec::with_capacity(mg.n);
        for neuron in mg.neurons() {
            let dict = PyDict::new(py);
            dict.set_item("uid", neuron.uid)?;
            dict.set_item("netuid", neuron.netuid)?;
            dict.set_item("active", neuron.active)?;
            dict.set_item("hotkey", neuron.hotkey.clone())?;
            dict.set_item("coldkey", neuron.coldkey.clone())?;
            dict.set_item("stake", neuron.stake.to_tao())?;
            dict.set_item("rank", neuron.rank)?;
            dict.set_item("trust", neuron.trust)?;
            dict.set_item("consensus", neuron.consensus)?;
            dict.set_item("incentive", neuron.incentive)?;
            dict.set_item("dividend", neuron.dividend)?;
            dict.set_item("emission", neuron.emission)?;
            dict.set_item("validator_trust", neuron.validator_trust)?;
            result.push(dict.into_any().unbind());
        }
        Ok(result)
    }

    /// Access a neuron by positional index (like metagraph[uid]).
    ///
    /// Returns a dict with neuron fields for the neuron at the given position.
    fn __getitem__(&self, py: Python<'_>, uid: usize) -> PyResult<PyObject> {
        let mg = self.require_inner()?;
        if uid >= mg.n {
            return Err(PyValueError::new_err(format!(
                "index {uid} out of range (metagraph has {} neurons)",
                mg.n
            )));
        }
        let neuron = mg.neuron_at(uid);
        let dict = PyDict::new(py);
        dict.set_item("uid", neuron.uid)?;
        dict.set_item("netuid", neuron.netuid)?;
        dict.set_item("active", neuron.active)?;
        dict.set_item("hotkey", neuron.hotkey.clone())?;
        dict.set_item("coldkey", neuron.coldkey.clone())?;
        dict.set_item("stake", neuron.stake.to_tao())?;
        dict.set_item("rank", neuron.rank)?;
        dict.set_item("trust", neuron.trust)?;
        dict.set_item("consensus", neuron.consensus)?;
        dict.set_item("incentive", neuron.incentive)?;
        dict.set_item("dividend", neuron.dividend)?;
        dict.set_item("emission", neuron.emission)?;
        dict.set_item("validator_trust", neuron.validator_trust)?;
        Ok(dict.into_any().unbind())
    }

    /// Number of neurons in the metagraph.
    fn __len__(&self) -> PyResult<usize> {
        let mg = self.require_inner()?;
        Ok(mg.n)
    }

    /// The subnet identifier.
    #[getter]
    fn netuid(&self) -> PyResult<u16> {
        let mg = self.require_inner()?;
        Ok(mg.netuid)
    }

    /// The block number at which this metagraph was synced.
    #[getter]
    fn block(&self) -> PyResult<u64> {
        let mg = self.require_inner()?;
        Ok(mg.block)
    }

    fn __repr__(&self) -> String {
        match &self.inner {
            Some(mg) => format!("Metagraph(netuid={}, n={}, block={})", mg.netuid, mg.n, mg.block),
            None => {
                format!("Metagraph(network='{}', netuid={}, not synced)", self.network, self.netuid)
            }
        }
    }
}

impl Metagraph {
    fn require_inner(&self) -> PyResult<&RustMetagraph> {
        self.inner
            .as_ref()
            .ok_or_else(|| PyRuntimeError::new_err("Metagraph not synced — call sync() first"))
    }
}

/// Resolve a network name to a NetworkConfig.
fn resolve_network_config(name: &str) -> PyResult<bittensor_core::config::NetworkConfig> {
    match name {
        "finney" | "mainnet" => Ok(bittensor_core::config::NetworkConfig::finney()),
        "test" | "testnet" => Ok(bittensor_core::config::NetworkConfig::test()),
        "local" => Ok(bittensor_core::config::NetworkConfig::local()),
        "archive" => Ok(bittensor_core::config::NetworkConfig::archive()),
        "latent-lite" => Ok(bittensor_core::config::NetworkConfig::latent_lite()),
        other => Err(PyValueError::new_err(format!(
            "unknown network '{other}'; choose finney, test, local, archive, or latent-lite"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_finney() {
        let config = resolve_network_config("finney").unwrap();
        assert_eq!(config.name, "finney");
    }

    #[test]
    fn test_resolve_mainnet() {
        let config = resolve_network_config("mainnet").unwrap();
        assert_eq!(config.name, "finney");
    }

    #[test]
    fn test_resolve_test() {
        let config = resolve_network_config("test").unwrap();
        assert_eq!(config.name, "test");
    }

    #[test]
    fn test_resolve_testnet() {
        let config = resolve_network_config("testnet").unwrap();
        assert_eq!(config.name, "test");
    }

    #[test]
    fn test_resolve_local() {
        let config = resolve_network_config("local").unwrap();
        assert_eq!(config.name, "local");
    }

    #[test]
    fn test_resolve_archive() {
        let config = resolve_network_config("archive").unwrap();
        assert_eq!(config.name, "archive");
    }

    #[test]
    fn test_resolve_latent_lite() {
        let config = resolve_network_config("latent-lite").unwrap();
        assert_eq!(config.name, "latent-lite");
    }

    #[test]
    fn test_resolve_unknown_fails() {
        assert!(resolve_network_config("invalid").is_err());
    }

    #[test]
    fn test_resolve_empty_fails() {
        assert!(resolve_network_config("").is_err());
    }

    #[test]
    fn test_metagraph_new() {
        let mg = Metagraph::new("finney", 1);
        assert!(mg.inner.is_none());
        assert_eq!(mg.network, "finney");
        assert_eq!(mg.netuid, 1);
    }

    #[test]
    fn test_metagraph_new_custom_netuid() {
        let mg = Metagraph::new("test", 42);
        assert!(mg.inner.is_none());
        assert_eq!(mg.network, "test");
        assert_eq!(mg.netuid, 42);
    }

    #[test]
    fn test_metagraph_repr_not_synced() {
        let mg = Metagraph::new("finney", 1);
        let repr = mg.__repr__();
        assert!(repr.contains("not synced"));
        assert!(repr.contains("finney"));
    }

    #[test]
    fn test_metagraph_repr_synced() {
        let inner = bittensor_metagraph::Metagraph::new(1);
        let mg = Metagraph { inner: Some(inner), network: "finney".to_string(), netuid: 1 };
        let repr = mg.__repr__();
        assert!(repr.contains("n=0"));
        assert!(repr.contains("netuid=1"));
    }

    #[test]
    fn test_metagraph_require_inner_fails_when_not_synced() {
        let mg = Metagraph::new("finney", 1);
        let result = mg.require_inner();
        assert!(result.is_err());
    }

    #[test]
    fn test_metagraph_len_fails_when_not_synced() {
        let mg = Metagraph::new("finney", 1);
        let result = mg.__len__();
        assert!(result.is_err());
    }

    #[test]
    fn test_metagraph_netuid_fails_when_not_synced() {
        let mg = Metagraph::new("finney", 1);
        let result = mg.netuid();
        assert!(result.is_err());
    }

    #[test]
    fn test_metagraph_block_fails_when_not_synced() {
        let mg = Metagraph::new("finney", 1);
        let result = mg.block();
        assert!(result.is_err());
    }

    #[test]
    fn test_metagraph_synced_len() {
        let inner = bittensor_metagraph::Metagraph::new(5);
        let mg = Metagraph { inner: Some(inner), network: "finney".to_string(), netuid: 5 };
        assert_eq!(mg.__len__().unwrap(), 0);
    }

    #[test]
    fn test_metagraph_synced_netuid() {
        let inner = bittensor_metagraph::Metagraph::new(9);
        let mg = Metagraph { inner: Some(inner), network: "test".to_string(), netuid: 9 };
        assert_eq!(mg.netuid().unwrap(), 9);
    }
}
