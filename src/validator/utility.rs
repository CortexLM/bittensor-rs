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
        let uid_values: Vec<Value> = uids.iter().map(|uid| Value::u128(*uid as u128)).collect();
        let weight_values: Vec<Value> = weights.iter().map(|w| Value::u128(*w as u128)).collect();

        Self::new(
            "SubtensorModule",
            "set_mechanism_weights",
            vec![
                Value::u128(netuid as u128),
                Value::u128(mechanism_id as u128),
                Value::unnamed_composite(uid_values),
                Value::unnamed_composite(weight_values),
                Value::u128(version_key as u128),
            ],
        )
    }

    /// Create a set_weights call for batching (mechanism 0)
    pub fn set_weights(netuid: u16, uids: &[u16], weights: &[u16], version_key: u64) -> Self {
        let uid_values: Vec<Value> = uids.iter().map(|uid| Value::u128(*uid as u128)).collect();
        let weight_values: Vec<Value> = weights.iter().map(|w| Value::u128(*w as u128)).collect();

        Self::new(
            "SubtensorModule",
            "set_weights",
            vec![
                Value::u128(netuid as u128),
                Value::unnamed_composite(uid_values),
                Value::unnamed_composite(weight_values),
                Value::u128(version_key as u128),
            ],
        )
    }

    /// Create a commit_mechanism_weights call for batching
    pub fn commit_mechanism_weights(netuid: u16, mechanism_id: u8, commit_hash: &[u8; 32]) -> Self {
        Self::new(
            "SubtensorModule",
            "commit_mechanism_weights",
            vec![
                Value::u128(netuid as u128),
                Value::u128(mechanism_id as u128),
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
                "set_mechanism_weights" => {
                    vec![
                        (
                            "netuid",
                            call.args.first().cloned().unwrap_or(Value::u128(0)),
                        ),
                        ("mecid", call.args.get(1).cloned().unwrap_or(Value::u128(0))),
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
                            call.args.get(4).cloned().unwrap_or(Value::u128(0)),
                        ),
                    ]
                }
                "set_weights" => {
                    vec![
                        (
                            "netuid",
                            call.args.first().cloned().unwrap_or(Value::u128(0)),
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
                            call.args.get(3).cloned().unwrap_or(Value::u128(0)),
                        ),
                    ]
                }
                "commit_mechanism_weights" => {
                    vec![
                        (
                            "netuid",
                            call.args.first().cloned().unwrap_or(Value::u128(0)),
                        ),
                        ("mecid", call.args.get(1).cloned().unwrap_or(Value::u128(0))),
                        (
                            "commit_hash",
                            call.args
                                .get(2)
                                .cloned()
                                .unwrap_or(Value::from_bytes([0u8; 32])),
                        ),
                    ]
                }
                _ => {
                    // Generic case - pass args as-is with indexed names
                    call.args
                        .iter()
                        .enumerate()
                        .map(|(i, v)| {
                            let name: &'static str = match i {
                                0 => "arg0",
                                1 => "arg1",
                                2 => "arg2",
                                3 => "arg3",
                                4 => "arg4",
                                5 => "arg5",
                                _ => "argN",
                            };
                            (name, v.clone())
                        })
                        .collect()
                }
            };

            // Build the inner call variant
            let inner_call = Value::named_variant(&call.function, call_args);

            // Wrap in the pallet variant (RuntimeCall::SubtensorModule(...))
            Value::named_variant(&call.module, [("call", inner_call)])
        })
        .collect();

    let args = vec![Value::unnamed_composite(call_values)];

    client
        .submit_extrinsic(UTILITY_MODULE, "batch_all", args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to execute batch_all: {}", e))
}

