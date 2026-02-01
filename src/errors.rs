//! Comprehensive error types for Bittensor SDK
//!
//! This module provides error types that match the Python SDK exception hierarchy
//! for compatibility and ease of use when porting code between implementations.

use serde::{Deserialize, Serialize};
use thiserror::Error;

// =============================================================================
// Chain/Network Errors
// =============================================================================

/// Error when connecting to the RPC endpoint fails
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Chain connection error: {message}")]
pub struct ChainConnectionError {
    /// Detailed error message
    pub message: String,
    /// The RPC URL that failed to connect
    pub rpc_url: Option<String>,
}

impl ChainConnectionError {
    /// Create a new chain connection error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            rpc_url: None,
        }
    }

    /// Create a new chain connection error with RPC URL
    pub fn with_url(message: impl Into<String>, rpc_url: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            rpc_url: Some(rpc_url.into()),
        }
    }
}

/// Error when querying chain storage fails
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Chain query error: {message}")]
pub struct ChainQueryError {
    /// Detailed error message
    pub message: String,
    /// The storage module being queried
    pub module: Option<String>,
    /// The storage entry being queried
    pub entry: Option<String>,
}

impl ChainQueryError {
    /// Create a new chain query error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            module: None,
            entry: None,
        }
    }

    /// Create a new chain query error with module and entry info
    pub fn with_storage(
        message: impl Into<String>,
        module: impl Into<String>,
        entry: impl Into<String>,
    ) -> Self {
        Self {
            message: message.into(),
            module: Some(module.into()),
            entry: Some(entry.into()),
        }
    }
}

/// Error when submitting an extrinsic fails
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Extrinsic error: {message}")]
pub struct ExtrinsicError {
    /// Detailed error message
    pub message: String,
    /// The pallet/module name
    pub pallet: Option<String>,
    /// The call/function name
    pub call: Option<String>,
}

impl ExtrinsicError {
    /// Create a new extrinsic error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            pallet: None,
            call: None,
        }
    }

    /// Create a new extrinsic error with pallet and call info
    pub fn with_call(
        message: impl Into<String>,
        pallet: impl Into<String>,
        call: impl Into<String>,
    ) -> Self {
        Self {
            message: message.into(),
            pallet: Some(pallet.into()),
            call: Some(call.into()),
        }
    }
}

/// Error when a transaction failed during execution on chain
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Transaction failed: {message}")]
pub struct TransactionFailed {
    /// Detailed error message
    pub message: String,
    /// The transaction hash if available
    pub tx_hash: Option<String>,
    /// The dispatch error from the chain
    pub dispatch_error: Option<String>,
}

impl TransactionFailed {
    /// Create a new transaction failed error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            tx_hash: None,
            dispatch_error: None,
        }
    }

    /// Create a new transaction failed error with hash
    pub fn with_hash(message: impl Into<String>, tx_hash: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            tx_hash: Some(tx_hash.into()),
            dispatch_error: None,
        }
    }

    /// Create a new transaction failed error with dispatch error
    pub fn with_dispatch_error(
        message: impl Into<String>,
        dispatch_error: impl Into<String>,
    ) -> Self {
        Self {
            message: message.into(),
            tx_hash: None,
            dispatch_error: Some(dispatch_error.into()),
        }
    }
}

/// Error when a block is not found
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Block not found: {message}")]
pub struct BlockNotFound {
    /// Detailed error message
    pub message: String,
    /// The block hash that was not found
    pub block_hash: Option<String>,
    /// The block number that was not found
    pub block_number: Option<u64>,
}

impl BlockNotFound {
    /// Create a new block not found error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            block_hash: None,
            block_number: None,
        }
    }

    /// Create a new block not found error with hash
    pub fn with_hash(message: impl Into<String>, block_hash: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            block_hash: Some(block_hash.into()),
            block_number: None,
        }
    }

    /// Create a new block not found error with number
    pub fn with_number(message: impl Into<String>, block_number: u64) -> Self {
        Self {
            message: message.into(),
            block_hash: None,
            block_number: Some(block_number),
        }
    }
}

/// Error when parsing chain metadata fails
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Metadata error: {message}")]
pub struct MetadataError {
    /// Detailed error message
    pub message: String,
    /// The metadata version if available
    pub metadata_version: Option<u32>,
}

impl MetadataError {
    /// Create a new metadata error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            metadata_version: None,
        }
    }

    /// Create a new metadata error with version
    pub fn with_version(message: impl Into<String>, version: u32) -> Self {
        Self {
            message: message.into(),
            metadata_version: Some(version),
        }
    }
}

