//! WASM-compatible type wrappers for the bittensor-rs SDK.
//!
//! These types mirror the bittensor-core types but are annotated with
//! `#[wasm_bindgen]` to be accessible from JavaScript. The original
//! bittensor-core crate cannot be used directly because it depends on
//! subxt (which pulls in tokio), making it incompatible with
//! wasm32-unknown-unknown.

use wasm_bindgen::prelude::*;

// ---------------------------------------------------------------------------
// Balance
// ---------------------------------------------------------------------------

const RAO_PER_TAO: u64 = 1_000_000_000;

/// Represents a Bittensor balance in rao (1 TAO = 1,000,000,000 rao).
///
/// All arithmetic is done in rao to avoid floating-point precision loss.
/// JS consumers can construct via `Balance.from_tao(1.5)` or
/// `Balance.from_rao(1500000000)` and read back with `.to_tao()` / `.to_rao()`.
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Balance {
    rao: u64,
}

#[wasm_bindgen]
impl Balance {
    /// Create a zero balance.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { rao: 0 }
    }

    /// Create from tao (1.0 TAO = 1,000,000,000 rao).
    pub fn from_tao(tao: f64) -> Self {
        let rao = (tao * RAO_PER_TAO as f64).round() as u64;
        Self { rao }
    }

    /// Create from rao.
    pub fn from_rao(rao: u64) -> Self {
        Self { rao }
    }

    /// Convert to tao as f64.
    pub fn to_tao(&self) -> f64 {
        self.rao as f64 / RAO_PER_TAO as f64
    }

    /// Convert to rao as u64.
    pub fn to_rao(&self) -> u64 {
        self.rao
    }

    /// Saturating addition (clamps at u64::MAX).
    pub fn add(&self, other: &Balance) -> Balance {
        Balance { rao: self.rao.saturating_add(other.rao) }
    }

    /// Saturating subtraction (clamps at 0).
    pub fn sub(&self, other: &Balance) -> Balance {
        Balance { rao: self.rao.saturating_sub(other.rao) }
    }

    /// Human-readable display string (9 decimal places).
    pub fn display(&self) -> String {
        let tao = self.to_tao();
        format!("{tao:.9}")
    }
}

impl Default for Balance {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// NetworkConfig
// ---------------------------------------------------------------------------

/// Network configuration for connecting to a Subtensor chain endpoint.
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    name: String,
    ws_endpoint: String,
    chain_id: u16,
}

#[wasm_bindgen]
impl NetworkConfig {
    /// Finney mainnet configuration.
    pub fn finney() -> Self {
        Self {
            name: "finney".into(),
            ws_endpoint: "wss://entrypoint-finney.opentensor.ai:443".into(),
            chain_id: 42,
        }
    }

    /// Testnet configuration.
    pub fn test() -> Self {
        Self {
            name: "test".into(),
            ws_endpoint: "wss://test.finney.opentensor.ai:443".into(),
            chain_id: 42,
        }
    }

    /// Local development node configuration.
    pub fn local() -> Self {
        Self { name: "local".into(), ws_endpoint: "ws://127.0.0.1:9944".into(), chain_id: 42 }
    }

    /// Human-readable network name.
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// WebSocket endpoint URL.
    pub fn ws_url(&self) -> String {
        self.ws_endpoint.clone()
    }

    /// Chain identifier (SS58 prefix).
    pub fn chain_id(&self) -> u16 {
        self.chain_id
    }
}

// ---------------------------------------------------------------------------
// AxonInfo
// ---------------------------------------------------------------------------

/// Information about a neuron's axon (server-side endpoint).
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct AxonInfo {
    inner: AxonInfoData,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct AxonInfoData {
    pub ip: u64,
    pub port: u16,
    pub ip_type: u8,
    pub protocol: u8,
    pub version: u32,
    pub hotkey: String,
    pub coldkey: String,
}

#[wasm_bindgen]
impl AxonInfo {
    /// Create a new `AxonInfo` with the given endpoint fields.
    #[wasm_bindgen(constructor)]
    pub fn new(
        ip: u64,
        port: u16,
        ip_type: u8,
        protocol: u8,
        version: u32,
        hotkey: String,
        coldkey: String,
    ) -> Self {
        Self { inner: AxonInfoData { ip, port, ip_type, protocol, version, hotkey, coldkey } }
    }

    /// Encoded IP address as u64 (e.g. 127.0.0.1 → 2130706433).
    pub fn ip(&self) -> u64 {
        self.inner.ip
    }
    /// TCP port number.
    pub fn port(&self) -> u16 {
        self.inner.port
    }
    /// IP protocol version (4 = IPv4, 6 = IPv6).
    pub fn ip_type(&self) -> u8 {
        self.inner.ip_type
    }
    /// Transport protocol (0 = HTTP, 1 = HTTPS).
    pub fn protocol(&self) -> u8 {
        self.inner.protocol
    }
    /// Bittensor node version.
    pub fn version(&self) -> u32 {
        self.inner.version
    }
    /// SS58-encoded hotkey public key.
    pub fn hotkey(&self) -> String {
        self.inner.hotkey.clone()
    }
    /// SS58-encoded coldkey public key.
    pub fn coldkey(&self) -> String {
        self.inner.coldkey.clone()
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.inner)
            .map_err(|e| JsValue::from_str(&format!("serialization error: {e}")))
    }

    /// Deserialize from JSON string.
    pub fn from_json(json: &str) -> Result<AxonInfo, JsValue> {
        serde_json::from_str(json)
            .map(|inner| AxonInfo { inner })
            .map_err(|e| JsValue::from_str(&format!("deserialization error: {e}")))
    }
}

