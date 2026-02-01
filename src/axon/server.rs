//! Axon HTTP server implementation
//!
//! The Axon is an HTTP server that receives requests from Dendrites in the
//! Bittensor network. It handles request verification, routing, and response
//! generation.

use crate::axon::handlers::{
    build_error_response, build_success_response, extract_synapse, status_codes, verify_request,
    AXON_VERSION,
};
use crate::axon::info::{AxonConfig, AxonInfo};
use crate::axon::middleware::{
    blacklist_middleware, counter_middleware, logging_middleware, priority_middleware,
    timeout_middleware, verify_middleware,
};
use crate::errors::{AxonConfigError, AxonError};
use crate::types::Synapse;
use crate::wallet::Keypair;
use axum::body::Bytes;

use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{middleware as axum_middleware, Router};
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::net::{IpAddr, SocketAddr};
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};

use tower_http::trace::TraceLayer;
use tracing::{error, info};

/// Type alias for synapse handler function
pub type SynapseHandler = Arc<
    dyn Fn(Synapse) -> Pin<Box<dyn Future<Output = Synapse> + Send>> + Send + Sync,
>;

/// Type alias for blacklist check function
pub type BlacklistFn = Arc<dyn Fn(&str, &str) -> bool + Send + Sync>;

/// Type alias for priority function
pub type PriorityFn = Arc<dyn Fn(&str, &str) -> f32 + Send + Sync>;

/// Type alias for verify function
pub type VerifyFn = Arc<dyn Fn(&str) -> bool + Send + Sync>;

/// Axon server state
pub struct AxonState {
    /// Number of currently active requests
    pub request_count: u64,
    /// Total requests received since startup
    pub total_requests: u64,
    /// Set of blacklisted hotkeys
    pub blacklist: HashSet<String>,
    /// Set of blacklisted IPs
    pub ip_blacklist: HashSet<String>,
    /// Priority mapping: hotkey -> priority (higher = more priority)
    pub priority_list: HashMap<String, f32>,
    /// The axon's hotkey SS58 address
    pub axon_hotkey: String,
    /// Whether to verify request signatures
    pub verify_signatures: bool,
    /// Whether to trust X-Forwarded-For and X-Real-IP headers.
    /// Only enable when running behind a trusted reverse proxy.
    pub trust_proxy_headers: bool,
    /// Custom blacklist function
    pub blacklist_fn: Option<BlacklistFn>,
    /// Custom priority function
    pub priority_fn: Option<PriorityFn>,
    /// Custom verify function
    pub verify_fn: Option<VerifyFn>,
}

impl Default for AxonState {
    fn default() -> Self {
        Self {
            request_count: 0,
            total_requests: 0,
            blacklist: HashSet::new(),
            ip_blacklist: HashSet::new(),
            priority_list: HashMap::new(),
            axon_hotkey: String::new(),
            verify_signatures: true,
            trust_proxy_headers: false,
            blacklist_fn: None,
            priority_fn: None,
            verify_fn: None,
        }
    }
}

/// Axon HTTP server for receiving Bittensor network requests
///
/// The Axon handles:
/// - Request signature verification
/// - Blacklist/whitelist enforcement
/// - Priority-based request queuing
/// - Custom synapse handlers
///
/// # Example
///
/// ```ignore
/// use bittensor_rs::axon::{Axon, AxonConfig};
/// use bittensor_rs::wallet::Keypair;
///
/// let keypair = Keypair::from_uri("//Alice").unwrap();
/// let config = AxonConfig::new().with_port(8091);
///
/// let mut axon = Axon::new(keypair, config);
///
/// // Attach a handler for a specific synapse
/// axon.attach("MyQuery", |synapse| async move {
///     // Process the synapse and return response
///     synapse
/// });
///
/// // Start serving
/// axon.serve().await?;
/// ```
pub struct Axon {
    /// The keypair for signing responses
    keypair: Keypair,
    /// Server configuration
    config: AxonConfig,
    /// Server state (shared across handlers)
    state: Arc<RwLock<AxonState>>,
    /// Registered synapse handlers
    handlers: HashMap<String, SynapseHandler>,
}

