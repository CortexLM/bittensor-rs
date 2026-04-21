//! Python bindings for bittensor-axon: Axon server with middleware chain.

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

use bittensor_axon::config::AxonConfig as RustAxonConfig;
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::{Arc, LazyLock, Mutex as StdMutex};

use tokio::sync::{RwLock, broadcast};

use crate::core_types::BittensorError;
use axum::response::IntoResponse;

// ---------------------------------------------------------------------------
// Global registries
// ---------------------------------------------------------------------------

/// Python callables keyed by route path (e.g. "/TextPrompt").
static PY_HANDLER_REGISTRY: LazyLock<StdMutex<HashMap<String, Py<PyAny>>>> =
    LazyLock::new(|| StdMutex::new(HashMap::new()));

/// Shutdown senders keyed by bound address string.
static SHUTDOWN_REGISTRY: LazyLock<StdMutex<HashMap<String, broadcast::Sender<()>>>> =
    LazyLock::new(|| StdMutex::new(HashMap::new()));

// ---------------------------------------------------------------------------
// Shared middleware state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct PyMiddlewareState {
    _axon_hotkey: Option<String>,
    blacklist: Arc<RwLock<HashSet<String>>>,
    priority_map: Arc<RwLock<HashMap<String, u32>>>,
}

// ---------------------------------------------------------------------------
// AxonConfig
// ---------------------------------------------------------------------------

/// Configuration for the Axon HTTP server.
#[pyclass]
#[derive(Clone)]
pub struct AxonConfig {
    inner: RustAxonConfig,
}

#[pymethods]
impl AxonConfig {
    #[new]
    #[pyo3(signature = (
        ip="0.0.0.0".to_string(),
        port=8090,
        max_connections=0,
        external_ip=None,
        hotkey=None,
    ))]
    fn new(
        ip: String,
        port: u16,
        max_connections: usize,
        external_ip: Option<String>,
        hotkey: Option<String>,
    ) -> Self {
        Self { inner: RustAxonConfig { ip, port, max_connections, external_ip, hotkey } }
    }

    #[getter]
    fn ip(&self) -> &str {
        &self.inner.ip
    }

    #[setter]
    fn set_ip(&mut self, val: String) {
        self.inner.ip = val;
    }

    #[getter]
    fn port(&self) -> u16 {
        self.inner.port
    }

    #[setter]
    fn set_port(&mut self, val: u16) {
        self.inner.port = val;
    }

    #[getter]
    fn max_connections(&self) -> usize {
        self.inner.max_connections
    }

    #[setter]
    fn set_max_connections(&mut self, val: usize) {
        self.inner.max_connections = val;
    }

    #[getter]
    fn external_ip(&self) -> Option<&str> {
        self.inner.external_ip.as_deref()
    }

    #[setter]
    fn set_external_ip(&mut self, val: Option<String>) {
        self.inner.external_ip = val;
    }

    #[getter]
    fn hotkey(&self) -> Option<&str> {
        self.inner.hotkey.as_deref()
    }

    #[setter]
    fn set_hotkey(&mut self, val: Option<String>) {
        self.inner.hotkey = val;
    }

    fn __repr__(&self) -> String {
        format!(
            "AxonConfig(ip='{}', port={}, hotkey={:?})",
            self.inner.ip, self.inner.port, self.inner.hotkey
        )
    }
}

// ---------------------------------------------------------------------------
// Axon
// ---------------------------------------------------------------------------

/// Axon HTTP server with middleware chain.
///
/// Usage:
///     config = bt.AxonConfig(port=0)
///     axon = bt.Axon(config)
///     axon.attach("TextPrompt", my_handler)
///     addr = await axon.start()
///     # ... later ...
///     axon.stop(addr)
#[pyclass]
pub struct Axon {
    config: AxonConfig,
    state: Arc<PyMiddlewareState>,
    _bound_addr: Option<String>,
}

#[pymethods]
impl Axon {
    #[new]
    #[pyo3(signature = (config=None))]
    fn new(config: Option<AxonConfig>) -> Self {
        let config = config.unwrap_or_else(|| AxonConfig { inner: RustAxonConfig::default() });
        let state = Arc::new(PyMiddlewareState {
            _axon_hotkey: config.inner.hotkey.clone(),
            blacklist: Arc::new(RwLock::new(HashSet::new())),
            priority_map: Arc::new(RwLock::new(HashMap::new())),
        });
        Self { config, state, _bound_addr: None }
    }

    /// Register a synapse handler.
    ///
    /// Args:
    ///     synapse_type: Name of the synapse (used as the URL path).
    ///     handler: Python callable(dict) -> dict.
    fn attach(&self, synapse_type: &str, handler: Py<PyAny>) -> PyResult<()> {
        let path = format!("/{}", synapse_type);
        PY_HANDLER_REGISTRY
            .lock()
            .map_err(|e| PyRuntimeError::new_err(format!("registry lock poisoned: {e}")))?
            .insert(path, handler);
        Ok(())
    }

