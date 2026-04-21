//! Python bindings for bittensor-core types: Balance, NetworkConfig, BittensorError, and chain data models.

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyType;

use bittensor_core::balance::Balance as RustBalance;
use bittensor_core::config::NetworkConfig as RustNetworkConfig;
use bittensor_core::types::{
    AxonInfo as RustAxonInfo, DelegateInfo as RustDelegateInfo, MetagraphInfo as RustMetagraphInfo,
    NeuronCertificate as RustNeuronCertificate, NeuronInfo as RustNeuronInfo,
    NeuronInfoLite as RustNeuronInfoLite, PrometheusInfo as RustPrometheusInfo,
    StakeInfo as RustStakeInfo, SubnetHyperparameters as RustSubnetHyperparameters,
    SubnetInfo as RustSubnetInfo,
};

// ---------------------------------------------------------------------------
// BittensorError → Python Exception (simple subclass of PyRuntimeError)
// ---------------------------------------------------------------------------

pyo3::create_exception!(bittensor_rs, BittensorError, PyRuntimeError);

// NOTE: We cannot impl From<RustBittensorError> for PyErr due to orphan rules.
// Use `BittensorError::new_err(e.to_string())` explicitly instead.

// ---------------------------------------------------------------------------
// Balance
// ---------------------------------------------------------------------------

/// Bittensor Balance with arithmetic operators and tao/rao conversions.
///
/// One TAO = 1,000,000,000 RAO.
#[pyclass]
#[derive(Clone)]
pub struct Balance {
    inner: RustBalance,
}

#[pymethods]
impl Balance {
    #[new]
    #[pyo3(signature = (rao=0))]
    fn new(rao: u64) -> Self {
        Self { inner: RustBalance::from_rao(rao) }
    }

    /// Create a Balance from tao (1 TAO = 10^9 RAO).
    #[classmethod]
    fn from_tao(_cls: &Bound<'_, PyType>, tao: f64) -> Self {
        Self { inner: RustBalance::from_tao(tao) }
    }

    /// Create a Balance from rao.
    #[classmethod]
    fn from_rao(_cls: &Bound<'_, PyType>, rao: u64) -> Self {
        Self { inner: RustBalance::from_rao(rao) }
    }

    /// Zero balance.
    #[classmethod]
    fn zero(_cls: &Bound<'_, PyType>) -> Self {
        Self { inner: RustBalance::ZERO }
    }

    /// One TAO balance.
    #[classmethod]
    fn one_tao(_cls: &Bound<'_, PyType>) -> Self {
        Self { inner: RustBalance::ONE_TAO }
    }

    /// Balance in rao (smallest unit).
    #[getter]
    fn rao(&self) -> u64 {
        self.inner.to_rao()
    }

    /// Balance in tao.
    #[getter]
    fn tao(&self) -> f64 {
        self.inner.to_tao()
    }

    fn __add__(&self, other: &Self) -> Self {
        Self { inner: self.inner + other.inner }
    }

    fn __sub__(&self, other: &Self) -> Self {
        Self { inner: self.inner - other.inner }
    }

    fn __mul__(&self, scalar: u64) -> Self {
        Self { inner: self.inner * scalar }
    }

    /// Divide Balance by another Balance → integer ratio (u64),
    /// or divide Balance by an integer scalar → Balance.
    fn __truediv__(&self, other: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        let py = other.py();
        if let Ok(other_bal) = other.extract::<Self>() {
            let ratio = self.inner / other_bal.inner;
            Ok(ratio.into_pyobject(py)?.into_any().unbind())
        } else if let Ok(scalar) = other.extract::<u64>() {
            if scalar == 0 {
                return Err(PyRuntimeError::new_err("division by zero"));
            }
            let result = RustBalance::from_rao(self.inner.to_rao() / scalar);
            Ok(Balance::from(result).into_pyobject(py)?.into_any().unbind())
        } else {
            Err(PyRuntimeError::new_err(
                "unsupported operand type(s) for /: 'Balance' and non-Balance/non-int",
            ))
        }
    }

    fn __str__(&self) -> String {
        format!("{}", self.inner)
    }

    fn __repr__(&self) -> String {
        format!("Balance(rao={})", self.inner.to_rao())
    }