impl Axon {
    /// Create a new Axon server
    ///
    /// # Arguments
    ///
    /// * `keypair` - The hotkey keypair for signing responses
    /// * `config` - The server configuration
    ///
    /// # Returns
    ///
    /// A new Axon instance
    pub fn new(keypair: Keypair, config: AxonConfig) -> Self {
        let state = AxonState {
            axon_hotkey: keypair.ss58_address().to_string(),
            verify_signatures: config.verify_signatures,
            trust_proxy_headers: config.trust_proxy_headers,
            ..Default::default()
        };

        Self {
            keypair,
            config,
            state: Arc::new(RwLock::new(state)),
            handlers: HashMap::new(),
        }
    }

    /// Attach a synapse handler for a specific route
    ///
    /// # Arguments
    ///
    /// * `name` - The synapse name (route path)
    /// * `handler` - The async handler function
    ///
    /// # Returns
    ///
    /// Mutable reference to self for chaining
    ///
    /// # Example
    ///
    /// ```ignore
    /// axon.attach("Query", |synapse| async move {
    ///     // Process synapse
    ///     synapse
    /// });
    /// ```
    pub fn attach<F, Fut>(&mut self, name: &str, handler: F) -> &mut Self
    where
        F: Fn(Synapse) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Synapse> + Send + 'static,
    {
        let handler = Arc::new(move |synapse: Synapse| {
            let fut = handler(synapse);
            Box::pin(fut) as Pin<Box<dyn Future<Output = Synapse> + Send>>
        });
        self.handlers.insert(name.to_string(), handler);
        self
    }

    /// Set a custom blacklist check function
    ///
    /// The function receives (hotkey, synapse_name) and returns true if blacklisted.
    ///
    /// # Arguments
    ///
    /// * `f` - The blacklist check function
    ///
    /// # Returns
    ///
    /// Mutable reference to self for chaining
    pub fn set_blacklist<F>(&mut self, f: F) -> &mut Self
    where
        F: Fn(&str, &str) -> bool + Send + Sync + 'static,
    {
        // Use try_write first for immediate update without blocking.
        // If the lock is held, spawn a task to update when available.
        // This is acceptable since set_blacklist is typically called during setup
        // before the server starts handling requests.
        let blacklist_fn = Arc::new(f);
        if let Ok(mut state_write) = self.state.try_write() {
            state_write.blacklist_fn = Some(blacklist_fn);
        } else {
            // Lock is held, spawn a task to update when available
            let state = self.state.clone();
            tokio::spawn(async move {
                let mut state_write = state.write().await;
                state_write.blacklist_fn = Some(blacklist_fn);
            });
        }
        self
    }

    /// Set a custom priority function
    ///
    /// The function receives (hotkey, synapse_name) and returns a priority value.
    /// Higher values indicate higher priority.
    ///
    /// # Arguments
    ///
    /// * `f` - The priority function
    ///
    /// # Returns
    ///
    /// Mutable reference to self for chaining
    pub fn set_priority<F>(&mut self, f: F) -> &mut Self
    where
        F: Fn(&str, &str) -> f32 + Send + Sync + 'static,
    {
        // Use try_write first for immediate update without blocking.
        // If the lock is held, spawn a task to update when available.
        // This is acceptable since set_priority is typically called during setup
        // before the server starts handling requests.
        let priority_fn = Arc::new(f);
        if let Ok(mut state_write) = self.state.try_write() {
            state_write.priority_fn = Some(priority_fn);
        } else {
            // Lock is held, spawn a task to update when available
            let state = self.state.clone();
            tokio::spawn(async move {
                let mut state_write = state.write().await;
                state_write.priority_fn = Some(priority_fn);
            });
        }
        self
    }