    /// Start the Axon server. Returns the bound address string.
    fn start(&mut self, py: Python<'_>) -> PyResult<PyObject> {
        let config = self.config.inner.clone();
        let state = self.state.clone();

        let coro = pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let addr: SocketAddr =
                config.bind_addr().parse().map_err(|e: std::net::AddrParseError| {
                    BittensorError::new_err(format!("invalid bind address: {e}"))
                })?;

            let listener = tokio::net::TcpListener::bind(addr)
                .await
                .map_err(|e| BittensorError::new_err(format!("bind failed: {e}")))?;

            let actual_addr = listener
                .local_addr()
                .map_err(|e| BittensorError::new_err(format!("local_addr: {e}")))?;

            let addr_str = actual_addr.to_string();

            let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
            {
                let mut guard = SHUTDOWN_REGISTRY
                    .lock()
                    .map_err(|e| BittensorError::new_err(format!("registry lock poisoned: {e}")))?;
                guard.insert(addr_str.clone(), shutdown_tx);
            }

            let router = build_router(state);

            tokio::spawn(async move {
                let _ = axum::serve(listener, router)
                    .with_graceful_shutdown(shutdown_signal(shutdown_rx))
                    .await;
            });

            Ok(addr_str)
        })?;

        Ok(coro.into_any().unbind())
    }

    /// Stop the Axon server at the given address.
    fn stop(&self, addr: &str) -> PyResult<()> {
        let mut guard = SHUTDOWN_REGISTRY
            .lock()
            .map_err(|e| PyRuntimeError::new_err(format!("registry lock poisoned: {e}")))?;
        if let Some(tx) = guard.remove(addr) {
            let _ = tx.send(());
            Ok(())
        } else {
            Err(PyRuntimeError::new_err(format!("no running Axon at address '{addr}'")))
        }
    }

    /// Blacklist a hotkey.
    fn blacklist(&self, key: &str, py: Python<'_>) -> PyResult<PyObject> {
        let bl = self.state.blacklist.clone();
        let key = key.to_string();
        let coro = pyo3_async_runtimes::tokio::future_into_py(py, async move {
            bl.write().await.insert(key);
            Ok(())
        })?;
        Ok(coro.into_any().unbind())
    }

    /// Remove a hotkey from the blacklist.
    fn unblacklist(&self, key: &str, py: Python<'_>) -> PyResult<PyObject> {
        let bl = self.state.blacklist.clone();
        let key = key.to_string();
        let coro = pyo3_async_runtimes::tokio::future_into_py(py, async move {
            bl.write().await.remove(&key);
            Ok(())
        })?;
        Ok(coro.into_any().unbind())
    }

    /// Set the priority for a hotkey.
    fn set_priority(&self, key: &str, priority: u32, py: Python<'_>) -> PyResult<PyObject> {
        let pm = self.state.priority_map.clone();
        let key = key.to_string();
        let coro = pyo3_async_runtimes::tokio::future_into_py(py, async move {
            pm.write().await.insert(key, priority);
            Ok(())
        })?;
        Ok(coro.into_any().unbind())
    }

    fn __repr__(&self) -> String {
        format!("Axon(ip='{}', port={})", self.config.inner.ip, self.config.inner.port)
    }
}

// ---------------------------------------------------------------------------
// Axum router with catch-all handler that dispatches to Python
// ---------------------------------------------------------------------------

fn build_router(state: Arc<PyMiddlewareState>) -> axum::Router {
    axum::Router::new().fallback(py_handler_fallback).layer(axum::Extension(state))
}