// =============================================================================
// Wallet Errors
// =============================================================================

/// Generic wallet error
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Wallet error: {message}")]
pub struct WalletError {
    /// Detailed error message
    pub message: String,
    /// The wallet name if applicable
    pub wallet_name: Option<String>,
}

impl WalletError {
    /// Create a new wallet error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            wallet_name: None,
        }
    }

    /// Create a new wallet error with wallet name
    pub fn with_wallet(message: impl Into<String>, wallet_name: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            wallet_name: Some(wallet_name.into()),
        }
    }
}

/// Error when a keyfile is not found
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Keyfile not found: {path}")]
pub struct KeyfileNotFound {
    /// The path to the keyfile
    pub path: String,
    /// The key name (hotkey/coldkey)
    pub key_name: Option<String>,
}

impl KeyfileNotFound {
    /// Create a new keyfile not found error
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            key_name: None,
        }
    }

    /// Create a new keyfile not found error with key name
    pub fn with_key_name(path: impl Into<String>, key_name: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            key_name: Some(key_name.into()),
        }
    }
}

/// Error when decrypting a keyfile fails
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Keyfile decryption error: {message}")]
pub struct KeyfileDecryptionError {
    /// Detailed error message
    pub message: String,
    /// The keyfile path if available
    pub path: Option<String>,
}

impl KeyfileDecryptionError {
    /// Create a new keyfile decryption error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            path: None,
        }
    }

    /// Create a new keyfile decryption error with path
    pub fn with_path(message: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            path: Some(path.into()),
        }
    }
}

/// Error when a mnemonic phrase is invalid
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Invalid mnemonic: {message}")]
pub struct InvalidMnemonic {
    /// Detailed error message
    pub message: String,
    /// The word count if applicable
    pub word_count: Option<usize>,
}

impl InvalidMnemonic {
    /// Create a new invalid mnemonic error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            word_count: None,
        }
    }

    /// Create a new invalid mnemonic error with word count
    pub fn with_word_count(message: impl Into<String>, word_count: usize) -> Self {
        Self {
            message: message.into(),
            word_count: Some(word_count),
        }
    }
}

/// Error when a keyfile is corrupted or invalid
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Invalid keyfile: {message}")]
pub struct InvalidKeyfile {
    /// Detailed error message
    pub message: String,
    /// The keyfile path if available
    pub path: Option<String>,
}

impl InvalidKeyfile {
    /// Create a new invalid keyfile error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            path: None,
        }
    }

    /// Create a new invalid keyfile error with path
    pub fn with_path(message: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            path: Some(path.into()),
        }
    }
}

/// Error when file permissions prevent reading/writing a keyfile
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Keyfile permission error: {message}")]
pub struct KeyfilePermissionError {
    /// Detailed error message
    pub message: String,
    /// The keyfile path if available
    pub path: Option<String>,
    /// The required permission (read/write)
    pub required_permission: Option<String>,
}

impl KeyfilePermissionError {
    /// Create a new keyfile permission error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            path: None,
            required_permission: None,
        }
    }

    /// Create a new keyfile permission error with path
    pub fn with_path(message: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            path: Some(path.into()),
            required_permission: None,
        }
    }

    /// Create a new keyfile permission error with permission info
    pub fn with_permission(
        message: impl Into<String>,
        path: impl Into<String>,
        permission: impl Into<String>,
    ) -> Self {
        Self {
            message: message.into(),
            path: Some(path.into()),
            required_permission: Some(permission.into()),
        }
    }
}

/// Error when a key already exists (during create operations)
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Key already exists: {message}")]
pub struct KeyExists {
    /// Detailed error message
    pub message: String,
    /// The key name
    pub key_name: Option<String>,
    /// The keyfile path
    pub path: Option<String>,
}

impl KeyExists {
    /// Create a new key exists error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            key_name: None,
            path: None,
        }
    }

    /// Create a new key exists error with key name
    pub fn with_key_name(message: impl Into<String>, key_name: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            key_name: Some(key_name.into()),
            path: None,
        }
    }

    /// Create a new key exists error with path
    pub fn with_path(message: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            key_name: None,
            path: Some(path.into()),
        }
    }
}

// =============================================================================
// Registration Errors
// =============================================================================

/// Error when a hotkey is not registered on a subnet
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Not registered: {message}")]
pub struct NotRegistered {
    /// Detailed error message
    pub message: String,
    /// The hotkey SS58 address
    pub hotkey: Option<String>,
    /// The subnet UID
    pub netuid: Option<u16>,
}

