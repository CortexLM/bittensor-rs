//! Python bindings for bittensor-synapse: Synapse, TerminalInfo, StreamingSynapse.

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyType;

use bittensor_synapse::TerminalInfo as RustTerminalInfo;
use bittensor_synapse::header::keys;
use bittensor_synapse::sha3_256_hex;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// TerminalInfo
// ---------------------------------------------------------------------------

/// Terminal information for synapse endpoints — mirrors Python's dataclass behavior.
///
/// All fields are optional and mutable (get/set), matching Python's mutable dataclass pattern.
#[pyclass]
#[derive(Clone)]
pub struct TerminalInfo {
    pub(crate) inner: RustTerminalInfo,
}

#[pymethods]
impl TerminalInfo {
    #[new]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (
        status_code=None,
        status_message=None,
        process_time=None,
        ip=None,
        port=None,
        version=None,
        nonce=None,
        uuid=None,
        hotkey=None,
        signature=None,
    ))]
    fn new(
        status_code: Option<u16>,
        status_message: Option<String>,
        process_time: Option<f64>,
        ip: Option<String>,
        port: Option<u16>,
        version: Option<u32>,
        nonce: Option<u64>,
        uuid: Option<String>,
        hotkey: Option<String>,
        signature: Option<String>,
    ) -> Self {
        Self {
            inner: RustTerminalInfo {
                status_code,
                status_message,
                process_time,
                ip,
                port,
                version,
                nonce,
                uuid,
                hotkey,
                signature,
            },
        }
    }

    // --- Getters/Setters for all fields ---

    #[getter]
    fn status_code(&self) -> Option<u16> {
        self.inner.status_code
    }

    #[setter]
    fn set_status_code(&mut self, val: Option<u16>) {
        self.inner.status_code = val;
    }

    #[getter]
    fn status_message(&self) -> Option<&str> {
        self.inner.status_message.as_deref()
    }

    #[setter]
    fn set_status_message(&mut self, val: Option<String>) {
        self.inner.status_message = val;
    }

    #[getter]
    fn process_time(&self) -> Option<f64> {
        self.inner.process_time
    }

    #[setter]
    fn set_process_time(&mut self, val: Option<f64>) {
        self.inner.process_time = val;
    }

    #[getter]
    fn ip(&self) -> Option<&str> {
        self.inner.ip.as_deref()
    }

    #[setter]
    fn set_ip(&mut self, val: Option<String>) {
        self.inner.ip = val;
    }

    #[getter]
    fn port(&self) -> Option<u16> {
        self.inner.port
    }

    #[setter]
    fn set_port(&mut self, val: Option<u16>) {
        self.inner.port = val;
    }

    #[getter]
    fn version(&self) -> Option<u32> {
        self.inner.version
    }

    #[setter]
    fn set_version(&mut self, val: Option<u32>) {
        self.inner.version = val;
    }

    #[getter]
    fn nonce(&self) -> Option<u64> {
        self.inner.nonce
    }

    #[setter]
    fn set_nonce(&mut self, val: Option<u64>) {
        self.inner.nonce = val;
    }

    #[getter]
    fn uuid(&self) -> Option<&str> {
        self.inner.uuid.as_deref()
    }

    #[setter]
    fn set_uuid(&mut self, val: Option<String>) {
        self.inner.uuid = val;
    }

    #[getter]
    fn hotkey(&self) -> Option<&str> {
        self.inner.hotkey.as_deref()
    }

    #[setter]
    fn set_hotkey(&mut self, val: Option<String>) {
        self.inner.hotkey = val;
    }

    #[getter]
    fn signature(&self) -> Option<&str> {
        self.inner.signature.as_deref()
    }

    #[setter]
    fn set_signature(&mut self, val: Option<String>) {
        self.inner.signature = val;
    }

    /// Serialize non-None fields into a header map using the given prefix.
    ///
    /// Prefix is typically "bt_header_axon_" or "bt_header_dendrite_".
    fn to_headers(&self, prefix: &str) -> HashMap<String, String> {
        self.inner.to_headers_with_prefix(prefix)
    }

    /// Deserialize from a header map with the given prefix.
    #[classmethod]
    fn from_headers(
        _cls: &Bound<'_, PyType>,
        headers: HashMap<String, String>,
        prefix: &str,
    ) -> Self {
        Self { inner: RustTerminalInfo::from_headers_with_prefix(&headers, prefix) }
    }

    fn __repr__(&self) -> String {
        format!(
            "TerminalInfo(status_code={:?}, status_message={:?}, nonce={:?}, hotkey={:?})",
            self.inner.status_code, self.inner.status_message, self.inner.nonce, self.inner.hotkey,
        )
    }
}