    fn __richcmp__(&self, other: &Self, op: pyo3::basic::CompareOp) -> bool {
        match op {
            pyo3::basic::CompareOp::Eq => self.inner == other.inner,
            pyo3::basic::CompareOp::Ne => self.inner != other.inner,
            pyo3::basic::CompareOp::Lt => self.inner < other.inner,
            pyo3::basic::CompareOp::Le => self.inner <= other.inner,
            pyo3::basic::CompareOp::Gt => self.inner > other.inner,
            pyo3::basic::CompareOp::Ge => self.inner >= other.inner,
        }
    }

    fn __hash__(&self) -> u64 {
        self.inner.to_rao()
    }
}

// ---------------------------------------------------------------------------
// NetworkConfig
// ---------------------------------------------------------------------------

/// Network configuration for connecting to a Subtensor chain endpoint.
#[pyclass]
#[derive(Clone)]
pub struct NetworkConfig {
    pub(crate) inner: RustNetworkConfig,
}

#[pymethods]
impl NetworkConfig {
    #[new]
    #[pyo3(signature = (name, ws_endpoint, archive_endpoint=None, chain_id=42))]
    fn new(
        name: String,
        ws_endpoint: String,
        archive_endpoint: Option<String>,
        chain_id: u16,
    ) -> Self {
        Self { inner: RustNetworkConfig { name, ws_endpoint, archive_endpoint, chain_id } }
    }

    /// Finney mainnet configuration.
    #[classmethod]
    fn finney(_cls: &Bound<'_, PyType>) -> Self {
        Self { inner: RustNetworkConfig::finney() }
    }

    /// Testnet configuration.
    #[classmethod]
    fn test(_cls: &Bound<'_, PyType>) -> Self {
        Self { inner: RustNetworkConfig::test() }
    }

    /// Local development node configuration.
    #[classmethod]
    fn local(_cls: &Bound<'_, PyType>) -> Self {
        Self { inner: RustNetworkConfig::local() }
    }

    /// Archive node configuration.
    #[classmethod]
    fn archive(_cls: &Bound<'_, PyType>) -> Self {
        Self { inner: RustNetworkConfig::archive() }
    }

    /// Latent-lite endpoint configuration.
    #[classmethod]
    fn latent_lite(_cls: &Bound<'_, PyType>) -> Self {
        Self { inner: RustNetworkConfig::latent_lite() }
    }

    /// Human-readable network name.
    #[getter]
    fn name(&self) -> &str {
        &self.inner.name
    }

    /// WebSocket endpoint URL.
    #[getter]
    fn ws_endpoint(&self) -> &str {
        &self.inner.ws_endpoint
    }

    /// Optional archive node endpoint URL.
    #[getter]
    fn archive_endpoint(&self) -> Option<&str> {
        self.inner.archive_endpoint.as_deref()
    }

    /// Chain identifier (SS58 prefix).
    #[getter]
    fn chain_id(&self) -> u16 {
        self.inner.chain_id
    }

    fn __repr__(&self) -> String {
        format!(
            "NetworkConfig(name='{}', ws_endpoint='{}', chain_id={})",
            self.inner.name, self.inner.ws_endpoint, self.inner.chain_id
        )
    }
}

// ---------------------------------------------------------------------------
// AxonInfo
// ---------------------------------------------------------------------------

/// Information about a neuron's axon (serving endpoint).
#[pyclass]
#[derive(Clone)]
pub struct AxonInfo {
    pub(crate) inner: RustAxonInfo,
}

#[pymethods]
impl AxonInfo {
    #[new]
    #[pyo3(signature = (ip=0, port=8090, ip_type=4, protocol=0, version=0, hotkey="".to_string(), coldkey="".to_string()))]
    fn new(
        ip: u64,
        port: u16,
        ip_type: u8,
        protocol: u8,
        version: u32,
        hotkey: String,
        coldkey: String,
    ) -> Self {
        Self { inner: RustAxonInfo { ip, port, ip_type, protocol, version, hotkey, coldkey } }
    }

    #[getter]
    fn ip(&self) -> u64 {
        self.inner.ip
    }

    #[getter]
    fn port(&self) -> u16 {
        self.inner.port
    }

    #[getter]
    fn ip_type(&self) -> u8 {
        self.inner.ip_type
    }

    #[getter]
    fn protocol(&self) -> u8 {
        self.inner.protocol
    }

    #[getter]
    fn version(&self) -> u32 {
        self.inner.version
    }