    /// Set a custom verification function
    ///
    /// The function receives the synapse_name and returns true if verification passes.
    /// This is called before signature verification.
    ///
    /// # Arguments
    ///
    /// * `f` - The verification function
    ///
    /// # Returns
    ///
    /// Mutable reference to self for chaining
    pub fn set_verify<F>(&mut self, f: F) -> &mut Self
    where
        F: Fn(&str) -> bool + Send + Sync + 'static,
    {
        // Use try_write first for immediate update without blocking.
        // If the lock is held, spawn a task to update when available.
        // This is acceptable since set_verify is typically called during setup
        // before the server starts handling requests.
        let verify_fn = Arc::new(f);
        if let Ok(mut state_write) = self.state.try_write() {
            state_write.verify_fn = Some(verify_fn);
        } else {
            // Lock is held, spawn a task to update when available
            let state = self.state.clone();
            tokio::spawn(async move {
                let mut state_write = state.write().await;
                state_write.verify_fn = Some(verify_fn);
            });
        }
        self
    }

    /// Add a hotkey to the blacklist
    ///
    /// # Arguments
    ///
    /// * `hotkey` - The hotkey SS58 address to blacklist
    pub async fn blacklist_hotkey(&self, hotkey: impl Into<String>) {
        let mut state_write = self.state.write().await;
        state_write.blacklist.insert(hotkey.into());
    }

    /// Remove a hotkey from the blacklist
    ///
    /// # Arguments
    ///
    /// * `hotkey` - The hotkey SS58 address to remove
    pub async fn unblacklist_hotkey(&self, hotkey: &str) {
        let mut state_write = self.state.write().await;
        state_write.blacklist.remove(hotkey);
    }

    /// Add an IP address to the blacklist
    ///
    /// # Arguments
    ///
    /// * `ip` - The IP address to blacklist
    pub async fn blacklist_ip(&self, ip: impl Into<String>) {
        let mut state_write = self.state.write().await;
        state_write.ip_blacklist.insert(ip.into());
    }

    /// Remove an IP address from the blacklist
    ///
    /// # Arguments
    ///
    /// * `ip` - The IP address to remove
    pub async fn unblacklist_ip(&self, ip: &str) {
        let mut state_write = self.state.write().await;
        state_write.ip_blacklist.remove(ip);
    }

    /// Set the priority for a hotkey
    ///
    /// # Arguments
    ///
    /// * `hotkey` - The hotkey SS58 address
    /// * `priority` - The priority value (higher = more priority)
    pub async fn set_hotkey_priority(&self, hotkey: impl Into<String>, priority: f32) {
        let mut state_write = self.state.write().await;
        state_write.priority_list.insert(hotkey.into(), priority);
    }

    /// Get the current request count
    pub async fn request_count(&self) -> u64 {
        self.state.read().await.request_count
    }

    /// Get the total requests received
    pub async fn total_requests(&self) -> u64 {
        self.state.read().await.total_requests
    }

    /// Get the axon's hotkey SS58 address
    pub fn hotkey(&self) -> &str {
        self.keypair.ss58_address()
    }

    /// Get the configuration
    pub fn config(&self) -> &AxonConfig {
        &self.config
    }