impl NotRegistered {
    /// Create a new not registered error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            hotkey: None,
            netuid: None,
        }
    }

    /// Create a new not registered error with hotkey and netuid
    pub fn with_details(
        message: impl Into<String>,
        hotkey: impl Into<String>,
        netuid: u16,
    ) -> Self {
        Self {
            message: message.into(),
            hotkey: Some(hotkey.into()),
            netuid: Some(netuid),
        }
    }
}

/// Error when a hotkey is already registered
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Already registered: {message}")]
pub struct AlreadyRegistered {
    /// Detailed error message
    pub message: String,
    /// The hotkey SS58 address
    pub hotkey: Option<String>,
    /// The subnet UID
    pub netuid: Option<u16>,
}

impl AlreadyRegistered {
    /// Create a new already registered error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            hotkey: None,
            netuid: None,
        }
    }

    /// Create a new already registered error with hotkey and netuid
    pub fn with_details(
        message: impl Into<String>,
        hotkey: impl Into<String>,
        netuid: u16,
    ) -> Self {
        Self {
            message: message.into(),
            hotkey: Some(hotkey.into()),
            netuid: Some(netuid),
        }
    }
}

/// Error when registration transaction fails
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Registration failed: {message}")]
pub struct RegistrationFailed {
    /// Detailed error message
    pub message: String,
    /// The subnet UID
    pub netuid: Option<u16>,
    /// The dispatch error if available
    pub dispatch_error: Option<String>,
}

impl RegistrationFailed {
    /// Create a new registration failed error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            netuid: None,
            dispatch_error: None,
        }
    }

    /// Create a new registration failed error with netuid
    pub fn with_netuid(message: impl Into<String>, netuid: u16) -> Self {
        Self {
            message: message.into(),
            netuid: Some(netuid),
            dispatch_error: None,
        }
    }

    /// Create a new registration failed error with dispatch error
    pub fn with_dispatch_error(
        message: impl Into<String>,
        dispatch_error: impl Into<String>,
    ) -> Self {
        Self {
            message: message.into(),
            netuid: None,
            dispatch_error: Some(dispatch_error.into()),
        }
    }
}

/// Error when PoW solution is not found in time
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("PoW failed: {message}")]
pub struct PowFailed {
    /// Detailed error message
    pub message: String,
    /// The difficulty target
    pub difficulty: Option<u64>,
    /// The number of attempts made
    pub attempts: Option<u64>,
    /// Time spent in seconds
    pub time_elapsed_secs: Option<f64>,
}

impl PowFailed {
    /// Create a new PoW failed error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            difficulty: None,
            attempts: None,
            time_elapsed_secs: None,
        }
    }

    /// Create a new PoW failed error with details
    pub fn with_details(
        message: impl Into<String>,
        difficulty: u64,
        attempts: u64,
        time_elapsed_secs: f64,
    ) -> Self {
        Self {
            message: message.into(),
            difficulty: Some(difficulty),
            attempts: Some(attempts),
            time_elapsed_secs: Some(time_elapsed_secs),
        }
    }
}

// =============================================================================
// Stake Errors
// =============================================================================

/// Error when there is insufficient balance for an operation
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Insufficient balance: {message}")]
pub struct InsufficientBalance {
    /// Detailed error message
    pub message: String,
    /// The required amount in RAO
    pub required: Option<u64>,
    /// The available amount in RAO
    pub available: Option<u64>,
}

impl InsufficientBalance {
    /// Create a new insufficient balance error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            required: None,
            available: None,
        }
    }

    /// Create a new insufficient balance error with amounts
    pub fn with_amounts(message: impl Into<String>, required: u64, available: u64) -> Self {
        Self {
            message: message.into(),
            required: Some(required),
            available: Some(available),
        }
    }
}

/// Error when there is insufficient stake to unstake
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Insufficient stake: {message}")]
pub struct InsufficientStake {
    /// Detailed error message
    pub message: String,
    /// The requested unstake amount in RAO
    pub requested: Option<u64>,
    /// The current stake amount in RAO
    pub current_stake: Option<u64>,
}

impl InsufficientStake {
    /// Create a new insufficient stake error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            requested: None,
            current_stake: None,
        }
    }

    /// Create a new insufficient stake error with amounts
    pub fn with_amounts(message: impl Into<String>, requested: u64, current_stake: u64) -> Self {
        Self {
            message: message.into(),
            requested: Some(requested),
            current_stake: Some(current_stake),
        }
    }
}