/// Catch-all: dispatches every request to the Python handler registry.
async fn py_handler_fallback(
    axum::Extension(state): axum::Extension<Arc<PyMiddlewareState>>,
    req: axum::extract::Request,
) -> impl axum::response::IntoResponse {
    let path = req.uri().path().to_string();

    // Blacklist check
    {
        let blacklist = state.blacklist.read().await;
        let dendrite_hk = req
            .headers()
            .get("bt_header_dendrite_hotkey")
            .or_else(|| req.headers().get("bt-dendrite-hotkey"))
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if !dendrite_hk.is_empty() && blacklist.contains(dendrite_hk) {
            return (axum::http::StatusCode::FORBIDDEN, "hotkey is blacklisted").into_response();
        }
    }

    // Look up Python handler
    let handler = match PY_HANDLER_REGISTRY.lock() {
        Ok(guard) => Python::with_gil(|py| guard.get(&path).map(|h| h.clone_ref(py))),
        Err(_) => {
            return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "registry lock poisoned")
                .into_response();
        }
    };

    let handler = match handler {
        Some(h) => h,
        None => {
            return (axum::http::StatusCode::NOT_FOUND, "no handler registered").into_response();
        }
    };

    // Read body
    let (_, body) = req.into_parts();
    let bytes = match axum::body::to_bytes(body, 1024 * 1024).await {
        Ok(b) => b,
        Err(_) => {
            return (axum::http::StatusCode::BAD_REQUEST, "failed to read body").into_response();
        }
    };

    // Parse as JSON
    let body_value: serde_json::Value = if bytes.is_empty() {
        serde_json::Value::Object(serde_json::Map::new())
    } else {
        match serde_json::from_slice(&bytes) {
            Ok(v) => v,
            Err(_) => {
                return (axum::http::StatusCode::BAD_REQUEST, "invalid JSON").into_response();
            }
        }
    };

    // Call Python handler within the GIL
    let result = Python::with_gil(|py| {
        let callback = handler.bind(py);
        let py_arg = json_value_to_py(py, &body_value)?;
        let result_obj = callback.call1((py_arg,))?;
        // Try dict → JSON string, or string directly
        let result_str = if result_obj.is_instance_of::<pyo3::types::PyDict>() {
            let dict = result_obj.downcast::<pyo3::types::PyDict>()?;
            let mut map = serde_json::Map::new();
            for (k, v) in dict.iter() {
                let key: String = k.extract::<String>()?;
                let val = py_value_to_json(&v)?;
                map.insert(key, val);
            }
            serde_json::Value::Object(map).to_string()
        } else if result_obj.is_instance_of::<pyo3::types::PyString>() {
            result_obj.extract::<String>()?
        } else {
            let str_repr = result_obj.call_method0("__str__")?;
            str_repr.extract::<String>()?
        };
        Ok::<String, PyErr>(result_str)
    });

    match result {
        Ok(json_str) => (
            axum::http::StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "application/json")],
            json_str,
        )
            .into_response(),
        Err(_) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "handler error").into_response(),
    }
}

// ---------------------------------------------------------------------------
// JSON ↔ Python conversion helpers
// ---------------------------------------------------------------------------

fn json_value_to_py(py: Python<'_>, val: &serde_json::Value) -> PyResult<PyObject> {
    use pyo3::types::PyDict;
    match val {
        serde_json::Value::Object(map) => {
            let dict = PyDict::new(py);
            for (k, v) in map {
                dict.set_item(k, json_value_to_py(py, v)?)?;
            }
            Ok(dict.into_any().unbind())
        }
        serde_json::Value::String(s) => Ok(s.into_pyobject(py)?.into_any().unbind()),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.into_pyobject(py)?.into_any().unbind())
            } else if let Some(f) = n.as_f64() {
                Ok(f.into_pyobject(py)?.into_any().unbind())
            } else {
                Ok(n.to_string().into_pyobject(py)?.into_any().unbind())
            }
        }
        serde_json::Value::Bool(b) => {
            let bool_borrowed = (*b).into_pyobject(py)?;
            Ok(bool_borrowed.to_owned().into_any().unbind())
        }
        serde_json::Value::Null => Ok(py.None()),
        serde_json::Value::Array(arr) => {
            let list: Vec<PyObject> =
                arr.iter().map(|v| json_value_to_py(py, v)).collect::<PyResult<Vec<PyObject>>>()?;
            Ok(list.into_pyobject(py)?.into_any().unbind())
        }
    }
}

fn py_value_to_json(obj: &Bound<'_, PyAny>) -> PyResult<serde_json::Value> {
    if obj.is_none() {
        return Ok(serde_json::Value::Null);
    }
    if let Ok(b) = obj.extract::<bool>() {
        return Ok(serde_json::Value::Bool(b));
    }
    if let Ok(s) = obj.extract::<String>() {
        return Ok(serde_json::Value::String(s));
    }
    if let Ok(i) = obj.extract::<i64>() {
        return Ok(serde_json::Value::Number(i.into()));
    }
    if let Ok(f) = obj.extract::<f64>() {
        if let Some(n) = serde_json::Number::from_f64(f) {
            return Ok(serde_json::Value::Number(n));
        }
    }
    if let Ok(dict) = obj.downcast::<pyo3::types::PyDict>() {
        let mut map = serde_json::Map::new();
        for (k, v) in dict.iter() {
            let key: String = k.extract()?;
            let val = py_value_to_json(&v)?;
            map.insert(key, val);
        }
        return Ok(serde_json::Value::Object(map));
    }
    if let Ok(list) = obj.downcast::<pyo3::types::PyList>() {
        let arr: Vec<serde_json::Value> = list
            .iter()
            .map(|v| py_value_to_json(&v))
            .collect::<PyResult<Vec<serde_json::Value>>>()?;
        return Ok(serde_json::Value::Array(arr));
    }
    let s: String = obj.extract()?;
    Ok(serde_json::Value::String(s))
}

async fn shutdown_signal(mut rx: broadcast::Receiver<()>) {
    let _ = rx.recv().await;
}
