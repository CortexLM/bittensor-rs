//! Integration tests for the Dendrite HTTP client.
//!
//! Tests the Dendrite client's ability to sign requests and communicate with
//! a real HTTP server. Uses simple axum servers as test backends.
//!
//! # Header Mismatch Note
//!
//! The Dendrite signing module produces headers with `bt-` hyphenated names
//! (`bt-nonce`, `bt-dendrite-hotkey`, etc.), while the Axon middleware expects
//! `bt_header_dendrite_` underscore-prefixed names. These tests verify the
//! Dendrite side's behavior independently.
//!
//! # Running
//!
//! ```sh
//! cargo test -p bittensor-dendrite --test integration
//! ```

use axum::Router;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::routing::post;
use bittensor_core::types::AxonInfo;
use bittensor_dendrite::config::DendriteConfig;
use bittensor_dendrite::dendrite::Dendrite;
use bittensor_dendrite::signing;
use bittensor_synapse::{Synapse, TerminalInfo, sha3_256_hex, signing_message};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct TextPrompt {
    prompt: String,
    max_tokens: u32,
    #[serde(rename = "name_val")]
    name_val: String,
    #[serde(rename = "timeout_val")]
    timeout_val: f64,
    dendrite_info: TerminalInfo,
    axon_info: TerminalInfo,
    computed_hash: String,
    total: u64,
    header: u64,
}

impl Synapse for TextPrompt {
    type Output = serde_json::Value;

    fn name(&self) -> &str {
        &self.name_val
    }
    fn timeout(&self) -> f64 {
        self.timeout_val
    }
    fn set_timeout(&mut self, t: f64) {
        self.timeout_val = t;
    }
    fn dendrite(&self) -> &TerminalInfo {
        &self.dendrite_info
    }
    fn set_dendrite(&mut self, info: TerminalInfo) {
        self.dendrite_info = info;
    }
    fn axon(&self) -> &TerminalInfo {
        &self.axon_info
    }
    fn set_axon(&mut self, info: TerminalInfo) {
        self.axon_info = info;
    }
    fn computed_body_hash(&self) -> &str {
        &self.computed_hash
    }
    fn set_computed_body_hash(&mut self, h: String) {
        self.computed_hash = h;
    }
    fn total_size(&self) -> u64 {
        self.total
    }
    fn set_total_size(&mut self, s: u64) {
        self.total = s;
    }
    fn header_size(&self) -> u64 {
        self.header
    }
    fn set_header_size(&mut self, s: u64) {
        self.header = s;
    }
}

impl TextPrompt {
    fn new(prompt: &str) -> Self {
        Self {
            prompt: prompt.to_string(),
            max_tokens: 100,
            name_val: "TextPrompt".to_string(),
            timeout_val: 12.0,
            dendrite_info: TerminalInfo::default(),
            axon_info: TerminalInfo::default(),
            computed_hash: String::new(),
            total: 0,
            header: 0,
        }
    }
}

fn alice() -> subxt_signer::sr25519::Keypair {
    subxt_signer::sr25519::dev::alice()
}

fn bob() -> subxt_signer::sr25519::Keypair {
    subxt_signer::sr25519::dev::bob()
}

fn localhost_axon(port: u16, hotkey: &str) -> AxonInfo {
    AxonInfo {
        ip: 2130706433,
        port,
        ip_type: 4,
        protocol: 0,
        version: 1,
        hotkey: hotkey.to_string(),
        coldkey: "5ColdkeyTest".to_string(),
    }
}

async fn start_test_app(app: Router) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    port
}

async fn start_echo_server() -> (u16, Arc<Mutex<HashMap<String, String>>>) {
    let captured = Arc::new(Mutex::new(HashMap::new()));
    let captured_clone = captured.clone();

    // Dendrite posts to the root URL derived from AxonInfo (http://ip:port),
    // so we must listen on "/" not on "/TextPrompt".
    let app = Router::new().route(
        "/",
        post(move |req: Request| {
            let captured = captured_clone.clone();
            async move {
                let headers = req.headers();
                let mut map = HashMap::new();
                for (k, v) in headers.iter() {
                    if k.as_str().starts_with("bt-") || k.as_str().starts_with("bt_header") {
                        map.insert(k.to_string(), v.to_str().unwrap_or("").to_string());
                    }
                }
                *captured.lock().await = map.clone();
                (StatusCode::OK, axum::Json(map))
            }
        }),
    );

    let port = start_test_app(app).await;
    (port, captured)
}

async fn start_401_server() -> u16 {
    let app =
        Router::new().route("/", post(|| async { (StatusCode::UNAUTHORIZED, "unauthorized") }));
    start_test_app(app).await
}

