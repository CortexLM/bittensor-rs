//! Middleware for the Axon HTTP server
//!
//! This module provides middleware functions for request processing including:
//! - Blacklist checking
//! - Priority queuing
//! - Signature verification
//! - Request logging

use crate::axon::handlers::{build_error_response, status_codes, status_messages};
use crate::axon::server::AxonState;
use crate::dendrite::request::header_names;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Blacklist middleware - reject requests from blacklisted hotkeys
///
/// Checks if the dendrite's hotkey is in the blacklist and rejects
/// the request with a 403 Forbidden status if so.
pub async fn blacklist_middleware(
    State(state): State<Arc<RwLock<AxonState>>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let start_time = Instant::now();

    // Extract dendrite hotkey from headers
    let dendrite_hotkey = req
        .headers()
        .get(header_names::DENDRITE_HOTKEY)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Check blacklist
    let state_read = state.read().await;

    // Extract IP address from request.
    // Only trust proxy headers (X-Forwarded-For, X-Real-IP) if trust_proxy_headers is enabled.
    // This prevents IP blacklist bypass via header spoofing when not behind a trusted proxy.
    let client_ip = if state_read.trust_proxy_headers {
        req.headers()
            .get("x-forwarded-for")
            .or_else(|| req.headers().get("x-real-ip"))
            .and_then(|v| v.to_str().ok())
            .and_then(|s| {
                // X-Forwarded-For may contain multiple IPs, take the first (original client)
                s.split(',').next().map(|ip| ip.trim().to_string())
            })
    } else {
        // When trust_proxy_headers is disabled, we don't use proxy headers.
        // The actual client IP would be obtained from the connection itself,
        // but that's not available in this middleware context without ConnectInfo.
        // For now, return None to avoid trusting spoofable headers.
        None
    };

    // Check hotkey blacklist
    if let Some(ref hotkey) = dendrite_hotkey {
        if state_read.blacklist.contains(hotkey) {
            warn!("Blocked blacklisted hotkey: {}", hotkey);
            let process_time = start_time.elapsed().as_secs_f64();
            return build_error_response(
                &state_read.axon_hotkey,
                StatusCode::FORBIDDEN,
                status_codes::FORBIDDEN,
                status_messages::FORBIDDEN,
                process_time,
            );
        }
    }

    // Check IP blacklist
    if let Some(ref ip) = client_ip {
        if state_read.ip_blacklist.contains(ip) {
            warn!("Blocked blacklisted IP: {}", ip);
            let process_time = start_time.elapsed().as_secs_f64();
            return build_error_response(
                &state_read.axon_hotkey,
                StatusCode::FORBIDDEN,
                status_codes::FORBIDDEN,
                status_messages::FORBIDDEN,
                process_time,
            );
        }
    }

    // Check custom blacklist function
    if let Some(ref blacklist_fn) = state_read.blacklist_fn {
        if let Some(ref hotkey) = dendrite_hotkey {
            let synapse_name = req
                .headers()
                .get(header_names::NAME)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("unknown");

            if blacklist_fn(hotkey, synapse_name) {
                warn!(
                    "Blocked by custom blacklist function: hotkey={}, synapse={}",
                    hotkey, synapse_name
                );
                let process_time = start_time.elapsed().as_secs_f64();
                return build_error_response(
                    &state_read.axon_hotkey,
                    StatusCode::FORBIDDEN,
                    status_codes::FORBIDDEN,
                    status_messages::FORBIDDEN,
                    process_time,
                );
            }
        }
    }

    drop(state_read);
    next.run(req).await
}

/// Priority middleware - track request priority
///
/// Extracts the priority for this request based on the dendrite's hotkey
/// and adds it to the request extensions for later use.
pub async fn priority_middleware(
    State(state): State<Arc<RwLock<AxonState>>>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    // Extract dendrite hotkey from headers
    let dendrite_hotkey = req
        .headers()
        .get(header_names::DENDRITE_HOTKEY)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Get priority for this hotkey
    let priority = if let Some(ref hotkey) = dendrite_hotkey {
        let state_read = state.read().await;

        // Check custom priority function first
        if let Some(ref priority_fn) = state_read.priority_fn {
            let synapse_name = req
                .headers()
                .get(header_names::NAME)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("unknown");
            priority_fn(hotkey, synapse_name)
        } else {
            // Fall back to priority list
            state_read.priority_list.get(hotkey).copied().unwrap_or(0.0)
        }
    } else {
        0.0
    };

    // Add priority to request extensions
    req.extensions_mut().insert(RequestPriority(priority));

    next.run(req).await
}

/// Request priority extension
#[derive(Debug, Clone, Copy)]
pub struct RequestPriority(pub f32);

