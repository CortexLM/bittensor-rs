pub mod runtime;
pub mod signer;

use anyhow::Result;
use sp_core::crypto::AccountId32;
use subxt::{dynamic::Value, PolkadotConfig};
use thiserror::Error;

pub use runtime::*;
pub use signer::{create_signer, signer_from_seed, BittensorSigner};

/// Default RPC endpoint (managed by Opentensor)
/// Same as Bittensor Python's DEFAULT_ENDPOINT
pub const DEFAULT_RPC_URL: &str = "wss://entrypoint-finney.opentensor.ai:443";

/// Error types for chain operations
#[derive(Debug, Error)]
pub enum Error {
    #[error("Subxt error: {0}")]
    Subxt(#[from] subxt::Error),
    #[error("RPC error: {0}")]
    Rpc(String),
    #[error("Encoding error: {0}")]
    Encoding(String),
    #[error("Decoding error: {0}")]
    Decoding(String),
    #[error("Invalid account: {0}")]
    InvalidAccount(String),
    #[error("Transaction error: {0}")]
    Transaction(String),
}

/// Bittensor client for interacting with the chain
pub struct BittensorClient {
    pub api: subxt::OnlineClient<PolkadotConfig>,
    pub rpc_url: String,
}

impl BittensorClient {
    /// Create a new Bittensor client connected to the specified RPC endpoint
    pub async fn new(rpc_url: impl Into<String>) -> Result<Self, Error> {
        let url = rpc_url.into();
        let api = subxt::OnlineClient::<PolkadotConfig>::from_url(&url).await?;

        Ok(Self { api, rpc_url: url })
    }

    /// Get the underlying subxt API client
    pub fn api(&self) -> &subxt::OnlineClient<PolkadotConfig> {
        &self.api
    }

    /// Get the RPC URL
    pub fn rpc_url(&self) -> &str {
        &self.rpc_url
    }

    /// Query a storage entry at the latest finalized block
    pub async fn storage(
        &self,
        module: &str,
        entry: &str,
        key: Option<Vec<Value>>,
    ) -> Result<Option<Value>, Error> {
        let keys = key.unwrap_or_default();
        let storage_query = subxt::dynamic::storage(module, entry, keys);
        let storage = self.api.storage().at_latest().await?;
        let value = storage.fetch(&storage_query).await?;

        // Convert DecodedValueThunk to Value
        match value {
            Some(thunk) => {
                // to_value() returns Result<Value, DecodeError>
                match thunk.to_value() {
                    Ok(v) => Ok(Some(v.remove_context())),
                    Err(e) => Err(Error::Decoding(format!(
                        "Failed to decode storage value: {}",
                        e
                    ))),
                }
            }
            None => Ok(None),
        }
    }

    /// Query a storage entry with multiple keys at the latest finalized block
    pub async fn storage_with_keys(
        &self,
        module: &str,
        entry: &str,
        keys: Vec<Value>,
    ) -> Result<Option<Value>, Error> {
        let storage_query = subxt::dynamic::storage(module, entry, keys);
        let storage = self.api.storage().at_latest().await?;
        let value = storage.fetch(&storage_query).await?;

        // Convert DecodedValueThunk to Value
        match value {
            Some(thunk) => match thunk.to_value() {
                Ok(v) => Ok(Some(v.remove_context())),
                Err(e) => Err(Error::Decoding(format!(
                    "Failed to decode storage value: {}",
                    e
                ))),
            },
            None => Ok(None),
        }
    }

    /// Query storage at a specific block hash
    pub async fn storage_at_block(
        &self,
        module: &str,
        entry: &str,
        keys: Vec<Value>,
        block_hash: sp_core::H256,
    ) -> Result<Option<Value>, Error> {
        let storage_query = subxt::dynamic::storage(module, entry, keys);
        let storage = self.api.storage().at(block_hash);
        let value = storage.fetch(&storage_query).await?;

        // Convert DecodedValueThunk to Value
        match value {
            Some(thunk) => match thunk.to_value() {
                Ok(v) => Ok(Some(v.remove_context())),
                Err(e) => Err(Error::Decoding(format!(
                    "Failed to decode storage value: {}",
                    e
                ))),
            },
            None => Ok(None),
        }
    }

