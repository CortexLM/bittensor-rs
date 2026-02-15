pub mod connection;
pub mod runtime;
pub mod signer;

use anyhow::Result;
use sp_core::crypto::AccountId32;
use std::sync::Arc;
use std::time::Duration;
use subxt::{dynamic::Value, PolkadotConfig};
use thiserror::Error;
use tracing::{debug, info, warn};

pub use connection::*;
pub use runtime::*;
pub use signer::{
    create_signer, signer_from_seed, BittensorSigner, ManagedSigner, NonceManager,
    SharedNonceManager,
};

pub const DEFAULT_RPC_URL: &str = "wss://entrypoint-finney.opentensor.ai:443";

pub const FALLBACK_ENDPOINTS: &[&str] = &[
    "wss://entrypoint-finney.opentensor.ai:443",
    "wss://finney.opentensor.ai:443",
];

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
    #[error("Nonce error: {0}")]
    Nonce(String),
    #[error("Rate limited: {0}")]
    RateLimited(String),
    #[error("Dispatch error: {0}")]
    DispatchError(String),
}

#[derive(Debug, Clone)]
pub struct ChainEvent {
    pub pallet_name: String,
    pub variant_name: String,
    pub pallet_index: u8,
    pub variant_index: u8,
    pub field_bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum DispatchResult {
    Success,
    Error {
        pallet_name: String,
        error_name: String,
        pallet_index: u8,
        error_index: u8,
        description: String,
    },
}

impl DispatchResult {
    pub fn is_success(&self) -> bool {
        matches!(self, DispatchResult::Success)
    }

    pub fn is_error(&self) -> bool {
        matches!(self, DispatchResult::Error { .. })
    }
}

#[derive(Debug, Clone)]
pub struct ExtrinsicResult {
    pub tx_hash: String,
    pub block_hash: Option<String>,
    pub events: Vec<ChainEvent>,
    pub dispatch_result: DispatchResult,
}

impl ExtrinsicResult {
    pub fn is_success(&self) -> bool {
        self.dispatch_result.is_success()
    }

    pub fn has_event(&self, pallet: &str, variant: &str) -> bool {
        self.events
            .iter()
            .any(|e| e.pallet_name == pallet && e.variant_name == variant)
    }

    pub fn find_events(&self, pallet: &str, variant: &str) -> Vec<&ChainEvent> {
        self.events
            .iter()
            .filter(|e| e.pallet_name == pallet && e.variant_name == variant)
            .collect()
    }
}

impl std::fmt::Display for ExtrinsicResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ExtrinsicResult(tx_hash={}", self.tx_hash)?;
        match &self.dispatch_result {
            DispatchResult::Success => write!(f, ", status=Success")?,
            DispatchResult::Error {
                pallet_name,
                error_name,
                ..
            } => write!(f, ", status=Error({}.{})", pallet_name, error_name)?,
        }
        write!(f, ", events={})", self.events.len())
    }
}

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub max_elapsed_time: Duration,
    pub retry_nonce_errors: bool,
    pub retry_rpc_errors: bool,
}

impl RetryPolicy {
    pub fn no_retry() -> Self {
        Self {
            max_retries: 0,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(1),
            max_elapsed_time: Duration::from_secs(5),
            retry_nonce_errors: false,
            retry_rpc_errors: false,
        }
    }

    pub fn standard() -> Self {
        Self::default()
    }

    pub fn aggressive() -> Self {
        Self {
            max_retries: 10,
            initial_delay: Duration::from_millis(50),
            max_delay: Duration::from_secs(30),
            max_elapsed_time: Duration::from_secs(120),
            retry_nonce_errors: true,
            retry_rpc_errors: true,
        }
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(200),
            max_delay: Duration::from_secs(10),
            max_elapsed_time: Duration::from_secs(60),
            retry_nonce_errors: true,
            retry_rpc_errors: true,
        }
    }
}

fn is_nonce_error(err: &Error) -> bool {
    let msg = err.to_string().to_lowercase();
    msg.contains("1014")
        || msg.contains("priority is too low")
        || msg.contains("stale")
        || msg.contains("nonce")
        || msg.contains("transaction is outdated")
}

