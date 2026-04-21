//! Dendrite HTTP client for Bittensor network communication
//!
//! The Dendrite client is responsible for making HTTP requests to Axon servers.
//! It handles request signing, connection pooling, timeouts, and response parsing.

use crate::dendrite::request::{DendriteRequest, RequestError};
use crate::dendrite::response::{
    build_error_synapse, status_codes, DendriteResponse, ResponseError,
};
use crate::dendrite::streaming::{StreamError, StreamingResponse, StreamingSynapse};
use crate::types::{AxonInfo, Synapse, TerminalInfo};
use crate::utils::ss58::AccountId32ToSS58;
use futures::Stream;
use reqwest::Client;
use sp_core::{sr25519, Pair};
use std::time::{Duration, Instant};
use thiserror::Error;
use uuid::Uuid;

/// Default timeout for Dendrite requests (12 seconds, matching Python SDK)
pub const DEFAULT_TIMEOUT_SECS: u64 = 12;

/// Default Dendrite version
pub const DEFAULT_DENDRITE_VERSION: u64 = 100;

/// Errors that can occur during Dendrite operations
#[derive(Debug, Error)]
pub enum DendriteError {
    #[error("Request building error: {0}")]
    Request(#[from] RequestError),
    #[error("Response error: {0}")]
    Response(#[from] ResponseError),
    #[error("HTTP client error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Timeout after {0:?}")]
    Timeout(Duration),
    #[error("Connection refused to {0}")]
    ConnectionRefused(String),
    #[error("Invalid axon: {0}")]
    InvalidAxon(String),
    #[error("Signing error: {0}")]
    Signing(String),
    #[error("Stream error: {0}")]
    Stream(#[from] StreamError),
}

/// Dendrite HTTP client for making requests to Axon servers
///
/// The Dendrite handles all aspects of communicating with Axon servers:
/// - Building and signing requests
/// - Managing connection pooling
/// - Handling timeouts
/// - Parsing responses
///
/// # Example
///
/// ```ignore
/// use bittensor_rs::dendrite::Dendrite;
///
/// // Create a dendrite without signing (anonymous)
/// let dendrite = Dendrite::new(None);
///
/// // Or with a keypair for signed requests
/// let keypair = sr25519::Pair::from_string("//Alice", None)?;
/// let dendrite = Dendrite::new(Some(keypair));
///
/// // Make a request
/// let synapse = Synapse::new().with_name("Query");
/// let response = dendrite.call(&axon, synapse).await?;
/// ```
pub struct Dendrite {
    /// The HTTP client (with connection pooling)
    client: Client,
    /// Optional keypair for signing requests
    keypair: Option<sr25519::Pair>,
    /// Default timeout for requests
    timeout: Duration,
    /// Dendrite version
    version: u64,
    /// Dendrite IP (optional, for headers)
    ip: Option<String>,
    /// Dendrite port (optional, for headers)
    port: Option<u16>,
}

impl Dendrite {
    /// Create a new Dendrite client
    ///
    /// # Arguments
    ///
    /// * `keypair` - Optional SR25519 keypair for signing requests
    ///
    /// # Returns
    ///
    /// A new Dendrite instance with default settings
    pub fn new(keypair: Option<sr25519::Pair>) -> Self {
        let client = Client::builder()
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            keypair,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            version: DEFAULT_DENDRITE_VERSION,
            ip: None,
            port: None,
        }
    }

    /// Set the default timeout for requests
    ///
    /// # Arguments
    ///
    /// * `timeout` - The timeout duration
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the Dendrite version
    ///
    /// # Arguments
    ///
    /// * `version` - The version number
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_version(mut self, version: u64) -> Self {
        self.version = version;
        self
    }

    /// Set the Dendrite IP address for headers
    ///
    /// # Arguments
    ///
    /// * `ip` - The IP address string
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_ip(mut self, ip: impl Into<String>) -> Self {
        self.ip = Some(ip.into());
        self
    }

    /// Set the Dendrite port for headers
    ///
    /// # Arguments
    ///
    /// * `port` - The port number
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// Get the hotkey SS58 address if a keypair is set
    pub fn hotkey(&self) -> Option<String> {
        self.keypair.as_ref().map(|kp| kp.public().to_ss58())
    }

    /// Build the dendrite terminal info for request headers
    fn build_dendrite_info(&self) -> TerminalInfo {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);

        TerminalInfo {
            ip: self.ip.clone(),
            port: self.port,
            version: Some(self.version),
            nonce: Some(nonce),
            uuid: Some(Uuid::new_v4().to_string()),
            hotkey: self.hotkey(),
            ..Default::default()
        }
    }

    /// Send a synapse to a single axon
    ///
    /// # Arguments
    ///
    /// * `axon` - The target Axon server
    /// * `synapse` - The Synapse to send
    ///
    /// # Returns
    ///
    /// The response Synapse with updated terminal info and response data
    pub async fn call(&self, axon: &AxonInfo, synapse: Synapse) -> Result<Synapse, DendriteError> {
        self.call_with_timeout(axon, synapse, self.timeout).await
    }

    /// Send a synapse to a single axon with a specific timeout
    ///
    /// # Arguments
    ///
    /// * `axon` - The target Axon server
    /// * `synapse` - The Synapse to send
    /// * `timeout` - Request timeout
    ///
    /// # Returns
    ///
    /// The response Synapse with updated terminal info and response data
    pub async fn call_with_timeout(
        &self,
        axon: &AxonInfo,
        synapse: Synapse,
        timeout: Duration,
    ) -> Result<Synapse, DendriteError> {
        // Validate axon is serving
        if !axon.is_serving() {
            return Err(DendriteError::InvalidAxon(
                "Axon is not serving (0.0.0.0)".to_string(),
            ));
        }

        let start_time = Instant::now();

        // Build the request
        let dendrite_info = self.build_dendrite_info();
        let mut request = DendriteRequest::new(axon, &synapse, &dendrite_info, timeout)?;

        // Sign the request if we have a keypair
        // For signing, prefer the axon's hotkey if known, otherwise fall back to IP:port
        let axon_hotkey = axon.hotkey.clone().unwrap_or_else(|| axon.ip_str());
        if let Some(ref keypair) = self.keypair {
            request.sign(keypair, &axon_hotkey)?;
        }

        // Convert to HTTP headers
        let headers = crate::dendrite::request::synapse_to_headers(&Synapse {
            name: synapse.name.clone(),
            timeout: Some(timeout.as_secs_f64()),
            dendrite: Some(TerminalInfo {
                ip: request.headers.dendrite_ip.clone(),
                port: request
                    .headers
                    .dendrite_port
                    .as_ref()
                    .and_then(|p| p.parse().ok()),
                version: request
                    .headers
                    .dendrite_version
                    .as_ref()
                    .and_then(|v| v.parse().ok()),
                nonce: request
                    .headers
                    .dendrite_nonce
                    .as_ref()
                    .and_then(|n| n.parse().ok()),
                uuid: request.headers.dendrite_uuid.clone(),
                hotkey: request.headers.dendrite_hotkey.clone(),
                signature: request.headers.dendrite_signature.clone(),
                ..Default::default()
            }),
            computed_body_hash: request.headers.computed_body_hash.clone(),
            ..Default::default()
        });
        // Build the HTTP request
        let http_request = self
            .client
            .post(&request.url)
            .headers(headers)
            .body(request.body)
            .timeout(timeout);

        // Execute the request
        let result = http_request.send().await;
        let process_time = start_time.elapsed().as_secs_f64();

        match result {
            Ok(response) => {
                let status = response.status().as_u16();
                let response_headers = response.headers().clone();
                let body = response.bytes().await?.to_vec();

                let dendrite_response =
                    DendriteResponse::new(status, response_headers, body, process_time);
                Ok(dendrite_response.into_synapse()?)
            }
            Err(e) => {
                if e.is_timeout() {
                    Ok(build_error_synapse(
                        &synapse,
                        status_codes::TIMEOUT,
                        "Request timeout",
                        process_time,
                    ))
                } else if e.is_connect() {
                    Ok(build_error_synapse(
                        &synapse,
                        status_codes::SERVICE_UNAVAILABLE,
                        &format!("Connection failed: {}", e),
                        process_time,
                    ))
                } else {
                    Err(DendriteError::Http(e))
                }
            }
        }
    }

    /// Send a synapse to multiple axons concurrently
    ///
    /// # Arguments
    ///
    /// * `axons` - List of target Axon servers
    /// * `synapse` - The Synapse to send (cloned for each request)
    ///
    /// # Returns
    ///
    /// A vector of results, one for each axon in the same order
    pub async fn call_many(
        &self,
        axons: &[AxonInfo],
        synapse: Synapse,
    ) -> Vec<Result<Synapse, DendriteError>> {
        self.forward(axons, synapse, None).await
    }

    /// Forward a synapse to multiple axons (like Python dendrite.forward)
    ///
    /// This is the main method for sending requests to multiple axons,
    /// equivalent to the Python SDK's `dendrite.forward()` method.
    ///
    /// # Arguments
    ///
    /// * `axons` - List of target Axon servers
    /// * `synapse` - The Synapse to send (cloned for each request)
    /// * `timeout` - Optional timeout override
    ///
    /// # Returns
    ///
    /// A vector of results, one for each axon in the same order
    pub async fn forward(
        &self,
        axons: &[AxonInfo],
        synapse: Synapse,
        timeout: Option<Duration>,
    ) -> Vec<Result<Synapse, DendriteError>> {
        let timeout = timeout.unwrap_or(self.timeout);

        // Create futures for all requests
        let futures: Vec<_> = axons
            .iter()
            .map(|axon| {
                let synapse_clone = synapse.clone();
                self.call_with_timeout(axon, synapse_clone, timeout)
            })
            .collect();

        // Execute all concurrently
        futures::future::join_all(futures).await
    }

    /// Send a streaming synapse to a single axon
    ///
    /// # Arguments
    ///
    /// * `axon` - The target Axon server
    /// * `synapse` - The streaming synapse to send
    ///
    /// # Returns
    ///
    /// A Stream that yields chunks as they arrive
    pub async fn call_stream<S>(
        &self,
        axon: &AxonInfo,
        synapse: S,
    ) -> Result<impl Stream<Item = Result<S::Chunk, StreamError>>, DendriteError>
    where
        S: StreamingSynapse + Unpin + 'static,
    {
        self.call_stream_with_timeout(axon, synapse, self.timeout)
            .await
    }

    /// Send a streaming synapse to a single axon with a specific timeout
    ///
    /// # Arguments
    ///
    /// * `axon` - The target Axon server
    /// * `synapse` - The streaming synapse to send
    /// * `timeout` - Connection timeout (not stream timeout)
    ///
    /// # Returns
    ///
    /// A Stream that yields chunks as they arrive
    pub async fn call_stream_with_timeout<S>(
        &self,
        axon: &AxonInfo,
        synapse: S,
        timeout: Duration,
    ) -> Result<impl Stream<Item = Result<S::Chunk, StreamError>>, DendriteError>
    where
        S: StreamingSynapse + Unpin + 'static,
    {
        // Validate axon is serving
        if !axon.is_serving() {
            return Err(DendriteError::InvalidAxon(
                "Axon is not serving (0.0.0.0)".to_string(),
            ));
        }

        // Build the endpoint URL
        let url = format!("{}/{}", axon.to_endpoint(), synapse.name());

        // Build headers
        let dendrite_info = self.build_dendrite_info();
        let mut headers = http::HeaderMap::new();

        // Add dendrite headers
        if let Some(ref ip) = dendrite_info.ip {
            if let Ok(hv) = http::HeaderValue::from_str(ip) {
                headers.insert("bt_header_dendrite_ip", hv);
            }
        }
        if let Some(port) = dendrite_info.port {
            if let Ok(hv) = http::HeaderValue::from_str(&port.to_string()) {
                headers.insert("bt_header_dendrite_port", hv);
            }
        }
        if let Some(version) = dendrite_info.version {
            if let Ok(hv) = http::HeaderValue::from_str(&version.to_string()) {
                headers.insert("bt_header_dendrite_version", hv);
            }
        }
        if let Some(nonce) = dendrite_info.nonce {
            if let Ok(hv) = http::HeaderValue::from_str(&nonce.to_string()) {
                headers.insert("bt_header_dendrite_nonce", hv);
            }
        }
        if let Some(ref uuid) = dendrite_info.uuid {
            if let Ok(hv) = http::HeaderValue::from_str(uuid) {
                headers.insert("bt_header_dendrite_uuid", hv);
            }
        }
        if let Some(ref hotkey) = dendrite_info.hotkey {
            if let Ok(hv) = http::HeaderValue::from_str(hotkey) {
                headers.insert("bt_header_dendrite_hotkey", hv);
            }
        }

        // Add name header
        if let Ok(hv) = http::HeaderValue::from_str(synapse.name()) {
            headers.insert("name", hv);
        }

        // Add timeout header
        if let Ok(hv) = http::HeaderValue::from_str(&timeout.as_secs_f64().to_string()) {
            headers.insert("bt_header_timeout", hv);
        }

        // Build the HTTP request
        let http_request = self.client.post(&url).headers(headers).timeout(timeout);

        // Execute the request and get the response stream
        let response = http_request.send().await?;

        if !response.status().is_success() {
            return Err(DendriteError::Response(ResponseError::HttpError {
                status: response.status().as_u16(),
            }));
        }

        // Get the byte stream from the response
        let byte_stream = response.bytes_stream();

        // Create the streaming response
        Ok(StreamingResponse::new(synapse, byte_stream))
    }
}

impl Clone for Dendrite {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            keypair: self.keypair.clone(),
            timeout: self.timeout,
            version: self.version,
            ip: self.ip.clone(),
            port: self.port,
        }
    }
}

