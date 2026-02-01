//! Response handling for Dendrite HTTP responses
//!
//! This module provides types and utilities for processing responses
//! from Axon servers, including status interpretation and synapse reconstruction.

use crate::dendrite::request::{headers_to_synapse, RequestError};
use crate::types::{Synapse, TerminalInfo};
use http::header::HeaderMap;
use thiserror::Error;

/// Errors that can occur during response processing
#[derive(Debug, Error)]
pub enum ResponseError {
    #[error("HTTP error: status {status}")]
    HttpError { status: u16 },
    #[error("Timeout")]
    Timeout,
    #[error("Deserialization error: {0}")]
    Deserialization(String),
    #[error("Request error: {0}")]
    Request(#[from] RequestError),
    #[error("Network error: {0}")]
    Network(String),
}

/// A response from a Dendrite HTTP request
#[derive(Debug, Clone)]
pub struct DendriteResponse {
    /// HTTP status code
    pub status: u16,
    /// Response headers
    pub headers: HeaderMap,
    /// Response body
    pub body: Vec<u8>,
    /// Time taken to process the request (in seconds)
    pub process_time: f64,
}

impl DendriteResponse {
    /// Create a new DendriteResponse
    ///
    /// # Arguments
    ///
    /// * `status` - HTTP status code
    /// * `headers` - Response headers
    /// * `body` - Response body bytes
    /// * `process_time` - Processing time in seconds
    pub fn new(status: u16, headers: HeaderMap, body: Vec<u8>, process_time: f64) -> Self {
        Self {
            status,
            headers,
            body,
            process_time,
        }
    }

    /// Convert the response into a Synapse
    ///
    /// Parses headers and body to reconstruct the full synapse with
    /// terminal information and response data.
    ///
    /// # Returns
    ///
    /// The reconstructed Synapse or an error
    pub fn into_synapse(self) -> Result<Synapse, ResponseError> {
        let mut synapse = headers_to_synapse(&self.headers, &self.body)?;

        // Update the dendrite terminal info with response status
        if let Some(ref mut dendrite) = synapse.dendrite {
            dendrite.status_code = Some(self.status as i32);
            dendrite.process_time = Some(self.process_time);
        } else {
            synapse.dendrite = Some(TerminalInfo {
                status_code: Some(self.status as i32),
                process_time: Some(self.process_time),
                ..Default::default()
            });
        }

        Ok(synapse)
    }

    /// Check if the response indicates success (2xx status code)
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// Check if the response indicates a timeout (408 or 504)
    pub fn is_timeout(&self) -> bool {
        self.status == 408 || self.status == 504
    }

    /// Check if the response indicates a client error (4xx)
    pub fn is_client_error(&self) -> bool {
        (400..500).contains(&self.status)
    }

    /// Check if the response indicates a server error (5xx)
    pub fn is_server_error(&self) -> bool {
        (500..600).contains(&self.status)
    }

    /// Get the axon status code from headers if present
    pub fn axon_status_code(&self) -> Option<i32> {
        self.headers
            .get("bt_header_axon_status_code")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok())
    }

    /// Get the axon status message from headers if present
    pub fn axon_status_message(&self) -> Option<String> {
        self.headers
            .get("bt_header_axon_status_message")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
    }

    /// Get the axon process time from headers if present
    pub fn axon_process_time(&self) -> Option<f64> {
        self.headers
            .get("bt_header_axon_process_time")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok())
    }

    /// Get the body as a string (if valid UTF-8)
    pub fn body_as_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.body).ok()
    }

    /// Deserialize the body as JSON into a specific type
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, ResponseError> {
        serde_json::from_slice(&self.body)
            .map_err(|e| ResponseError::Deserialization(e.to_string()))
    }
}

/// Build a failed response synapse for error conditions
///
/// This is used when the request fails before reaching the axon,
/// such as connection errors or timeouts.
///
/// # Arguments
///
/// * `original` - The original synapse that was sent
/// * `status_code` - Error status code
/// * `message` - Error message
/// * `process_time` - Time taken before failure
///
/// # Returns
///
/// A synapse marked with failure status
pub fn build_error_synapse(
    original: &Synapse,
    status_code: i32,
    message: &str,
    process_time: f64,
) -> Synapse {
    let mut synapse = original.clone();

    // Update dendrite terminal info with error status
    let dendrite = synapse.dendrite.get_or_insert_with(TerminalInfo::default);
    dendrite.status_code = Some(status_code);
    dendrite.status_message = Some(message.to_string());
    dendrite.process_time = Some(process_time);

    synapse
}

