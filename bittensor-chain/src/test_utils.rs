use bittensor_core::config::SubtensorConfig;
use std::collections::HashMap;
use subxt::OnlineClient;
use subxt_rpcs::client::{MockRpcClient, RpcClient, mock_rpc_client::Json};

fn load_metadata_bytes() -> Vec<u8> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let path = std::path::Path::new(&manifest_dir).parent().unwrap().join("metadata/finney.scale");
    std::fs::read(&path).unwrap_or_else(|e| panic!("Failed to read metadata from {:?}: {e}", path))
}

fn scale_compact_u32(val: u32) -> Vec<u8> {
    if val < 64 {
        vec![(val << 2) as u8]
    } else if val < 16384 {
        let v = (val << 2) | 1;
        vec![(v & 0xFF) as u8, ((v >> 8) & 0xFF) as u8]
    } else {
        let v = (val << 2) | 2;
        vec![
            (v & 0xFF) as u8,
            ((v >> 8) & 0xFF) as u8,
            ((v >> 16) & 0xFF) as u8,
            ((v >> 24) & 0xFF) as u8,
        ]
    }
}

fn to_hex(bytes: &[u8]) -> String {
    "0x".to_string() + &bytes.iter().map(|b| format!("{b:02x}")).collect::<String>()
}

const GENESIS_HASH: &str = "0x0000000000000000000000000000000000000000000000000000000000000000";
const RPC_METHODS: &str = r#"{"methods":["rpc_methods","system_health","system_version","system_chain","system_properties","chain_getBlockHash","chain_getHeader","chain_getBlock","chain_getFinalizedHead","state_getMetadata","state_getRuntimeVersion","state_getStorage","state_getKeysPaged","state_call","chain_subscribeNewHeads","chain_subscribeFinalizedHeads","chain_subscribeAllHeads"]}"#;

fn runtime_version() -> serde_json::Value {
    serde_json::json!({"specName":"node-subtensor","implName":"node-subtensor","specVersion":393,"implVersion":0,"apis":[],"transactionVersion":2})
}

fn block_header() -> serde_json::Value {
    serde_json::json!({
        "parentHash": GENESIS_HASH,
        "number": "0x1",
        "stateRoot": GENESIS_HASH,
        "extrinsicsRoot": GENESIS_HASH,
        "digest": { "logs": [] }
    })
}

fn empty_block() -> serde_json::Value {
    serde_json::json!({
        "header": block_header(),
        "block": { "extrinsics": [] }
    })
}

pub async fn mock_client_empty() -> OnlineClient<SubtensorConfig> {
    build_mock(load_metadata_bytes(), HashMap::new()).await
}

#[allow(dead_code)]
pub async fn mock_client_with_storage(
    storage: HashMap<String, String>,
) -> OnlineClient<SubtensorConfig> {
    build_mock(load_metadata_bytes(), storage).await
}