    #[getter]
    fn hotkey(&self) -> &str {
        &self.inner.hotkey
    }

    #[getter]
    fn coldkey(&self) -> &str {
        &self.inner.coldkey
    }

    fn __repr__(&self) -> String {
        format!(
            "AxonInfo(ip={}, port={}, hotkey='{}')",
            self.inner.ip, self.inner.port, self.inner.hotkey
        )
    }
}

// ---------------------------------------------------------------------------
// PrometheusInfo
// ---------------------------------------------------------------------------

/// Prometheus monitoring information for a neuron.
#[pyclass]
#[derive(Clone)]
pub struct PrometheusInfo {
    inner: RustPrometheusInfo,
}

#[pymethods]
impl PrometheusInfo {
    #[getter]
    fn ip(&self) -> u64 {
        self.inner.ip
    }

    #[getter]
    fn port(&self) -> u16 {
        self.inner.port
    }

    #[getter]
    fn version(&self) -> u32 {
        self.inner.version
    }

    #[getter]
    fn block(&self) -> u64 {
        self.inner.block
    }

    fn __repr__(&self) -> String {
        format!(
            "PrometheusInfo(ip={}, port={}, block={})",
            self.inner.ip, self.inner.port, self.inner.block
        )
    }
}

// ---------------------------------------------------------------------------
// StakeInfo
// ---------------------------------------------------------------------------

/// Stake information for a hotkey/coldkey pair.
#[pyclass]
#[derive(Clone)]
pub struct StakeInfo {
    inner: RustStakeInfo,
}

#[pymethods]
impl StakeInfo {
    #[getter]
    fn hotkey(&self) -> &str {
        &self.inner.hotkey
    }

    #[getter]
    fn coldkey(&self) -> &str {
        &self.inner.coldkey
    }

    #[getter]
    fn stake(&self) -> Balance {
        Balance { inner: self.inner.stake }
    }

    fn __repr__(&self) -> String {
        format!(
            "StakeInfo(hotkey='{}', coldkey='{}', stake={})",
            self.inner.hotkey, self.inner.coldkey, self.inner.stake
        )
    }
}

// ---------------------------------------------------------------------------
// DelegateInfo
// ---------------------------------------------------------------------------

/// Delegate information including take, nominators, and registrations.
#[pyclass]
#[derive(Clone)]
pub struct DelegateInfo {
    inner: RustDelegateInfo,
}

#[pymethods]
impl DelegateInfo {
    #[getter]
    fn delegate_ss58(&self) -> &str {
        &self.inner.delegate_ss58
    }

    #[getter]
    fn delegate_hotkey(&self) -> &str {
        &self.inner.delegate_hotkey
    }

    #[getter]
    fn total_stake(&self) -> Balance {
        Balance { inner: self.inner.total_stake }
    }

    #[getter]
    fn owner_hotkey(&self) -> &str {
        &self.inner.owner_hotkey
    }

    #[getter]
    fn take(&self) -> u16 {
        self.inner.take
    }

    #[getter]
    fn owner_ss58(&self) -> &str {
        &self.inner.owner_ss58
    }

    #[getter]
    fn registrations(&self) -> Vec<u16> {
        self.inner.registrations.clone()
    }

    #[getter]
    fn validator_permits(&self) -> Vec<u16> {
        self.inner.validator_permits.clone()
    }

    /// Nominator list as list of (ss58_address, stake) tuples.
    #[getter]
    fn nominators(&self) -> Vec<(String, Balance)> {
        self.inner
            .nominators
            .iter()
            .map(|(addr, bal)| (addr.clone(), Balance { inner: *bal }))
            .collect()
    }

    fn __repr__(&self) -> String {
        format!(
            "DelegateInfo(hotkey='{}', take={}, total_stake={})",
            self.inner.delegate_hotkey, self.inner.take, self.inner.total_stake
        )
    }
}

// ---------------------------------------------------------------------------
// NeuronInfo / NeuronInfoLite
// ---------------------------------------------------------------------------

/// Full neuron information including weights and bonds.
#[pyclass]
#[derive(Clone)]
pub struct NeuronInfo {
    inner: RustNeuronInfo,
}

#[pymethods]
impl NeuronInfo {
    #[getter]
    fn uid(&self) -> u16 {
        self.inner.uid
    }

