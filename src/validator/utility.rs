use crate::chain::{BittensorClient, BittensorSigner, ExtrinsicWait};
use anyhow::Result;
use subxt::dynamic::Value;

const UTILITY_MODULE: &str = "Utility";

/// A call to be batched
#[derive(Clone, Debug)]
pub struct BatchCall {
    pub module: String,
    pub function: String,
    pub args: Vec<Value>,
}

impl BatchCall {
    /// Create a new batch call
    pub fn new(module: impl Into<String>, function: impl Into<String>, args: Vec<Value>) -> Self {
        Self {
            module: module.into(),
            function: function.into(),
            args,
        }
    }

    /// Create a set_mechanism_weights call for batching
    pub fn set_mechanism_weights(
        netuid: u16,
        mechanism_id: u8,
        uids: &[u16],
        weights: &[u16],
        version_key: u64,
    ) -> Self {
        let uid_values: Vec<Value> = uids.iter().map(|uid| Value::from(*uid)).collect();
        let weight_values: Vec<Value> = weights.iter().map(|w| Value::from(*w)).collect();

        Self::new(
            "SubtensorModule",
            "set_mechanism_weights",
            vec![
                Value::from(netuid),
                Value::from(mechanism_id),
                Value::unnamed_composite(uid_values),
                Value::unnamed_composite(weight_values),
                Value::from(version_key),
            ],
        )
    }

    /// Create a set_weights call for batching (mechanism 0)
    pub fn set_weights(netuid: u16, uids: &[u16], weights: &[u16], version_key: u64) -> Self {
        let uid_values: Vec<Value> = uids.iter().map(|uid| Value::from(*uid)).collect();
        let weight_values: Vec<Value> = weights.iter().map(|w| Value::from(*w)).collect();

        Self::new(
            "SubtensorModule",
            "set_weights",
            vec![
                Value::from(netuid),
                Value::unnamed_composite(uid_values),
                Value::unnamed_composite(weight_values),
                Value::from(version_key),
            ],
        )
    }

    /// Create a commit_mechanism_weights call for batching
    pub fn commit_mechanism_weights(netuid: u16, mechanism_id: u8, commit_hash: &[u8; 32]) -> Self {
        Self::new(
            "SubtensorModule",
            "commit_mechanism_weights",
            vec![
                Value::from(netuid),
                Value::from(mechanism_id),
                Value::from_bytes(commit_hash),
            ],
        )
    }
}

/// Execute a batch of calls atomically (all succeed or all fail)
/// Uses Utility.batch_all which rolls back if any call fails
pub async fn batch_all(
    client: &BittensorClient,
    signer: &BittensorSigner,
    calls: Vec<BatchCall>,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    if calls.is_empty() {
        return Err(anyhow::anyhow!("Cannot batch empty call list"));
    }

    // Build call values using the RuntimeCall enum format
    // Each call needs to be encoded as a RuntimeCall variant
    let call_values: Vec<Value> = calls
        .iter()
        .map(|call| {
            // Create call as named variant matching RuntimeCall enum structure
            // RuntimeCall::SubtensorModule(pallet_subtensor::Call::function_name { args })
            let call_args: Vec<(&str, Value)> = match call.function.as_str() {
                "set_mechanism_weights" => vec![
                    (
                        "netuid",
                        call.args.first().cloned().unwrap_or(Value::from(0u16)),
                    ),
                    (
                        "mecid",
                        call.args.get(1).cloned().unwrap_or(Value::from(0u8)),
                    ),
                    (
                        "dests",
                        call.args
                            .get(2)
                            .cloned()
                            .unwrap_or(Value::unnamed_composite(vec![])),
                    ),
                    (
                        "weights",
                        call.args
                            .get(3)
                            .cloned()
                            .unwrap_or(Value::unnamed_composite(vec![])),
                    ),
                    (
                        "version_key",
                        call.args.get(4).cloned().unwrap_or(Value::from(0u64)),
                    ),
                ],
                "set_weights" => vec![
                    (
                        "netuid",
                        call.args.first().cloned().unwrap_or(Value::from(0u16)),
                    ),
                    (
                        "dests",
                        call.args
                            .get(1)
                            .cloned()
                            .unwrap_or(Value::unnamed_composite(vec![])),
                    ),
                    (
                        "weights",
                        call.args
                            .get(2)
                            .cloned()
                            .unwrap_or(Value::unnamed_composite(vec![])),
                    ),
                    (
                        "version_key",
                        call.args.get(3).cloned().unwrap_or(Value::from(0u64)),
                    ),
                ],
                "commit_mechanism_weights" => vec![
                    (
                        "netuid",
                        call.args.first().cloned().unwrap_or(Value::from(0u16)),
                    ),
                    (
                        "mecid",
                        call.args.get(1).cloned().unwrap_or(Value::from(0u8)),
                    ),
                    (
                        "hash",
                        call.args.get(2).cloned().unwrap_or(Value::from_bytes([])),
                    ),
                ],
                _ => call
                    .args
                    .iter()
                    .enumerate()
                    .map(|(idx, val)| (format!("arg_{}", idx), val.clone()))
                    .map(|(name, val)| (Box::leak(name.into_boxed_str()) as &str, val))
                    .collect(),
            };

            // Create the pallet call as a variant
            let pallet_call = Value::named_variant(
                call.function.clone(),
                call_args
                    .into_iter()
                    .map(|(name, val)| (name.to_string(), val))
                    .collect::<Vec<(String, Value)>>(),
            );

            // Wrap in the RuntimeCall variant for the module
            Value::named_variant(call.module.clone(), vec![("call", pallet_call)])
        })
        .collect();

    let args = vec![Value::unnamed_composite(call_values)];

    let tx_hash = client
        .submit_extrinsic(UTILITY_MODULE, "batch_all", args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to submit batch_all: {}", e))?;

    Ok(tx_hash)
}
