//! Dendrite — the HTTP client for sending signed Synapse requests to Axons.

use crate::config::DendriteConfig;
use crate::signing;
use bittensor_core::error::BittensorError;
use bittensor_core::types::AxonInfo;
use bittensor_synapse::{StreamingSynapse, Synapse, TerminalInfo, sha3_256_hex};
use futures::StreamExt;
use reqwest::Client;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use subxt_signer::sr25519::Keypair;

/// The Dendrite HTTP client.
///
/// Sends signed Synapse requests to axon endpoints, matching the Python SDK's
/// `bittensor.dendrite` behaviour.
pub struct Dendrite {
    client: Client,
    hotkey: Option<Keypair>,
    nonce: AtomicU64,
}

impl Dendrite {
    /// Create a new Dendrite from the given config.
    pub fn new(config: DendriteConfig) -> Result<Self, BittensorError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .pool_max_idle_per_host(config.max_connections)
            .build()
            .map_err(|e| BittensorError::Network(format!("failed to build reqwest client: {e}")))?;

        Ok(Self {
            client,
            hotkey: config.hotkey,
            nonce: AtomicU64::new(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
            ),
        })
    }

    /// Return the next monotonic nonce.
    fn next_nonce(&self) -> u64 {
        self.nonce.fetch_add(1, Ordering::Relaxed)
    }

    /// Build the target URL from an [`AxonInfo`].
    fn axon_url(axon_info: &AxonInfo) -> String {
        let protocol = if axon_info.protocol == 0 { "http" } else { "https" };
        let ip =
            if axon_info.ip == 0 { "127.0.0.1".to_string() } else { ip_from_u64(axon_info.ip) };
        format!("{protocol}://{ip}:{}", axon_info.port)
    }

    /// Send a signed synapse request and return the raw HTTP response.
    ///
    /// This is the lowest-level method. Callers typically use [`Self::query`], [`Self::forward`],
    /// [`Self::call`], or [`Self::call_stream`] instead.
    pub async fn query<S: Synapse + serde::Serialize>(
        &self,
        mut synapse: S,
        axon_info: &AxonInfo,
    ) -> Result<S, BittensorError> {
        let url = Self::axon_url(axon_info);
        let body =
            serde_json::to_vec(&synapse).map_err(|e| BittensorError::Codec(e.to_string()))?;
        let body_hash = sha3_256_hex(&body);
        synapse.set_computed_body_hash(body_hash.clone());
        synapse.set_total_size(body.len() as u64);

        let request = self.client.post(&url);

        let request = if let Some(ref keypair) = self.hotkey {
            let nonce = self.next_nonce();
            let signed = signing::sign_request(keypair, &axon_info.hotkey, &body, nonce)?;
            let request = request.headers(signed.headers);

            let dendrite_info = TerminalInfo {
                hotkey: Some(signed.dendrite_hotkey),
                nonce: Some(signed.nonce),
                uuid: Some(signed.uuid),
                ip: None,
                port: None,
                version: None,
                status_code: None,
                status_message: None,
                process_time: None,
                signature: None,
            };
            synapse.set_dendrite(dendrite_info);
            request
        } else {
            request.header("accept", "application/json")
        };

        let synapse_headers = synapse.to_headers();
        let mut all_headers = reqwest::header::HeaderMap::new();
        for (k, v) in &synapse_headers {
            if let Ok(name) = reqwest::header::HeaderName::from_bytes(k.as_bytes()) {
                if let Ok(val) = reqwest::header::HeaderValue::from_str(v) {
                    all_headers.insert(name, val);
                }
            }
        }

        let response = request.headers(all_headers).body(body).send().await.map_err(|e| {
            if e.is_timeout() {
                BittensorError::Timeout(e.to_string())
            } else {
                BittensorError::Network(e.to_string())
            }
        })?;

        let status = response.status();
        if status.as_u16() == 401 {
            return Err(BittensorError::Signing("received 401 Unauthorized from axon".into()));
        }
        if !status.is_success() {
            return Err(BittensorError::Rpc(format!("HTTP {} from axon", status)));
        }

        let resp_headers: std::collections::HashMap<String, String> = response
            .headers()
            .iter()
            .map(|(k, v): (&reqwest::header::HeaderName, &reqwest::header::HeaderValue)| {
                (k.to_string(), v.to_str().unwrap_or("").to_string())
            })
            .collect();

        let axon_resp = TerminalInfo::from_headers_with_prefix(
            &resp_headers,
            bittensor_synapse::header::keys::AXON_PREFIX,
        );

        synapse.set_axon(axon_resp);
        Ok(synapse)
    }

    /// Send a signed synapse request with streaming support.
    ///
    /// Similar to [`Self::query`] but intended for use when the response body is
    /// streamed incrementally.
    pub async fn forward<S: Synapse + serde::Serialize>(
        &self,
        synapse: S,
        axon_info: &AxonInfo,
    ) -> Result<S, BittensorError> {
        self.query(synapse, axon_info).await
    }

    /// Send a signed synapse request and return the full Synapse with response
    /// metadata populated (axon terminal info, status, etc.).
    pub async fn call<S: Synapse + serde::Serialize>(
        &self,
        synapse: S,
        axon_info: &AxonInfo,
    ) -> Result<S, BittensorError> {
        self.query(synapse, axon_info).await
    }

    /// Send a signed synapse request and return an SSE stream of response chunks.
    ///
    /// For [`StreamingSynapse`] types, each chunk is parsed into the stream item type.
    pub async fn call_stream<S>(
        &self,
        synapse: S,
        axon_info: &AxonInfo,
    ) -> Result<S::StreamItem, BittensorError>
    where
        S: StreamingSynapse + serde::Serialize,
    {
        let url = Self::axon_url(axon_info);
        let body =
            serde_json::to_vec(&synapse).map_err(|e| BittensorError::Codec(e.to_string()))?;

        let mut request_builder = self.client.post(&url);

        if let Some(ref keypair) = self.hotkey {
            let nonce = self.next_nonce();
            let signed = signing::sign_request(keypair, &axon_info.hotkey, &body, nonce)?;
            request_builder = request_builder.headers(signed.headers);
        } else {
            request_builder = request_builder.header("accept", "text/event-stream");
        }

        let response =
            request_builder.header("accept", "text/event-stream").body(body).send().await.map_err(
                |e| {
                    if e.is_timeout() {
                        BittensorError::Timeout(e.to_string())
                    } else {
                        BittensorError::Network(e.to_string())
                    }
                },
            )?;

        let status = response.status();
        if status.as_u16() == 401 {
            return Err(BittensorError::Signing("received 401 Unauthorized from axon".into()));
        }
        if !status.is_success() {
            return Err(BittensorError::Rpc(format!("HTTP {} from axon", status)));
        }

        let mut stream = response.bytes_stream();
        let mut buffer = Vec::new();

        while let Some(chunk_result) = stream.next().await {
            let chunk =
                chunk_result.map_err(|e: reqwest::Error| BittensorError::Network(e.to_string()))?;
            buffer.extend_from_slice(&chunk);

            while let Some(pos) = buffer.windows(2).position(|w| w == b"\n\n") {
                let event_data = buffer[..pos].to_vec();
                buffer.drain(..pos + 2);

                for line in event_data.split(|&b| b == b'\n') {
                    if let Some(data) = line.strip_prefix(b"data: ") {
                        if data == b"[DONE]" {
                            continue;
                        }
                        let item = S::process_chunk(data)
                            .map_err(|e| BittensorError::Codec(e.to_string()))?;
                        return Ok(item);
                    }
                }
            }
        }

        if !buffer.is_empty() {
            for line in buffer.split(|&b| b == b'\n') {
                if let Some(data) = line.strip_prefix(b"data: ") {
                    if data == b"[DONE]" {
                        continue;
                    }
                    let item =
                        S::process_chunk(data).map_err(|e| BittensorError::Codec(e.to_string()))?;
                    return Ok(item);
                }
            }
        }

        Err(BittensorError::Network("stream ended without producing items".into()))
    }
}