// ---------------------------------------------------------------------------
// RegistrationInfo
// ---------------------------------------------------------------------------

/// Neuron registration information.
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct RegistrationInfo {
    inner: RegistrationInfoData,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegistrationInfoData {
    pub netuid: u16,
    pub hotkey: String,
    pub block: u64,
    pub burn: u64,
}

#[wasm_bindgen]
impl RegistrationInfo {
    pub fn netuid(&self) -> u16 {
        self.inner.netuid
    }
    pub fn hotkey(&self) -> String {
        self.inner.hotkey.clone()
    }
    pub fn block(&self) -> u64 {
        self.inner.block
    }
    pub fn burn_rao(&self) -> u64 {
        self.inner.burn
    }
    pub fn burn_tao(&self) -> f64 {
        self.inner.burn as f64 / RAO_PER_TAO as f64
    }

    pub fn to_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.inner)
            .map_err(|e| JsValue::from_str(&format!("serialization error: {e}")))
    }

    /// Deserialize from JSON string.
    pub fn from_json(json: &str) -> Result<RegistrationInfo, JsValue> {
        serde_json::from_str(json)
            .map(|inner| RegistrationInfo { inner })
            .map_err(|e| JsValue::from_str(&format!("deserialization error: {e}")))
    }
}

// ---------------------------------------------------------------------------
// TerminalInfo (wrapper around bittensor_synapse::TerminalInfo)
// ---------------------------------------------------------------------------

/// Terminal information for synapse endpoints.
///
/// Wraps [`bittensor_synapse::TerminalInfo`] for JS access.
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct TerminalInfo {
    inner: bittensor_synapse::TerminalInfo,
}

#[wasm_bindgen]
impl TerminalInfo {
    /// Create an empty `TerminalInfo` with all fields `None`.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { inner: bittensor_synapse::TerminalInfo::new() }
    }

    /// HTTP status code of the response.
    pub fn status_code(&self) -> Option<u16> {
        self.inner.status_code
    }
    /// Status message (e.g. "OK" or "Timeout").
    pub fn status_message(&self) -> Option<String> {
        self.inner.status_message.clone()
    }
    /// Server-side processing time in seconds.
    pub fn process_time(&self) -> Option<f64> {
        self.inner.process_time
    }
    /// IP address string.
    pub fn ip(&self) -> Option<String> {
        self.inner.ip.clone()
    }
    /// Port number.
    pub fn port(&self) -> Option<u16> {
        self.inner.port
    }
    /// Bittensor node version.
    pub fn version(&self) -> Option<u32> {
        self.inner.version
    }
    /// Nonce for replay protection.
    pub fn nonce(&self) -> Option<u64> {
        self.inner.nonce
    }
    /// Unique request UUID.
    pub fn uuid(&self) -> Option<String> {
        self.inner.uuid.clone()
    }
    /// SS58-encoded hotkey.
    pub fn hotkey(&self) -> Option<String> {
        self.inner.hotkey.clone()
    }
    /// Hex-encoded Sr25519 signature.
    pub fn signature(&self) -> Option<String> {
        self.inner.signature.clone()
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.inner)
            .map_err(|e| JsValue::from_str(&format!("serialization error: {e}")))
    }

    /// Deserialize from JSON string.
    pub fn from_json(json: &str) -> Result<TerminalInfo, JsValue> {
        serde_json::from_str(json)
            .map(|inner| TerminalInfo { inner })
            .map_err(|e| JsValue::from_str(&format!("deserialization error: {e}")))
    }
}

