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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn balance_new_zero() {
        let b = Balance::new(0);
        assert_eq!(b.rao(), 0);
        assert_eq!(b.tao(), 0.0);
    }

    #[test]
    fn balance_new_one_tao() {
        let b = Balance::new(1_000_000_000);
        assert_eq!(b.rao(), 1_000_000_000);
        assert!((b.tao() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn balance_new_partial_tao() {
        let b = Balance::new(500_000_000);
        assert_eq!(b.rao(), 500_000_000);
        assert!((b.tao() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn balance_add() {
        let a = Balance::new(500);
        let b = Balance::new(300);
        let c = a.__add__(&b);
        assert_eq!(c.rao(), 800);
    }

    #[test]
    fn balance_sub() {
        let a = Balance::new(500);
        let b = Balance::new(300);
        let c = a.__sub__(&b);
        assert_eq!(c.rao(), 200);
    }

    #[test]
    fn balance_mul_scalar() {
        let a = Balance::new(100);
        let c = a.__mul__(3);
        assert_eq!(c.rao(), 300);
    }

    #[test]
    fn balance_richcmp_eq() {
        let a = Balance::new(100);
        let b = Balance::new(100);
        assert!(a.__richcmp__(&b, pyo3::basic::CompareOp::Eq));
    }

    #[test]
    fn balance_richcmp_ne() {
        let a = Balance::new(100);
        let b = Balance::new(200);
        assert!(a.__richcmp__(&b, pyo3::basic::CompareOp::Ne));
    }

    #[test]
    fn balance_richcmp_lt() {
        let a = Balance::new(100);
        let b = Balance::new(200);
        assert!(a.__richcmp__(&b, pyo3::basic::CompareOp::Lt));
        assert!(!b.__richcmp__(&a, pyo3::basic::CompareOp::Lt));
    }

    #[test]
    fn balance_richcmp_le() {
        let a = Balance::new(100);
        let b = Balance::new(100);
        assert!(a.__richcmp__(&b, pyo3::basic::CompareOp::Le));
    }

    #[test]
    fn balance_richcmp_gt() {
        let a = Balance::new(200);
        let b = Balance::new(100);
        assert!(a.__richcmp__(&b, pyo3::basic::CompareOp::Gt));
    }

    #[test]
    fn balance_richcmp_ge() {
        let a = Balance::new(200);
        let b = Balance::new(100);
        assert!(a.__richcmp__(&b, pyo3::basic::CompareOp::Ge));
    }

    #[test]
    fn balance_str_format() {
        let b = Balance::new(1_500_000_000);
        let s = b.__str__();
        assert!(!s.is_empty());
    }

    #[test]
    fn balance_repr_format() {
        let b = Balance::new(42);
        assert_eq!(b.__repr__(), "Balance(rao=42)");
    }

    #[test]
    fn balance_hash() {
        let b = Balance::new(42);
        assert_eq!(b.__hash__(), 42);
    }

    #[test]
    fn balance_clone() {
        let a = Balance::new(100);
        let b = a.clone();
        assert_eq!(a.rao(), b.rao());
    }

    #[test]
    fn balance_from_rust_balance() {
        let rb = RustBalance::from_rao(999);
        let pb: Balance = rb.into();
        assert_eq!(pb.rao(), 999);
    }

    #[test]
    fn network_config_new_custom() {
        let nc =
            NetworkConfig::new("custom".to_string(), "ws://localhost:9944".to_string(), None, 42);
        assert_eq!(nc.name(), "custom");
        assert_eq!(nc.ws_endpoint(), "ws://localhost:9944");
        assert_eq!(nc.archive_endpoint(), None);
        assert_eq!(nc.chain_id(), 42);
    }

    #[test]
    fn network_config_with_archive() {
        let nc = NetworkConfig::new(
            "arch".to_string(),
            "ws://x".to_string(),
            Some("ws://archive".to_string()),
            99,
        );
        assert_eq!(nc.archive_endpoint(), Some("ws://archive"));
    }

    #[test]
    fn network_config_repr() {
        let nc = NetworkConfig::new("test".to_string(), "ws://x".to_string(), None, 42);
        let repr = nc.__repr__();
        assert!(repr.contains("NetworkConfig"));
        assert!(repr.contains("test"));
    }

    #[test]
    fn network_config_clone() {
        let nc = NetworkConfig::new("c".to_string(), "ws://y".to_string(), None, 1);
        let nc2 = nc.clone();
        assert_eq!(nc.name(), nc2.name());
        assert_eq!(nc.ws_endpoint(), nc2.ws_endpoint());
    }

    #[test]
    fn axon_info_new_defaults() {
        let ai = AxonInfo::new(0, 8090, 4, 0, 0, "".to_string(), "".to_string());
        assert_eq!(ai.ip(), 0);
        assert_eq!(ai.port(), 8090);
        assert_eq!(ai.ip_type(), 4);
        assert_eq!(ai.protocol(), 0);
        assert_eq!(ai.version(), 0);
        assert_eq!(ai.hotkey(), "");
        assert_eq!(ai.coldkey(), "");
    }

    #[test]
    fn axon_info_new_custom() {
        let ai = AxonInfo::new(3232235521, 8080, 4, 1, 3, "hk".to_string(), "ck".to_string());
        assert_eq!(ai.ip(), 3232235521);
        assert_eq!(ai.port(), 8080);
        assert_eq!(ai.protocol(), 1);
        assert_eq!(ai.version(), 3);
        assert_eq!(ai.hotkey(), "hk");
        assert_eq!(ai.coldkey(), "ck");
    }

    #[test]
    fn axon_info_repr() {
        let ai = AxonInfo::new(0, 8090, 4, 0, 0, "".to_string(), "".to_string());
        let repr = ai.__repr__();
        assert!(repr.contains("AxonInfo"));
        assert!(repr.contains("8090"));
    }

    #[test]
    fn axon_info_clone() {
        let ai = AxonInfo::new(1, 2, 3, 4, 5, "hk".to_string(), "ck".to_string());
        let ai2 = ai.clone();
        assert_eq!(ai.ip(), ai2.ip());
        assert_eq!(ai.port(), ai2.port());
        assert_eq!(ai.hotkey(), ai2.hotkey());
    }

    #[test]
    fn stake_info_from_rust() {
        let rs = RustStakeInfo {
            hotkey: "hot".to_string(),
            coldkey: "cold".to_string(),
            stake: RustBalance::from_rao(1000),
        };
        let ps: StakeInfo = rs.into();
        assert_eq!(ps.hotkey(), "hot");
        assert_eq!(ps.coldkey(), "cold");
        assert_eq!(ps.stake().rao(), 1000);
    }

    #[test]
    fn metagraph_info_from_rust() {
        let rm = RustMetagraphInfo {
            netuid: 1,
            block: 100,
            n: 10,
            stake: RustBalance::from_rao(5000),
            total_issuance: RustBalance::from_rao(10000),
            total_weight: 0,
            total_bond: 0,
        };
        let pm: MetagraphInfo = rm.into();
        assert_eq!(pm.netuid(), 1);
        assert_eq!(pm.block(), 100);
        assert_eq!(pm.n(), 10);
        assert_eq!(pm.stake().rao(), 5000);
        assert_eq!(pm.total_issuance().rao(), 10000);
    }

    #[test]
    fn prometheus_info_getters() {
        let pi = PrometheusInfo {
            inner: RustPrometheusInfo { ip: 16777343, port: 9100, version: 1, block: 12345 },
        };
        assert_eq!(pi.ip(), 16777343);
        assert_eq!(pi.port(), 9100);
        assert_eq!(pi.version(), 1);
        assert_eq!(pi.block(), 12345);
    }

    #[test]
    fn prometheus_info_repr() {
        let pi =
            PrometheusInfo { inner: RustPrometheusInfo { ip: 0, port: 0, version: 0, block: 0 } };
        assert!(pi.__repr__().contains("PrometheusInfo"));
    }

    #[test]
    fn delegate_info_getters() {
        let rd = RustDelegateInfo {
            delegate_ss58: "5Del".to_string(),
            delegate_hotkey: "hk".to_string(),
            total_stake: RustBalance::from_rao(5000),
            nominators: vec![("n1".to_string(), RustBalance::from_rao(2500))],
            owner_hotkey: "owner".to_string(),
            take: 18,
            owner_ss58: "5Own".to_string(),
            registrations: vec![1, 3],
            validator_permits: vec![1],
        };
        let pd = DelegateInfo { inner: rd };
        assert_eq!(pd.delegate_ss58(), "5Del");
        assert_eq!(pd.delegate_hotkey(), "hk");
        assert_eq!(pd.total_stake().rao(), 5000);
        assert_eq!(pd.owner_hotkey(), "owner");
        assert_eq!(pd.take(), 18);
        assert_eq!(pd.owner_ss58(), "5Own");
        assert_eq!(pd.registrations(), vec![1u16, 3]);
        assert_eq!(pd.validator_permits(), vec![1u16]);
        assert_eq!(pd.nominators().len(), 1);
    }

    #[test]
    fn delegate_info_repr() {
        let rd = RustDelegateInfo {
            delegate_ss58: "5D".to_string(),
            delegate_hotkey: "hk".to_string(),
            total_stake: RustBalance::from_rao(100),
            nominators: vec![],
            owner_hotkey: "o".to_string(),
            take: 0,
            owner_ss58: "5O".to_string(),
            registrations: vec![],
            validator_permits: vec![],
        };
        let pd = DelegateInfo { inner: rd };
        assert!(pd.__repr__().contains("DelegateInfo"));
    }

    #[test]
    fn neuron_info_getters() {
        let rn = RustNeuronInfo {
            uid: 42,
            netuid: 1,
            active: true,
            stake: RustBalance::from_rao(1000),
            rank: 10,
            trust: 5,
            consensus: 3,
            incentive: 7,
            dividend: 2,
            emission: 999,
            prometheus_info: None,
            axon_info: None,
            hotkey: "hk".to_string(),
            coldkey: "ck".to_string(),
            last_update: 500,
            validator_trust: 8,
            weights: vec![],
            bonds: vec![],
            stake_dict: vec![],
        };
        let pn = NeuronInfo { inner: rn };
        assert_eq!(pn.uid(), 42);
        assert_eq!(pn.netuid(), 1);
        assert!(pn.active());
        assert_eq!(pn.stake().rao(), 1000);
        assert_eq!(pn.rank(), 10);
        assert_eq!(pn.trust(), 5);
        assert_eq!(pn.consensus(), 3);
        assert_eq!(pn.incentive(), 7);
        assert_eq!(pn.dividend(), 2);
        assert_eq!(pn.emission(), 999);
        assert_eq!(pn.hotkey(), "hk");
        assert_eq!(pn.coldkey(), "ck");
        assert_eq!(pn.last_update(), 500);
        assert_eq!(pn.validator_trust(), 8);
    }

    #[test]
    fn neuron_info_repr() {
        let rn = RustNeuronInfo {
            uid: 1,
            netuid: 2,
            active: false,
            stake: RustBalance::ZERO,
            rank: 0,
            trust: 0,
            consensus: 0,
            incentive: 0,
            dividend: 0,
            emission: 0,
            prometheus_info: None,
            axon_info: None,
            hotkey: "x".to_string(),
            coldkey: "y".to_string(),
            last_update: 0,
            validator_trust: 0,
            weights: vec![],
            bonds: vec![],
            stake_dict: vec![],
        };
        let pn = NeuronInfo { inner: rn };
        assert!(pn.__repr__().contains("NeuronInfo"));
    }

    #[test]
    fn neuron_info_lite_getters() {
        let rnl = RustNeuronInfoLite {
            uid: 7,
            hotkey: "hk2".to_string(),
            coldkey: "ck2".to_string(),
            active: true,
            stake: RustBalance::from_rao(2000),
            rank: 11,
            trust: 6,
            consensus: 4,
            incentive: 8,
        };
        let pnl = NeuronInfoLite { inner: rnl };
        assert_eq!(pnl.uid(), 7);
        assert_eq!(pnl.hotkey(), "hk2");
        assert_eq!(pnl.coldkey(), "ck2");
        assert!(pnl.active());
        assert_eq!(pnl.stake().rao(), 2000);
        assert_eq!(pnl.rank(), 11);
        assert_eq!(pnl.trust(), 6);
        assert_eq!(pnl.consensus(), 4);
        assert_eq!(pnl.incentive(), 8);
    }

    #[test]
    fn neuron_info_lite_repr() {
        let rnl = RustNeuronInfoLite {
            uid: 0,
            hotkey: String::new(),
            coldkey: String::new(),
            active: false,
            stake: RustBalance::ZERO,
            rank: 0,
            trust: 0,
            consensus: 0,
            incentive: 0,
        };
        let pnl = NeuronInfoLite { inner: rnl };
        assert!(pnl.__repr__().contains("NeuronInfoLite"));
    }

    #[test]
    fn subnet_info_getters() {
        let rs = RustSubnetInfo {
            netuid: 5,
            name: "root".to_string(),
            owner_hotkey: "owner".to_string(),
            tempo: 100,
            subnet_identity: None,
            maximum_uid: 256,
            modality: 0,
            network_uid: 5,
        };
        let ps = SubnetInfo { inner: rs };
        assert_eq!(ps.netuid(), 5);
        assert_eq!(ps.name(), "root");
        assert_eq!(ps.owner_hotkey(), "owner");
        assert_eq!(ps.tempo(), 100);
        assert_eq!(ps.maximum_uid(), 256);
        assert_eq!(ps.modality(), 0);
        assert_eq!(ps.network_uid(), 5);
    }

    #[test]
    fn subnet_info_repr() {
        let rs = RustSubnetInfo {
            netuid: 1,
            name: "x".to_string(),
            owner_hotkey: "y".to_string(),
            tempo: 0,
            subnet_identity: None,
            maximum_uid: 0,
            modality: 0,
            network_uid: 0,
        };
        let ps = SubnetInfo { inner: rs };
        assert!(ps.__repr__().contains("SubnetInfo"));
    }

    #[test]
    fn subnet_hyperparameters_getters() {
        let rh = RustSubnetHyperparameters {
            rho: 10,
            kappa: 32768,
            difficulty: 1000000,
            burn: 1000,
            immunity_ratio: 25,
            min_burn: 500,
            max_burn: 2000,
            weights_rate_limit: 100,
            weights_version: 0,
            weights_min_stake: 0,
            max_weight_limit: 1000,
            scaling_law_power: 50,
            subnetwork_n: 256,
            max_n: 1024,
            blocks_since_last_step: 0,
            tempo: 360,
            adjustment_alpha: 500,
            adjustment_interval: 100,
            bonds_moving_avg: 1000,
            alpha_high: 58,
            alpha_low: 60,
            liquid_alpha_enabled: false,
        };
        let ph = SubnetHyperparameters { inner: rh };
        assert_eq!(ph.rho(), 10);
        assert_eq!(ph.kappa(), 32768);
        assert_eq!(ph.difficulty(), 1000000);
        assert_eq!(ph.burn(), 1000);
        assert_eq!(ph.immunity_ratio(), 25);
        assert_eq!(ph.min_burn(), 500);
        assert_eq!(ph.max_burn(), 2000);
        assert_eq!(ph.weights_rate_limit(), 100);
        assert_eq!(ph.weights_version(), 0);
        assert_eq!(ph.max_weight_limit(), 1000);
        assert_eq!(ph.scaling_law_power(), 50);
        assert_eq!(ph.subnetwork_n(), 256);
        assert_eq!(ph.max_n(), 1024);
        assert_eq!(ph.tempo(), 360);
        assert!(!ph.liquid_alpha_enabled());
    }

    #[test]
    fn subnet_hyperparameters_repr() {
        let rh = RustSubnetHyperparameters {
            rho: 1,
            kappa: 2,
            difficulty: 3,
            burn: 0,
            immunity_ratio: 0,
            min_burn: 0,
            max_burn: 0,
            weights_rate_limit: 0,
            weights_version: 0,
            weights_min_stake: 0,
            max_weight_limit: 0,
            scaling_law_power: 0,
            subnetwork_n: 0,
            max_n: 0,
            blocks_since_last_step: 0,
            tempo: 0,
            adjustment_alpha: 0,
            adjustment_interval: 0,
            bonds_moving_avg: 0,
            alpha_high: 0,
            alpha_low: 0,
            liquid_alpha_enabled: false,
        };
        let ph = SubnetHyperparameters { inner: rh };
        assert!(ph.__repr__().contains("SubnetHyperparameters"));
    }

    #[test]
    fn neuron_certificate_getters() {
        let rc = RustNeuronCertificate {
            hotkey: "cert_hk".to_string(),
            certificate: vec![1u8, 2, 3],
            block: 42,
        };
        let pc = NeuronCertificate { inner: rc };
        assert_eq!(pc.hotkey(), "cert_hk");
        assert_eq!(pc.certificate(), vec![1u8, 2, 3]);
        assert_eq!(pc.block(), 42);
    }

    #[test]
    fn neuron_certificate_repr() {
        let rc = RustNeuronCertificate { hotkey: "h".to_string(), certificate: vec![], block: 0 };
        let pc = NeuronCertificate { inner: rc };
        assert!(pc.__repr__().contains("NeuronCertificate"));
    }
}
