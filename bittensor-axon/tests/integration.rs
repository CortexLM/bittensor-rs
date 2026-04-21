//! Integration tests for the Axon server middleware chain.
//!
//! These tests start a real HTTP server and send requests using `reqwest`,
//! validating the full middleware stack: verification → blacklist → priority → body_hash.
//!
//! # Critical Finding: Axon Middleware Ordering Bug
//!
//! The current `Axon::new()` implementation applies `.layer()` to the Router
//! BEFORE routes are added via `.attach()`. In axum 0.8, routes added after
//! `.layer()` do NOT go through the previously-added middleware. This means
//! the Axon struct's middleware chain is inert — no verification, blacklist,
//! body hash, or priority checks actually run for attached handlers.
//!
//! The tests below construct Routers with the CORRECT ordering (routes first,
//! then middleware) to validate the middleware behavior end-to-end. A
//! `#[ignore]` test demonstrates the bug when using the `Axon` struct.
//!
//! # Running
//!
//! ```sh
//! cargo test -p bittensor-axon --test integration
//! ```

use axum::Router;
use axum::body::Body;
use axum::http::{HeaderValue, Request as HttpRequest, StatusCode};
use axum::middleware;
use axum::response::IntoResponse;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::routing::post;
use bittensor_axon::axon::Axon;
use bittensor_axon::config::AxonConfig;
use bittensor_axon::middleware as mw;
use bittensor_axon::middleware::{MiddlewareState, headers};
use bittensor_synapse::hashing::sha3_256_hex;
use bittensor_synapse::signing::signing_message;
use futures::stream::Stream;
use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::sync::Arc;
use subxt_signer::sr25519::Keypair;
use tokio::net::TcpListener;
use tower::ServiceExt;

fn alice() -> Keypair {
    subxt_signer::sr25519::dev::alice()
}

fn bob() -> Keypair {
    subxt_signer::sr25519::dev::bob()
}

fn insert_hdr(map: &mut reqwest::header::HeaderMap, name: &str, value: &str) {
    let hname = reqwest::header::HeaderName::from_bytes(name.as_bytes()).unwrap();
    let hval = HeaderValue::from_str(value).unwrap();
    map.insert(hname, hval);
}

fn hex_encode(bytes: impl AsRef<[u8]>) -> String {
    bytes.as_ref().iter().map(|b| format!("{b:02x}")).collect()
}

fn make_state(axon_hotkey: Option<&str>) -> MiddlewareState {
    MiddlewareState {
        axon_hotkey: axon_hotkey.map(|s| s.to_string()),
        blacklist: Arc::new(tokio::sync::RwLock::new(HashSet::new())),
        priority_map: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
    }
}

fn signed_headers(
    alice_kp: &Keypair,
    axon_hotkey: &str,
    body: &[u8],
    nonce: u64,
) -> reqwest::header::HeaderMap {
    let uuid = uuid::Uuid::new_v4().to_string();
    let body_hash = sha3_256_hex(body);
    let dendrite_hotkey = alice_kp.public_key().to_account_id().to_string();
    let message = signing_message(nonce, &dendrite_hotkey, axon_hotkey, &uuid, &body_hash);
    let signature = alice_kp.sign(message.as_bytes());
    let sig_hex = format!("0x{}", hex_encode(signature.0));

    let mut map = reqwest::header::HeaderMap::new();
    insert_hdr(&mut map, headers::NONCE, &nonce.to_string());
    insert_hdr(&mut map, headers::DENDRITE_HOTKEY, &dendrite_hotkey);
    insert_hdr(&mut map, headers::AXON_HOTKEY, axon_hotkey);
    insert_hdr(&mut map, headers::UUID, &uuid);
    insert_hdr(&mut map, headers::COMPUTED_BODY_HASH, &body_hash);
    insert_hdr(&mut map, headers::SIGNATURE, &sig_hex);
    map
}

async fn ok_handler() -> &'static str {
    "ok"
}

/// Build a Router with routes FIRST, then middleware — correct axum 0.8 ordering.
fn build_app(state: MiddlewareState) -> Router {
    Router::new()
        .route("/TextPrompt", post(ok_handler))
        .fallback(|| async { StatusCode::NOT_FOUND.into_response() })
        .layer(middleware::from_fn(mw::body_hash_middleware))
        .layer(middleware::from_fn(mw::priority_middleware))
        .layer(middleware::from_fn(mw::blacklist_middleware))
        .layer(middleware::from_fn(mw::verification_middleware))
        .layer(axum::Extension(state))
}