/// Execute a batch of calls, continuing even if some fail
/// Uses Utility.batch which continues on errors
pub async fn batch(
    client: &BittensorClient,
    signer: &BittensorSigner,
    calls: Vec<BatchCall>,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    if calls.is_empty() {
        return Err(anyhow::anyhow!("Cannot batch empty call list"));
    }

    let call_values: Vec<Value> = calls
        .iter()
        .map(|call| {
            let call_args: Vec<(&str, Value)> = match call.function.as_str() {
                "set_mechanism_weights" => {
                    vec![
                        (
                            "netuid",
                            call.args.first().cloned().unwrap_or(Value::u128(0)),
                        ),
                        ("mecid", call.args.get(1).cloned().unwrap_or(Value::u128(0))),
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
                            call.args.get(4).cloned().unwrap_or(Value::u128(0)),
                        ),
                    ]
                }
                "set_weights" => {
                    vec![
                        (
                            "netuid",
                            call.args.first().cloned().unwrap_or(Value::u128(0)),
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
                            call.args.get(3).cloned().unwrap_or(Value::u128(0)),
                        ),
                    ]
                }
                "commit_mechanism_weights" => {
                    vec![
                        (
                            "netuid",
                            call.args.first().cloned().unwrap_or(Value::u128(0)),
                        ),
                        ("mecid", call.args.get(1).cloned().unwrap_or(Value::u128(0))),
                        (
                            "commit_hash",
                            call.args
                                .get(2)
                                .cloned()
                                .unwrap_or(Value::from_bytes([0u8; 32])),
                        ),
                    ]
                }
                _ => call
                    .args
                    .iter()
                    .enumerate()
                    .map(|(i, v)| {
                        let name: &'static str = match i {
                            0 => "arg0",
                            1 => "arg1",
                            2 => "arg2",
                            3 => "arg3",
                            4 => "arg4",
                            5 => "arg5",
                            _ => "argN",
                        };
                        (name, v.clone())
                    })
                    .collect(),
            };

            let inner_call = Value::named_variant(&call.function, call_args);
            Value::named_variant(&call.module, [("call", inner_call)])
        })
        .collect();

    let args = vec![Value::unnamed_composite(call_values)];

    client
        .submit_extrinsic(UTILITY_MODULE, "batch", args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to execute batch: {}", e))
}

/// Execute a batch of calls, ignoring failures (force batch)
/// Uses Utility.force_batch which never fails
pub async fn force_batch(
    client: &BittensorClient,
    signer: &BittensorSigner,
    calls: Vec<BatchCall>,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    if calls.is_empty() {
        return Err(anyhow::anyhow!("Cannot batch empty call list"));
    }

    let call_values: Vec<Value> = calls
        .iter()
        .map(|call| {
            let call_args: Vec<(&str, Value)> = match call.function.as_str() {
                "set_mechanism_weights" => {
                    vec![
                        (
                            "netuid",
                            call.args.first().cloned().unwrap_or(Value::u128(0)),
                        ),
                        ("mecid", call.args.get(1).cloned().unwrap_or(Value::u128(0))),
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
                            call.args.get(4).cloned().unwrap_or(Value::u128(0)),
                        ),
                    ]
                }
                "set_weights" => {
                    vec![
                        (
                            "netuid",
                            call.args.first().cloned().unwrap_or(Value::u128(0)),
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
                            call.args.get(3).cloned().unwrap_or(Value::u128(0)),
                        ),
                    ]
                }
                "commit_mechanism_weights" => {
                    vec![
                        (
                            "netuid",
                            call.args.first().cloned().unwrap_or(Value::u128(0)),
                        ),
                        ("mecid", call.args.get(1).cloned().unwrap_or(Value::u128(0))),
                        (
                            "commit_hash",
                            call.args
                                .get(2)
                                .cloned()
                                .unwrap_or(Value::from_bytes([0u8; 32])),
                        ),
                    ]
                }
                _ => call
                    .args
                    .iter()
                    .enumerate()
                    .map(|(i, v)| {
                        let name: &'static str = match i {
                            0 => "arg0",
                            1 => "arg1",
                            2 => "arg2",
                            3 => "arg3",
                            4 => "arg4",
                            5 => "arg5",
                            _ => "argN",
                        };
                        (name, v.clone())
                    })
                    .collect(),
            };

            let inner_call = Value::named_variant(&call.function, call_args);
            Value::named_variant(&call.module, [("call", inner_call)])
        })
        .collect();

    let args = vec![Value::unnamed_composite(call_values)];

    client
        .submit_extrinsic(UTILITY_MODULE, "force_batch", args, signer, wait_for)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to execute force_batch: {}", e))
}

/// Helper to batch set_mechanism_weights calls for multiple mechanisms
pub async fn batch_set_mechanism_weights(
    client: &BittensorClient,
    signer: &BittensorSigner,
    netuid: u16,
    mechanism_weights: Vec<(u8, Vec<u16>, Vec<u16>)>, // (mechanism_id, uids, weights)
    version_key: u64,
    wait_for: ExtrinsicWait,
) -> Result<String> {
    let calls: Vec<BatchCall> = mechanism_weights
        .into_iter()
        .map(|(mecid, uids, weights)| {
            BatchCall::set_mechanism_weights(netuid, mecid, &uids, &weights, version_key)
        })
        .collect();

    batch_all(client, signer, calls, wait_for).await
}