// ---------------------------------------------------------------------------
// Synapse (Python base class pattern)
// ---------------------------------------------------------------------------

/// Base Synapse class for Bittensor protocol serialization.
///
/// Python users subclass `bittensor_rs.Synapse` and override methods as needed.
/// The base class provides default `to_headers()`, `from_headers()`, and `body_hash()` methods.
#[pyclass(subclass)]
#[derive(Clone)]
pub struct Synapse {
    name_val: String,
    timeout_val: f64,
    dendrite_info: TerminalInfo,
    axon_info: TerminalInfo,
    computed_body_hash_val: String,
    total_size_val: u64,
    header_size_val: u64,
}

#[pymethods]
impl Synapse {
    #[new]
    #[pyo3(signature = (name="Synapse", timeout=12.0))]
    fn new(name: &str, timeout: f64) -> Self {
        Self {
            name_val: name.to_string(),
            timeout_val: timeout,
            dendrite_info: TerminalInfo::new(
                None, None, None, None, None, None, None, None, None, None,
            ),
            axon_info: TerminalInfo::new(
                None, None, None, None, None, None, None, None, None, None,
            ),
            computed_body_hash_val: String::new(),
            total_size_val: 0,
            header_size_val: 0,
        }
    }

    /// The name/route of this synapse type.
    #[getter]
    fn name(&self) -> &str {
        &self.name_val
    }

    #[setter]
    fn set_name(&mut self, val: String) {
        self.name_val = val;
    }

    /// The query timeout in seconds.
    #[getter]
    fn timeout(&self) -> f64 {
        self.timeout_val
    }

    #[setter]
    fn set_timeout(&mut self, val: f64) {
        self.timeout_val = val;
    }

    /// Dendrite (requesting) terminal information.
    #[getter]
    fn dendrite(&self) -> TerminalInfo {
        self.dendrite_info.clone()
    }

    #[setter]
    fn set_dendrite(&mut self, val: TerminalInfo) {
        self.dendrite_info = val;
    }

    /// Axon (responding) terminal information.
    #[getter]
    fn axon(&self) -> TerminalInfo {
        self.axon_info.clone()
    }

    #[setter]
    fn set_axon(&mut self, val: TerminalInfo) {
        self.axon_info = val;
    }

    /// The computed body hash string.
    #[getter]
    fn computed_body_hash(&self) -> &str {
        &self.computed_body_hash_val
    }

    #[setter]
    fn set_computed_body_hash(&mut self, val: String) {
        self.computed_body_hash_val = val;
    }

    /// The total size of the request body in bytes.
    #[getter]
    fn total_size(&self) -> u64 {
        self.total_size_val
    }

    #[setter]
    fn set_total_size(&mut self, val: u64) {
        self.total_size_val = val;
    }

    /// The size of the request header in bytes.
    #[getter]
    fn header_size(&self) -> u64 {
        self.header_size_val
    }

    #[setter]
    fn set_header_size(&mut self, val: u64) {
        self.header_size_val = val;
    }

    /// Compute the SHA3-256 body hash from the given body bytes.
    ///
    /// Args:
    ///     body: bytes or str to hash.
    ///
    /// Returns:
    ///     Hex-encoded SHA3-256 digest.
    #[staticmethod]
    fn body_hash(body: &[u8]) -> String {
        sha3_256_hex(body)
    }

    /// Serialize this synapse into a header map for HTTP transmission.
    ///
    /// Matches the Python SDK's `to_headers()` method exactly.
    fn to_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert(keys::NAME.to_string(), self.name_val.clone());
        headers.insert(keys::TIMEOUT.to_string(), self.timeout_val.to_string());

        let axon_headers = self.axon_info.inner.to_headers_with_prefix(keys::AXON_PREFIX);
        headers.extend(axon_headers);

        let dendrite_headers =
            self.dendrite_info.inner.to_headers_with_prefix(keys::DENDRITE_PREFIX);
        headers.extend(dendrite_headers);

        let header_size = headers.keys().map(|k| k.len()).sum::<usize>()
            + headers.values().map(|v| v.len()).sum::<usize>();
        headers.insert(keys::HEADER_SIZE.to_string(), header_size.to_string());
        headers.insert(keys::TOTAL_SIZE.to_string(), self.total_size_val.to_string());
        headers.insert(keys::COMPUTED_BODY_HASH.to_string(), self.computed_body_hash_val.clone());