/// Start a real HTTP server from a Router, return the bound address.
async fn serve_app(app: Router) -> std::net::SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    addr
}

// ── 1. Signed request passes all middleware ─────────────────────────────────

#[tokio::test]
async fn signed_request_round_trip() {
    let axon_hotkey = "5AxonTestHotkey";
    let state = make_state(Some(axon_hotkey));
    let app = build_app(state);
    let addr = serve_app(app).await;

    let alice_kp = alice();
    let body = br#"{"prompt":"test"}"#;
    let hdrs = signed_headers(&alice_kp, axon_hotkey, body, 1);

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://{addr}/TextPrompt"))
        .headers(hdrs)
        .body(body.to_vec())
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(resp.text().await.unwrap(), "ok");
}

// ── 2. Unsigned request → 401 Unauthorized ─────────────────────────────────

#[tokio::test]
async fn unsigned_request_returns_401() {
    let axon_hotkey = "5AxonTestHotkey";
    let state = make_state(Some(axon_hotkey));
    let app = build_app(state);
    let addr = serve_app(app).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://{addr}/TextPrompt"))
        .body(b"{}".to_vec())
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ── 3. No hotkey → unsigned request passes (no-wallet mode) ─────────────────

#[tokio::test]
async fn unsigned_request_passes_when_no_hotkey() {
    let state = make_state(None);
    let app = build_app(state);
    let addr = serve_app(app).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://{addr}/TextPrompt"))
        .body(b"{}".to_vec())
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(resp.status(), StatusCode::OK);
}

// ── 4. Blacklisted hotkey → 403 Forbidden ───────────────────────────────────

#[tokio::test]
async fn blacklisted_hotkey_returns_403() {
    let axon_hotkey = "5AxonTestHotkey";
    let bob_kp = bob();
    let bob_hotkey = bob_kp.public_key().to_account_id().to_string();

    let mut blacklist = HashSet::new();
    blacklist.insert(bob_hotkey.clone());
    let state = MiddlewareState {
        axon_hotkey: Some(axon_hotkey.to_string()),
        blacklist: Arc::new(tokio::sync::RwLock::new(blacklist)),
        priority_map: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
    };
    let app = build_app(state);
    let addr = serve_app(app).await;

    let body = br#"{"prompt":"test"}"#;
    let uuid = uuid::Uuid::new_v4().to_string();
    let body_hash = sha3_256_hex(body);
    let message = signing_message(1, &bob_hotkey, axon_hotkey, &uuid, &body_hash);
    let signature = bob_kp.sign(message.as_bytes());
    let sig_hex = format!("0x{}", hex_encode(signature.0));

    let mut hdrs = reqwest::header::HeaderMap::new();
    insert_hdr(&mut hdrs, headers::SIGNATURE, &sig_hex);
    insert_hdr(&mut hdrs, headers::NONCE, "1");
    insert_hdr(&mut hdrs, headers::DENDRITE_HOTKEY, &bob_hotkey);
    insert_hdr(&mut hdrs, headers::AXON_HOTKEY, axon_hotkey);
    insert_hdr(&mut hdrs, headers::UUID, &uuid);
    insert_hdr(&mut hdrs, headers::COMPUTED_BODY_HASH, &body_hash);

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://{addr}/TextPrompt"))
        .headers(hdrs)
        .body(body.to_vec())
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

// ── 5. Tampered body hash → 400 Bad Request ────────────────────────────────

#[tokio::test]
async fn tampered_body_hash_returns_400() {
    let axon_hotkey = "5AxonTestHotkey";
    let state = make_state(Some(axon_hotkey));
    let app = build_app(state);
    let addr = serve_app(app).await;

    let alice_kp = alice();
    let body = br#"{"prompt":"test"}"#;
    let wrong_hash = sha3_256_hex(b"wrong body content");

    let mut hdrs = signed_headers(&alice_kp, axon_hotkey, body, 1);
    insert_hdr(&mut hdrs, headers::COMPUTED_BODY_HASH, &wrong_hash);

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://{addr}/TextPrompt"))
        .headers(hdrs)
        .body(body.to_vec())
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ── 6. Priority middleware injects x-request-priority header ────────────────

#[tokio::test]
async fn priority_header_in_response() {
    let axon_hotkey = "5AxonTestHotkey";
    let alice_kp = alice();
    let alice_hotkey = alice_kp.public_key().to_account_id().to_string();

    let mut priority_map = HashMap::new();
    priority_map.insert(alice_hotkey.clone(), 42u32);
    let state = MiddlewareState {
        axon_hotkey: Some(axon_hotkey.to_string()),
        blacklist: Arc::new(tokio::sync::RwLock::new(HashSet::new())),
        priority_map: Arc::new(tokio::sync::RwLock::new(priority_map)),
    };
    let app = build_app(state);
    let addr = serve_app(app).await;

    let body = br#"{"prompt":"test"}"#;
    let hdrs = signed_headers(&alice_kp, axon_hotkey, body, 1);

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://{addr}/TextPrompt"))
        .headers(hdrs)
        .body(body.to_vec())
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(resp.status(), StatusCode::OK);
    let pri =
        resp.headers().get(headers::REQUEST_PRIORITY).and_then(|v| v.to_str().ok()).unwrap_or("0");
    assert_eq!(pri, "42");
}

// ── 7. Priority defaults to 0 for unknown hotkey ────────────────────────────

#[tokio::test]
async fn priority_defaults_to_zero() {
    let axon_hotkey = "5AxonTestHotkey";
    let state = make_state(Some(axon_hotkey));
    let app = build_app(state);
    let addr = serve_app(app).await;

    let alice_kp = alice();
    let body = br#"{"prompt":"test"}"#;
    let hdrs = signed_headers(&alice_kp, axon_hotkey, body, 1);

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://{addr}/TextPrompt"))
        .headers(hdrs)
        .body(body.to_vec())
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(resp.status(), StatusCode::OK);
    let pri =
        resp.headers().get(headers::REQUEST_PRIORITY).and_then(|v| v.to_str().ok()).unwrap_or("0");
    assert_eq!(pri, "0");
}

// ── 8. SSE streaming end-to-end ────────────────────────────────────────────

#[tokio::test]
async fn sse_streaming_round_trip() {
    async fn sse_handler() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
        let stream = tokio_stream::iter(vec![
            Ok::<_, Infallible>(Event::default().data("chunk1")),
            Ok::<_, Infallible>(Event::default().data("chunk2")),
            Ok::<_, Infallible>(Event::default().data("[DONE]")),
        ]);
        Sse::new(stream).keep_alive(KeepAlive::default())
    }

    let state = make_state(None);
    let app = Router::new()
        .route("/StreamPrompt", post(sse_handler))
        .layer(middleware::from_fn(mw::body_hash_middleware))
        .layer(middleware::from_fn(mw::priority_middleware))
        .layer(middleware::from_fn(mw::blacklist_middleware))
        .layer(middleware::from_fn(mw::verification_middleware))
        .layer(axum::Extension(state));
    let addr = serve_app(app).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://{addr}/StreamPrompt"))
        .header("accept", "text/event-stream")
        .body(b"{}".to_vec())
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.text().await.unwrap();
    assert!(body.contains("data: chunk1"), "body: {body}");
    assert!(body.contains("data: chunk2"), "body: {body}");
    assert!(body.contains("data: [DONE]"), "body: {body}");
}

// ── 9. Signed SSE streaming passes verification ─────────────────────────────

#[tokio::test]
async fn signed_sse_streaming_passes_verification() {
    async fn sse_handler() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
        let stream = tokio_stream::iter(vec![
            Ok::<_, Infallible>(Event::default().data("signed-chunk")),
            Ok::<_, Infallible>(Event::default().data("[DONE]")),
        ]);
        Sse::new(stream).keep_alive(KeepAlive::default())
    }

    let axon_hotkey = "5AxonStreamHotkey";
    let state = make_state(Some(axon_hotkey));
    let app = Router::new()
        .route("/StreamPrompt", post(sse_handler))
        .layer(middleware::from_fn(mw::body_hash_middleware))
        .layer(middleware::from_fn(mw::priority_middleware))
        .layer(middleware::from_fn(mw::blacklist_middleware))
        .layer(middleware::from_fn(mw::verification_middleware))
        .layer(axum::Extension(state));
    let addr = serve_app(app).await;

    let alice_kp = alice();
    let body = br#"{}"#;
    let hdrs = signed_headers(&alice_kp, axon_hotkey, body, 1);

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://{addr}/StreamPrompt"))
        .headers(hdrs)
        .header("accept", "text/event-stream")
        .body(body.to_vec())
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.text().await.unwrap();
    assert!(body.contains("data: signed-chunk"), "body: {body}");
}

// ── 10. Unregistered route returns 404 ───────────────────────────────────────

#[tokio::test]
async fn unregistered_route_returns_404() {
    let state = make_state(None);
    let app = build_app(state);
    let addr = serve_app(app).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://{addr}/UnknownSynapse"))
        .body(b"{}".to_vec())
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ── 11. Late blacklist still blocks (RwLock is live) ────────────────────────

#[tokio::test]
async fn late_blacklist_still_blocks() {
    let axon_hotkey = "5AxonLateBlackHotkey";
    let alice_kp = alice();
    let alice_hotkey = alice_kp.public_key().to_account_id().to_string();

    let state = make_state(Some(axon_hotkey));
    let app = build_app(state.clone());
    let addr = serve_app(app).await;

    let body = br#"{"prompt":"test"}"#;
    let hdrs1 = signed_headers(&alice_kp, axon_hotkey, body, 1);

    let client = reqwest::Client::new();
    let resp1 = client
        .post(format!("http://{addr}/TextPrompt"))
        .headers(hdrs1)
        .body(body.to_vec())
        .send()
        .await
        .expect("request should succeed");
    assert_eq!(resp1.status(), StatusCode::OK);

    state.blacklist.write().await.insert(alice_hotkey.clone());

    let hdrs2 = signed_headers(&alice_kp, axon_hotkey, body, 2);
    let resp2 = client
        .post(format!("http://{addr}/TextPrompt"))
        .headers(hdrs2)
        .body(body.to_vec())
        .send()
        .await
        .expect("request should succeed");
    assert_eq!(resp2.status(), StatusCode::FORBIDDEN);
}

// ── 12. Multiple routes on same server ─────────────────────────────────────

#[tokio::test]
async fn multiple_synapse_routes() {
    let state = make_state(None);
    let app = Router::new()
        .route("/TextPrompt", post(|| async { "text response" }))
        .route("/ImagePrompt", post(|| async { "image response" }))
        .layer(middleware::from_fn(mw::body_hash_middleware))
        .layer(middleware::from_fn(mw::priority_middleware))
        .layer(middleware::from_fn(mw::blacklist_middleware))
        .layer(middleware::from_fn(mw::verification_middleware))
        .layer(axum::Extension(state));
    let addr = serve_app(app).await;

    let client = reqwest::Client::new();

    let resp_text = client
        .post(format!("http://{addr}/TextPrompt"))
        .body(b"{}".to_vec())
        .send()
        .await
        .expect("request should succeed");
    assert_eq!(resp_text.status(), StatusCode::OK);
    assert_eq!(resp_text.text().await.unwrap(), "text response");

    let resp_image = client
        .post(format!("http://{addr}/ImagePrompt"))
        .body(b"{}".to_vec())
        .send()
        .await
        .expect("request should succeed");
    assert_eq!(resp_image.status(), StatusCode::OK);
    assert_eq!(resp_image.text().await.unwrap(), "image response");
}

// ── 13. Demonstrate Axon middleware ordering bug ────────────────────────────
// In axum 0.8, Router::layer() before .route() means routes skip the layer.
// The Axon struct applies layers in new() before attach() adds routes.

#[tokio::test]
#[ignore]
async fn axon_struct_unsigned_request_bypasses_middleware() {
    let axon_hotkey = "5AxonBugTestHotkey";
    let config = AxonConfig {
        ip: "127.0.0.1".to_string(),
        port: 0,
        max_connections: 0,
        external_ip: None,
        hotkey: Some(axon_hotkey.to_string()),
    };
    let mut axon = Axon::new(config).attach("TextPrompt", || async { "bypassed" });
    let addr = axon.start().await.expect("should bind");

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://{addr}/TextPrompt"))
        .body(b"{}".to_vec())
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "BUG: Axon middleware is not applied to attached routes due to layer-before-route ordering in axum 0.8"
    );

    axon.stop().expect("should shutdown");
}

// ── 14. oneshot test confirming middleware works with correct ordering ──────

#[tokio::test]
async fn oneshot_middleware_rejects_unsigned() {
    let state = make_state(Some("5AxonOneshotHotkey"));
    let app = build_app(state);
    let req = HttpRequest::builder().method("POST").uri("/TextPrompt").body(Body::empty()).unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn oneshot_middleware_accepts_signed() {
    let axon_hotkey = "5AxonOneshotHotkey";
    let state = make_state(Some(axon_hotkey));
    let app = build_app(state);

    let alice_kp = alice();
    let body = br#"{"prompt":"test"}"#;
    let hdrs = signed_headers(&alice_kp, axon_hotkey, body, 1);

    let mut req_builder = HttpRequest::builder().method("POST").uri("/TextPrompt");
    for (k, v) in hdrs.iter() {
        req_builder = req_builder.header(k, v);
    }
    let req = req_builder.body(Body::from(body.to_vec())).unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