    #[getter]
    fn netuid(&self) -> u16 {
        self.inner.netuid
    }

    #[getter]
    fn active(&self) -> bool {
        self.inner.active
    }

    #[getter]
    fn stake(&self) -> Balance {
        Balance { inner: self.inner.stake }
    }

    #[getter]
    fn rank(&self) -> u16 {
        self.inner.rank
    }

    #[getter]
    fn trust(&self) -> u16 {
        self.inner.trust
    }

    #[getter]
    fn consensus(&self) -> u16 {
        self.inner.consensus
    }

    #[getter]
    fn incentive(&self) -> u16 {
        self.inner.incentive
    }

    #[getter]
    fn dividend(&self) -> u16 {
        self.inner.dividend
    }

    #[getter]
    fn emission(&self) -> u64 {
        self.inner.emission
    }

    #[getter]
    fn hotkey(&self) -> &str {
        &self.inner.hotkey
    }

    #[getter]
    fn coldkey(&self) -> &str {
        &self.inner.coldkey
    }

    #[getter]
    fn last_update(&self) -> u64 {
        self.inner.last_update
    }

    #[getter]
    fn validator_trust(&self) -> u16 {
        self.inner.validator_trust
    }

    fn __repr__(&self) -> String {
        format!(
            "NeuronInfo(uid={}, netuid={}, active={}, hotkey='{}')",
            self.inner.uid, self.inner.netuid, self.inner.active, self.inner.hotkey
        )
    }
}

/// Lightweight neuron information (no weights/bonds).
#[pyclass]
#[derive(Clone)]
pub struct NeuronInfoLite {
    inner: RustNeuronInfoLite,
}

#[pymethods]
impl NeuronInfoLite {
    #[getter]
    fn uid(&self) -> u16 {
        self.inner.uid
    }

    #[getter]
    fn hotkey(&self) -> &str {
        &self.inner.hotkey
    }

    #[getter]
    fn coldkey(&self) -> &str {
        &self.inner.coldkey
    }

    #[getter]
    fn active(&self) -> bool {
        self.inner.active
    }

    #[getter]
    fn stake(&self) -> Balance {
        Balance { inner: self.inner.stake }
    }

    #[getter]
    fn rank(&self) -> u16 {
        self.inner.rank
    }

    #[getter]
    fn trust(&self) -> u16 {
        self.inner.trust
    }

    #[getter]
    fn consensus(&self) -> u16 {
        self.inner.consensus
    }

    #[getter]
    fn incentive(&self) -> u16 {
        self.inner.incentive
    }

    fn __repr__(&self) -> String {
        format!(
            "NeuronInfoLite(uid={}, active={}, hotkey='{}')",
            self.inner.uid, self.inner.active, self.inner.hotkey
        )
    }
}

// ---------------------------------------------------------------------------
// SubnetInfo / SubnetHyperparameters
// ---------------------------------------------------------------------------

/// Subnet metadata.
#[pyclass]
#[derive(Clone)]
pub struct SubnetInfo {
    inner: RustSubnetInfo,
}

#[pymethods]
impl SubnetInfo {
    #[getter]
    fn netuid(&self) -> u16 {
        self.inner.netuid
    }

    #[getter]
    fn name(&self) -> &str {
        &self.inner.name
    }

    #[getter]
    fn owner_hotkey(&self) -> &str {
        &self.inner.owner_hotkey
    }

    #[getter]
    fn tempo(&self) -> u16 {
        self.inner.tempo
    }

    #[getter]
    fn maximum_uid(&self) -> u16 {
        self.inner.maximum_uid
    }

    #[getter]
    fn modality(&self) -> u8 {
        self.inner.modality
    }

    #[getter]
    fn network_uid(&self) -> u16 {
        self.inner.network_uid
    }

    fn __repr__(&self) -> String {
        format!(
            "SubnetInfo(netuid={}, name='{}', owner='{}')",
            self.inner.netuid, self.inner.name, self.inner.owner_hotkey
        )
    }
}

/// Subnet hyperparameters controlling incentive distribution.
#[pyclass]
#[derive(Clone)]
pub struct SubnetHyperparameters {
    inner: RustSubnetHyperparameters,
}

#[pymethods]
impl SubnetHyperparameters {
    #[getter]
    fn rho(&self) -> u16 {
        self.inner.rho
    }