impl Default for Dendrite {
    fn default() -> Self {
        Self::new(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;

    fn create_test_axon() -> AxonInfo {
        AxonInfo {
            hotkey: Some("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".to_string()),
            block: 1000,
            version: 100,
            ip: IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
            port: 8091,
            ip_type: 4,
            protocol: 0,
            placeholder1: 0,
            placeholder2: 0,
        }
    }

    fn create_non_serving_axon() -> AxonInfo {
        AxonInfo {
            hotkey: None,
            block: 1000,
            version: 100,
            ip: IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
            port: 8091,
            ip_type: 4,
            protocol: 0,
            placeholder1: 0,
            placeholder2: 0,
        }
    }

    #[test]
    fn test_dendrite_new_without_keypair() {
        let dendrite = Dendrite::new(None);
        assert!(dendrite.keypair.is_none());
        assert!(dendrite.hotkey().is_none());
        assert_eq!(dendrite.timeout, Duration::from_secs(DEFAULT_TIMEOUT_SECS));
    }

    #[test]
    fn test_dendrite_new_with_keypair() {
        let keypair =
            sr25519::Pair::from_string("//Alice", None).expect("Failed to create test keypair");
        let dendrite = Dendrite::new(Some(keypair));

        assert!(dendrite.keypair.is_some());
        assert!(dendrite.hotkey().is_some());
    }

    #[test]
    fn test_dendrite_with_timeout() {
        let dendrite = Dendrite::new(None).with_timeout(Duration::from_secs(30));
        assert_eq!(dendrite.timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_dendrite_with_version() {
        let dendrite = Dendrite::new(None).with_version(200);
        assert_eq!(dendrite.version, 200);
    }

    #[test]
    fn test_dendrite_with_ip_and_port() {
        let dendrite = Dendrite::new(None).with_ip("192.168.1.1").with_port(8080);

        assert_eq!(dendrite.ip, Some("192.168.1.1".to_string()));
        assert_eq!(dendrite.port, Some(8080));
    }

    #[test]
    fn test_build_dendrite_info() {
        let keypair =
            sr25519::Pair::from_string("//Alice", None).expect("Failed to create test keypair");
        let dendrite = Dendrite::new(Some(keypair))
            .with_ip("10.0.0.1")
            .with_port(9000)
            .with_version(150);

        let info = dendrite.build_dendrite_info();

        assert_eq!(info.ip, Some("10.0.0.1".to_string()));
        assert_eq!(info.port, Some(9000));
        assert_eq!(info.version, Some(150));
        assert!(info.nonce.is_some());
        assert!(info.uuid.is_some());
        assert!(info.hotkey.is_some());
    }

    #[test]
    fn test_dendrite_clone() {
        let dendrite = Dendrite::new(None)
            .with_timeout(Duration::from_secs(20))
            .with_version(300);

        let cloned = dendrite.clone();

        assert_eq!(cloned.timeout, Duration::from_secs(20));
        assert_eq!(cloned.version, 300);
    }

    #[test]
    fn test_dendrite_default() {
        let dendrite = Dendrite::default();

        assert!(dendrite.keypair.is_none());
        assert_eq!(dendrite.timeout, Duration::from_secs(DEFAULT_TIMEOUT_SECS));
        assert_eq!(dendrite.version, DEFAULT_DENDRITE_VERSION);
    }

    #[tokio::test]
    async fn test_call_non_serving_axon() {
        let dendrite = Dendrite::new(None);
        let axon = create_non_serving_axon();
        let synapse = Synapse::new().with_name("Test");

        let result = dendrite.call(&axon, synapse).await;

        assert!(result.is_err());
        match result {
            Err(DendriteError::InvalidAxon(_)) => {}
            _ => panic!("Expected InvalidAxon error"),
        }
    }

    #[tokio::test]
    async fn test_call_many_empty() {
        let dendrite = Dendrite::new(None);
        let axons: Vec<AxonInfo> = vec![];
        let synapse = Synapse::new().with_name("Test");

        let results = dendrite.call_many(&axons, synapse).await;

        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_forward_with_timeout() {
        let dendrite = Dendrite::new(None);
        let axon = create_test_axon();
        let synapse = Synapse::new().with_name("Test");

        // This will fail to connect but should respect the timeout
        let results = dendrite
            .forward(&[axon], synapse, Some(Duration::from_millis(100)))
            .await;

        assert_eq!(results.len(), 1);
        // The result should be an error synapse (connection failed) or an error
        // We're just testing the API works correctly
    }
}