    /// Build the axum Router with all handlers and middleware
    fn build_router(&self) -> Router<()> {
        let state = self.state.clone();
        let handlers = self.handlers.clone();
        let keypair = self.keypair.clone();

        // Create base router with state
        let mut router: Router<Arc<RwLock<AxonState>>> = Router::new();

        // Health check endpoint
        router = router.route("/health", get(health_handler));

        // Add synapse handlers
        for (name, handler) in handlers.iter() {
            let handler = handler.clone();
            let keypair = keypair.clone();
            let state_clone = state.clone();

            let route_handler = move |headers: HeaderMap, body: Bytes| {
                let handler = handler.clone();
                let keypair = keypair.clone();
                let state = state_clone.clone();

                async move {
                    handle_synapse_request(state, keypair, headers, body, handler).await
                }
            };

            router = router.route(&format!("/{}", name), post(route_handler));
        }

        // Add middleware layers
        router
            .layer(axum_middleware::from_fn_with_state(state.clone(), counter_middleware))
            .layer(axum_middleware::from_fn_with_state(state.clone(), timeout_middleware))
            .layer(axum_middleware::from_fn_with_state(state.clone(), verify_middleware))
            .layer(axum_middleware::from_fn_with_state(state.clone(), priority_middleware))
            .layer(axum_middleware::from_fn_with_state(state.clone(), blacklist_middleware))
            .layer(axum_middleware::from_fn(logging_middleware))
            .layer(TraceLayer::new_for_http())
            .layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any),
            )
            .with_state(state)
    }

    /// Start the HTTP server
    ///
    /// # Returns
    ///
    /// Ok(()) on successful shutdown, or an error
    pub async fn serve(self) -> Result<(), AxonError> {
        let addr: SocketAddr = self
            .config
            .socket_addr()
            .parse()
            .map_err(|e| AxonError::new(format!("Invalid socket address: {}", e)))?;

        let router = self.build_router();

        info!(
            "Axon server starting on {} (hotkey: {})",
            addr,
            self.keypair.ss58_address()
        );

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| AxonError::new(format!("Failed to bind to {}: {}", addr, e)))?;

        axum::serve(listener, router)
            .await
            .map_err(|e| AxonError::new(format!("Server error: {}", e)))?;

        Ok(())
    }

    /// Start the HTTP server with TLS
    ///
    /// # Arguments
    ///
    /// * `cert_path` - Path to the TLS certificate file
    /// * `key_path` - Path to the TLS private key file
    ///
    /// # Returns
    ///
    /// Ok(()) on successful shutdown, or an error
    pub async fn serve_tls(self, cert_path: &Path, key_path: &Path) -> Result<(), AxonError> {
        use axum_server::tls_rustls::RustlsConfig;

        let addr: SocketAddr = self
            .config
            .socket_addr()
            .parse()
            .map_err(|e| AxonError::new(format!("Invalid socket address: {}", e)))?;

        let router = self.build_router();

        // Load TLS configuration
        let tls_config = RustlsConfig::from_pem_file(cert_path, key_path)
            .await
            .map_err(|e| AxonError::new(format!("Failed to load TLS config: {}", e)))?;

        info!(
            "Axon server starting on {} with TLS (hotkey: {})",
            addr,
            self.keypair.ss58_address()
        );

        axum_server::bind_rustls(addr, tls_config)
            .serve(router.into_make_service())
            .await
            .map_err(|e| AxonError::new(format!("Server error: {}", e)))?;

        Ok(())
    }

    /// Get the AxonInfo for chain registration
    ///
    /// This returns the information needed to register the axon on-chain.
    ///
    /// # Arguments
    ///
    /// * `block` - The current block number
    ///
    /// # Returns
    ///
    /// AxonInfo ready for chain registration
    pub fn info(&self, block: u64) -> Result<AxonInfo, AxonConfigError> {
        let external_ip = self.config.get_external_ip();
        let external_port = self.config.get_external_port();

        let ip: IpAddr = external_ip.parse().map_err(|e| {
            AxonConfigError::with_field(
                format!("Invalid IP address '{}': {}", external_ip, e),
                "external_ip",
            )
        })?;

        let ip_type = match ip {
            IpAddr::V4(_) => 4,
            IpAddr::V6(_) => 6,
        };

        Ok(AxonInfo {
            block,
            version: AXON_VERSION as u32,
            ip,
            port: external_port,
            ip_type,
            protocol: 4, // TCP
            placeholder1: 0,
            placeholder2: 0,
        })
    }
}