        headers
    }

    /// Deserialize a Synapse from a header map.
    ///
    /// Matches the Python SDK's `from_headers()` classmethod.
    #[classmethod]
    fn from_headers(_cls: &Bound<'_, PyType>, headers: HashMap<String, String>) -> PyResult<Self> {
        let name = headers.get(keys::NAME).cloned().unwrap_or_else(|| "Synapse".to_string());
        let timeout =
            headers.get(keys::TIMEOUT).and_then(|v| v.parse::<f64>().ok()).unwrap_or(12.0);
        let total_size =
            headers.get(keys::TOTAL_SIZE).and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
        let computed_body_hash = headers.get(keys::COMPUTED_BODY_HASH).cloned().unwrap_or_default();

        let axon_info = TerminalInfo {
            inner: RustTerminalInfo::from_headers_with_prefix(&headers, keys::AXON_PREFIX),
        };
        let dendrite_info = TerminalInfo {
            inner: RustTerminalInfo::from_headers_with_prefix(&headers, keys::DENDRITE_PREFIX),
        };

        Ok(Self {
            name_val: name,
            timeout_val: timeout,
            dendrite_info,
            axon_info,
            computed_body_hash_val: computed_body_hash,
            total_size_val: total_size,
            header_size_val: 0,
        })
    }

    fn __repr__(&self) -> String {
        format!("Synapse(name='{}', timeout={})", self.name_val, self.timeout_val)
    }
}

// ---------------------------------------------------------------------------
// StreamingSynapse
// ---------------------------------------------------------------------------

/// Streaming synapse for Server-Sent Events (SSE) handling.
///
/// Python users subclass `bittensor_rs.StreamingSynapse` and override
/// `process_chunk(chunk: bytes) -> str` to define how SSE data chunks are parsed.
#[pyclass(subclass)]
#[derive(Clone)]
pub struct StreamingSynapse {
    inner: Synapse,
}

#[pymethods]
impl StreamingSynapse {
    #[new]
    #[pyo3(signature = (name="StreamingSynapse", timeout=12.0))]
    fn new(name: &str, timeout: f64) -> Self {
        Self { inner: Synapse::new(name, timeout) }
    }

    /// The name/route of this streaming synapse.
    #[getter]
    fn name(&self) -> &str {
        self.inner.name()
    }

    #[setter]
    fn set_name(&mut self, val: String) {
        self.inner.set_name(val);
    }

    /// The query timeout in seconds.
    #[getter]
    fn timeout(&self) -> f64 {
        self.inner.timeout()
    }

    #[setter]
    fn set_timeout(&mut self, val: f64) {
        self.inner.set_timeout(val);
    }

    /// Dendrite (requesting) terminal information.
    #[getter]
    fn dendrite(&self) -> TerminalInfo {
        self.inner.dendrite()
    }

    #[setter]
    fn set_dendrite(&mut self, val: TerminalInfo) {
        self.inner.set_dendrite(val);
    }

    /// Axon (responding) terminal information.
    #[getter]
    fn axon(&self) -> TerminalInfo {
        self.inner.axon()
    }

    #[setter]
    fn set_axon(&mut self, val: TerminalInfo) {
        self.inner.set_axon(val);
    }

    /// Process a single SSE data chunk. Override in Python subclass.
    ///
    /// Default implementation returns the chunk as a UTF-8 string.
    fn process_chunk(&self, chunk: &[u8]) -> PyResult<String> {
        String::from_utf8(chunk.to_vec())
            .map_err(|e| PyRuntimeError::new_err(format!("invalid UTF-8 in chunk: {e}")))
    }

    /// Serialize to headers (delegates to inner Synapse).
    fn to_headers(&self) -> HashMap<String, String> {
        self.inner.to_headers()
    }

    /// Deserialize from headers (delegates to inner Synapse).
    #[classmethod]
    fn from_headers(_cls: &Bound<'_, PyType>, headers: HashMap<String, String>) -> PyResult<Self> {
        let syn = Synapse::from_headers(_cls, headers)?;
        Ok(Self { inner: syn })
    }

    fn __repr__(&self) -> String {
        format!("StreamingSynapse(name='{}', timeout={})", self.inner.name(), self.inner.timeout())
    }
}