async fn build_mock(
    metadata_raw: Vec<u8>,
    storage: HashMap<String, String>,
) -> OnlineClient<SubtensorConfig> {
    let genesis = GENESIS_HASH.to_string();
    let rt_ver = runtime_version();
    let header = block_header();
    let block = empty_block();

    let metadata_wrapped = {
        let mut buf = scale_compact_u32(metadata_raw.len() as u32);
        buf.extend_from_slice(&metadata_raw);
        to_hex(&buf)
    };

    let core_version_hex = {
        // SCALE-encoded SpecVersionHeader:
        // spec_name="node-subtensor", impl_name="node-subtensor",
        // authoring_version=0, spec_version=393, impl_version=0,
        // apis=Vec::new(), transaction_version=2
        "0x386e6f64652d73756274656e736f72386e6f64652d73756274656e736f720000000089010000000000000002000000".to_string()
    };

    let metadata_versions_hex = {
        // SCALE-encoded Vec<u32> = [15] (metadata v15)
        "0x043c000000".to_string()
    };

    let metadata_at_version_hex = {
        // Option<(Compact<u32>, RuntimeMetadataPrefixed)>
        // Some = 0x01, then compact(len), then raw metadata
        let mut buf = vec![0x01];
        buf.extend_from_slice(&scale_compact_u32(metadata_raw.len() as u32));
        buf.extend_from_slice(&metadata_raw);
        to_hex(&buf)
    };

    let metadata_for_get_metadata = metadata_wrapped.clone();
    let metadata_for_state_call = metadata_wrapped.clone();
    let core_version_for_call = core_version_hex.clone();
    let versions_for_call = metadata_versions_hex.clone();
    let at_version_for_call = metadata_at_version_hex.clone();

    let mock = MockRpcClient::builder()
        .method_handler("rpc_methods", |_p| async move {
            serde_json::from_str::<serde_json::Value>(RPC_METHODS).unwrap()
        })
        .method_handler("system_health", move |_p| async move {
            Json(serde_json::json!({"isSyncing":false,"peers":0,"shouldHavePeers":false}))
        })
        .method_handler("system_version", |_p| async move { Json("4.0.0") })
        .method_handler("system_chain", |_p| async move { Json("Development") })
        .method_handler("system_properties", |_p| async move {
            Json(serde_json::json!({"ss58Format":42,"tokenDecimals":9,"tokenSymbol":"TAO"}))
        })
        .method_handler("chain_getBlockHash", move |_p| {
            let genesis = genesis.clone();
            async move { Json(serde_json::json!(genesis)) }
        })
        .method_handler("chain_getHeader", move |_p| {
            let h = header.clone();
            async move { Json(h) }
        })
        .method_handler("chain_getBlock", move |_p| {
            let b = block.clone();
            async move { Json(b) }
        })
        .method_handler("chain_getFinalizedHead", move |_p| {
            let genesis = GENESIS_HASH.to_string();
            async move { Json(serde_json::json!(genesis)) }
        })
        .method_handler("state_getMetadata", move |_p| {
            let m = metadata_for_get_metadata.clone();
            async move {
                serde_json::from_str::<serde_json::Value>(&m)
                    .unwrap_or_else(|_| serde_json::json!(null))
            }
        })
        .method_handler("state_getRuntimeVersion", move |_p| {
            let rv = rt_ver.clone();
            async move { Json(rv) }
        })
        .method_handler("state_getStorage", move |p| {
            let storage = storage.clone();
            async move {
                let key = parse_storage_key(p);
                match storage.get(&key) {
                    Some(v) => serde_json::json!(v),
                    None => serde_json::json!(null),
                }
            }
        })
        .method_handler("state_getKeysPaged", |_p| async move { Json(Vec::<String>::new()) })
        .method_handler("state_call", move |p| {
            let core_ver = core_version_for_call.clone();
            let meta_ver = versions_for_call.clone();
            let meta_at = at_version_for_call.clone();
            let meta_fallback = metadata_for_state_call.clone();
            async move {
                let method_name = parse_state_call_method(p);
                match method_name.as_str() {
                    "Core_version" => Json(core_ver),
                    "Metadata_metadata_versions" => Json(meta_ver),
                    "Metadata_metadata_at_version" => Json(meta_at),
                    "Metadata_metadata" => Json(meta_fallback),
                    _ => Json("0x".to_string()),
                }
            }
        })
        .subscription_handler("chain_subscribeNewHeads", |_p, _u| {
            let h = block_header();
            async move {
                let (tx, rx) = tokio::sync::mpsc::channel(16);
                let _ = tx.send(Json(h)).await;
                tokio::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(60)).await
                });
                subxt_rpcs::client::mock_rpc_client::AndThen::<Vec<Json<serde_json::Value>>, _>(
                    vec![],
                    rx,
                )
            }
        })
        .subscription_handler("chain_subscribeFinalizedHeads", |_p, _u| {
            let h = block_header();
            async move {
                let (tx, rx) = tokio::sync::mpsc::channel(16);
                let _ = tx.send(Json(h)).await;
                tokio::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(60)).await
                });
                subxt_rpcs::client::mock_rpc_client::AndThen::<Vec<Json<serde_json::Value>>, _>(
                    vec![],
                    rx,
                )
            }
        })
        .subscription_handler("chain_subscribeAllHeads", |_p, _u| {
            let h = block_header();
            async move {
                let (tx, rx) = tokio::sync::mpsc::channel(16);
                let _ = tx.send(Json(h)).await;
                tokio::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(60)).await
                });
                subxt_rpcs::client::mock_rpc_client::AndThen::<Vec<Json<serde_json::Value>>, _>(
                    vec![],
                    rx,
                )
            }
        })
        .build();

    let rpc_client = RpcClient::new(mock);
    OnlineClient::from_rpc_client(rpc_client).await.expect("Failed to create mock OnlineClient")
}

fn parse_storage_key(params: Option<Box<serde_json::value::RawValue>>) -> String {
    params
        .as_ref()
        .and_then(|p| serde_json::from_str::<Vec<String>>(p.get()).ok())
        .and_then(|v| v.first().cloned())
        .unwrap_or_default()
}

fn parse_state_call_method(params: Option<Box<serde_json::value::RawValue>>) -> String {
    params
        .as_ref()
        .and_then(|p| serde_json::from_str::<Vec<String>>(p.get()).ok())
        .and_then(|v| v.first().cloned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_client_empty_connects() {
        let client = mock_client_empty().await;
        let at = client.at_current_block().await;
        assert!(at.is_ok(), "mock client should reach current block: {:?}", at.err());
    }

    #[tokio::test]
    async fn mock_client_empty_balance() {
        let client = mock_client_empty().await;
        let account = subxt::utils::AccountId32::from([0u8; 32]);
        let balance = crate::queries::account::get_balance(&client, &account).await;
        assert!(balance.is_ok(), "get_balance should succeed with mock: {:?}", balance.err());
        assert_eq!(balance.unwrap().to_rao(), 0);
    }

    #[tokio::test]
    async fn mock_client_empty_total_stake() {
        let client = mock_client_empty().await;
        let stake = crate::queries::account::get_total_network_stake(&client).await;
        assert!(stake.is_ok(), "get_total_network_stake should succeed: {:?}", stake.err());
        assert_eq!(stake.unwrap().to_rao(), 0);
    }
}
