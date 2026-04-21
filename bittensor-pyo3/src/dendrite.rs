//! Python bindings for bittensor-dendrite: Dendrite HTTP client with signing + streaming.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use bittensor_core::types::AxonInfo as RustAxonInfo;
use bittensor_dendrite::signing::sign_request;
use bittensor_synapse::TerminalInfo as RustTerminalInfo;
use bittensor_synapse::header::keys;
use bittensor_synapse::sha3_256_hex;
use subxt_signer::sr25519::Keypair;

use crate::core_types::{AxonInfo, BittensorError};

use std::sync::Arc;

// ---------------------------------------------------------------------------
// DendriteConfig
// ---------------------------------------------------------------------------

/// Configuration for constructing a Dendrite HTTP client.
#[pyclass]
#[derive(Clone)]
pub struct DendriteConfig {
    timeout_secs: u64,
    max_connections: usize,
    hotkey_seed: Option<String>,
}

#[pymethods]
impl DendriteConfig {
    #[new]
    #[pyo3(signature = (timeout_secs=12, max_connections=100, hotkey_seed=None))]
    fn new(timeout_secs: u64, max_connections: usize, hotkey_seed: Option<String>) -> Self {
        Self { timeout_secs, max_connections, hotkey_seed }
    }

    #[getter]
    fn timeout_secs(&self) -> u64 {
        self.timeout_secs
    }

    #[setter]
    fn set_timeout_secs(&mut self, val: u64) {
        self.timeout_secs = val;
    }

    #[getter]
    fn max_connections(&self) -> usize {
        self.max_connections
    }

    #[setter]
    fn set_max_connections(&mut self, val: usize) {
        self.max_connections = val;
    }

    /// Hex-encoded 32-byte secret key (with or without 0x prefix).
    #[getter]
    fn hotkey_seed(&self) -> Option<&str> {
        self.hotkey_seed.as_deref()
    }

    #[setter]
    fn set_hotkey_seed(&mut self, val: Option<String>) {
        self.hotkey_seed = val;
    }

    fn __repr__(&self) -> String {
        format!(
            "DendriteConfig(timeout_secs={}, max_connections={}, hotkey_seed={:?})",
            self.timeout_secs, self.max_connections, self.hotkey_seed
        )
    }
}

// ---------------------------------------------------------------------------
// Key parsing helper
// ---------------------------------------------------------------------------

/// Parse a hex seed string (with or without 0x prefix) into a Keypair.
fn parse_keypair(seed: &str) -> PyResult<Keypair> {
    let stripped = seed.strip_prefix("0x").unwrap_or(seed);
    let bytes: Vec<u8> = hex::decode(stripped)
        .map_err(|e| PyValueError::new_err(format!("invalid hex in hotkey_seed: {e}")))?;
    let secret_bytes: [u8; 32] = bytes.try_into().map_err(|_| {
        PyValueError::new_err("hotkey_seed must be exactly 32 bytes (64 hex chars)")
    })?;
    Keypair::from_secret_key(secret_bytes)
        .map_err(|e| PyValueError::new_err(format!("invalid secret key: {e}")))
}

// ---------------------------------------------------------------------------
// Dendrite
// ---------------------------------------------------------------------------

/// Dendrite — the HTTP client for sending signed Synapse requests to Axons.
///
/// Usage:
///     dendrite = bt.Dendrite(config)
///     synapse = bt.Synapse(name="TextPrompt")
///     result = await dendrite.query(synapse, axon_info)
#[pyclass]
pub struct Dendrite {
    config: DendriteConfig,
}

#[pymethods]
impl Dendrite {
    #[new]
    #[pyo3(signature = (config=None))]
    fn new(config: Option<DendriteConfig>) -> Self {
        Self {
            config: config.unwrap_or(DendriteConfig {
                timeout_secs: 12,
                max_connections: 100,
                hotkey_seed: None,
            }),
        }
    }

    /// Send a signed synapse query to an axon.
    ///
    /// Args:
    ///     synapse: A Python Synapse object (or subclass).
    ///     axon_info: An AxonInfo object specifying the target.
    ///
    /// Returns:
    ///     The synapse object with response metadata populated.
    fn query(&self, py: Python<'_>, synapse: PyObject, axon_info: &AxonInfo) -> PyResult<PyObject> {
        let timeout_secs = self.config.timeout_secs;
        let max_connections = self.config.max_connections;
        let hotkey_seed = self.config.hotkey_seed.clone();
        let rust_axon_info = axon_info.inner.clone();

        let coro = pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(timeout_secs))
                .pool_max_idle_per_host(max_connections)
                .build()
                .map_err(|e| BittensorError::new_err(format!("client build failed: {e}")))?;