/// Verification middleware - verify request signatures
///
/// Verifies the dendrite's signature on the request if signature
/// verification is enabled in the state.
pub async fn verify_middleware(
    State(state): State<Arc<RwLock<AxonState>>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let start_time = Instant::now();
    let state_read = state.read().await;

    // Skip verification if disabled
    if !state_read.verify_signatures {
        drop(state_read);
        return next.run(req).await;
    }

    let axon_hotkey = state_read.axon_hotkey.clone();

    // Check custom verify function first
    if let Some(ref verify_fn) = state_read.verify_fn {
        let synapse_name = req
            .headers()
            .get(header_names::NAME)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown");

        if !verify_fn(synapse_name) {
            debug!(
                "Request failed custom verification for synapse: {}",
                synapse_name
            );
            let process_time = start_time.elapsed().as_secs_f64();
            return build_error_response(
                &axon_hotkey,
                StatusCode::UNAUTHORIZED,
                status_codes::UNAUTHORIZED,
                status_messages::UNAUTHORIZED,
                process_time,
            );
        }
    }

    drop(state_read);

    // For full signature verification, we need the body
    // This is done in the handler since we need to consume the body
    next.run(req).await
}

/// Logging middleware - log request details
///
/// Logs incoming requests and their processing time.
pub async fn logging_middleware(req: Request<Body>, next: Next) -> Response {
    let start_time = Instant::now();

    let method = req.method().clone();
    let uri = req.uri().clone();
    let synapse_name = req
        .headers()
        .get(header_names::NAME)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    let dendrite_hotkey = req
        .headers()
        .get(header_names::DENDRITE_HOTKEY)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("anonymous")
        .to_string();

    debug!(
        "Incoming request: {} {} synapse={} from={}",
        method, uri, synapse_name, dendrite_hotkey
    );

    let response = next.run(req).await;

    let status = response.status();
    let process_time = start_time.elapsed().as_secs_f64();

    info!(
        "Request completed: {} {} synapse={} from={} status={} time={:.3}s",
        method, uri, synapse_name, dendrite_hotkey, status, process_time
    );

    response
}

/// Request counter middleware - track request counts
///
/// Increments the request counter in the axon state.
pub async fn counter_middleware(
    State(state): State<Arc<RwLock<AxonState>>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    // Increment request counter
    {
        let mut state_write = state.write().await;
        state_write.request_count += 1;
        state_write.total_requests += 1;
    }

    let response = next.run(req).await;

    // Decrement active request counter
    {
        let mut state_write = state.write().await;
        state_write.request_count = state_write.request_count.saturating_sub(1);
    }

    response
}

/// Timeout middleware - enforce request timeouts
///
/// Extracts the timeout from request headers and enforces it.
/// Uses the default timeout if not specified.
pub async fn timeout_middleware(
    State(state): State<Arc<RwLock<AxonState>>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let start_time = Instant::now();

    // Extract timeout from headers or use default
    let timeout_secs = req
        .headers()
        .get(header_names::TIMEOUT)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(12.0);

    // Clamp timeout to reasonable bounds: min 1 second, max 5 minutes
    let timeout_secs = timeout_secs.clamp(1.0, 300.0);

    let timeout_duration = std::time::Duration::from_secs_f64(timeout_secs);

    // Create a future that completes when the request is done or times out
    let response_future = next.run(req);

    match tokio::time::timeout(timeout_duration, response_future).await {
        Ok(response) => response,
        Err(_) => {
            let state_read = state.read().await;
            let process_time = start_time.elapsed().as_secs_f64();
            warn!("Request timed out after {:.3}s", process_time);
            build_error_response(
                &state_read.axon_hotkey,
                StatusCode::REQUEST_TIMEOUT,
                status_codes::TIMEOUT,
                status_messages::TIMEOUT,
                process_time,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    fn create_test_state() -> Arc<RwLock<AxonState>> {
        Arc::new(RwLock::new(AxonState {
            request_count: 0,
            total_requests: 0,
            blacklist: HashSet::new(),
            ip_blacklist: HashSet::new(),
            priority_list: HashMap::new(),
            axon_hotkey: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".to_string(),
            verify_signatures: true,
            trust_proxy_headers: false,
            blacklist_fn: None,
            priority_fn: None,
            verify_fn: None,
        }))
    }

    #[test]
    fn test_request_priority() {
        let priority = RequestPriority(0.75);
        assert_eq!(priority.0, 0.75);
    }

    #[tokio::test]
    async fn test_state_counter() {
        let state = create_test_state();

        // Simulate increment
        {
            let mut state_write = state.write().await;
            state_write.request_count += 1;
            state_write.total_requests += 1;
        }

        let state_read = state.read().await;
        assert_eq!(state_read.request_count, 1);
        assert_eq!(state_read.total_requests, 1);
    }

    #[tokio::test]
    async fn test_blacklist_check() {
        let state = create_test_state();

        // Add a hotkey to blacklist
        {
            let mut state_write = state.write().await;
            state_write
                .blacklist
                .insert("blacklisted_hotkey".to_string());
        }

        let state_read = state.read().await;
        assert!(state_read.blacklist.contains("blacklisted_hotkey"));
        assert!(!state_read.blacklist.contains("allowed_hotkey"));
    }

    #[tokio::test]
    async fn test_priority_list() {
        let state = create_test_state();

        // Add priorities
        {
            let mut state_write = state.write().await;
            state_write
                .priority_list
                .insert("high_priority".to_string(), 1.0);
            state_write
                .priority_list
                .insert("low_priority".to_string(), 0.1);
        }

        let state_read = state.read().await;
        assert_eq!(state_read.priority_list.get("high_priority"), Some(&1.0));
        assert_eq!(state_read.priority_list.get("low_priority"), Some(&0.1));
        assert_eq!(state_read.priority_list.get("unknown"), None);
    }
}
