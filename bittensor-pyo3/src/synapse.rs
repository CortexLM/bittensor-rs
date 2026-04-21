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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_info_new_defaults() {
        let ti = TerminalInfo::new(None, None, None, None, None, None, None, None, None, None);
        assert_eq!(ti.status_code(), None);
        assert_eq!(ti.status_message(), None);
        assert_eq!(ti.process_time(), None);
        assert_eq!(ti.ip(), None);
        assert_eq!(ti.port(), None);
        assert_eq!(ti.version(), None);
        assert_eq!(ti.nonce(), None);
        assert_eq!(ti.uuid(), None);
        assert_eq!(ti.hotkey(), None);
        assert_eq!(ti.signature(), None);
    }

    #[test]
    fn terminal_info_new_with_values() {
        let ti = TerminalInfo::new(
            Some(200),
            Some("OK".to_string()),
            Some(0.5),
            Some("127.0.0.1".to_string()),
            Some(8090),
            Some(4),
            Some(12345),
            Some("uuid-1".to_string()),
            Some("hk".to_string()),
            Some("sig".to_string()),
        );
        assert_eq!(ti.status_code(), Some(200));
        assert_eq!(ti.status_message(), Some("OK"));
        assert_eq!(ti.process_time(), Some(0.5));
        assert_eq!(ti.ip(), Some("127.0.0.1"));
        assert_eq!(ti.port(), Some(8090));
        assert_eq!(ti.version(), Some(4));
        assert_eq!(ti.nonce(), Some(12345));
        assert_eq!(ti.uuid(), Some("uuid-1"));
        assert_eq!(ti.hotkey(), Some("hk"));
        assert_eq!(ti.signature(), Some("sig"));
    }

    #[test]
    fn terminal_info_setters() {
        let mut ti = TerminalInfo::new(None, None, None, None, None, None, None, None, None, None);
        ti.set_status_code(Some(404));
        ti.set_status_message(Some("Not Found".to_string()));
        ti.set_process_time(Some(1.0));
        ti.set_ip(Some("10.0.0.1".to_string()));
        ti.set_port(Some(443));
        ti.set_version(Some(2));
        ti.set_nonce(Some(999));
        ti.set_uuid(Some("u2".to_string()));
        ti.set_hotkey(Some("hk2".to_string()));
        ti.set_signature(Some("sig2".to_string()));
        assert_eq!(ti.status_code(), Some(404));
        assert_eq!(ti.status_message(), Some("Not Found"));
        assert_eq!(ti.process_time(), Some(1.0));
        assert_eq!(ti.ip(), Some("10.0.0.1"));
        assert_eq!(ti.port(), Some(443));
        assert_eq!(ti.version(), Some(2));
        assert_eq!(ti.nonce(), Some(999));
        assert_eq!(ti.uuid(), Some("u2"));
        assert_eq!(ti.hotkey(), Some("hk2"));
        assert_eq!(ti.signature(), Some("sig2"));
    }

    #[test]
    fn terminal_info_to_headers() {
        let ti = TerminalInfo::new(
            Some(200),
            Some("OK".to_string()),
            None,
            None,
            None,
            None,
            Some(42),
            None,
            Some("hk".to_string()),
            None,
        );
        let headers = ti.to_headers("bt_header_axon_");
        assert_eq!(headers.get("bt_header_axon_status_code").unwrap(), "200");
        assert_eq!(headers.get("bt_header_axon_status_message").unwrap(), "OK");
        assert_eq!(headers.get("bt_header_axon_nonce").unwrap(), "42");
        assert_eq!(headers.get("bt_header_axon_hotkey").unwrap(), "hk");
        assert!(!headers.contains_key("bt_header_axon_process_time"));
    }

    #[test]
    fn terminal_info_from_headers_manual() {
        let mut headers = HashMap::new();
        headers.insert("bt_header_dendrite_status_code".to_string(), "200".to_string());
        headers.insert("bt_header_dendrite_nonce".to_string(), "77".to_string());
        let inner = RustTerminalInfo::from_headers_with_prefix(&headers, "bt_header_dendrite_");
        let ti = TerminalInfo { inner };
        assert_eq!(ti.status_code(), Some(200));
        assert_eq!(ti.nonce(), Some(77));
    }

    #[test]
    fn terminal_info_repr() {
        let ti = TerminalInfo::new(None, None, None, None, None, None, None, None, None, None);
        let repr = ti.__repr__();
        assert!(repr.contains("TerminalInfo"));
    }

    #[test]
    fn terminal_info_clone() {
        let ti = TerminalInfo::new(
            Some(200),
            Some("OK".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        let ti2 = ti.clone();
        assert_eq!(ti.status_code(), ti2.status_code());
        assert_eq!(ti.status_message(), ti2.status_message());
    }

    #[test]
    fn synapse_new_defaults() {
        let s = Synapse::new("Synapse", 12.0);
        assert_eq!(s.name(), "Synapse");
        assert_eq!(s.timeout(), 12.0);
        assert_eq!(s.computed_body_hash(), "");
        assert_eq!(s.total_size(), 0);
        assert_eq!(s.header_size(), 0);
    }

    #[test]
    fn synapse_new_custom() {
        let s = Synapse::new("TextPrompt", 30.0);
        assert_eq!(s.name(), "TextPrompt");
        assert_eq!(s.timeout(), 30.0);
    }

    #[test]
    fn synapse_setters() {
        let mut s = Synapse::new("X", 5.0);
        s.set_name("Y".to_string());
        s.set_timeout(60.0);
        s.set_computed_body_hash("hash123".to_string());
        s.set_total_size(1024);
        s.set_header_size(256);
        assert_eq!(s.name(), "Y");
        assert_eq!(s.timeout(), 60.0);
        assert_eq!(s.computed_body_hash(), "hash123");
        assert_eq!(s.total_size(), 1024);
        assert_eq!(s.header_size(), 256);
    }

    #[test]
    fn synapse_body_hash() {
        let hash = Synapse::body_hash(b"hello world");
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn synapse_body_hash_empty() {
        let hash = Synapse::body_hash(b"");
        assert!(!hash.is_empty());
    }

    #[test]
    fn synapse_body_hash_deterministic() {
        let h1 = Synapse::body_hash(b"test data");
        let h2 = Synapse::body_hash(b"test data");
        assert_eq!(h1, h2);
    }

    #[test]
    fn synapse_to_headers_contains_name() {
        let s = Synapse::new("TestSyn", 12.0);
        let headers = s.to_headers();
        assert_eq!(headers.get(keys::NAME).unwrap(), "TestSyn");
        assert!(headers.contains_key(keys::TIMEOUT));
    }

    #[test]
    fn synapse_repr() {
        let s = Synapse::new("TestSyn", 5.0);
        let repr = s.__repr__();
        assert!(repr.contains("Synapse"));
        assert!(repr.contains("TestSyn"));
    }

    #[test]
    fn synapse_clone() {
        let s = Synapse::new("X", 10.0);
        let s2 = s.clone();
        assert_eq!(s.name(), s2.name());
        assert_eq!(s.timeout(), s2.timeout());
    }

    #[test]
    fn streaming_synapse_new_defaults() {
        let ss = StreamingSynapse::new("StreamingSynapse", 12.0);
        assert_eq!(ss.name(), "StreamingSynapse");
        assert_eq!(ss.timeout(), 12.0);
    }

    #[test]
    fn streaming_synapse_new_custom() {
        let ss = StreamingSynapse::new("StreamTest", 60.0);
        assert_eq!(ss.name(), "StreamTest");
        assert_eq!(ss.timeout(), 60.0);
    }

    #[test]
    fn streaming_synapse_setters() {
        let mut ss = StreamingSynapse::new("X", 5.0);
        ss.set_name("Y".to_string());
        ss.set_timeout(30.0);
        assert_eq!(ss.name(), "Y");
        assert_eq!(ss.timeout(), 30.0);
    }

    #[test]
    fn streaming_synapse_process_chunk_valid_utf8() {
        let ss = StreamingSynapse::new("S", 12.0);
        let result = ss.process_chunk(b"hello");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn streaming_synapse_process_chunk_invalid_utf8() {
        let ss = StreamingSynapse::new("S", 12.0);
        let result = ss.process_chunk(&[0xff, 0xfe, 0xfd]);
        assert!(result.is_err());
    }

    #[test]
    fn streaming_synapse_to_headers() {
        let ss = StreamingSynapse::new("StreamTest", 12.0);
        let headers = ss.to_headers();
        assert_eq!(headers.get(keys::NAME).unwrap(), "StreamTest");
    }

    #[test]
    fn streaming_synapse_repr() {
        let ss = StreamingSynapse::new("S", 12.0);
        let repr = ss.__repr__();
        assert!(repr.contains("StreamingSynapse"));
        assert!(repr.contains("S"));
    }

    #[test]
    fn streaming_synapse_clone() {
        let ss = StreamingSynapse::new("S", 12.0);
        let ss2 = ss.clone();
        assert_eq!(ss.name(), ss2.name());
    }
}