/// Health check handler
async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// Handle a synapse request
async fn handle_synapse_request(
    state: Arc<RwLock<AxonState>>,
    keypair: Keypair,
    headers: HeaderMap,
    body: Bytes,
    handler: SynapseHandler,
) -> Response {
    let start_time = std::time::Instant::now();
    let hotkey = keypair.ss58_address().to_string();

    // Verify the request signature if enabled
    {
        let state_read = state.read().await;
        if state_read.verify_signatures {
            match verify_request(&headers, &body, &hotkey) {
                Ok(_verified) => {}
                Err(e) => {
                    let process_time = start_time.elapsed().as_secs_f64();
                    return build_error_response(
                        &hotkey,
                        StatusCode::UNAUTHORIZED,
                        status_codes::UNAUTHORIZED,
                        &e.message,
                        process_time,
                    );
                }
            }
        }
    }

    // Extract the synapse from the request
    let synapse = match extract_synapse(&headers, &body) {
        Ok(s) => s,
        Err(e) => {
            let process_time = start_time.elapsed().as_secs_f64();
            return build_error_response(
                &hotkey,
                StatusCode::BAD_REQUEST,
                status_codes::INTERNAL_ERROR,
                &e.message,
                process_time,
            );
        }
    };

    // Call the handler
    let response_synapse = handler(synapse).await;

    // Serialize the response
    let response_body = match serde_json::to_vec(&response_synapse.extra) {
        Ok(b) => Bytes::from(b),
        Err(e) => {
            let process_time = start_time.elapsed().as_secs_f64();
            error!("Failed to serialize response: {}", e);
            return build_error_response(
                &hotkey,
                StatusCode::INTERNAL_SERVER_ERROR,
                status_codes::INTERNAL_ERROR,
                "Failed to serialize response",
                process_time,
            );
        }
    };

    let process_time = start_time.elapsed().as_secs_f64();
    build_success_response(&hotkey, response_body, process_time)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_keypair() -> Keypair {
        Keypair::from_uri("//Alice").expect("Failed to create test keypair")
    }

    #[test]
    fn test_axon_new() {
        let keypair = create_test_keypair();
        let config = AxonConfig::default();
        let axon = Axon::new(keypair, config);

        assert!(!axon.hotkey().is_empty());
        assert_eq!(axon.config().port, 8091);
    }

    #[test]
    fn test_axon_attach_handler() {
        let keypair = create_test_keypair();
        let config = AxonConfig::default();
        let mut axon = Axon::new(keypair, config);

        axon.attach("TestQuery", |synapse| async move { synapse });

        assert!(axon.handlers.contains_key("TestQuery"));
    }

    #[test]
    fn test_axon_info() {
        let keypair = create_test_keypair();
        let config = AxonConfig::new()
            .with_ip("192.168.1.1")
            .with_port(9000)
            .with_external_ip("1.2.3.4")
            .with_external_port(9001);
        let axon = Axon::new(keypair, config);

        let info = axon.info(1000).unwrap();

        assert_eq!(info.block, 1000);
        assert_eq!(info.port, 9001);
        assert_eq!(info.ip_type, 4); // IPv4
    }

    #[tokio::test]
    async fn test_axon_blacklist() {
        let keypair = create_test_keypair();
        let config = AxonConfig::default();
        let axon = Axon::new(keypair, config);

        axon.blacklist_hotkey("test_hotkey").await;

        let state = axon.state.read().await;
        assert!(state.blacklist.contains("test_hotkey"));
    }

    #[tokio::test]
    async fn test_axon_priority() {
        let keypair = create_test_keypair();
        let config = AxonConfig::default();
        let axon = Axon::new(keypair, config);

        axon.set_hotkey_priority("high_priority", 1.0).await;
        axon.set_hotkey_priority("low_priority", 0.1).await;

        let state = axon.state.read().await;
        assert_eq!(state.priority_list.get("high_priority"), Some(&1.0));
        assert_eq!(state.priority_list.get("low_priority"), Some(&0.1));
    }

    #[tokio::test]
    async fn test_axon_request_counters() {
        let keypair = create_test_keypair();
        let config = AxonConfig::default();
        let axon = Axon::new(keypair, config);

        assert_eq!(axon.request_count().await, 0);
        assert_eq!(axon.total_requests().await, 0);

        // Simulate some requests
        {
            let mut state = axon.state.write().await;
            state.request_count = 5;
            state.total_requests = 100;
        }

        assert_eq!(axon.request_count().await, 5);
        assert_eq!(axon.total_requests().await, 100);
    }

    #[test]
    fn test_axon_state_default() {
        let state = AxonState::default();

        assert_eq!(state.request_count, 0);
        assert_eq!(state.total_requests, 0);
        assert!(state.blacklist.is_empty());
        assert!(state.priority_list.is_empty());
        assert!(state.verify_signatures);
        assert!(!state.trust_proxy_headers);
    }
}