            // Extract synapse name from Python object
            let synapse_name: String = Python::with_gil(|py| {
                let syn_obj = synapse.bind(py);
                syn_obj
                    .getattr("name")
                    .and_then(|n| n.extract())
                    .unwrap_or_else(|_| "Synapse".to_string())
            });

            let body = serde_json::json!({ "name": synapse_name });
            let body_bytes = serde_json::to_vec(&body)
                .map_err(|e| BittensorError::new_err(format!("serialize failed: {e}")))?;

            let body_hash = sha3_256_hex(&body_bytes);
            let url = build_axon_url(&rust_axon_info);

            // Sign if hotkey available
            let keypair =
                if let Some(ref seed) = hotkey_seed { Some(parse_keypair(seed)?) } else { None };

            let mut request = client.post(&url);

            if let Some(ref kp) = keypair {
                let nonce = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                let signed = sign_request(kp, &rust_axon_info.hotkey, &body_bytes, nonce)
                    .map_err(|e| BittensorError::new_err(e.to_string()))?;

                let mut all_headers = signed.headers;
                if let Ok(name) =
                    reqwest::header::HeaderName::from_bytes(keys::COMPUTED_BODY_HASH.as_bytes())
                {
                    if let Ok(val) = reqwest::header::HeaderValue::from_str(&body_hash) {
                        all_headers.insert(name, val);
                    }
                }
                request = request.headers(all_headers);
            } else {
                request = request.header("accept", "application/json");
                request = request.header(keys::COMPUTED_BODY_HASH, &body_hash);
            }

            let response = request.body(body_bytes).send().await.map_err(|e| {
                if e.is_timeout() {
                    BittensorError::new_err(format!("timeout: {e}"))
                } else {
                    BittensorError::new_err(format!("network: {e}"))
                }
            })?;

            let status = response.status();
            if status.as_u16() == 401 {
                return Err(BittensorError::new_err("received 401 Unauthorized from axon"));
            }
            if !status.is_success() {
                return Err(BittensorError::new_err(format!("HTTP {} from axon", status)));
            }

            // Parse response headers into TerminalInfo
            let resp_headers: std::collections::HashMap<String, String> = response
                .headers()
                .iter()
                .map(|(k, v): (&reqwest::header::HeaderName, &reqwest::header::HeaderValue)| {
                    (k.to_string(), v.to_str().unwrap_or("").to_string())
                })
                .collect();

            let axon_resp =
                RustTerminalInfo::from_headers_with_prefix(&resp_headers, keys::AXON_PREFIX);