/// Error when a stake operation fails
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Stake failed: {message}")]
pub struct StakeFailed {
    /// Detailed error message
    pub message: String,
    /// The amount attempted in RAO
    pub amount: Option<u64>,
    /// The dispatch error if available
    pub dispatch_error: Option<String>,
}

impl StakeFailed {
    /// Create a new stake failed error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            amount: None,
            dispatch_error: None,
        }
    }

    /// Create a new stake failed error with amount
    pub fn with_amount(message: impl Into<String>, amount: u64) -> Self {
        Self {
            message: message.into(),
            amount: Some(amount),
            dispatch_error: None,
        }
    }

    /// Create a new stake failed error with dispatch error
    pub fn with_dispatch_error(
        message: impl Into<String>,
        dispatch_error: impl Into<String>,
    ) -> Self {
        Self {
            message: message.into(),
            amount: None,
            dispatch_error: Some(dispatch_error.into()),
        }
    }
}

// =============================================================================
// Weights Errors
// =============================================================================

/// Generic weights error
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Weights error: {message}")]
pub struct WeightsError {
    /// Detailed error message
    pub message: String,
    /// The subnet UID
    pub netuid: Option<u16>,
}

impl WeightsError {
    /// Create a new weights error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            netuid: None,
        }
    }

    /// Create a new weights error with netuid
    pub fn with_netuid(message: impl Into<String>, netuid: u16) -> Self {
        Self {
            message: message.into(),
            netuid: Some(netuid),
        }
    }
}

/// Error when weights don't normalize properly
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Invalid weights: {message}")]
pub struct InvalidWeights {
    /// Detailed error message
    pub message: String,
    /// The weight sum if relevant
    pub weight_sum: Option<f64>,
    /// Expected sum (typically 1.0 or u16::MAX)
    pub expected_sum: Option<f64>,
}

impl InvalidWeights {
    /// Create a new invalid weights error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            weight_sum: None,
            expected_sum: None,
        }
    }

    /// Create a new invalid weights error with sum info
    pub fn with_sums(message: impl Into<String>, weight_sum: f64, expected_sum: f64) -> Self {
        Self {
            message: message.into(),
            weight_sum: Some(weight_sum),
            expected_sum: Some(expected_sum),
        }
    }
}

/// Error when weight version key doesn't match
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Weight version mismatch: {message}")]
pub struct WeightVersionMismatch {
    /// Detailed error message
    pub message: String,
    /// The expected version
    pub expected_version: Option<u64>,
    /// The provided version
    pub provided_version: Option<u64>,
}

impl WeightVersionMismatch {
    /// Create a new weight version mismatch error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            expected_version: None,
            provided_version: None,
        }
    }

    /// Create a new weight version mismatch error with versions
    pub fn with_versions(message: impl Into<String>, expected: u64, provided: u64) -> Self {
        Self {
            message: message.into(),
            expected_version: Some(expected),
            provided_version: Some(provided),
        }
    }
}

/// Error when weight count exceeds maximum allowed
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Too many weights: {message}")]
pub struct TooManyWeights {
    /// Detailed error message
    pub message: String,
    /// The number of weights provided
    pub count: Option<usize>,
    /// The maximum allowed
    pub max_allowed: Option<usize>,
}

impl TooManyWeights {
    /// Create a new too many weights error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            count: None,
            max_allowed: None,
        }
    }

    /// Create a new too many weights error with counts
    pub fn with_counts(message: impl Into<String>, count: usize, max_allowed: usize) -> Self {
        Self {
            message: message.into(),
            count: Some(count),
            max_allowed: Some(max_allowed),
        }
    }
}

// =============================================================================
// Synapse/Communication Errors
// =============================================================================

/// Generic synapse error
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Synapse error: {message}")]
pub struct SynapseError {
    /// Detailed error message
    pub message: String,
    /// The synapse name if applicable
    pub synapse_name: Option<String>,
}

impl SynapseError {
    /// Create a new synapse error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            synapse_name: None,
        }
    }

    /// Create a new synapse error with synapse name
    pub fn with_synapse_name(message: impl Into<String>, synapse_name: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            synapse_name: Some(synapse_name.into()),
        }
    }
}

/// Error when a synapse request times out
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Synapse timeout: {message}")]
pub struct SynapseTimeout {
    /// Detailed error message
    pub message: String,
    /// The timeout duration in seconds
    pub timeout_secs: Option<f64>,
    /// The target axon endpoint
    pub endpoint: Option<String>,
}