fn is_retryable_rpc_error(err: &Error) -> bool {
    let msg = err.to_string().to_lowercase();
    msg.contains("503")
        || msg.contains("connection reset")
        || msg.contains("connection refused")
        || msg.contains("broken pipe")
        || msg.contains("timed out")
        || msg.contains("timeout")
        || msg.contains("websocket")
        || msg.contains("eof")
        || msg.contains("channel closed")
        || msg.contains("restart")
}

fn parse_events_from_in_block(
    in_block: &subxt::tx::TxInBlock<PolkadotConfig, subxt::OnlineClient<PolkadotConfig>>,
    metadata: &subxt::Metadata,
) -> (Vec<ChainEvent>, DispatchResult) {
    let events_result = std::thread::scope(|_| {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(in_block.fetch_events())
        })
    });

    match events_result {
        Ok(extrinsic_events) => {
            let mut events = Vec::new();
            let mut dispatch_result = DispatchResult::Success;

            for event_result in extrinsic_events.iter() {
                match event_result {
                    Ok(event_details) => {
                        let pallet_name = event_details.pallet_name().to_string();
                        let variant_name = event_details.variant_name().to_string();
                        let pallet_index = event_details.pallet_index();
                        let variant_index = event_details.variant_index();

                        if pallet_name == "System" && variant_name == "ExtrinsicFailed" {
                            dispatch_result =
                                decode_dispatch_error_from_event(&event_details, metadata);
                        }

                        events.push(ChainEvent {
                            pallet_name,
                            variant_name,
                            pallet_index,
                            variant_index,
                            field_bytes: event_details.field_bytes().to_vec(),
                        });
                    }
                    Err(e) => {
                        warn!("Failed to decode event: {}", e);
                    }
                }
            }

            (events, dispatch_result)
        }
        Err(e) => {
            warn!("Failed to fetch events: {}", e);
            (Vec::new(), DispatchResult::Success)
        }
    }
}

fn decode_dispatch_error_from_event(
    event_details: &subxt::events::EventDetails<PolkadotConfig>,
    metadata: &subxt::Metadata,
) -> DispatchResult {
    let field_bytes = event_details.field_bytes();
    if field_bytes.len() >= 4 && field_bytes[0] == 3 {
        let pallet_index = field_bytes[1];
        let error_index = field_bytes[2];

        let (pallet_name, error_name, description) =
            resolve_module_error(metadata, pallet_index, error_index);

        DispatchResult::Error {
            pallet_name,
            error_name,
            pallet_index,
            error_index,
            description,
        }
    } else {
        let desc = if !field_bytes.is_empty() {
            match field_bytes[0] {
                0 => "Other error".to_string(),
                1 => "Cannot lookup".to_string(),
                2 => "Bad origin".to_string(),
                4 => "Token error".to_string(),
                5 => "Arithmetic error".to_string(),
                6 => "Transactional error".to_string(),
                7 => "Exhausted".to_string(),
                8 => "Corruption".to_string(),
                9 => "Unavailable".to_string(),
                10 => "Root not allowed".to_string(),
                _ => format!("Unknown dispatch error variant: {}", field_bytes[0]),
            }
        } else {
            "Unknown dispatch error".to_string()
        };

        DispatchResult::Error {
            pallet_name: "System".to_string(),
            error_name: "DispatchError".to_string(),
            pallet_index: 0,
            error_index: 0,
            description: desc,
        }
    }
}

fn resolve_module_error(
    metadata: &subxt::Metadata,
    pallet_index: u8,
    error_index: u8,
) -> (String, String, String) {
    if let Some(pallet) = metadata.pallet_by_index(pallet_index) {
        let pallet_name = pallet.name().to_string();

        if let Some(error_variant) = pallet
            .error_variants()
            .and_then(|vars| vars.get(error_index as usize))
        {
            let error_name = error_variant.name.clone();
            let description = error_variant
                .docs
                .first()
                .cloned()
                .unwrap_or_else(|| format!("{}.{}", pallet_name, error_name));
            return (pallet_name, error_name, description);
        }

        let error_name = format!("Error({})", error_index);
        let description = format!("{}.{}", pallet_name, error_name);
        return (pallet_name, error_name, description);
    }

    (
        format!("Pallet({})", pallet_index),
        format!("Error({})", error_index),
        format!(
            "Module error: pallet_index={}, error_index={}",
            pallet_index, error_index
        ),
    )
}