    /// Query a runtime API call
    pub async fn runtime_api(
        &self,
        runtime_api: &str,
        method: &str,
        params: Vec<Value>,
    ) -> Result<Option<Value>, Error> {
        let api_call = subxt::dynamic::runtime_api_call(runtime_api, method, params);
        let result = self
            .api
            .runtime_api()
            .at_latest()
            .await?
            .call(api_call)
            .await?;

        // Runtime API returns DecodedValueThunk - convert to Value
        match result.to_value() {
            Ok(v) => Ok(Some(v.remove_context())),
            Err(e) => Err(Error::Decoding(format!(
                "Failed to decode runtime API result: {}",
                e
            ))),
        }
    }

    /// Call a runtime API
    pub async fn runtime_api_call(
        &self,
        runtime_api: &str,
        method: &str,
        params: Option<Vec<u8>>,
    ) -> Result<Vec<u8>, Error> {
        let params_ref = params.as_deref();
        let runtime_api_call = format!("{}_{}", runtime_api, method);
        let payload = self
            .api
            .runtime_api()
            .at_latest()
            .await?
            .call_raw(&runtime_api_call, params_ref)
            .await?;
        Ok(payload)
    }

    /// Get the balance of an account
    pub async fn account_balance(&self, account: &AccountId32) -> Result<u128, Error> {
        use parity_scale_codec::Encode;

        let account_bytes = account.encode();
        let account_value = Value::from_bytes(&account_bytes);

        let storage_query = subxt::dynamic::storage("System", "Account", vec![account_value]);
        let storage = self.api.storage().at_latest().await?;
        let data = storage.fetch(&storage_query).await?;

        // Decode AccountInfo structure: { nonce, consumers, providers, sufficients, data: { free, reserved, frozen } }
        // We need to extract the balance from data.free
        match data {
            Some(thunk) => {
                let value = thunk
                    .to_value()
                    .map_err(|e| Error::Decoding(format!("Failed to decode account data: {}", e)))?
                    .remove_context();

                // Try to extract balance from the account data structure
                // AccountInfo structure: account data contains free balance
                // System.Account returns AccountInfo structure which includes data field

                // Use proper SCALE decoding to extract balance
                // The account structure contains a 'data' field with balance information
                if let Ok(named) = crate::utils::scale_decode::decode_named_composite(&value) {
                    // Look for 'data' field which contains balance info
                    if let Some(data_value) = named.get("data") {
                        if let Ok(data_fields) =
                            crate::utils::scale_decode::decode_named_composite(data_value)
                        {
                            // Extract 'free' balance
                            if let Some(free_value) = data_fields.get("free") {
                                if let Ok(balance) =
                                    crate::utils::scale_decode::decode_u128(free_value)
                                {
                                    return Ok(balance);
                                }
                            }
                        }
                    }
                }

                Err(Error::Decoding(
                    "Failed to extract balance from account data".to_string(),
                ))
            }
            None => Err(Error::Decoding(
                "Account balance not found in storage".to_string(),
            )),
        }
    }