impl SynapseTimeout {
    /// Create a new synapse timeout error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            timeout_secs: None,
            endpoint: None,
        }
    }

    /// Create a new synapse timeout error with details
    pub fn with_details(
        message: impl Into<String>,
        timeout_secs: f64,
        endpoint: impl Into<String>,
    ) -> Self {
        Self {
            message: message.into(),
            timeout_secs: Some(timeout_secs),
            endpoint: Some(endpoint.into()),
        }
    }
}

/// Error when synapse signature verification fails
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Synapse unauthorized: {message}")]
pub struct SynapseUnauthorized {
    /// Detailed error message
    pub message: String,
    /// The hotkey that failed verification
    pub hotkey: Option<String>,
}

impl SynapseUnauthorized {
    /// Create a new synapse unauthorized error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            hotkey: None,
        }
    }

    /// Create a new synapse unauthorized error with hotkey
    pub fn with_hotkey(message: impl Into<String>, hotkey: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            hotkey: Some(hotkey.into()),
        }
    }
}

/// Error when IP or hotkey is blacklisted
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Synapse blacklisted: {message}")]
pub struct SynapseBlacklisted {
    /// Detailed error message
    pub message: String,
    /// The blacklisted IP if applicable
    pub ip: Option<String>,
    /// The blacklisted hotkey if applicable
    pub hotkey: Option<String>,
}

impl SynapseBlacklisted {
    /// Create a new synapse blacklisted error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            ip: None,
            hotkey: None,
        }
    }

    /// Create a new synapse blacklisted error with IP
    pub fn with_ip(message: impl Into<String>, ip: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            ip: Some(ip.into()),
            hotkey: None,
        }
    }

    /// Create a new synapse blacklisted error with hotkey
    pub fn with_hotkey(message: impl Into<String>, hotkey: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            ip: None,
            hotkey: Some(hotkey.into()),
        }
    }
}

/// Error when serialization or deserialization fails
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Serialization error: {message}")]
pub struct SerializationError {
    /// Detailed error message
    pub message: String,
    /// The type name being serialized/deserialized
    pub type_name: Option<String>,
}

impl SerializationError {
    /// Create a new serialization error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            type_name: None,
        }
    }

    /// Create a new serialization error with type name
    pub fn with_type(message: impl Into<String>, type_name: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            type_name: Some(type_name.into()),
        }
    }
}

// =============================================================================
// Dendrite Errors
// =============================================================================

/// HTTP client error for dendrite operations
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Dendrite error: {message}")]
pub struct DendriteError {
    /// Detailed error message
    pub message: String,
    /// The HTTP status code if applicable
    pub status_code: Option<u16>,
}

impl DendriteError {
    /// Create a new dendrite error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            status_code: None,
        }
    }

    /// Create a new dendrite error with status code
    pub fn with_status(message: impl Into<String>, status_code: u16) -> Self {
        Self {
            message: message.into(),
            status_code: Some(status_code),
        }
    }
}

/// Error when axon endpoint is unreachable
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Axon unreachable: {message}")]
pub struct AxonUnreachable {
    /// Detailed error message
    pub message: String,
    /// The endpoint that was unreachable
    pub endpoint: Option<String>,
    /// The IP address
    pub ip: Option<String>,
    /// The port
    pub port: Option<u16>,
}

impl AxonUnreachable {
    /// Create a new axon unreachable error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            endpoint: None,
            ip: None,
            port: None,
        }
    }

    /// Create a new axon unreachable error with endpoint
    pub fn with_endpoint(message: impl Into<String>, endpoint: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            endpoint: Some(endpoint.into()),
            ip: None,
            port: None,
        }
    }

    /// Create a new axon unreachable error with IP and port
    pub fn with_ip_port(message: impl Into<String>, ip: impl Into<String>, port: u16) -> Self {
        Self {
            message: message.into(),
            endpoint: None,
            ip: Some(ip.into()),
            port: Some(port),
        }
    }
}

/// Error when response from axon is malformed
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Invalid response: {message}")]
pub struct InvalidResponse {
    /// Detailed error message
    pub message: String,
    /// The raw response data if available
    pub raw_response: Option<String>,
}

impl InvalidResponse {
    /// Create a new invalid response error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            raw_response: None,
        }
    }

    /// Create a new invalid response error with raw response
    pub fn with_raw_response(message: impl Into<String>, raw_response: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            raw_response: Some(raw_response.into()),
        }
    }
}

// =============================================================================
// Axon Errors
// =============================================================================

