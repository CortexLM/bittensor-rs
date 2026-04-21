//! Request signing — adds `bt-*` headers to outgoing HTTP requests.
//!
//! The signing message format is `"{nonce}.{dendrite_hotkey}.{axon_hotkey}.{uuid}.{body_hash}"`,
//! matching the Python SDK's `synapse.signing_message()` exactly.

use bittensor_core::error::BittensorError;
use bittensor_synapse::{sha3_256_hex, signing_message};
use reqwest::header::{ACCEPT, HeaderMap, HeaderValue};
use subxt_signer::sr25519::Keypair;

/// Bittensor-specific header names.
mod header_names {
    pub const NONCE: &str = "bt-nonce";
    pub const DENDRITE_HOTKEY: &str = "bt-dendrite-hotkey";
    pub const AXON_HOTKEY: &str = "bt-axon-hotkey";
    pub const UUID: &str = "bt-uuid";
    pub const BODY_HASH: &str = "bt-body-hash";
    pub const SIGNATURE: &str = "bt-signature";
}

/// The result of signing a request — includes all bt-* header values and the
/// computed body hash so callers can update the synapse accordingly.
pub struct SignedRequest {
    pub nonce: u64,
    pub uuid: String,
    pub body_hash: String,
    pub dendrite_hotkey: String,
    pub headers: HeaderMap,
}

/// Sign an outgoing synapse request and return the signed header map.
///
/// `body` is the serialised request body (may be empty).
/// `axon_hotkey` is the SS58 address of the target axon.
/// `nonce` is a monotonically-increasing counter (caller supplies it).
pub fn sign_request(
    keypair: &Keypair,
    axon_hotkey: &str,
    body: &[u8],
    nonce: u64,
) -> Result<SignedRequest, BittensorError> {
    let uuid = uuid::Uuid::new_v4().to_string();
    let body_hash = sha3_256_hex(body);
    let dendrite_hotkey = keypair.public_key().to_account_id().to_string();

    let message = signing_message(nonce, &dendrite_hotkey, axon_hotkey, &uuid, &body_hash);
    let signature = keypair.sign(message.as_bytes());
    let signature_hex = format!("0x{}", hex_encode(signature.0));

    let mut headers = HeaderMap::new();
    insert_header(&mut headers, header_names::NONCE, &nonce.to_string())?;
    insert_header(&mut headers, header_names::DENDRITE_HOTKEY, &dendrite_hotkey)?;
    insert_header(&mut headers, header_names::AXON_HOTKEY, axon_hotkey)?;
    insert_header(&mut headers, header_names::UUID, &uuid)?;
    insert_header(&mut headers, header_names::BODY_HASH, &body_hash)?;
    insert_header(&mut headers, header_names::SIGNATURE, &signature_hex)?;
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

    Ok(SignedRequest { nonce, uuid, body_hash, dendrite_hotkey, headers })
}

fn insert_header(headers: &mut HeaderMap, name: &str, value: &str) -> Result<(), BittensorError> {
    let header_name = reqwest::header::HeaderName::from_bytes(name.as_bytes())
        .map_err(|e| BittensorError::Signing(format!("invalid header name '{name}': {e}")))?;
    let header_value = HeaderValue::from_str(value)
        .map_err(|e| BittensorError::Signing(format!("invalid header value for '{name}': {e}")))?;
    headers.insert(header_name, header_value);
    Ok(())
}

fn hex_encode(bytes: impl AsRef<[u8]>) -> String {
    bytes.as_ref().iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alice() -> Keypair {
        subxt_signer::sr25519::dev::alice()
    }

    #[test]
    fn sign_request_produces_all_bt_headers() {
        let keypair = alice();
        let axon_hotkey = "5AxonHotkey1234567890abcdefghijk";
        let body = br#"{"prompt":"hello"}"#;
        let result = sign_request(&keypair, axon_hotkey, body, 1).unwrap();

        assert!(result.headers.contains_key(header_names::NONCE));
        assert!(result.headers.contains_key(header_names::DENDRITE_HOTKEY));
        assert!(result.headers.contains_key(header_names::AXON_HOTKEY));
        assert!(result.headers.contains_key(header_names::UUID));
        assert!(result.headers.contains_key(header_names::BODY_HASH));
        assert!(result.headers.contains_key(header_names::SIGNATURE));
        assert!(result.headers.contains_key("accept"));
    }

    #[test]
    fn nonce_is_preserved() {
        let keypair = alice();
        let result = sign_request(&keypair, "axon", b"", 42).unwrap();
        assert_eq!(result.nonce, 42);
        assert_eq!(result.headers.get(header_names::NONCE).unwrap(), "42");
    }

    #[test]
    fn body_hash_matches_sha3_256() {
        let keypair = alice();
        let body = br#"{"prompt":"hello"}"#;
        let expected = sha3_256_hex(body);
        let result = sign_request(&keypair, "axon", body, 0).unwrap();
        assert_eq!(result.body_hash, expected);
    }

    #[test]
    fn signature_starts_with_0x() {
        let keypair = alice();
        let result = sign_request(&keypair, "axon", b"", 0).unwrap();
        let sig = result.headers.get(header_names::SIGNATURE).unwrap().to_str().unwrap();
        assert!(sig.starts_with("0x"));
        assert_eq!(sig.len(), 2 + 128);
    }

    #[test]
    fn uuid_is_v4() {
        let keypair = alice();
        let result = sign_request(&keypair, "axon", b"", 0).unwrap();
        let parsed = uuid::Uuid::parse_str(&result.uuid).unwrap();
        assert_eq!(parsed.get_version(), Some(uuid::Version::Random));
    }

    #[test]
    fn different_nonces_produce_different_signatures() {
        let keypair = alice();
        let r1 = sign_request(&keypair, "axon", b"", 1).unwrap();
        let r2 = sign_request(&keypair, "axon", b"", 2).unwrap();
        let s1 = r1.headers.get(header_names::SIGNATURE).unwrap();
        let s2 = r2.headers.get(header_names::SIGNATURE).unwrap();
        assert_ne!(s1, s2);
    }

    #[test]
    fn empty_body_hashes_to_known_value() {
        let keypair = alice();
        let result = sign_request(&keypair, "axon", b"", 0).unwrap();
        assert_eq!(
            result.body_hash,
            "a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a"
        );
    }
}
