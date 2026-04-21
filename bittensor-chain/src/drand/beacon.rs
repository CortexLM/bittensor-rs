//! DRAND beacon client — fetch, verify, and cache DRAND randomness rounds.
//!
//! Uses the DRAND HTTP API (`/public/{round}` or `/public/latest`) and
//! verifies BLS12-381 signatures. Quicknet puts the public key on G2
//! (96 bytes compressed) and the signature on G1 (48 bytes compressed).
//!
//! All network calls go through an injectable HTTP client so tests can
//! replace them with mocks.

use std::num::NonZeroUsize;
use std::sync::Arc;

use lru::LruCache;
use serde::Deserialize;
use sha2::{Digest, Sha256};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Chain hash for DRAND quicknet mainnet.
pub const MAINNET_CHAIN_HASH: &str =
    "52db9ba70e0cc0f6eaf7803dd07147dcf88962a807d5724d0c6f8f63c3d0da0d";

/// DRAND quicknet public key (96 bytes, G2 point, compressed).
/// Sourced from https://api.drand.sh/52db9ba7.../info
pub const MAINNET_PUBLIC_KEY_HEX: &str = "83cf0f2896adee7eb8b5f01fcad3912212c437e0073e911fb90022d3e760183c\
     8c4b450b6a0a6c3ac6a5776a2d1064510d1fec758c921cc22b0e17e63aaf4bcb\
     5ed66304de9cf809bd274ca73bab4af5a6e9c76a4bc09e76eae8991ef5ece45a";

/// Default DRAND API base URL.
pub const DEFAULT_DRAND_URL: &str = "https://api.drand.sh";

/// Default cache capacity for recent rounds.
pub const DEFAULT_CACHE_CAPACITY: usize = 100;

/// BLS signature domain separation tag for Quicknet (unchained).
/// See: https://drand.love/developer/specification/#signature-domain-separation
const DST_QUICKNET: &[u8] = b"BLS_SIG_BLS12381G1_XMD:SHA-256_SSWU_RO_NUL_";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A single DRAND round returned by the HTTP API.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DrandRound {
    /// Round number (monotonically increasing).
    pub round: u64,
    /// Randomness value (hex-encoded SHA-256 of signature).
    pub randomness: String,
    /// BLS signature (hex-encoded).
    pub signature: String,
    /// Previous signature (hex-encoded, used for chained mode).
    #[serde(default)]
    pub previous_signature: Option<String>,
}

/// Chain info returned by the DRAND `/info` endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct DrandChainInfo {
    /// Hex-encoded public key.
    pub public_key: String,
    /// Round period in seconds.
    pub period: u64,
    /// Genesis time (UNIX timestamp).
    pub genesis_time: u64,
    /// Chain hash (hex).
    pub hash: String,
    /// Group hash (hex).
    #[serde(default)]
    pub group_hash: String,
}

/// Errors from DRAND beacon operations.
#[derive(Debug, thiserror::Error)]
pub enum DrandBeaconError {
    #[error("HTTP request failed: {0}")]
    Http(String),

    #[error("JSON parse error: {0}")]
    Json(String),

    #[error("BLS signature verification failed")]
    SignatureVerification,

    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),

    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("Cache miss for round {0}")]
    CacheMiss(u64),

    #[error("Hex decode error: {0}")]
    HexDecode(String),
}

// ---------------------------------------------------------------------------
// HTTP abstraction — returns raw String JSON to be dyn-compatible
// ---------------------------------------------------------------------------