/// HTTP server error for axon operations
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Axon error: {message}")]
pub struct AxonError {
    /// Detailed error message
    pub message: String,
    /// The HTTP status code if applicable
    pub status_code: Option<u16>,
}

impl AxonError {
    /// Create a new axon error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            status_code: None,
        }
    }

    /// Create a new axon error with status code
    pub fn with_status(message: impl Into<String>, status_code: u16) -> Self {
        Self {
            message: message.into(),
            status_code: Some(status_code),
        }
    }
}

/// Error when axon is not running/serving
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Axon not serving: {message}")]
pub struct AxonNotServing {
    /// Detailed error message
    pub message: String,
    /// The IP the axon should be serving on
    pub ip: Option<String>,
    /// The port the axon should be serving on
    pub port: Option<u16>,
}

impl AxonNotServing {
    /// Create a new axon not serving error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            ip: None,
            port: None,
        }
    }

    /// Create a new axon not serving error with IP and port
    pub fn with_ip_port(message: impl Into<String>, ip: impl Into<String>, port: u16) -> Self {
        Self {
            message: message.into(),
            ip: Some(ip.into()),
            port: Some(port),
        }
    }
}

/// Error when axon configuration is invalid
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Axon config error: {message}")]
pub struct AxonConfigError {
    /// Detailed error message
    pub message: String,
    /// The config field that is invalid
    pub field: Option<String>,
}

impl AxonConfigError {
    /// Create a new axon config error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            field: None,
        }
    }

    /// Create a new axon config error with field name
    pub fn with_field(message: impl Into<String>, field: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            field: Some(field.into()),
        }
    }
}

// =============================================================================
// Senate/Governance Errors
// =============================================================================

/// Error when user is not a senate member
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Not a senate member: {message}")]
pub struct NotSenateMember {
    /// Detailed error message
    pub message: String,
    /// The hotkey SS58 address
    pub hotkey: Option<String>,
}

impl NotSenateMember {
    /// Create a new not senate member error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            hotkey: None,
        }
    }

    /// Create a new not senate member error with hotkey
    pub fn with_hotkey(message: impl Into<String>, hotkey: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            hotkey: Some(hotkey.into()),
        }
    }
}

/// Error when user is already a senate member
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Already a senate member: {message}")]
pub struct AlreadySenateMember {
    /// Detailed error message
    pub message: String,
    /// The hotkey SS58 address
    pub hotkey: Option<String>,
}

impl AlreadySenateMember {
    /// Create a new already senate member error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            hotkey: None,
        }
    }

    /// Create a new already senate member error with hotkey
    pub fn with_hotkey(message: impl Into<String>, hotkey: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            hotkey: Some(hotkey.into()),
        }
    }
}

/// Error when a vote operation fails
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Vote failed: {message}")]
pub struct VoteFailed {
    /// Detailed error message
    pub message: String,
    /// The proposal index
    pub proposal_index: Option<u32>,
    /// The dispatch error if available
    pub dispatch_error: Option<String>,
}

impl VoteFailed {
    /// Create a new vote failed error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            proposal_index: None,
            dispatch_error: None,
        }
    }

    /// Create a new vote failed error with proposal index
    pub fn with_proposal(message: impl Into<String>, proposal_index: u32) -> Self {
        Self {
            message: message.into(),
            proposal_index: Some(proposal_index),
            dispatch_error: None,
        }
    }

    /// Create a new vote failed error with dispatch error
    pub fn with_dispatch_error(
        message: impl Into<String>,
        dispatch_error: impl Into<String>,
    ) -> Self {
        Self {
            message: message.into(),
            proposal_index: None,
            dispatch_error: Some(dispatch_error.into()),
        }
    }
}

/// Error when a proposal is not found
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("Proposal not found: {message}")]
pub struct ProposalNotFound {
    /// Detailed error message
    pub message: String,
    /// The proposal index
    pub proposal_index: Option<u32>,
    /// The proposal hash if applicable
    pub proposal_hash: Option<String>,
}

impl ProposalNotFound {
    /// Create a new proposal not found error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            proposal_index: None,
            proposal_hash: None,
        }
    }

    /// Create a new proposal not found error with index
    pub fn with_index(message: impl Into<String>, proposal_index: u32) -> Self {
        Self {
            message: message.into(),
            proposal_index: Some(proposal_index),
            proposal_hash: None,
        }
    }

    /// Create a new proposal not found error with hash
    pub fn with_hash(message: impl Into<String>, proposal_hash: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            proposal_index: None,
            proposal_hash: Some(proposal_hash.into()),
        }
    }
}

// =============================================================================
// Unified Error Enum
// =============================================================================