/// Standard status codes used in Bittensor protocol
pub mod status_codes {
    /// Request successful
    pub const SUCCESS: i32 = 200;
    /// Request successful, no content
    pub const NO_CONTENT: i32 = 204;
    /// Bad request (malformed)
    pub const BAD_REQUEST: i32 = 400;
    /// Unauthorized (invalid signature)
    pub const UNAUTHORIZED: i32 = 401;
    /// Forbidden (blacklisted)
    pub const FORBIDDEN: i32 = 403;
    /// Not found (endpoint doesn't exist)
    pub const NOT_FOUND: i32 = 404;
    /// Request timeout
    pub const TIMEOUT: i32 = 408;
    /// Too many requests (rate limited)
    pub const TOO_MANY_REQUESTS: i32 = 429;
    /// Internal server error
    pub const INTERNAL_ERROR: i32 = 500;
    /// Service unavailable
    pub const SERVICE_UNAVAILABLE: i32 = 503;
    /// Gateway timeout
    pub const GATEWAY_TIMEOUT: i32 = 504;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dendrite_response_is_success() {
        let response = DendriteResponse::new(200, HeaderMap::new(), vec![], 0.5);
        assert!(response.is_success());
        assert!(!response.is_timeout());
        assert!(!response.is_client_error());
        assert!(!response.is_server_error());
    }

    #[test]
    fn test_dendrite_response_is_timeout() {
        let response_408 = DendriteResponse::new(408, HeaderMap::new(), vec![], 12.0);
        assert!(response_408.is_timeout());
        assert!(!response_408.is_success());

        let response_504 = DendriteResponse::new(504, HeaderMap::new(), vec![], 12.0);
        assert!(response_504.is_timeout());
    }

    #[test]
    fn test_dendrite_response_is_client_error() {
        let response = DendriteResponse::new(404, HeaderMap::new(), vec![], 0.1);
        assert!(response.is_client_error());
        assert!(!response.is_success());
        assert!(!response.is_server_error());
    }

    #[test]
    fn test_dendrite_response_is_server_error() {
        let response = DendriteResponse::new(500, HeaderMap::new(), vec![], 0.1);
        assert!(response.is_server_error());
        assert!(!response.is_success());
        assert!(!response.is_client_error());
    }

    #[test]
    fn test_dendrite_response_json() {
        use serde::Deserialize;

        #[derive(Deserialize, Debug, PartialEq)]
        struct TestData {
            value: i32,
        }

        let body = br#"{"value": 42}"#;
        let response = DendriteResponse::new(200, HeaderMap::new(), body.to_vec(), 0.1);

        let data: TestData = response.json().unwrap();
        assert_eq!(data.value, 42);
    }

    #[test]
    fn test_dendrite_response_body_as_str() {
        let body = b"Hello, world!";
        let response = DendriteResponse::new(200, HeaderMap::new(), body.to_vec(), 0.1);

        assert_eq!(response.body_as_str(), Some("Hello, world!"));
    }

    #[test]
    fn test_build_error_synapse() {
        let original = Synapse::new().with_name("TestSynapse");
        let error_synapse = build_error_synapse(&original, 408, "Request timeout", 12.5);

        assert_eq!(error_synapse.name, Some("TestSynapse".to_string()));

        let dendrite = error_synapse.dendrite.unwrap();
        assert_eq!(dendrite.status_code, Some(408));
        assert_eq!(dendrite.status_message, Some("Request timeout".to_string()));
        assert_eq!(dendrite.process_time, Some(12.5));
    }

    #[test]
    fn test_into_synapse() {
        let mut headers = HeaderMap::new();
        headers.insert("name", "ConvertedSynapse".parse().unwrap());
        headers.insert("bt_header_timeout", "15.0".parse().unwrap());
        headers.insert("bt_header_axon_status_code", "200".parse().unwrap());

        let body = br#"{}"#;
        let response = DendriteResponse::new(200, headers, body.to_vec(), 0.5);

        let synapse = response.into_synapse().unwrap();
        assert_eq!(synapse.name, Some("ConvertedSynapse".to_string()));

        let dendrite = synapse.dendrite.unwrap();
        assert_eq!(dendrite.status_code, Some(200));
        assert_eq!(dendrite.process_time, Some(0.5));
    }

    #[test]
    fn test_axon_header_accessors() {
        let mut headers = HeaderMap::new();
        headers.insert("bt_header_axon_status_code", "200".parse().unwrap());
        headers.insert("bt_header_axon_status_message", "OK".parse().unwrap());
        headers.insert("bt_header_axon_process_time", "0.123".parse().unwrap());

        let response = DendriteResponse::new(200, headers, vec![], 0.2);

        assert_eq!(response.axon_status_code(), Some(200));
        assert_eq!(response.axon_status_message(), Some("OK".to_string()));
        assert_eq!(response.axon_process_time(), Some(0.123));
    }
}