impl Default for TerminalInfo {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// NeuronInfoLite
// ---------------------------------------------------------------------------

/// Lightweight neuron info for a subnet (no weight/bond vectors).
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct NeuronInfoLite {
    inner: NeuronInfoLiteData,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct NeuronInfoLiteData {
    pub uid: u16,
    pub hotkey: String,
    pub coldkey: String,
    pub active: bool,
    pub incentive: u16,
    pub stake_rao: u64,
}

#[wasm_bindgen]
impl NeuronInfoLite {
    /// Neuron UID within the subnet.
    pub fn uid(&self) -> u16 {
        self.inner.uid
    }
    /// SS58-encoded hotkey.
    pub fn hotkey(&self) -> String {
        self.inner.hotkey.clone()
    }
    /// SS58-encoded coldkey.
    pub fn coldkey(&self) -> String {
        self.inner.coldkey.clone()
    }
    /// Whether the neuron is currently active.
    pub fn active(&self) -> bool {
        self.inner.active
    }
    /// Incentive value (0–65535).
    pub fn incentive(&self) -> u16 {
        self.inner.incentive
    }
    /// Stake in rao.
    pub fn stake_rao(&self) -> u64 {
        self.inner.stake_rao
    }
    /// Stake in TAO (f64).
    pub fn stake_tao(&self) -> f64 {
        self.inner.stake_rao as f64 / RAO_PER_TAO as f64
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.inner)
            .map_err(|e| JsValue::from_str(&format!("serialization error: {e}")))
    }

    /// Deserialize from JSON string.
    pub fn from_json(json: &str) -> Result<NeuronInfoLite, JsValue> {
        serde_json::from_str(json)
            .map(|inner| NeuronInfoLite { inner })
            .map_err(|e| JsValue::from_str(&format!("deserialization error: {e}")))
    }
}

// ---------------------------------------------------------------------------
// SubnetInfo
// ---------------------------------------------------------------------------

/// Subnet metadata: name, owner, tempo, and UID limits.
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct SubnetInfo {
    inner: SubnetInfoData,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubnetInfoData {
    pub netuid: u16,
    pub name: String,
    pub owner_hotkey: String,
    pub tempo: u16,
    pub maximum_uid: u16,
    pub modality: u8,
    pub network_uid: u16,
}

#[wasm_bindgen]
impl SubnetInfo {
    /// Subnet unique identifier.
    pub fn netuid(&self) -> u16 {
        self.inner.netuid
    }
    /// Human-readable subnet name.
    pub fn name(&self) -> String {
        self.inner.name.clone()
    }
    /// SS58-encoded owner hotkey.
    pub fn owner_hotkey(&self) -> String {
        self.inner.owner_hotkey.clone()
    }
    /// Block tempo (interval between weight-setting rounds).
    pub fn tempo(&self) -> u16 {
        self.inner.tempo
    }
    /// Maximum UID count in the subnet.
    pub fn maximum_uid(&self) -> u16 {
        self.inner.maximum_uid
    }
    /// Subnet modality (0 = text, 1 = image, 2 = audio, etc.).
    pub fn modality(&self) -> u8 {
        self.inner.modality
    }
    /// Network-level UID.
    pub fn network_uid(&self) -> u16 {
        self.inner.network_uid
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.inner)
            .map_err(|e| JsValue::from_str(&format!("serialization error: {e}")))
    }

    /// Deserialize from JSON string.
    pub fn from_json(json: &str) -> Result<SubnetInfo, JsValue> {
        serde_json::from_str(json)
            .map(|inner| SubnetInfo { inner })
            .map_err(|e| JsValue::from_str(&format!("deserialization error: {e}")))
    }
}

// ---------------------------------------------------------------------------
// SubnetHyperparams
// ---------------------------------------------------------------------------

/// Tunable hyperparameters for a subnet (rho, kappa, difficulty, etc.).
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct SubnetHyperparams {
    inner: SubnetHyperparamsData,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubnetHyperparamsData {
    pub rho: u16,
    pub kappa: u16,
    pub difficulty: u64,
    pub burn: u64,
    pub immunity_ratio: u16,
    pub min_burn: u64,
    pub max_burn: u64,
    pub weights_rate_limit: u64,
    pub weights_version: u64,
    pub weights_min_stake: u64,
    pub max_weight_limit: u16,
    pub scaling_law_power: u16,
    pub subnetwork_n: u16,
    pub max_n: u16,
    pub blocks_since_last_step: u64,
    pub tempo: u16,
    pub adjustment_alpha: u64,
    pub adjustment_interval: u64,
    pub bonds_moving_avg: u64,
    pub alpha_high: u16,
    pub alpha_low: u16,
    pub liquid_alpha_enabled: bool,
}

#[wasm_bindgen]
impl SubnetHyperparams {
    /// Rho: trust ratio denominator.
    pub fn rho(&self) -> u16 {
        self.inner.rho
    }
    /// Kappa: trust ratio numerator.
    pub fn kappa(&self) -> u16 {
        self.inner.kappa
    }
    /// Registration difficulty.
    pub fn difficulty(&self) -> u64 {
        self.inner.difficulty
    }
    /// Registration burn cost in rao.
    pub fn burn(&self) -> u64 {
        self.inner.burn
    }
    /// Immunity period ratio (percentage).
    pub fn immunity_ratio(&self) -> u16 {
        self.inner.immunity_ratio
    }
    /// Minimum registration burn in rao.
    pub fn min_burn(&self) -> u64 {
        self.inner.min_burn
    }
    /// Maximum registration burn in rao.
    pub fn max_burn(&self) -> u64 {
        self.inner.max_burn
    }
    /// Minimum blocks between weight-setting calls.
    pub fn weights_rate_limit(&self) -> u64 {
        self.inner.weights_rate_limit
    }
    /// Expected weights version key.
    pub fn weights_version(&self) -> u64 {
        self.inner.weights_version
    }
    /// Whether liquid alpha is enabled for the subnet.
    pub fn liquid_alpha_enabled(&self) -> bool {
        self.inner.liquid_alpha_enabled
    }
    /// Block tempo (interval between weight-setting rounds).
    pub fn tempo(&self) -> u16 {
        self.inner.tempo
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.inner)
            .map_err(|e| JsValue::from_str(&format!("serialization error: {e}")))
    }

    /// Deserialize from JSON string.
    pub fn from_json(json: &str) -> Result<SubnetHyperparams, JsValue> {
        serde_json::from_str(json)
            .map(|inner| SubnetHyperparams { inner })
            .map_err(|e| JsValue::from_str(&format!("deserialization error: {e}")))
    }
}

// ---------------------------------------------------------------------------
// StakeInfo
// ---------------------------------------------------------------------------

/// Stake entry linking a hotkey/coldkey pair to a stake amount.
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct StakeInfo {
    inner: StakeInfoData,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct StakeInfoData {
    pub hotkey: String,
    pub coldkey: String,
    pub stake: u64,
}

#[wasm_bindgen]
impl StakeInfo {
    /// SS58-encoded hotkey.
    pub fn hotkey(&self) -> String {
        self.inner.hotkey.clone()
    }
    /// SS58-encoded coldkey.
    pub fn coldkey(&self) -> String {
        self.inner.coldkey.clone()
    }
    /// Stake in rao.
    pub fn stake_rao(&self) -> u64 {
        self.inner.stake
    }
    /// Stake in TAO (f64).
    pub fn stake_tao(&self) -> f64 {
        self.inner.stake as f64 / RAO_PER_TAO as f64
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.inner)
            .map_err(|e| JsValue::from_str(&format!("serialization error: {e}")))
    }

    /// Deserialize from JSON string.
    pub fn from_json(json: &str) -> Result<StakeInfo, JsValue> {
        serde_json::from_str(json)
            .map(|inner| StakeInfo { inner })
            .map_err(|e| JsValue::from_str(&format!("deserialization error: {e}")))
    }
}

// ---------------------------------------------------------------------------
// DelegateInfo
// ---------------------------------------------------------------------------

/// Delegate metadata: hotkey, total stake, nominators, take, and registrations.
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct DelegateInfo {
    inner: DelegateInfoData,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct DelegateInfoData {
    pub delegate_ss58: String,
    pub delegate_hotkey: String,
    pub total_stake: u64,
    pub nominators: Vec<(String, u64)>,
    pub owner_hotkey: String,
    pub take: u16,
    pub owner_ss58: String,
    pub registrations: Vec<u16>,
    pub validator_permits: Vec<u16>,
}

#[wasm_bindgen]
impl DelegateInfo {
    /// SS58-encoded delegate address.
    pub fn delegate_ss58(&self) -> String {
        self.inner.delegate_ss58.clone()
    }
    /// Delegate hotkey string.
    pub fn delegate_hotkey(&self) -> String {
        self.inner.delegate_hotkey.clone()
    }
    /// Total delegated stake in rao.
    pub fn total_stake_rao(&self) -> u64 {
        self.inner.total_stake
    }
    /// Total delegated stake in TAO (f64).
    pub fn total_stake_tao(&self) -> f64 {
        self.inner.total_stake as f64 / RAO_PER_TAO as f64
    }
    /// Owner hotkey string.
    pub fn owner_hotkey(&self) -> String {
        self.inner.owner_hotkey.clone()
    }
    /// Delegate take percentage (0–10000 basis points).
    pub fn take(&self) -> u16 {
        self.inner.take
    }
    /// SS58-encoded owner address.
    pub fn owner_ss58(&self) -> String {
        self.inner.owner_ss58.clone()
    }
    /// Number of nominators for this delegate.
    pub fn nominator_count(&self) -> usize {
        self.inner.nominators.len()
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.inner)
            .map_err(|e| JsValue::from_str(&format!("serialization error: {e}")))
    }

    /// Deserialize from JSON string.
    pub fn from_json(json: &str) -> Result<DelegateInfo, JsValue> {
        serde_json::from_str(json)
            .map(|inner| DelegateInfo { inner })
            .map_err(|e| JsValue::from_str(&format!("deserialization error: {e}")))
    }
}

// ---------------------------------------------------------------------------
// Internal helpers: conversion from JSON-RPC responses
// ---------------------------------------------------------------------------

impl NeuronInfoLite {
    /// Construct from a raw JSON value (used by queries).
    // Dead code allowed: reserved for future WASM query endpoints that return raw JSON
    #[allow(dead_code)]
    pub(crate) fn from_serde_value(v: serde_json::Value) -> Result<Self, String> {
        let data: NeuronInfoLiteData =
            serde_json::from_value(v).map_err(|e| format!("deserialize NeuronInfoLite: {e}"))?;
        Ok(NeuronInfoLite { inner: data })
    }
}

impl SubnetInfo {
    pub(crate) fn from_serde_value(v: serde_json::Value) -> Result<Self, String> {
        let data: SubnetInfoData =
            serde_json::from_value(v).map_err(|e| format!("deserialize SubnetInfo: {e}"))?;
        Ok(SubnetInfo { inner: data })
    }
}

impl SubnetHyperparams {
    // Dead code allowed: reserved for future WASM query endpoints that return raw JSON
    #[allow(dead_code)]
    pub(crate) fn from_serde_value(v: serde_json::Value) -> Result<Self, String> {
        let data: SubnetHyperparamsData =
            serde_json::from_value(v).map_err(|e| format!("deserialize SubnetHyperparams: {e}"))?;
        Ok(SubnetHyperparams { inner: data })
    }
}

impl StakeInfo {
    // Dead code allowed: reserved for future WASM query endpoints that return raw JSON
    #[allow(dead_code)]
    pub(crate) fn from_serde_value(v: serde_json::Value) -> Result<Self, String> {
        let data: StakeInfoData =
            serde_json::from_value(v).map_err(|e| format!("deserialize StakeInfo: {e}"))?;
        Ok(StakeInfo { inner: data })
    }
}

// ---------------------------------------------------------------------------
// Tests (native target)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn balance_from_tao() {
        let b = Balance::from_tao(1.5);
        assert_eq!(b.to_rao(), 1_500_000_000);
        let diff = (b.to_tao() - 1.5).abs();
        assert!(diff < 1e-10);
    }

    #[test]
    fn balance_from_rao() {
        let b = Balance::from_rao(42);
        assert_eq!(b.to_rao(), 42);
    }

    #[test]
    fn balance_zero() {
        let b = Balance::new();
        assert_eq!(b.to_rao(), 0);
        assert_eq!(b.to_tao(), 0.0);
    }

    #[test]
    fn balance_add_sub() {
        let a = Balance::from_tao(3.0);
        let b = Balance::from_tao(1.5);
        let sum = a.add(&b);
        assert_eq!(sum.to_tao(), 4.5);
        let diff = a.sub(&b);
        assert_eq!(diff.to_tao(), 1.5);
    }

    #[test]
    fn balance_display() {
        let b = Balance::from_tao(1.5);
        assert_eq!(b.display(), "1.500000000");
        let zero = Balance::new();
        assert_eq!(zero.display(), "0.000000000");
        let small = Balance::from_rao(1);
        assert_eq!(small.display(), "0.000000001");
    }

    #[test]
    fn network_config_finney() {
        let cfg = NetworkConfig::finney();
        assert_eq!(cfg.name(), "finney");
        assert!(cfg.ws_url().starts_with("wss://"));
    }

    #[test]
    fn network_config_test() {
        let cfg = NetworkConfig::test();
        assert_eq!(cfg.name(), "test");
    }

    #[test]
    fn network_config_local() {
        let cfg = NetworkConfig::local();
        assert_eq!(cfg.name(), "local");
        assert_eq!(cfg.ws_url(), "ws://127.0.0.1:9944");
    }

    #[test]
    fn axon_info_json_roundtrip() {
        let info = AxonInfo::new(
            2130706433,
            8090,
            4,
            0,
            1,
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".into(),
            "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty".into(),
        );
        let json = info.to_json().unwrap();
        let restored = AxonInfo::from_json(&json).unwrap();
        assert_eq!(restored.ip(), 2130706433);
        assert_eq!(restored.port(), 8090);
        assert_eq!(restored.hotkey(), "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY");
    }

    #[test]
    fn subnet_info_json_roundtrip() {
        let data = SubnetInfoData {
            netuid: 1,
            name: "root".into(),
            owner_hotkey: "owner".into(),
            tempo: 100,
            maximum_uid: 256,
            modality: 0,
            network_uid: 1,
        };
        let info = SubnetInfo { inner: data };
        let json = info.to_json().unwrap();
        let restored = SubnetInfo::from_json(&json).unwrap();
        assert_eq!(restored.netuid(), 1);
        assert_eq!(restored.name(), "root");
    }

    #[test]
    fn terminal_info_default() {
        let info = TerminalInfo::new();
        assert!(info.status_code().is_none());
        assert!(info.hotkey().is_none());
    }

    #[test]
    fn stake_info_json_roundtrip() {
        let data =
            StakeInfoData { hotkey: "hk".into(), coldkey: "ck".into(), stake: 1_500_000_000 };
        let info = StakeInfo { inner: data };
        let json = info.to_json().unwrap();
        let restored = StakeInfo::from_json(&json).unwrap();
        assert_eq!(restored.hotkey(), "hk");
        assert_eq!(restored.stake_tao(), 1.5);
    }

    #[test]
    fn registration_info_json_roundtrip() {
        let data = RegistrationInfoData {
            netuid: 1,
            hotkey: "hk".into(),
            block: 100,
            burn: 1_000_000_000,
        };
        let info = RegistrationInfo { inner: data };
        let json = info.to_json().unwrap();
        let restored = RegistrationInfo::from_json(&json).unwrap();
        assert_eq!(restored.netuid(), 1);
        assert_eq!(restored.burn_tao(), 1.0);
    }

    #[test]
    fn delegate_info_json_roundtrip() {
        let data = DelegateInfoData {
            delegate_ss58: "5Test".into(),
            delegate_hotkey: "hk".into(),
            total_stake: 500_000_000_000,
            nominators: vec![("nom1".into(), 250_000_000_000)],
            owner_hotkey: "owner".into(),
            take: 18,
            owner_ss58: "5Owner".into(),
            registrations: vec![1, 3],
            validator_permits: vec![1],
        };
        let info = DelegateInfo { inner: data };
        let json = info.to_json().unwrap();
        let restored = DelegateInfo::from_json(&json).unwrap();
        assert_eq!(restored.take(), 18);
        assert_eq!(restored.nominator_count(), 1);
    }

    #[test]
    fn subnet_hyperparams_json_roundtrip() {
        let data = SubnetHyperparamsData {
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
        let info = SubnetHyperparams { inner: data };
        let json = info.to_json().unwrap();
        let restored = SubnetHyperparams::from_json(&json).unwrap();
        assert_eq!(restored.rho(), 10);
        assert!(!restored.liquid_alpha_enabled());
    }

    // -----------------------------------------------------------------------
    // Additional Balance tests
    // -----------------------------------------------------------------------

    #[test]
    fn balance_saturating_add_overflow() {
        let a = Balance::from_rao(u64::MAX);
        let b = Balance::from_rao(1);
        let sum = a.add(&b);
        assert_eq!(sum.to_rao(), u64::MAX);
    }

    #[test]
    fn balance_saturating_sub_underflow() {
        let a = Balance::from_rao(5);
        let b = Balance::from_rao(10);
        let diff = a.sub(&b);
        assert_eq!(diff.to_rao(), 0);
    }

    #[test]
    fn balance_partial_cmp_ordering() {
        let small = Balance::from_rao(100);
        let big = Balance::from_rao(200);
        assert!(small < big);
        assert!(big > small);
        assert_eq!(small, Balance::from_rao(100));
    }

    #[test]
    fn balance_display_max_u64() {
        let b = Balance::from_rao(u64::MAX);
        let s = b.display();
        assert!(s.contains("18446744073"));
    }

    #[test]
    fn balance_default_trait() {
        let b = Balance::default();
        assert_eq!(b.to_rao(), 0);
    }

    #[test]
    fn balance_from_tao_zero() {
        let b = Balance::from_tao(0.0);
        assert_eq!(b.to_rao(), 0);
    }

    #[test]
    fn balance_from_tao_fractional() {
        let b = Balance::from_tao(0.5);
        assert_eq!(b.to_rao(), 500_000_000);
    }

    // -----------------------------------------------------------------------
    // Additional AxonInfo tests
    // -----------------------------------------------------------------------

    #[test]
    fn axon_info_all_getters() {
        let info =
            AxonInfo::new(2130706433, 8090, 4, 1, 200, "hotkey123".into(), "coldkey456".into());
        assert_eq!(info.ip(), 2130706433);
        assert_eq!(info.port(), 8090);
        assert_eq!(info.ip_type(), 4);
        assert_eq!(info.protocol(), 1);
        assert_eq!(info.version(), 200);
        assert_eq!(info.hotkey(), "hotkey123");
        assert_eq!(info.coldkey(), "coldkey456");
    }

    #[test]
    fn axon_info_from_json_roundtrip_all_fields() {
        let info = AxonInfo::new(12345, 99, 6, 1, 42, "hk".into(), "ck".into());
        let json = info.to_json().unwrap();
        let restored = AxonInfo::from_json(&json).unwrap();
        assert_eq!(restored.ip(), 12345);
        assert_eq!(restored.port(), 99);
        assert_eq!(restored.ip_type(), 6);
        assert_eq!(restored.protocol(), 1);
        assert_eq!(restored.version(), 42);
        assert_eq!(restored.hotkey(), "hk");
        assert_eq!(restored.coldkey(), "ck");
    }

    // -----------------------------------------------------------------------
    // NeuronInfoLite tests
    // -----------------------------------------------------------------------

    #[test]
    fn neuron_info_lite_json_roundtrip() {
        let data = NeuronInfoLiteData {
            uid: 42,
            hotkey: "hk_lite".into(),
            coldkey: "ck_lite".into(),
            active: true,
            incentive: 500,
            stake_rao: 3_000_000_000,
        };
        let info = NeuronInfoLite { inner: data };
        let json = info.to_json().unwrap();
        let restored = NeuronInfoLite::from_json(&json).unwrap();
        assert_eq!(restored.uid(), 42);
        assert_eq!(restored.hotkey(), "hk_lite");
        assert_eq!(restored.coldkey(), "ck_lite");
        assert!(restored.active());
        assert_eq!(restored.incentive(), 500);
        assert_eq!(restored.stake_rao(), 3_000_000_000);
        let diff = (restored.stake_tao() - 3.0).abs();
        assert!(diff < 1e-10);
    }

    #[test]
    fn neuron_info_lite_inactive_zero_stake() {
        let data = NeuronInfoLiteData {
            uid: 0,
            hotkey: "".into(),
            coldkey: "".into(),
            active: false,
            incentive: 0,
            stake_rao: 0,
        };
        let info = NeuronInfoLite { inner: data };
        let json = info.to_json().unwrap();
        let restored = NeuronInfoLite::from_json(&json).unwrap();
        assert!(!restored.active());
        assert_eq!(restored.stake_rao(), 0);
        assert_eq!(restored.stake_tao(), 0.0);
    }

    #[test]
    fn neuron_info_lite_from_serde_value() {
        let v = serde_json::json!({
            "uid": 7,
            "hotkey": "hk",
            "coldkey": "ck",
            "active": true,
            "incentive": 100,
            "stakeRao": 999
        });
        let info = NeuronInfoLite::from_serde_value(v).unwrap();
        assert_eq!(info.uid(), 7);
        assert_eq!(info.stake_rao(), 999);
    }

    // -----------------------------------------------------------------------
    // SubnetInfo full getter tests
    // -----------------------------------------------------------------------

    #[test]
    fn subnet_info_all_getters() {
        let data = SubnetInfoData {
            netuid: 5,
            name: "subnet5".into(),
            owner_hotkey: "owner_hk".into(),
            tempo: 200,
            maximum_uid: 512,
            modality: 1,
            network_uid: 5,
        };
        let info = SubnetInfo { inner: data };
        let json = info.to_json().unwrap();
        let restored = SubnetInfo::from_json(&json).unwrap();
        assert_eq!(restored.netuid(), 5);
        assert_eq!(restored.name(), "subnet5");
        assert_eq!(restored.owner_hotkey(), "owner_hk");
        assert_eq!(restored.tempo(), 200);
        assert_eq!(restored.maximum_uid(), 512);
        assert_eq!(restored.modality(), 1);
        assert_eq!(restored.network_uid(), 5);
    }

    // -----------------------------------------------------------------------
    // SubnetHyperparams full getter tests
    // -----------------------------------------------------------------------

    #[test]
    fn subnet_hyperparams_all_getters() {
        let data = SubnetHyperparamsData {
            rho: 20,
            kappa: 1000,
            difficulty: 5000,
            burn: 200,
            immunity_ratio: 30,
            min_burn: 100,
            max_burn: 400,
            weights_rate_limit: 50,
            weights_version: 2,
            weights_min_stake: 10,
            max_weight_limit: 500,
            scaling_law_power: 75,
            subnetwork_n: 128,
            max_n: 2048,
            blocks_since_last_step: 99,
            tempo: 400,
            adjustment_alpha: 250,
            adjustment_interval: 50,
            bonds_moving_avg: 800,
            alpha_high: 70,
            alpha_low: 30,
            liquid_alpha_enabled: true,
        };
        let info = SubnetHyperparams { inner: data };
        let json = info.to_json().unwrap();
        let restored = SubnetHyperparams::from_json(&json).unwrap();
        assert_eq!(restored.rho(), 20);
        assert_eq!(restored.kappa(), 1000);
        assert_eq!(restored.difficulty(), 5000);
        assert_eq!(restored.burn(), 200);
        assert_eq!(restored.immunity_ratio(), 30);
        assert_eq!(restored.min_burn(), 100);
        assert_eq!(restored.max_burn(), 400);
        assert_eq!(restored.weights_rate_limit(), 50);
        assert_eq!(restored.weights_version(), 2);
        assert_eq!(restored.tempo(), 400);
        assert!(restored.liquid_alpha_enabled());
    }

    // -----------------------------------------------------------------------
    // StakeInfo full getter tests
    // -----------------------------------------------------------------------

    #[test]
    fn stake_info_all_getters() {
        let data = StakeInfoData {
            hotkey: "hk_stake".into(),
            coldkey: "ck_stake".into(),
            stake: 7_500_000_000,
        };
        let info = StakeInfo { inner: data };
        let json = info.to_json().unwrap();
        let restored = StakeInfo::from_json(&json).unwrap();
        assert_eq!(restored.hotkey(), "hk_stake");
        assert_eq!(restored.coldkey(), "ck_stake");
        assert_eq!(restored.stake_rao(), 7_500_000_000);
        let diff = (restored.stake_tao() - 7.5).abs();
        assert!(diff < 1e-10);
    }

    // -----------------------------------------------------------------------
    // DelegateInfo full getter tests
    // -----------------------------------------------------------------------

    #[test]
    fn delegate_info_all_getters() {
        let data = DelegateInfoData {
            delegate_ss58: "5Delegate".into(),
            delegate_hotkey: "del_hk".into(),
            total_stake: 2_000_000_000,
            nominators: vec![("nom_a".into(), 1_000_000_000), ("nom_b".into(), 500_000_000)],
            owner_hotkey: "own_hk".into(),
            take: 15,
            owner_ss58: "5Owner".into(),
            registrations: vec![1, 2, 3],
            validator_permits: vec![1, 2],
        };
        let info = DelegateInfo { inner: data };
        let json = info.to_json().unwrap();
        let restored = DelegateInfo::from_json(&json).unwrap();
        assert_eq!(restored.delegate_ss58(), "5Delegate");
        assert_eq!(restored.delegate_hotkey(), "del_hk");
        assert_eq!(restored.total_stake_rao(), 2_000_000_000);
        let diff = (restored.total_stake_tao() - 2.0).abs();
        assert!(diff < 1e-10);
        assert_eq!(restored.owner_hotkey(), "own_hk");
        assert_eq!(restored.take(), 15);
        assert_eq!(restored.owner_ss58(), "5Owner");
        assert_eq!(restored.nominator_count(), 2);
    }

    // -----------------------------------------------------------------------
    // TerminalInfo full tests
    // -----------------------------------------------------------------------

    #[test]
    fn terminal_info_json_roundtrip() {
        let inner = bittensor_synapse::TerminalInfo {
            status_code: Some(200),
            status_message: Some("OK".into()),
            process_time: Some(0.42),
            ip: Some("127.0.0.1".into()),
            port: Some(8080),
            version: Some(5),
            nonce: Some(12345),
            uuid: Some("uuid-1234".into()),
            hotkey: Some("5Hotkey".into()),
            signature: Some("0xsig".into()),
        };
        let info = TerminalInfo { inner };
        let json = info.to_json().unwrap();
        let restored = TerminalInfo::from_json(&json).unwrap();
        assert_eq!(restored.status_code(), Some(200));
        assert_eq!(restored.status_message(), Some("OK".to_string()));
        assert_eq!(restored.process_time(), Some(0.42));
        assert_eq!(restored.ip(), Some("127.0.0.1".to_string()));
        assert_eq!(restored.port(), Some(8080));
        assert_eq!(restored.version(), Some(5));
        assert_eq!(restored.nonce(), Some(12345));
        assert_eq!(restored.uuid(), Some("uuid-1234".to_string()));
        assert_eq!(restored.hotkey(), Some("5Hotkey".to_string()));
        assert_eq!(restored.signature(), Some("0xsig".to_string()));
    }

    #[test]
    fn terminal_info_default_all_none() {
        let info = TerminalInfo::default();
        assert!(info.status_code().is_none());
        assert!(info.status_message().is_none());
        assert!(info.process_time().is_none());
        assert!(info.ip().is_none());
        assert!(info.port().is_none());
        assert!(info.version().is_none());
        assert!(info.nonce().is_none());
        assert!(info.uuid().is_none());
        assert!(info.hotkey().is_none());
        assert!(info.signature().is_none());
    }

    #[test]
    fn terminal_info_json_roundtrip_empty() {
        let info = TerminalInfo::new();
        let json = info.to_json().unwrap();
        let restored = TerminalInfo::from_json(&json).unwrap();
        assert!(restored.status_code().is_none());
        assert!(restored.hotkey().is_none());
    }

    // -----------------------------------------------------------------------
    // RegistrationInfo additional tests
    // -----------------------------------------------------------------------

    #[test]
    fn registration_info_all_getters() {
        let data = RegistrationInfoData {
            netuid: 3,
            hotkey: "hk_reg".into(),
            block: 500,
            burn: 2_500_000_000,
        };
        let info = RegistrationInfo { inner: data };
        let json = info.to_json().unwrap();
        let restored = RegistrationInfo::from_json(&json).unwrap();
        assert_eq!(restored.netuid(), 3);
        assert_eq!(restored.hotkey(), "hk_reg");
        assert_eq!(restored.block(), 500);
        assert_eq!(restored.burn_rao(), 2_500_000_000);
        let diff = (restored.burn_tao() - 2.5).abs();
        assert!(diff < 1e-10);
    }

    // -----------------------------------------------------------------------
    // NetworkConfig additional tests
    // -----------------------------------------------------------------------

    #[test]
    fn network_config_chain_id() {
        let cfg = NetworkConfig::finney();
        assert_eq!(cfg.chain_id(), 42);
        let cfg = NetworkConfig::test();
        assert_eq!(cfg.chain_id(), 42);
        let cfg = NetworkConfig::local();
        assert_eq!(cfg.chain_id(), 42);
    }

    #[test]
    fn network_config_ws_urls() {
        let finney = NetworkConfig::finney();
        assert!(finney.ws_url().starts_with("wss://"));
        let local = NetworkConfig::local();
        assert!(local.ws_url().starts_with("ws://"));
    }
}