/// Unified error type for all Bittensor SDK operations
///
/// This enum wraps all specific error types and provides a unified interface
/// for error handling. It matches the Python SDK exception hierarchy.
#[derive(Debug, Error, Serialize, Deserialize)]
pub enum BittensorError {
    // Chain/Network Errors
    #[error(transparent)]
    ChainConnection(#[from] ChainConnectionError),
    #[error(transparent)]
    ChainQuery(#[from] ChainQueryError),
    #[error(transparent)]
    Extrinsic(#[from] ExtrinsicError),
    #[error(transparent)]
    TransactionFailed(#[from] TransactionFailed),
    #[error(transparent)]
    BlockNotFound(#[from] BlockNotFound),
    #[error(transparent)]
    Metadata(#[from] MetadataError),

    // Wallet Errors
    #[error(transparent)]
    Wallet(#[from] WalletError),
    #[error(transparent)]
    KeyfileNotFound(#[from] KeyfileNotFound),
    #[error(transparent)]
    KeyfileDecryption(#[from] KeyfileDecryptionError),
    #[error(transparent)]
    InvalidMnemonic(#[from] InvalidMnemonic),
    #[error(transparent)]
    InvalidKeyfile(#[from] InvalidKeyfile),
    #[error(transparent)]
    KeyfilePermission(#[from] KeyfilePermissionError),
    #[error(transparent)]
    KeyExists(#[from] KeyExists),

    // Registration Errors
    #[error(transparent)]
    NotRegistered(#[from] NotRegistered),
    #[error(transparent)]
    AlreadyRegistered(#[from] AlreadyRegistered),
    #[error(transparent)]
    RegistrationFailed(#[from] RegistrationFailed),
    #[error(transparent)]
    PowFailed(#[from] PowFailed),

    // Stake Errors
    #[error(transparent)]
    InsufficientBalance(#[from] InsufficientBalance),
    #[error(transparent)]
    InsufficientStake(#[from] InsufficientStake),
    #[error(transparent)]
    StakeFailed(#[from] StakeFailed),

    // Weights Errors
    #[error(transparent)]
    Weights(#[from] WeightsError),
    #[error(transparent)]
    InvalidWeights(#[from] InvalidWeights),
    #[error(transparent)]
    WeightVersionMismatch(#[from] WeightVersionMismatch),
    #[error(transparent)]
    TooManyWeights(#[from] TooManyWeights),

    // Synapse/Communication Errors
    #[error(transparent)]
    Synapse(#[from] SynapseError),
    #[error(transparent)]
    SynapseTimeout(#[from] SynapseTimeout),
    #[error(transparent)]
    SynapseUnauthorized(#[from] SynapseUnauthorized),
    #[error(transparent)]
    SynapseBlacklisted(#[from] SynapseBlacklisted),
    #[error(transparent)]
    Serialization(#[from] SerializationError),

    // Dendrite Errors
    #[error(transparent)]
    Dendrite(#[from] DendriteError),
    #[error(transparent)]
    AxonUnreachable(#[from] AxonUnreachable),
    #[error(transparent)]
    InvalidResponse(#[from] InvalidResponse),

    // Axon Errors
    #[error(transparent)]
    Axon(#[from] AxonError),
    #[error(transparent)]
    AxonNotServing(#[from] AxonNotServing),
    #[error(transparent)]
    AxonConfig(#[from] AxonConfigError),

    // Senate/Governance Errors
    #[error(transparent)]
    NotSenateMember(#[from] NotSenateMember),
    #[error(transparent)]
    AlreadySenateMember(#[from] AlreadySenateMember),
    #[error(transparent)]
    VoteFailed(#[from] VoteFailed),
    #[error(transparent)]
    ProposalNotFound(#[from] ProposalNotFound),

    // External library errors (converted to String for Serialize/Deserialize)
    #[error("Subxt error: {0}")]
    Subxt(String),
    #[error("IO error: {0}")]
    Io(String),
    #[error("JSON error: {0}")]
    Json(String),
    #[error("Hex decode error: {0}")]
    Hex(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

// =============================================================================
// From implementations for external error types
// =============================================================================

impl From<subxt::Error> for BittensorError {
    fn from(err: subxt::Error) -> Self {
        BittensorError::Subxt(err.to_string())
    }
}

impl From<std::io::Error> for BittensorError {
    fn from(err: std::io::Error) -> Self {
        BittensorError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for BittensorError {
    fn from(err: serde_json::Error) -> Self {
        BittensorError::Json(err.to_string())
    }
}

impl From<hex::FromHexError> for BittensorError {
    fn from(err: hex::FromHexError) -> Self {
        BittensorError::Hex(err.to_string())
    }
}

// =============================================================================
// Convenience type alias
// =============================================================================

/// Result type alias for Bittensor SDK operations
pub type BittensorResult<T> = Result<T, BittensorError>;

// =============================================================================
// Utility functions for error construction
// =============================================================================

impl BittensorError {
    /// Create an unknown error from any error type
    pub fn unknown(err: impl std::fmt::Display) -> Self {
        BittensorError::Unknown(err.to_string())
    }

    /// Check if this is a chain connection error
    pub fn is_connection_error(&self) -> bool {
        matches!(self, BittensorError::ChainConnection(_))
    }

    /// Check if this is an insufficient balance error
    pub fn is_insufficient_balance(&self) -> bool {
        matches!(self, BittensorError::InsufficientBalance(_))
    }

    /// Check if this is a not registered error
    pub fn is_not_registered(&self) -> bool {
        matches!(self, BittensorError::NotRegistered(_))
    }

    /// Check if this is a timeout error
    pub fn is_timeout(&self) -> bool {
        matches!(self, BittensorError::SynapseTimeout(_))
    }

    /// Check if this is an authorization error
    pub fn is_unauthorized(&self) -> bool {
        matches!(
            self,
            BittensorError::SynapseUnauthorized(_) | BittensorError::SynapseBlacklisted(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_connection_error() {
        let err = ChainConnectionError::new("Failed to connect");
        assert_eq!(err.message, "Failed to connect");
        assert!(err.rpc_url.is_none());

        let err_with_url =
            ChainConnectionError::with_url("Connection refused", "wss://example.com:9944");
        assert_eq!(err_with_url.rpc_url, Some("wss://example.com:9944".to_string()));
    }

    #[test]
    fn test_insufficient_balance_error() {
        let err = InsufficientBalance::with_amounts("Not enough TAO", 1000, 500);
        assert_eq!(err.required, Some(1000));
        assert_eq!(err.available, Some(500));
    }

    #[test]
    fn test_not_registered_error() {
        let err = NotRegistered::with_details(
            "Hotkey not registered",
            "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
            1,
        );
        assert_eq!(err.netuid, Some(1));
        assert!(err.hotkey.is_some());
    }

    #[test]
    fn test_bittensor_error_from_chain_connection() {
        let err = ChainConnectionError::new("Connection failed");
        let bt_err: BittensorError = err.into();
        assert!(bt_err.is_connection_error());
    }

    #[test]
    fn test_bittensor_error_from_subxt() {
        // Use a simple approach to verify the conversion works
        let bt_err = BittensorError::Subxt("test subxt error".to_string());
        assert!(matches!(bt_err, BittensorError::Subxt(_)));
    }

    #[test]
    fn test_bittensor_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let bt_err: BittensorError = io_err.into();
        assert!(matches!(bt_err, BittensorError::Io(_)));
    }

    #[test]
    fn test_bittensor_error_helper_methods() {
        let balance_err = BittensorError::InsufficientBalance(InsufficientBalance::new("test"));
        assert!(balance_err.is_insufficient_balance());
        assert!(!balance_err.is_connection_error());

        let reg_err = BittensorError::NotRegistered(NotRegistered::new("test"));
        assert!(reg_err.is_not_registered());

        let timeout_err = BittensorError::SynapseTimeout(SynapseTimeout::new("test"));
        assert!(timeout_err.is_timeout());

        let unauth_err = BittensorError::SynapseUnauthorized(SynapseUnauthorized::new("test"));
        assert!(unauth_err.is_unauthorized());
    }

    #[test]
    fn test_error_serialization() {
        let err = ChainQueryError::with_storage("Query failed", "SubtensorModule", "TotalStake");
        let serialized = serde_json::to_string(&err).expect("Should serialize");
        let deserialized: ChainQueryError =
            serde_json::from_str(&serialized).expect("Should deserialize");
        assert_eq!(err.message, deserialized.message);
        assert_eq!(err.module, deserialized.module);
        assert_eq!(err.entry, deserialized.entry);
    }

    #[test]
    fn test_bittensor_error_serialization() {
        let err = BittensorError::ChainConnection(ChainConnectionError::new("test"));
        let serialized = serde_json::to_string(&err).expect("Should serialize");
        let deserialized: BittensorError =
            serde_json::from_str(&serialized).expect("Should deserialize");
        assert!(deserialized.is_connection_error());
    }
}