#[derive(Debug)]
pub struct BittensorClient {
    pub api: subxt::OnlineClient<PolkadotConfig>,
    pub rpc_url: String,
    nonce_manager: Arc<NonceManager>,
    rate_limiter: Option<
        Arc<
            governor::RateLimiter<
                governor::state::NotKeyed,
                governor::state::InMemoryState,
                governor::clock::DefaultClock,
                governor::middleware::NoOpMiddleware,
            >,
        >,
    >,
}

impl BittensorClient {
    pub async fn new(rpc_url: impl Into<String>) -> Result<Self, Error> {
        let url = rpc_url.into();
        let api = subxt::OnlineClient::<PolkadotConfig>::from_url(&url).await?;

        Ok(Self {
            api,
            rpc_url: url,
            nonce_manager: Arc::new(NonceManager::new()),
            rate_limiter: None,
        })
    }

    pub async fn with_default() -> Result<Self, Error> {
        let url = std::env::var("BITTENSOR_RPC").unwrap_or_else(|_| DEFAULT_RPC_URL.to_string());
        Self::new(url).await
    }

    pub async fn with_failover() -> Result<Self, Error> {
        let env_url = std::env::var("BITTENSOR_RPC").ok();
        let endpoints: Vec<&str> = if let Some(ref url) = env_url {
            let mut eps = vec![url.as_str()];
            for ep in FALLBACK_ENDPOINTS {
                if *ep != url.as_str() {
                    eps.push(ep);
                }
            }
            eps
        } else {
            FALLBACK_ENDPOINTS.to_vec()
        };

        let mut last_error = None;
        for endpoint in &endpoints {
            info!("Attempting connection to {}", endpoint);
            match Self::new(*endpoint).await {
                Ok(client) => {
                    info!("Successfully connected to {}", endpoint);
                    return Ok(client);
                }
                Err(e) => {
                    warn!("Failed to connect to {}: {}", endpoint, e);
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| Error::Rpc("No endpoints available".to_string())))
    }

    pub fn with_nonce_manager(mut self, nonce_manager: Arc<NonceManager>) -> Self {
        self.nonce_manager = nonce_manager;
        self
    }

    pub fn with_rate_limiter(
        mut self,
        limiter: Arc<
            governor::RateLimiter<
                governor::state::NotKeyed,
                governor::state::InMemoryState,
                governor::clock::DefaultClock,
                governor::middleware::NoOpMiddleware,
            >,
        >,
    ) -> Self {
        self.rate_limiter = Some(limiter);
        self
    }

    pub fn nonce_manager(&self) -> &Arc<NonceManager> {
        &self.nonce_manager
    }

    pub fn api(&self) -> &subxt::OnlineClient<PolkadotConfig> {
        &self.api
    }

    pub fn rpc_url(&self) -> &str {
        &self.rpc_url
    }

    pub async fn fetch_nonce_from_chain(
        &self,
        account: &subxt::config::substrate::AccountId32,
    ) -> Result<u64, Error> {
        let nonce = self.api.tx().account_nonce(account).await?;
        Ok(nonce)
    }

    async fn ensure_nonce_initialized(&self, signer: &BittensorSigner) -> Result<(), Error> {
        let account_id = <BittensorSigner as subxt::tx::Signer<PolkadotConfig>>::account_id(signer);
        if self.nonce_manager.needs_refresh(&account_id).await {
            let on_chain_nonce = self.fetch_nonce_from_chain(&account_id).await?;
            self.nonce_manager
                .initialize_account(&account_id, on_chain_nonce)
                .await;
            debug!(
                "Initialized nonce for {:?} to {}",
                account_id, on_chain_nonce
            );
        }
        Ok(())
    }

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

    pub async fn storage_with_keys(
        &self,
        module: &str,
        entry: &str,
        keys: Vec<Value>,
    ) -> Result<Option<Value>, Error> {
        let storage_query = subxt::dynamic::storage(module, entry, keys);
        let storage = self.api.storage().at_latest().await?;
        let value = storage.fetch(&storage_query).await?;

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

        match result.to_value() {
            Ok(v) => Ok(Some(v.remove_context())),
            Err(e) => Err(Error::Decoding(format!(
                "Failed to decode runtime API result: {}",
                e
            ))),
        }
    }

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

    pub async fn account_balance(&self, account: &AccountId32) -> Result<u128, Error> {
        use parity_scale_codec::Encode;

        let account_bytes = account.encode();
        let account_value = Value::from_bytes(&account_bytes);

        let storage_query = subxt::dynamic::storage("System", "Account", vec![account_value]);
        let storage = self.api.storage().at_latest().await?;
        let data = storage.fetch(&storage_query).await?;

        match data {
            Some(thunk) => {
                let value = thunk
                    .to_value()
                    .map_err(|e| Error::Decoding(format!("Failed to decode account data: {}", e)))?
                    .remove_context();

                if let Ok(named) = crate::utils::decoders::decode_named_composite(&value) {
                    if let Some(data_value) = named.get("data") {
                        if let Ok(data_fields) =
                            crate::utils::decoders::decode_named_composite(data_value)
                        {
                            if let Some(free_value) = data_fields.get("free") {
                                if let Ok(balance) = crate::utils::decoders::decode_u128(free_value)
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

    pub async fn submit_extrinsic(
        &self,
        module: &str,
        function: &str,
        args: Vec<Value>,
        signer: &BittensorSigner,
        wait_for: ExtrinsicWait,
    ) -> Result<String, Error> {
        let result = self
            .submit_extrinsic_with_result(module, function, args, signer, wait_for)
            .await?;

        match result.dispatch_result {
            DispatchResult::Success => Ok(result.tx_hash),
            DispatchResult::Error {
                ref pallet_name,
                ref error_name,
                ref description,
                ..
            } => Err(Error::DispatchError(format!(
                "{}.{}: {}",
                pallet_name, error_name, description
            ))),
        }
    }

    pub async fn submit_extrinsic_with_result(
        &self,
        module: &str,
        function: &str,
        args: Vec<Value>,
        signer: &BittensorSigner,
        wait_for: ExtrinsicWait,
    ) -> Result<ExtrinsicResult, Error> {
        let policy = RetryPolicy::standard();
        self.submit_extrinsic_with_policy(module, function, args, signer, wait_for, &policy)
            .await
    }

    pub async fn submit_extrinsic_with_policy(
        &self,
        module: &str,
        function: &str,
        args: Vec<Value>,
        signer: &BittensorSigner,
        wait_for: ExtrinsicWait,
        policy: &RetryPolicy,
    ) -> Result<ExtrinsicResult, Error> {
        if let Some(ref limiter) = self.rate_limiter {
            if limiter.check().is_err() {
                return Err(Error::RateLimited(format!(
                    "Client-side rate limit exceeded for {}.{}",
                    module, function
                )));
            }
        }

        self.ensure_nonce_initialized(signer).await?;

        let account_id = <BittensorSigner as subxt::tx::Signer<PolkadotConfig>>::account_id(signer);

        let max_attempts = policy.max_retries + 1;
        let mut last_error: Option<Error> = None;

        for attempt in 0..max_attempts {
            if attempt > 0 {
                let delay = std::cmp::min(
                    policy.initial_delay * 2u32.saturating_pow(attempt - 1),
                    policy.max_delay,
                );
                debug!(
                    "Retry attempt {} for {}.{}, waiting {:?}",
                    attempt, module, function, delay
                );
                tokio::time::sleep(delay).await;
            }

            let nonce = self.nonce_manager.allocate_nonce(&account_id).await;
            let nonce = match nonce {
                Some(n) => n,
                None => {
                    let on_chain = self.fetch_nonce_from_chain(&account_id).await?;
                    self.nonce_manager
                        .initialize_account(&account_id, on_chain)
                        .await;
                    self.nonce_manager
                        .allocate_nonce(&account_id)
                        .await
                        .unwrap_or(on_chain)
                }
            };

            let call = subxt::dynamic::tx(module, function, args.clone());

            let tx_params = subxt::config::polkadot::PolkadotExtrinsicParamsBuilder::new()
                .nonce(nonce)
                .build();

            let submit_result = self.api.tx().create_signed(&call, signer, tx_params).await;

            let signed_tx = match submit_result {
                Ok(tx) => tx,
                Err(e) => {
                    let err = Error::Subxt(e);
                    if is_nonce_error(&err)
                        && policy.retry_nonce_errors
                        && attempt < max_attempts - 1
                    {
                        warn!(
                            "Nonce error during signing (attempt {}): {}",
                            attempt + 1,
                            err
                        );
                        self.nonce_manager.fail_nonce(&account_id, nonce).await;
                        let fresh = self.fetch_nonce_from_chain(&account_id).await?;
                        self.nonce_manager.reset_account(&account_id, fresh).await;
                        last_error = Some(err);
                        continue;
                    }
                    self.nonce_manager.fail_nonce(&account_id, nonce).await;
                    return Err(err);
                }
            };

            let mut tx_progress = match signed_tx.submit_and_watch().await {
                Ok(progress) => progress,
                Err(e) => {
                    let err = Error::Subxt(e);
                    self.nonce_manager.fail_nonce(&account_id, nonce).await;

                    if is_nonce_error(&err)
                        && policy.retry_nonce_errors
                        && attempt < max_attempts - 1
                    {
                        warn!("Nonce error on submit (attempt {}): {}", attempt + 1, err);
                        let fresh = self.fetch_nonce_from_chain(&account_id).await?;
                        self.nonce_manager.reset_account(&account_id, fresh).await;
                        last_error = Some(err);
                        continue;
                    }

                    if is_retryable_rpc_error(&err)
                        && policy.retry_rpc_errors
                        && attempt < max_attempts - 1
                    {
                        warn!(
                            "Retryable RPC error on submit (attempt {}): {}",
                            attempt + 1,
                            err
                        );
                        last_error = Some(err);
                        continue;
                    }

                    return Err(err);
                }
            };

            let tx_hash = format!("{:?}", tx_progress.extrinsic_hash());

            match wait_for {
                ExtrinsicWait::None => {
                    self.nonce_manager.confirm_nonce(&account_id, nonce).await;
                    return Ok(ExtrinsicResult {
                        tx_hash,
                        block_hash: None,
                        events: Vec::new(),
                        dispatch_result: DispatchResult::Success,
                    });
                }
                ExtrinsicWait::Included => {
                    let in_block_result = Self::wait_for_in_block(&mut tx_progress).await;
                    match in_block_result {
                        Ok(in_block) => {
                            self.nonce_manager.confirm_nonce(&account_id, nonce).await;
                            let block_hash = format!("{:?}", in_block.block_hash());
                            let metadata = self.api.metadata();
                            let (events, dispatch_result) =
                                parse_events_from_in_block(&in_block, &metadata);

                            return Ok(ExtrinsicResult {
                                tx_hash,
                                block_hash: Some(block_hash),
                                events,
                                dispatch_result,
                            });
                        }
                        Err(err) => {
                            self.nonce_manager.fail_nonce(&account_id, nonce).await;
                            if is_retryable_rpc_error(&err)
                                && policy.retry_rpc_errors
                                && attempt < max_attempts - 1
                            {
                                warn!(
                                    "Retryable error waiting for inclusion (attempt {}): {}",
                                    attempt + 1,
                                    err
                                );
                                last_error = Some(err);
                                continue;
                            }
                            return Err(err);
                        }
                    }
                }
                ExtrinsicWait::Finalized => match tx_progress.wait_for_finalized().await {
                    Ok(in_block) => {
                        self.nonce_manager.confirm_nonce(&account_id, nonce).await;
                        let block_hash = format!("{:?}", in_block.block_hash());
                        let metadata = self.api.metadata();
                        let (events, dispatch_result) =
                            parse_events_from_in_block(&in_block, &metadata);

                        return Ok(ExtrinsicResult {
                            tx_hash,
                            block_hash: Some(block_hash),
                            events,
                            dispatch_result,
                        });
                    }
                    Err(e) => {
                        self.nonce_manager.fail_nonce(&account_id, nonce).await;
                        let err = Error::Subxt(e);
                        if is_retryable_rpc_error(&err)
                            && policy.retry_rpc_errors
                            && attempt < max_attempts - 1
                        {
                            warn!(
                                "Retryable error waiting for finalization (attempt {}): {}",
                                attempt + 1,
                                err
                            );
                            last_error = Some(err);
                            continue;
                        }
                        return Err(err);
                    }
                },
            }
        }

        Err(last_error.unwrap_or_else(|| {
            Error::Transaction(format!(
                "Max retries ({}) exceeded for {}.{}",
                policy.max_retries, module, function
            ))
        }))
    }

    async fn wait_for_in_block(
        tx_progress: &mut subxt::tx::TxProgress<
            PolkadotConfig,
            subxt::OnlineClient<PolkadotConfig>,
        >,
    ) -> Result<subxt::tx::TxInBlock<PolkadotConfig, subxt::OnlineClient<PolkadotConfig>>, Error>
    {
        loop {
            match tx_progress.next().await {
                Some(Ok(status)) => match status {
                    subxt::tx::TxStatus::InBestBlock(in_block)
                    | subxt::tx::TxStatus::InFinalizedBlock(in_block) => return Ok(in_block),
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
        }
    }

    pub fn metadata(&self) -> subxt::Metadata {
        self.api.metadata()
    }

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

    pub async fn subscribe_finalized_blocks(
        &self,
    ) -> Result<impl futures::Stream<Item = Result<u64, Error>> + Send + '_, Error> {
        use futures::StreamExt;

        let block_stream = self.api.blocks().subscribe_finalized().await?;

        Ok(block_stream.map(|result| {
            result
                .map(|block| block.number() as u64)
                .map_err(Error::Subxt)
        }))
    }

    pub async fn block_hash(&self, block_number: u64) -> Result<Option<sp_core::H256>, Error> {
        let backend = self.api.backend();

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

    pub async fn query_constant(
        &self,
        module: &str,
        constant: &str,
    ) -> Result<Option<Value>, Error> {
        let metadata = self.api.metadata();

        if let Some(pallet) = metadata.pallet_by_name(module) {
            if let Some(constant_def) = pallet.constant_by_name(constant) {
                let constant_bytes = constant_def.value();
                return Ok(Some(Value::from_bytes(constant_bytes)));
            }
        }

        Err(Error::Decoding(format!(
            "Constant {}.{} not found in metadata",
            module, constant
        )))
    }

    pub async fn query_tx_rate_limit(&self) -> Result<u64, Error> {
        match self.storage("SubtensorModule", "TxRateLimit", None).await? {
            Some(val) => {
                if let Ok(v) = crate::utils::decoders::decode_u128(&val) {
                    Ok(v as u64)
                } else {
                    Ok(1)
                }
            }
            None => Ok(1),
        }
    }

    pub async fn query_weights_rate_limit(&self, netuid: u16) -> Result<u64, Error> {
        match self
            .storage(
                "SubtensorModule",
                "WeightsSetRateLimit",
                Some(vec![Value::from(netuid)]),
            )
            .await?
        {
            Some(val) => {
                if let Ok(v) = crate::utils::decoders::decode_u128(&val) {
                    Ok(v as u64)
                } else {
                    Ok(0)
                }
            }
            None => Ok(0),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ExtrinsicWait {
    None,
    Included,
    Finalized,
}

pub fn create_client_rate_limiter(
    ops_per_second: u32,
) -> Arc<
    governor::RateLimiter<
        governor::state::NotKeyed,
        governor::state::InMemoryState,
        governor::clock::DefaultClock,
        governor::middleware::NoOpMiddleware,
    >,
> {
    use std::num::NonZeroU32;

    let quota = governor::Quota::per_second(
        NonZeroU32::new(ops_per_second).unwrap_or(NonZeroU32::new(1).unwrap()),
    );
    Arc::new(governor::RateLimiter::direct(quota))
}