/// Trait for fetching raw JSON text over HTTP.
///
/// Returns a `String` instead of a generic `T` so the trait is
/// object-safe (dyn-compatible). Callers deserialize themselves.
pub trait DrandHttpFetcher: Send + Sync {
    /// Fetch the URL and return the response body as a String.
    fn fetch_json(
        &self,
        url: &str,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<String, Box<dyn std::error::Error + Send + Sync>>,
                > + Send
                + '_,
        >,
    >;
}

/// Production HTTP fetcher using reqwest.
struct ReqwestFetcher;

impl DrandHttpFetcher for ReqwestFetcher {
    fn fetch_json(
        &self,
        url: &str,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<String, Box<dyn std::error::Error + Send + Sync>>,
                > + Send
                + '_,
        >,
    > {
        let url = url.to_string();
        Box::pin(async move {
            let resp = reqwest::get(&url).await?;
            let text = resp.text().await?;
            Ok(text)
        })
    }
}

// ---------------------------------------------------------------------------
// Beacon client
// ---------------------------------------------------------------------------

/// HTTP-based DRAND beacon client with signature verification and LRU cache.
pub struct DrandBeacon {
    http: Arc<dyn DrandHttpFetcher>,
    base_url: String,
    chain_hash: String,
    public_key_bytes: Vec<u8>,
    cache: std::sync::Mutex<LruCache<u64, DrandRound>>,
}

impl DrandBeacon {
    /// Create a new beacon client with default settings.
    pub fn new() -> Result<Self, DrandBeaconError> {
        Self::with_config(DEFAULT_DRAND_URL, MAINNET_CHAIN_HASH, DEFAULT_CACHE_CAPACITY)
    }

    /// Create a new beacon client with custom config.
    pub fn with_config(
        base_url: &str,
        chain_hash: &str,
        cache_capacity: usize,
    ) -> Result<Self, DrandBeaconError> {
        let pk_bytes = hex_decode(MAINNET_PUBLIC_KEY_HEX).map_err(DrandBeaconError::HexDecode)?;
        Ok(Self {
            http: Arc::new(ReqwestFetcher),
            base_url: base_url.to_string(),
            chain_hash: chain_hash.to_string(),
            public_key_bytes: pk_bytes,
            cache: std::sync::Mutex::new(LruCache::new(
                NonZeroUsize::new(cache_capacity)
                    .unwrap_or_else(|| NonZeroUsize::new(100).expect("100 is always non-zero")),
            )),
        })
    }

    /// Create a new beacon with a custom HTTP fetcher (for testing).
    pub fn with_fetcher(
        fetcher: Arc<dyn DrandHttpFetcher>,
        chain_hash: &str,
        cache_capacity: usize,
    ) -> Result<Self, DrandBeaconError> {
        let pk_bytes = hex_decode(MAINNET_PUBLIC_KEY_HEX).map_err(DrandBeaconError::HexDecode)?;
        Ok(Self {
            http: fetcher,
            base_url: DEFAULT_DRAND_URL.to_string(),
            chain_hash: chain_hash.to_string(),
            public_key_bytes: pk_bytes,
            cache: std::sync::Mutex::new(LruCache::new(
                NonZeroUsize::new(cache_capacity)
                    .unwrap_or_else(|| NonZeroUsize::new(100).expect("100 is always non-zero")),
            )),
        })
    }

    /// Fetch the latest DRAND round, verify signature, and cache it.
    pub async fn get_latest(&self) -> Result<DrandRound, DrandBeaconError> {
        let url = format!("{}/{}/public/latest", self.base_url, self.chain_hash);
        let text =
            self.http.fetch_json(&url).await.map_err(|e| DrandBeaconError::Http(e.to_string()))?;
        let round: DrandRound =
            serde_json::from_str(&text).map_err(|e| DrandBeaconError::Json(e.to_string()))?;
        self.verify_and_cache(&round)?;
        Ok(round)
    }

    /// Fetch a specific DRAND round by number, verify, and cache.
    pub async fn get_round(&self, round_number: u64) -> Result<DrandRound, DrandBeaconError> {
        // Check cache first
        {
            let mut cache = self.cache.lock().expect("drand beacon cache lock");
            if let Some(cached) = cache.get(&round_number) {
                return Ok(cached.clone());
            }
        }

        let url = format!("{}/{}/public/{}", self.base_url, self.chain_hash, round_number);
        let text =
            self.http.fetch_json(&url).await.map_err(|e| DrandBeaconError::Http(e.to_string()))?;
        let round: DrandRound =
            serde_json::from_str(&text).map_err(|e| DrandBeaconError::Json(e.to_string()))?;
        self.verify_and_cache(&round)?;
        Ok(round)
    }

    /// Get a cached round without making a network call.
    pub fn get_cached(&self, round_number: u64) -> Option<DrandRound> {
        let mut cache = self.cache.lock().expect("drand beacon cache lock");
        cache.get(&round_number).cloned()
    }