            // Update the Python synapse with response metadata
            Python::with_gil(|py| {
                let syn_obj = synapse.bind(py);

                let axon_ti = crate::synapse::TerminalInfo { inner: axon_resp };
                syn_obj.setattr("axon", axon_ti)?;
                syn_obj.setattr("computed_body_hash", body_hash)?;

                Ok::<PyObject, PyErr>(syn_obj.clone().unbind())
            })
        })?;

        Ok(coro.into_any().unbind())
    }

    /// Send a signed synapse request (alias for query).
    #[pyo3(signature = (synapse, axon_info))]
    fn forward(
        &self,
        py: Python<'_>,
        synapse: PyObject,
        axon_info: &AxonInfo,
    ) -> PyResult<PyObject> {
        self.query(py, synapse, axon_info)
    }

    /// Send a signed synapse request (alias for query).
    #[pyo3(signature = (synapse, axon_info))]
    fn call(&self, py: Python<'_>, synapse: PyObject, axon_info: &AxonInfo) -> PyResult<PyObject> {
        self.query(py, synapse, axon_info)
    }

    /// Send a signed synapse request and return a stream of response chunks.
    ///
    /// Returns an async generator yielding str chunks from the SSE stream.
    fn call_stream(
        &self,
        py: Python<'_>,
        synapse: PyObject,
        axon_info: &AxonInfo,
    ) -> PyResult<PyObject> {
        let timeout_secs = self.config.timeout_secs;
        let max_connections = self.config.max_connections;
        let hotkey_seed = self.config.hotkey_seed.clone();
        let rust_axon_info = axon_info.inner.clone();

        let coro = pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(timeout_secs))
                .pool_max_idle_per_host(max_connections)
                .build()
                .map_err(|e| BittensorError::new_err(format!("client build failed: {e}")))?;

            let url = build_axon_url(&rust_axon_info);

            let synapse_name: String = Python::with_gil(|py| {
                let syn_obj = synapse.bind(py);
                syn_obj
                    .getattr("name")
                    .and_then(|n| n.extract())
                    .unwrap_or_else(|_| "Synapse".to_string())
            });

            let body = serde_json::json!({"name": synapse_name});
            let body_bytes = serde_json::to_vec(&body)
                .map_err(|e| BittensorError::new_err(format!("serialize failed: {e}")))?;

            // Sign if hotkey available — compute signature before moving body_bytes
            let signed_opt = if let Some(ref seed) = hotkey_seed {
                let kp = parse_keypair(seed)?;
                let nonce = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                Some(
                    sign_request(&kp, &rust_axon_info.hotkey, &body_bytes, nonce)
                        .map_err(|e| BittensorError::new_err(e.to_string()))?,
                )
            } else {
                None
            };

            let mut request =
                client.post(&url).header("accept", "text/event-stream").body(body_bytes);

            if let Some(signed) = signed_opt {
                request = request.headers(signed.headers);
            }

            let response = request
                .send()
                .await
                .map_err(|e| BittensorError::new_err(format!("network: {e}")))?;

            let status = response.status();
            if status.as_u16() == 401 {
                return Err(BittensorError::new_err("received 401 Unauthorized from axon"));
            }
            if !status.is_success() {
                return Err(BittensorError::new_err(format!("HTTP {} from axon", status)));
            }

            // Create a channel to stream chunks to Python
            let (tx, rx) = tokio::sync::mpsc::channel::<Option<String>>(64);

            // Spawn task that reads SSE stream and sends chunks
            tokio::spawn(async move {
                use futures::StreamExt;
                let mut stream = response.bytes_stream();
                let mut buffer = Vec::new();

                while let Some(chunk_result) = stream.next().await {
                    match chunk_result {
                        Ok(chunk) => {
                            buffer.extend_from_slice(&chunk);
                            // Parse SSE events from buffer
                            while let Some(pos) = buffer.windows(2).position(|w| w == b"\n\n") {
                                let event_data = buffer[..pos].to_vec();
                                buffer.drain(..pos + 2);

                                for line in event_data.split(|&b| b == b'\n') {
                                    if let Some(data) = line.strip_prefix(b"data: ") {
                                        if data == b"[DONE]" {
                                            continue;
                                        }
                                        if let Ok(s) = String::from_utf8(data.to_vec()) {
                                            if tx.send(Some(s)).await.is_err() {
                                                return; // Receiver dropped
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Err(_) => break,
                    }
                }

                // Flush remaining buffer
                if !buffer.is_empty() {
                    for line in buffer.split(|&b| b == b'\n') {
                        if let Some(data) = line.strip_prefix(b"data: ") {
                            if data != b"[DONE]" {
                                if let Ok(s) = String::from_utf8(data.to_vec()) {
                                    let _ = tx.send(Some(s)).await;
                                }
                            }
                        }
                    }
                }

                // Signal end of stream
                let _ = tx.send(None).await;
            });

            // Return an async iterator that yields chunks from the channel
            let py_iter = PyStreamIterator { rx: Arc::new(tokio::sync::Mutex::new(Some(rx))) };
            Ok(py_iter)
        })?;

        Ok(coro.into_any().unbind())
    }

    fn __repr__(&self) -> String {
        format!(
            "Dendrite(timeout_secs={}, max_connections={})",
            self.config.timeout_secs, self.config.max_connections
        )
    }
}

// ---------------------------------------------------------------------------
// Async stream iterator for call_stream
// ---------------------------------------------------------------------------

/// Python async iterator that yields str chunks from a tokio mpsc channel.
/// `None` signals end of stream.
#[pyclass]
struct PyStreamIterator {
    rx: Arc<tokio::sync::Mutex<Option<tokio::sync::mpsc::Receiver<Option<String>>>>>,
}

#[pymethods]
impl PyStreamIterator {
    fn __aiter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __anext__(slf: PyRef<'_, Self>, py: Python<'_>) -> PyResult<PyObject> {
        let rx_arc = slf.rx.clone();
        let coro = pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let mut guard = rx_arc.lock().await;
            match guard.as_mut() {
                Some(rx) => match rx.recv().await {
                    Some(Some(chunk)) => Ok(Some(chunk)),
                    Some(None) | None => {
                        // End of stream — clear the receiver
                        *guard = None;
                        Ok(None)
                    }
                },
                None => Ok(None), // Already exhausted
            }
        })?;
        Ok(coro.into_any().unbind())
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

fn build_axon_url(axon_info: &RustAxonInfo) -> String {
    let protocol = if axon_info.protocol == 0 { "http" } else { "https" };
    let ip = if axon_info.ip == 0 { "127.0.0.1".to_string() } else { ip_from_u64(axon_info.ip) };
    format!("{protocol}://{ip}:{}", axon_info.port)
}

fn ip_from_u64(ip: u64) -> String {
    let a = (ip >> 24) as u8;
    let b = (ip >> 16) as u8;
    let c = (ip >> 8) as u8;
    let d = ip as u8;
    format!("{a}.{b}.{c}.{d}")
}