    #[getter]
    fn kappa(&self) -> u16 {
        self.inner.kappa
    }

    #[getter]
    fn difficulty(&self) -> u32 {
        self.inner.difficulty
    }

    #[getter]
    fn burn(&self) -> u64 {
        self.inner.burn
    }

    #[getter]
    fn immunity_ratio(&self) -> u16 {
        self.inner.immunity_ratio
    }

    #[getter]
    fn min_burn(&self) -> u64 {
        self.inner.min_burn
    }

    #[getter]
    fn max_burn(&self) -> u64 {
        self.inner.max_burn
    }

    #[getter]
    fn weights_rate_limit(&self) -> u64 {
        self.inner.weights_rate_limit
    }

    #[getter]
    fn weights_version(&self) -> u16 {
        self.inner.weights_version
    }

    #[getter]
    fn max_weight_limit(&self) -> u16 {
        self.inner.max_weight_limit
    }

    #[getter]
    fn scaling_law_power(&self) -> u16 {
        self.inner.scaling_law_power
    }

    #[getter]
    fn subnetwork_n(&self) -> u16 {
        self.inner.subnetwork_n
    }

    #[getter]
    fn max_n(&self) -> u16 {
        self.inner.max_n
    }

    #[getter]
    fn tempo(&self) -> u16 {
        self.inner.tempo
    }

    #[getter]
    fn liquid_alpha_enabled(&self) -> bool {
        self.inner.liquid_alpha_enabled
    }

    fn __repr__(&self) -> String {
        format!(
            "SubnetHyperparameters(rho={}, kappa={}, difficulty={}, tempo={})",
            self.inner.rho, self.inner.kappa, self.inner.difficulty, self.inner.tempo
        )
    }
}

// ---------------------------------------------------------------------------
// MetagraphInfo
// ---------------------------------------------------------------------------

/// Metagraph summary for a subnet.
#[pyclass]
#[derive(Clone)]
pub struct MetagraphInfo {
    pub(crate) inner: RustMetagraphInfo,
}

#[pymethods]
impl MetagraphInfo {
    #[getter]
    fn netuid(&self) -> u16 {
        self.inner.netuid
    }

    #[getter]
    fn block(&self) -> u64 {
        self.inner.block
    }

    #[getter]
    fn n(&self) -> u16 {
        self.inner.n
    }

    #[getter]
    fn stake(&self) -> Balance {
        Balance { inner: self.inner.stake }
    }

    #[getter]
    fn total_issuance(&self) -> Balance {
        Balance { inner: self.inner.total_issuance }
    }

    fn __repr__(&self) -> String {
        format!(
            "MetagraphInfo(netuid={}, block={}, n={}, stake={})",
            self.inner.netuid, self.inner.block, self.inner.n, self.inner.stake
        )
    }
}

// ---------------------------------------------------------------------------
// NeuronCertificate
// ---------------------------------------------------------------------------

/// Neuron certificate information.
#[pyclass]
#[derive(Clone)]
pub struct NeuronCertificate {
    inner: RustNeuronCertificate,
}

#[pymethods]
impl NeuronCertificate {
    #[getter]
    fn hotkey(&self) -> &str {
        &self.inner.hotkey
    }

    #[getter]
    fn certificate(&self) -> Vec<u8> {
        self.inner.certificate.clone()
    }

    #[getter]
    fn block(&self) -> u64 {
        self.inner.block
    }

    fn __repr__(&self) -> String {
        format!("NeuronCertificate(hotkey='{}', block={})", self.inner.hotkey, self.inner.block)
    }
}

// ---------------------------------------------------------------------------
// Conversion helpers (pub(crate))
// ---------------------------------------------------------------------------

// NOTE: balance_to_py helper removed — use `Balance::from(rust_balance)` instead.

// ---------------------------------------------------------------------------
// From impls for converting Rust types → Python wrapper types
// ---------------------------------------------------------------------------

impl From<RustBalance> for Balance {
    fn from(b: RustBalance) -> Self {
        Balance { inner: b }
    }
}

impl From<RustStakeInfo> for StakeInfo {
    fn from(s: RustStakeInfo) -> Self {
        StakeInfo { inner: s }
    }
}

impl From<RustMetagraphInfo> for MetagraphInfo {
    fn from(m: RustMetagraphInfo) -> Self {
        MetagraphInfo { inner: m }
    }
}