    /// Verify the BLS12-381 signature on a round and insert into cache.
    fn verify_and_cache(&self, round: &DrandRound) -> Result<(), DrandBeaconError> {
        let sig_bytes = hex_decode(&round.signature)
            .map_err(|e| DrandBeaconError::InvalidSignature(e.to_string()))?;

        let message = round_message(round.round, round.previous_signature.as_deref());
        verify_bls_signature_quicknet(&self.public_key_bytes, &message, &sig_bytes)?;

        let mut cache = self.cache.lock().expect("drand beacon cache lock");
        cache.put(round.round, round.clone());
        Ok(())
    }

    /// Return the chain hash.
    pub fn chain_hash(&self) -> &str {
        &self.chain_hash
    }

    /// Return the public key bytes.
    pub fn public_key(&self) -> &[u8] {
        &self.public_key_bytes
    }
}

// ---------------------------------------------------------------------------
// BLS12-381 signature verification (Quicknet: G2 PK, G1 sig)
// ---------------------------------------------------------------------------

/// Verify a Quicknet BLS12-381 signature.
///
/// Quicknet puts the public key on G2 (96 bytes compressed) and the
/// signature on G1 (48 bytes compressed). Verification checks:
///   e(sig, g2) == e(H(m), pk)
fn verify_bls_signature_quicknet(
    pk_bytes: &[u8],
    message: &[u8],
    sig_bytes: &[u8],
) -> Result<(), DrandBeaconError> {
    // blst::min_sig = PK on G2 (96B), sig on G1 (48B)
    use blst::BLST_ERROR;
    use blst::min_sig::{PublicKey as G2PublicKey, Signature as G1Signature};

    let pk = G2PublicKey::from_bytes(pk_bytes)
        .map_err(|e| DrandBeaconError::InvalidPublicKey(format!("{e:?}")))?;
    pk.validate().map_err(|e| DrandBeaconError::InvalidPublicKey(format!("validation: {e:?}")))?;

    let sig = G1Signature::from_bytes(sig_bytes)
        .map_err(|e| DrandBeaconError::InvalidSignature(format!("{e:?}")))?;

    let result = sig.verify(
        true, // sig_groupcheck
        message,
        DST_QUICKNET, // DST
        &[],          // augment
        &pk,
        true, // pk_validate
    );

    match result {
        BLST_ERROR::BLST_SUCCESS => Ok(()),
        _ => Err(DrandBeaconError::SignatureVerification),
    }
}

/// Construct the message that was signed for a DRAND round.
///
/// Quicknet (unchained): `sha256(round_number.to_be_bytes())`
///   — Per the drand source (`NewPedersenBLSUnchainedG1`), the beacon digest
///     is SHA-256 of the big-endian round number, NOT the raw 8 bytes.
/// Classic (chained):   `sha256(previous_signature || round_number.to_be_bytes())`
fn round_message(round: u64, previous_signature: Option<&str>) -> Vec<u8> {
    match previous_signature {
        // Unchained / Quicknet mode: message is SHA-256 of round number
        None => {
            let mut hasher = Sha256::new();
            hasher.update(round.to_be_bytes());
            hasher.finalize().to_vec()
        }
        // Chained / Classic mode: message is hash of previous sig + round
        Some(prev_sig_hex) => {
            let prev_sig = hex_decode(prev_sig_hex).unwrap_or_default();
            let mut hasher = Sha256::new();
            hasher.update(&prev_sig);
            hasher.update(round.to_be_bytes());
            hasher.finalize().to_vec()
        }
    }
}

/// Derive the randomness value from a signature.
///
/// `randomness = SHA-256(signature_bytes)`
pub fn derive_randomness(signature_hex: &str) -> Result<[u8; 32], DrandBeaconError> {
    let sig_bytes =
        hex_decode(signature_hex).map_err(|e| DrandBeaconError::HexDecode(e.to_string()))?;
    let hash = Sha256::digest(&sig_bytes);
    let mut out = [0u8; 32];
    out.copy_from_slice(&hash);
    Ok(out)
}

// ---------------------------------------------------------------------------
// Hex helpers
// ---------------------------------------------------------------------------

fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    if s.len() % 2 != 0 {
        return Err(format!("odd length hex string: {s}"));
    }
    let mut buf = Vec::with_capacity(s.len() / 2);
    for i in (0..s.len()).step_by(2) {
        let byte = u8::from_str_radix(&s[i..i + 2], 16)
            .map_err(|e| format!("hex decode at offset {i}: {e}"))?;
        buf.push(byte);
    }
    Ok(buf)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // ---- Known test vectors from Quicknet ----