mod hex {
    pub fn decode_to_slice(input: &str, output: &mut [u8]) -> Result<(), String> {
        if input.len() != output.len() * 2 {
            return Err(format!("length mismatch: {} vs {}", input.len(), output.len() * 2));
        }
        for (i, byte) in output.iter_mut().enumerate() {
            *byte = u8::from_str_radix(&input[i * 2..i * 2 + 2], 16).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn signed_request_includes_bt_headers() {
    let (port, captured) = start_echo_server().await;
    let keypair = alice();
    let dendrite =
        Dendrite::new(DendriteConfig::default().with_timeout_secs(5).with_hotkey(keypair.clone()))
            .unwrap();

    let axon = localhost_axon(port, "5AxonHotkey");
    let synapse = TextPrompt::new("hello");
    let _result = dendrite.query(synapse, &axon).await;

    let headers = captured.lock().await;
    assert!(headers.contains_key("bt-nonce"), "missing bt-nonce: {headers:?}");
    assert!(headers.contains_key("bt-dendrite-hotkey"), "missing bt-dendrite-hotkey");
    assert!(headers.contains_key("bt-axon-hotkey"), "missing bt-axon-hotkey");
    assert!(headers.contains_key("bt-uuid"), "missing bt-uuid");
    assert!(headers.contains_key("bt-body-hash"), "missing bt-body-hash");
    assert!(headers.contains_key("bt-signature"), "missing bt-signature");
}

#[tokio::test]
async fn signature_is_valid_sr25519() {
    let keypair = alice();
    let axon_hotkey = "5AxonHotkeyTest";
    let body = br#"{"prompt":"verify me"}"#;
    let nonce: u64 = 42;

    let result = signing::sign_request(&keypair, axon_hotkey, body, nonce).unwrap();

    let dendrite_hotkey = keypair.public_key().to_account_id().to_string();
    let message =
        signing_message(nonce, &dendrite_hotkey, axon_hotkey, &result.uuid, &result.body_hash);
    let sig_bytes: [u8; 64] = {
        let hex_str = result.headers.get("bt-signature").unwrap().to_str().unwrap();
        let no_prefix = hex_str.strip_prefix("0x").unwrap();
        let mut buf = [0u8; 64];
        hex::decode_to_slice(no_prefix, &mut buf).unwrap();
        buf
    };
    let signature = subxt_signer::sr25519::Signature(sig_bytes);
    let public = keypair.public_key();
    assert!(
        subxt_signer::sr25519::verify(&signature, message.as_bytes(), &public),
        "signature verification failed"
    );
}

#[tokio::test]
async fn unsigned_request_sends_no_bt_headers() {
    let (port, captured) = start_echo_server().await;
    let dendrite = Dendrite::new(DendriteConfig::default().with_timeout_secs(5)).unwrap();

    let axon = localhost_axon(port, "5AxonHotkey");
    let synapse = TextPrompt::new("unsigned");
    let _result = dendrite.query(synapse, &axon).await;

    let headers = captured.lock().await;
    assert!(!headers.contains_key("bt-signature"), "unsigned request should not have bt-signature");
    assert!(!headers.contains_key("bt-nonce"), "unsigned request should not have bt-nonce");
}

#[tokio::test]
async fn dendrite_returns_signing_error_on_401() {
    let port = start_401_server().await;
    let keypair = alice();
    let dendrite =
        Dendrite::new(DendriteConfig::default().with_timeout_secs(5).with_hotkey(keypair)).unwrap();

    let axon = localhost_axon(port, "5AxonHotkey");
    let synapse = TextPrompt::new("test 401");
    let result = dendrite.query(synapse, &axon).await;
    assert!(result.is_err(), "should return error for 401");
    if let Err(e) = result {
        let msg = format!("{e}");
        assert!(
            msg.contains("Unauthorized") || msg.contains("401"),
            "error should mention 401: {msg}"
        );
    }
}

#[tokio::test]
async fn different_signers_produce_different_signatures() {
    let alice_kp = alice();
    let bob_kp = bob();
    let axon_hotkey = "5AxonHotkey";
    let body = b"same body";
    let nonce = 1u64;

    let r1 = signing::sign_request(&alice_kp, axon_hotkey, body, nonce).unwrap();
    let r2 = signing::sign_request(&bob_kp, axon_hotkey, body, nonce).unwrap();

    let s1 = r1.headers.get("bt-signature").unwrap();
    let s2 = r2.headers.get("bt-signature").unwrap();
    assert_ne!(s1, s2, "different signers should produce different signatures");
}

#[tokio::test]
async fn body_hash_matches_sha3_256() {
    let keypair = alice();
    let body = br#"{"prompt":"hash test"}"#;
    let expected = sha3_256_hex(body);
    let result = signing::sign_request(&keypair, "5Axon", body, 0).unwrap();
    assert_eq!(result.body_hash, expected);
}

#[tokio::test]
async fn nonce_is_preserved_in_signed_request() {
    let keypair = alice();
    let body = b"";

    let r1 = signing::sign_request(&keypair, "5Axon", body, 100).unwrap();
    let r2 = signing::sign_request(&keypair, "5Axon", body, 200).unwrap();

    assert_eq!(r1.nonce, 100);
    assert_eq!(r2.nonce, 200);
    assert_ne!(r1.nonce, r2.nonce);
}