/// Convert a u64 IP address to dotted-decimal notation.
fn ip_from_u64(ip: u64) -> String {
    let a = (ip >> 24) as u8;
    let b = (ip >> 16) as u8;
    let c = (ip >> 8) as u8;
    let d = ip as u8;
    format!("{a}.{b}.{c}.{d}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use bittensor_core::error::BittensorError;

    #[test]
    fn ip_from_u64_localhost() {
        assert_eq!(ip_from_u64(2130706433), "127.0.0.1");
    }

    #[test]
    fn ip_from_u64_zero() {
        assert_eq!(ip_from_u64(0), "0.0.0.0");
    }

    #[test]
    fn axon_url_http() {
        let axon = AxonInfo {
            ip: 2130706433,
            port: 8090,
            ip_type: 4,
            protocol: 0,
            version: 1,
            hotkey: "5TestHotkey".into(),
            coldkey: "5TestColdkey".into(),
        };
        assert_eq!(Dendrite::axon_url(&axon), "http://127.0.0.1:8090");
    }

    #[test]
    fn axon_url_https() {
        let axon = AxonInfo {
            ip: 2130706433,
            port: 443,
            ip_type: 4,
            protocol: 1,
            version: 1,
            hotkey: "5TestHotkey".into(),
            coldkey: "5TestColdkey".into(),
        };
        assert_eq!(Dendrite::axon_url(&axon), "https://127.0.0.1:443");
    }

    #[test]
    fn new_dendrite_default_config() {
        let dendrite = Dendrite::new(DendriteConfig::default()).unwrap();
        assert!(dendrite.hotkey.is_none());
    }

    #[test]
    fn new_dendrite_with_hotkey() {
        let keypair = subxt_signer::sr25519::dev::alice();
        let config = DendriteConfig::default().with_hotkey(keypair);
        let dendrite = Dendrite::new(config).unwrap();
        assert!(dendrite.hotkey.is_some());
    }

    #[test]
    fn nonce_increments() {
        let dendrite = Dendrite::new(DendriteConfig::default()).unwrap();
        let n1 = dendrite.next_nonce();
        let n2 = dendrite.next_nonce();
        assert!(n2 > n1);
    }

    #[test]
    fn error_mapping_timeout() {
        let err = BittensorError::Timeout("timed out".into());
        assert!(err.is_retryable());
    }

    #[test]
    fn error_mapping_network() {
        let err = BittensorError::Network("connection refused".into());
        assert!(err.is_retryable());
    }

    #[test]
    fn error_mapping_signing_not_retryable() {
        let err = BittensorError::Signing("bad sig".into());
        assert!(!err.is_retryable());
    }

    mod wiremock_tests {
        use super::*;
        use wiremock::matchers::{header, method};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        fn test_axon(port: u16) -> AxonInfo {
            AxonInfo {
                ip: 2130706433,
                port,
                ip_type: 4,
                protocol: 0,
                version: 1,
                hotkey: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".into(),
                coldkey: "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty".into(),
            }
        }

        #[tokio::test]
        async fn query_returns_200_ok() {
            let server = MockServer::start().await;
            let port = server.address().port();

            Mock::given(method("POST"))
                .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"result":"ok"}"#))
                .mount(&server)
                .await;

            let _axon = test_axon(port);
            let dendrite = Dendrite::new(DendriteConfig::default().with_timeout_secs(5)).unwrap();

            let result = dendrite.client.post(format!("http://127.0.0.1:{port}")).send().await;
            assert!(result.is_ok());
            let resp = result.unwrap();
            assert_eq!(resp.status(), 200);
        }

        #[tokio::test]
        async fn query_returns_401_signing_error() {
            let server = MockServer::start().await;
            let port = server.address().port();

            Mock::given(method("POST"))
                .respond_with(ResponseTemplate::new(401))
                .mount(&server)
                .await;

            let _axon = test_axon(port);
            let keypair = subxt_signer::sr25519::dev::alice();
            let dendrite =
                Dendrite::new(DendriteConfig::default().with_timeout_secs(5).with_hotkey(keypair))
                    .unwrap();

            let resp =
                dendrite.client.post(format!("http://127.0.0.1:{port}")).send().await.unwrap();
            assert_eq!(resp.status(), 401);
        }

        #[tokio::test]
        async fn signed_request_includes_bt_headers() {
            let server = MockServer::start().await;
            let port = server.address().port();

            Mock::given(method("POST"))
                .and(header("bt-nonce", "42"))
                .and(header("bt-axon-hotkey", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"))
                .and(header(
                    "bt-body-hash",
                    "a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a",
                ))
                .respond_with(ResponseTemplate::new(200))
                .mount(&server)
                .await;

            let axon = AxonInfo {
                ip: 2130706433,
                port,
                ip_type: 4,
                protocol: 0,
                version: 1,
                hotkey: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".into(),
                coldkey: "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty".into(),
            };
            let keypair = subxt_signer::sr25519::dev::alice();
            let signed = signing::sign_request(&keypair, &axon.hotkey, b"", 42).unwrap();

            let dendrite =
                Dendrite::new(DendriteConfig::default().with_timeout_secs(5).with_hotkey(keypair))
                    .unwrap();

            let resp = dendrite
                .client
                .post(format!("http://127.0.0.1:{port}"))
                .headers(signed.headers)
                .body(b"".to_vec())
                .send()
                .await
                .unwrap();
            assert_eq!(resp.status(), 200);
        }

        #[tokio::test]
        async fn timeout_returns_timeout_error() {
            let server = MockServer::start().await;
            let port = server.address().port();

            Mock::given(method("POST"))
                .respond_with(
                    ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(10)),
                )
                .mount(&server)
                .await;

            let _axon = test_axon(port);
            let dendrite = Dendrite::new(DendriteConfig::default().with_timeout_secs(1)).unwrap();

            let result = dendrite.client.post(format!("http://127.0.0.1:{port}")).send().await;
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.is_timeout());
        }

        #[tokio::test]
        async fn connection_refused_returns_network_error() {
            let dendrite = Dendrite::new(DendriteConfig::default().with_timeout_secs(2)).unwrap();

            let result = dendrite.client.post("http://127.0.0.1:1").send().await;
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.is_connect());
        }

        #[tokio::test]
        async fn http_500_returns_rpc_error() {
            let server = MockServer::start().await;
            let port = server.address().port();

            Mock::given(method("POST"))
                .respond_with(ResponseTemplate::new(500))
                .mount(&server)
                .await;

            let dendrite = Dendrite::new(DendriteConfig::default().with_timeout_secs(5)).unwrap();

            let resp =
                dendrite.client.post(format!("http://127.0.0.1:{port}")).send().await.unwrap();
            assert!(resp.status().is_server_error());
        }
    }
}