    /// Round 123 signature (from quicknet public endpoint)
    const ROUND_123_SIG: &str = "b75c69d0b72a5d906e854e808ba7e2accb1542ac355ae486d591aa9d43765482\
         e26cd02df835d3546d23c4b13e0dfc92";

    // ---- Mock fetcher for testing ----

    struct MockFetcher {
        rounds_json: std::collections::HashMap<String, String>,
        call_count: AtomicUsize,
    }

    impl MockFetcher {
        fn new(rounds: Vec<DrandRound>) -> Self {
            let mut map = std::collections::HashMap::new();
            for r in &rounds {
                let key = format!("/{}", r.round);
                let json = serde_json::to_string(r).unwrap_or_default();
                map.insert(key, json);
            }
            // Add latest entry pointing to last round
            if let Some(last) = rounds.last() {
                let json = serde_json::to_string(last).unwrap_or_default();
                map.insert("/latest".to_string(), json);
            }
            Self { rounds_json: map, call_count: AtomicUsize::new(0) }
        }

        fn call_count(&self) -> usize {
            self.call_count.load(Ordering::SeqCst)
        }
    }

    impl DrandHttpFetcher for MockFetcher {
        fn fetch_json(
            &self,
            url: &str,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = Result<String, Box<dyn std::error::Error + Send + Sync>>,
                    > + Send
                    + '_,
            >,
        > {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            let result = if url.contains("/latest") {
                self.rounds_json.get("/latest").cloned()
            } else {
                // Extract round number from URL
                let round_part = url.rsplit('/').next().unwrap_or("");
                let key = format!("/{round_part}");
                self.rounds_json.get(&key).cloned()
            };
            let url_owned = url.to_string();
            match result {
                Some(json) => Box::pin(async move { Ok(json) }),
                None => Box::pin(async move {
                    Err(format!("round not found in mock for url: {url_owned}").into())
                }),
            }
        }
    }

    fn make_mock_round(round: u64, sig: &str, prev_sig: Option<&str>) -> DrandRound {
        let sig_bytes = hex_decode(sig).unwrap_or_default();
        let hash = Sha256::digest(&sig_bytes);
        let mut randomness_hex = String::with_capacity(64);
        for &b in &hash {
            randomness_hex.push_str(&format!("{b:02x}"));
        }
        DrandRound {
            round,
            randomness: randomness_hex,
            signature: sig.to_string(),
            previous_signature: prev_sig.map(|s| s.to_string()),
        }
    }

    // ---- Test 1: Hex decode roundtrip ----

    #[test]
    fn hex_decode_roundtrip() {
        let original = MAINNET_PUBLIC_KEY_HEX;
        let bytes = hex_decode(original).expect("decode");
        assert_eq!(bytes.len(), 96);
        let re_encoded: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(re_encoded, original);
    }

    // ---- Test 2: Round message construction (unchained) ----

    #[test]
    fn round_message_unchained() {
        let msg = round_message(123u64, None);
        // Quicknet unchained: digest is SHA-256(round_be_bytes), NOT raw 8 bytes
        let mut hasher = Sha256::new();
        hasher.update(123u64.to_be_bytes());
        let expected = hasher.finalize().to_vec();
        assert_eq!(msg, expected);
    }

    // ---- Test 3: Round message construction (chained) ----

    #[test]
    fn round_message_chained() {
        let prev_sig = "aabbccdd";
        let msg = round_message(5u64, Some(prev_sig));
        let mut hasher = Sha256::new();
        hasher.update(&hex_decode(prev_sig).unwrap());
        hasher.update(5u64.to_be_bytes());
        let expected = hasher.finalize().to_vec();
        assert_eq!(msg, expected);
    }

