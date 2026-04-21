//! Middleware chain for the Axon server.
//!
//! Order matches the Python SDK:
//! 1. VerificationMiddleware — signature check on the signing message
//! 2. BlacklistMiddleware — reject blacklisted dendrite hotkeys
//! 3. PriorityMiddleware — assign request priority based on stake
//! 4. BodyHashMiddleware — verify SHA3-256 body hash

use axum::body::Body;
use axum::extract::Request;
use axum::http::{HeaderMap, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use bittensor_synapse::hashing::sha3_256_hex;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Header keys used by the middleware chain.
pub mod headers {
    pub const NONCE: &str = "bt_header_dendrite_nonce";
    pub const DENDRITE_HOTKEY: &str = "bt_header_dendrite_hotkey";
    pub const AXON_HOTKEY: &str = "bt_header_axon_hotkey";
    pub const UUID: &str = "bt_header_dendrite_uuid";
    pub const COMPUTED_BODY_HASH: &str = "computed_body_hash";
    pub const SIGNATURE: &str = "bt_header_dendrite_signature";
    pub const REQUEST_PRIORITY: &str = "x-request-priority";
}

/// Shared middleware state passed through request extensions.
#[derive(Debug, Clone)]
pub struct MiddlewareState {
    /// Hotkey of this axon (used in verification).
    pub axon_hotkey: Option<String>,
    /// Set of blacklisted hotkeys.
    pub blacklist: Arc<RwLock<HashSet<String>>>,
    /// Static priority map: hotkey → priority value.
    pub priority_map: Arc<RwLock<HashMap<String, u32>>>,
}

/// Extracts the signing-message fields from request headers.
fn extract_signing_fields(
    headers: &HeaderMap,
) -> Result<(u64, String, String, String, String), StatusCode> {
    let nonce = headers
        .get(headers::NONCE)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let dendrite_hotkey = headers
        .get(headers::DENDRITE_HOTKEY)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let axon_hotkey =
        headers.get(headers::AXON_HOTKEY).and_then(|v| v.to_str().ok()).unwrap_or("").to_string();

    let uuid = headers.get(headers::UUID).and_then(|v| v.to_str().ok()).unwrap_or("").to_string();

    let body_hash = headers
        .get(headers::COMPUTED_BODY_HASH)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    Ok((nonce, dendrite_hotkey, axon_hotkey, uuid, body_hash))
}

/// 1. VerificationMiddleware: checks that the `bt_header_dendrite_signature`
///    header is a valid signature over the message
///    `"{nonce}.{dendrite_hotkey}.{axon_hotkey}.{uuid}.{body_hash}"`.
///
///    If `MiddlewareState::axon_hotkey` is `None` the check is skipped
///    (no key configured ⇒ accept all). Otherwise the signature header
///    must be present. In production the signature would be verified
///    against the dendrite hotkey's public key; here we verify that
///    the signing message was constructed and the header is non-empty.
pub async fn verification_middleware(request: Request, next: Next) -> Response {
    let state = request.extensions().get::<MiddlewareState>().cloned();

    if let Some(state) = state {
        if let Some(ref _axon_hk) = state.axon_hotkey {
            let headers = request.headers();
            if headers.get(headers::SIGNATURE).is_none() {
                return (StatusCode::UNAUTHORIZED, "missing signature header").into_response();
            }

            if let Ok((_nonce, _dhk, _ahk, _uuid, _bhash)) = extract_signing_fields(headers) {
                // In a full implementation we would verify the Sr25519
                // signature against the dendrite hotkey's public key here.
                // For now we just ensure the signing fields are present.
            } else {
                return (StatusCode::UNAUTHORIZED, "invalid signing fields").into_response();
            }
        }
    }

    next.run(request).await
}

/// 2. BlacklistMiddleware: rejects requests from blacklisted dendrite hotkeys.
pub async fn blacklist_middleware(request: Request, next: Next) -> Response {
    let state = request.extensions().get::<MiddlewareState>().cloned();

    if let Some(state) = state {
        let dendrite_hotkey = request
            .headers()
            .get(headers::DENDRITE_HOTKEY)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        if !dendrite_hotkey.is_empty() {
            let blacklist = state.blacklist.read().await;
            if blacklist.contains(&dendrite_hotkey) {
                return (StatusCode::FORBIDDEN, "hotkey is blacklisted").into_response();
            }
        }
    }

    next.run(request).await
}

/// 3. PriorityMiddleware: assigns a priority value to the request.
///
/// Looks up the dendrite hotkey in the static priority map. If not
/// found, defaults to priority 0. The value is injected as the
/// `x-request-priority` response header so downstream handlers can
/// inspect it.
pub async fn priority_middleware(mut request: Request, next: Next) -> Response {
    let state = request.extensions().get::<MiddlewareState>().cloned();

    let priority = if let Some(state) = state {
        let dendrite_hotkey = request
            .headers()
            .get(headers::DENDRITE_HOTKEY)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        if dendrite_hotkey.is_empty() {
            0u32
        } else {
            let map = state.priority_map.read().await;
            *map.get(&dendrite_hotkey).unwrap_or(&0)
        }
    } else {
        0u32
    };

    request.extensions_mut().insert(RequestPriority(priority));

    let mut response = next.run(request).await;
    response.headers_mut().insert(
        headers::REQUEST_PRIORITY,
        header::HeaderValue::from_str(&priority.to_string())
            .unwrap_or_else(|_| header::HeaderValue::from_static("0")),
    );
    response
}

/// Extension carrying the assigned priority value.
#[derive(Debug, Clone, Copy)]
pub struct RequestPriority(pub u32);

/// 4. BodyHashMiddleware: verifies that SHA3-256(body) matches the
///    `computed_body_hash` header. Returns 400 on mismatch.
pub async fn body_hash_middleware(request: Request, next: Next) -> Response {
    let expected_hash = request
        .headers()
        .get(headers::COMPUTED_BODY_HASH)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    if let Some(expected) = expected_hash {
        let (parts, body) = request.into_parts();
        let bytes = match axum::body::to_bytes(body, 1024 * 1024).await {
            Ok(b) => b,
            Err(_) => {
                return (StatusCode::BAD_REQUEST, "failed to read body").into_response();
            }
        };

        let actual = sha3_256_hex(&bytes);

        if !constant_time_eq(&actual, &expected) {
            let rebuilt = Request::from_parts(parts, Body::from(bytes));
            let _ = rebuilt;
            return (StatusCode::BAD_REQUEST, "body hash mismatch").into_response();
        }

        let rebuilt = Request::from_parts(parts, Body::from(bytes));
        return next.run(rebuilt).await;
    }

    next.run(request).await
}

/// Constant-time string comparison to prevent timing attacks on hash checks.
fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.bytes().zip(b.bytes()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use axum::body::Body;
    use axum::http::{Request as HttpRequest, StatusCode};
    use axum::middleware;
    use axum::routing::post;
    use tower::ServiceExt;

    async fn ok_handler() -> &'static str {
        "ok"
    }

    fn make_state() -> MiddlewareState {
        let mut blacklist = HashSet::new();
        blacklist.insert("5BadActor".to_string());

        let mut priority_map = HashMap::new();
        priority_map.insert("5HighPri".to_string(), 10);
        priority_map.insert("5MedPri".to_string(), 5);

        MiddlewareState {
            axon_hotkey: Some("5AxonKey".to_string()),
            blacklist: Arc::new(RwLock::new(blacklist)),
            priority_map: Arc::new(RwLock::new(priority_map)),
        }
    }

    #[tokio::test]
    async fn verification_rejects_missing_signature_when_hotkey_set() {
        let state = make_state();
        let app = Router::new()
            .route("/test", post(ok_handler))
            .layer(middleware::from_fn(verification_middleware))
            .layer(axum::Extension(state));

        let req = HttpRequest::builder().method("POST").uri("/test").body(Body::empty()).unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn verification_passes_with_signature_header() {
        let state = make_state();
        let app = Router::new()
            .route("/test", post(ok_handler))
            .layer(middleware::from_fn(verification_middleware))
            .layer(axum::Extension(state));

        let req = HttpRequest::builder()
            .method("POST")
            .uri("/test")
            .header(headers::SIGNATURE, "0xsomesig")
            .header(headers::NONCE, "42")
            .header(headers::DENDRITE_HOTKEY, "5Dendrite")
            .header(headers::AXON_HOTKEY, "5AxonKey")
            .header(headers::UUID, "uuid-123")
            .header(headers::COMPUTED_BODY_HASH, "abc")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn verification_skips_when_no_axon_hotkey() {
        let state = MiddlewareState { axon_hotkey: None, ..make_state() };
        let app = Router::new()
            .route("/test", post(ok_handler))
            .layer(middleware::from_fn(verification_middleware))
            .layer(axum::Extension(state));

        let req = HttpRequest::builder().method("POST").uri("/test").body(Body::empty()).unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn blacklist_rejects_blacklisted_hotkey() {
        let state = make_state();
        let app = Router::new()
            .route("/test", post(ok_handler))
            .layer(middleware::from_fn(blacklist_middleware))
            .layer(axum::Extension(state));

        let req = HttpRequest::builder()
            .method("POST")
            .uri("/test")
            .header(headers::DENDRITE_HOTKEY, "5BadActor")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn blacklist_passes_non_blacklisted_hotkey() {
        let state = make_state();
        let app = Router::new()
            .route("/test", post(ok_handler))
            .layer(middleware::from_fn(blacklist_middleware))
            .layer(axum::Extension(state));

        let req = HttpRequest::builder()
            .method("POST")
            .uri("/test")
            .header(headers::DENDRITE_HOTKEY, "5GoodActor")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn priority_assigns_known_priority() {
        let state = make_state();
        let app = Router::new()
            .route("/test", post(ok_handler))
            .layer(middleware::from_fn(priority_middleware))
            .layer(axum::Extension(state));

        let req = HttpRequest::builder()
            .method("POST")
            .uri("/test")
            .header(headers::DENDRITE_HOTKEY, "5HighPri")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let pri = resp
            .headers()
            .get(headers::REQUEST_PRIORITY)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("0");
        assert_eq!(pri, "10");
    }

    #[tokio::test]
    async fn priority_defaults_to_zero_for_unknown_hotkey() {
        let state = make_state();
        let app = Router::new()
            .route("/test", post(ok_handler))
            .layer(middleware::from_fn(priority_middleware))
            .layer(axum::Extension(state));

        let req = HttpRequest::builder()
            .method("POST")
            .uri("/test")
            .header(headers::DENDRITE_HOTKEY, "5Unknown")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        let pri = resp
            .headers()
            .get(headers::REQUEST_PRIORITY)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("0");
        assert_eq!(pri, "0");
    }

    #[tokio::test]
    async fn body_hash_rejects_mismatch() {
        let body = b"hello world";
        let wrong_hash = sha3_256_hex(b"not this body");

        let app = Router::new()
            .route("/test", post(ok_handler))
            .layer(middleware::from_fn(body_hash_middleware));

        let req = HttpRequest::builder()
            .method("POST")
            .uri("/test")
            .header(headers::COMPUTED_BODY_HASH, wrong_hash)
            .body(Body::from(body.to_vec()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn body_hash_passes_matching_hash() {
        let body = b"hello world";
        let correct_hash = sha3_256_hex(body);

        let app = Router::new()
            .route("/test", post(ok_handler))
            .layer(middleware::from_fn(body_hash_middleware));

        let req = HttpRequest::builder()
            .method("POST")
            .uri("/test")
            .header(headers::COMPUTED_BODY_HASH, correct_hash)
            .body(Body::from(body.to_vec()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn body_hash_passes_when_no_hash_header() {
        let app = Router::new()
            .route("/test", post(ok_handler))
            .layer(middleware::from_fn(body_hash_middleware));

        let req = HttpRequest::builder()
            .method("POST")
            .uri("/test")
            .body(Body::from(b"any body".to_vec()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[test]
    fn constant_time_eq_same_strings() {
        assert!(constant_time_eq("abc", "abc"));
    }

    #[test]
    fn constant_time_eq_different_strings() {
        assert!(!constant_time_eq("abc", "abd"));
        assert!(!constant_time_eq("abc", "ab"));
    }

    #[tokio::test]
    async fn full_middleware_chain_passes_valid_request() {
        let state = make_state();
        let body = b"test body";
        let body_hash = sha3_256_hex(body);

        let app = Router::new()
            .route("/test", post(ok_handler))
            .layer(middleware::from_fn(body_hash_middleware))
            .layer(middleware::from_fn(priority_middleware))
            .layer(middleware::from_fn(blacklist_middleware))
            .layer(middleware::from_fn(verification_middleware))
            .layer(axum::Extension(state));

        let req = HttpRequest::builder()
            .method("POST")
            .uri("/test")
            .header(headers::SIGNATURE, "0xsomesig")
            .header(headers::NONCE, "42")
            .header(headers::DENDRITE_HOTKEY, "5HighPri")
            .header(headers::AXON_HOTKEY, "5AxonKey")
            .header(headers::UUID, "uuid-123")
            .header(headers::COMPUTED_BODY_HASH, body_hash)
            .body(Body::from(body.to_vec()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn full_chain_rejects_blacklisted_after_verification() {
        let state = make_state();
        let body = b"test body";
        let body_hash = sha3_256_hex(body);

        let app = Router::new()
            .route("/test", post(ok_handler))
            .layer(middleware::from_fn(body_hash_middleware))
            .layer(middleware::from_fn(priority_middleware))
            .layer(middleware::from_fn(blacklist_middleware))
            .layer(middleware::from_fn(verification_middleware))
            .layer(axum::Extension(state));

        let req = HttpRequest::builder()
            .method("POST")
            .uri("/test")
            .header(headers::SIGNATURE, "0xsomesig")
            .header(headers::NONCE, "42")
            .header(headers::DENDRITE_HOTKEY, "5BadActor")
            .header(headers::AXON_HOTKEY, "5AxonKey")
            .header(headers::UUID, "uuid-123")
            .header(headers::COMPUTED_BODY_HASH, body_hash)
            .body(Body::from(body.to_vec()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }
}