    /// Submit an extrinsic using a dynamic call
    pub async fn submit_extrinsic(
        &self,
        module: &str,
        function: &str,
        args: Vec<Value>,
        signer: &BittensorSigner,
        wait_for: ExtrinsicWait,
    ) -> Result<String, Error> {
        let call = subxt::dynamic::tx(module, function, args);

        // Sign and submit transaction using subxt 0.44 API
        let mut tx_client = self
            .api
            .tx()
            .sign_and_submit_then_watch_default(&call, signer)
            .await?;

        match wait_for {
            ExtrinsicWait::Included => {
                // Wait for transaction to be included in a block
                let in_block = loop {
                    match tx_client.next().await {
                        Some(Ok(status)) => match status {
                            subxt::tx::TxStatus::InBestBlock(in_block)
                            | subxt::tx::TxStatus::InFinalizedBlock(in_block) => break in_block,
                            subxt::tx::TxStatus::Error { message } => {
                                return Err(Error::Transaction(format!(
                                    "Transaction error: {}",
                                    message
                                )))
                            }
                            subxt::tx::TxStatus::Invalid { message } => {
                                return Err(Error::Transaction(format!(
                                    "Invalid transaction: {}",
                                    message
                                )))
                            }
                            subxt::tx::TxStatus::Dropped { message } => {
                                return Err(Error::Transaction(format!(
                                    "Transaction dropped: {}",
                                    message
                                )))
                            }
                            _ => continue,
                        },
                        Some(Err(e)) => {
                            return Err(Error::Transaction(format!(
                                "Transaction status error: {}",
                                e
                            )))
                        }
                        None => {
                            return Err(Error::Transaction(
                                "Transaction stream ended unexpectedly".to_string(),
                            ))
                        }
                    }
                };
                Ok(format!("{:?}", in_block.extrinsic_hash()))
            }
            ExtrinsicWait::Finalized => {
                let finalized = tx_client.wait_for_finalized_success().await?;
                Ok(format!("{:?}", finalized.extrinsic_hash()))
            }
            ExtrinsicWait::None => Ok(format!("{:?}", tx_client.extrinsic_hash())),
        }
    }

    /// Get metadata
    pub fn metadata(&self) -> subxt::Metadata {
        self.api.metadata()
    }

    /// Get the current block number
    pub async fn block_number(&self) -> Result<u64, Error> {
        let finalized_head = self.api.backend().latest_finalized_block_ref().await?;
        let header = self
            .api
            .backend()
            .block_header(finalized_head.hash())
            .await
            .map_err(|e| Error::Rpc(format!("Failed to get block header: {}", e)))?;

        if let Some(header) = header {
            let number = header.number;
            Ok(number as u64)
        } else {
            Err(Error::Rpc("Block header not found".to_string()))
        }
    }

    /// Get block hash for a given block number
    pub async fn block_hash(&self, block_number: u64) -> Result<Option<sp_core::H256>, Error> {
        // Get block hash via backend
        let backend = self.api.backend();

        // Fetch block header by number to get its hash
        match backend
            .block_header(backend.latest_finalized_block_ref().await?.hash())
            .await
        {
            Ok(Some(header)) if header.number as u64 == block_number => {
                Ok(Some(backend.latest_finalized_block_ref().await?.hash()))
            }
            _ => Ok(None),
        }
    }

    /// Query a constant value via metadata lookup
    pub async fn query_constant(
        &self,
        module: &str,
        constant: &str,
    ) -> Result<Option<Value>, Error> {
        // Constants are stored in metadata - use metadata to get constant value
        let metadata = self.api.metadata();

        // Find the pallet/constant in metadata and return its value
        // This requires metadata introspection
        // For production: use metadata.pallet(module)?.constant(constant)?.value_bytes
        if let Some(pallet) = metadata.pallet_by_name(module) {
            if let Some(constant_def) = pallet.constant_by_name(constant) {
                // Constants are already encoded values
                let constant_bytes = constant_def.value();
                return Ok(Some(Value::from_bytes(constant_bytes)));
            }
        }

        Err(Error::Decoding(format!(
            "Constant {}.{} not found in metadata",
            module, constant
        )))
    }
}

/// Wait options for extrinsics
#[derive(Debug, Clone, Copy)]
pub enum ExtrinsicWait {
    /// Don't wait
    None,
    /// Wait for inclusion in a block
    Included,
    /// Wait for finalization
    Finalized,
}