    // ---- Test 4: Derive randomness ----

    #[test]
    fn derive_randomness_from_signature() {
        let sig_hex = ROUND_123_SIG;
        let result = derive_randomness(sig_hex).expect("derive");
        let sig_bytes = hex_decode(sig_hex).unwrap();
        let expected = Sha256::digest(&sig_bytes);
        assert_eq!(result[..], expected[..]);
    }

    // ---- Test 5: BLS signature verification with known quicknet vector ----

    #[test]
    fn verify_quicknet_round_123_signature() {
        let pk_bytes = hex_decode(MAINNET_PUBLIC_KEY_HEX).unwrap();
        let sig_bytes = hex_decode(ROUND_123_SIG).unwrap();
        let message = round_message(123u64, None);
        let result = verify_bls_signature_quicknet(&pk_bytes, &message, &sig_bytes);
        assert!(result.is_ok(), "BLS verification failed: {:?}", result);
    }

    // ---- Test 6: BLS signature verification rejects bad signature ----

    #[test]
    fn verify_rejects_tampered_signature() {
        let pk_bytes = hex_decode(MAINNET_PUBLIC_KEY_HEX).unwrap();
        let mut sig_bytes = hex_decode(ROUND_123_SIG).unwrap();
        sig_bytes[0] ^= 0xff;
        let message = round_message(123u64, None);
        assert!(verify_bls_signature_quicknet(&pk_bytes, &message, &sig_bytes).is_err());
    }

    // ---- Test 7: Cache stores and retrieves ----

    #[test]
    fn cache_stores_retrieved_round() {
        let beacon = DrandBeacon::new().unwrap();
        let round = DrandRound {
            round: 42,
            randomness: "abc123".to_string(),
            signature: "def456".to_string(),
            previous_signature: None,
        };
        {
            let mut cache = beacon.cache.lock().unwrap();
            cache.put(42, round.clone());
        }
        let cached = beacon.get_cached(42);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().round, 42);
    }

    // ---- Test 8: Cache miss returns None ----

    #[test]
    fn cache_miss_returns_none() {
        let beacon = DrandBeacon::new().unwrap();
        assert!(beacon.get_cached(99999).is_none());
    }

    // ---- Test 9: Mock fetcher returns correct round ----

    #[tokio::test]
    async fn mock_fetcher_get_round() {
        let mock_rounds = vec![make_mock_round(123, ROUND_123_SIG, None)];
        let fetcher = Arc::new(MockFetcher::new(mock_rounds));
        let beacon = DrandBeacon::with_fetcher(fetcher.clone(), MAINNET_CHAIN_HASH, 100).unwrap();

        let result = beacon.get_round(123).await;
        assert!(result.is_ok());
        let round = result.unwrap();
        assert_eq!(round.round, 123);
        assert_eq!(round.signature, ROUND_123_SIG);
    }

    // ---- Test 10: Cache avoids duplicate fetch ----

    #[tokio::test]
    async fn cache_avoids_duplicate_fetch() {
        let mock_rounds = vec![make_mock_round(123, ROUND_123_SIG, None)];
        let fetcher = Arc::new(MockFetcher::new(mock_rounds));
        let beacon = DrandBeacon::with_fetcher(fetcher.clone(), MAINNET_CHAIN_HASH, 100).unwrap();

        let _ = beacon.get_round(123).await;
        assert_eq!(fetcher.call_count(), 1);

        let _ = beacon.get_round(123).await;
        assert_eq!(fetcher.call_count(), 1);
    }

    // ---- Test 11: Invalid public key is rejected ----

    #[test]
    fn invalid_public_key_rejected() {
        let bad_pk = vec![0u8; 96];
        let sig_bytes = hex_decode(ROUND_123_SIG).unwrap();
        let message = round_message(123u64, None);
        assert!(verify_bls_signature_quicknet(&bad_pk, &message, &sig_bytes).is_err());
    }

    // ---- Test 12: Chain info returns correct hash ----

    #[test]
    fn beacon_chain_hash_matches() {
        let beacon = DrandBeacon::new().unwrap();
        assert_eq!(beacon.chain_hash(), MAINNET_CHAIN_HASH);
    }
}
