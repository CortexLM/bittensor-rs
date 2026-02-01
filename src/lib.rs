
pub mod axon;
pub mod blocks;
pub mod chain;
pub mod cli;
pub mod config;
pub mod core;
pub mod crv4;
pub mod dendrite;
pub mod errors;
pub mod logging;
pub mod metagraph;
pub mod queries;
pub mod subtensor;
pub mod types;
pub mod utils;
pub mod validator;
pub mod wallet;

pub use chain::{BittensorClient, Error as ChainError};
pub use config::{AxonConfig, Config, LoggingConfig as ConfigLoggingConfig, SubtensorConfig};
pub use metagraph::{sync_metagraph, Metagraph};

// Re-export logging module
pub use logging::{
    init_default_logging, init_logging, is_initialized, BittensorFormatter, CompactFormatter,
    JsonFormatter, LogFormat, LoggingConfig,
};

// Re-export types first (includes liquidity types)
pub use types::*;

// Re-export queries with specific naming to avoid conflicts
pub use queries::{
    balances::*, chain_info::*, delegates::*, identity::*, metagraph_queries::*, stakes::*,
    subnets::*,
};

// Re-export neurons module (use prefix for children/parents to avoid conflict with validator)
pub use queries::neurons::{
    fetch_axon_info, fetch_prometheus_info, get_all_neuron_certificates, get_neuron_certificate,
    get_neuron_for_pubkey_and_subnet, neuron, neurons, query_neuron_from_storage, Certificate,
};

// Children/parents queries accessible via module path
pub use queries::neurons as neuron_queries;

// Re-export liquidity queries with module prefix to avoid conflict
pub use queries::liquidity as liquidity_queries;

// Re-export validator functions (except weights to avoid conflict)
pub use validator::{
    liquidity::*, mechanism::*, registration::*, root::*, serving::*, staking::*, take::*,
    transfer::*, utility::*,
};

// Children module accessible via module path to avoid conflict with query children
pub use validator::children as validator_children;

// Re-export weights module with prefix to avoid conflict
pub use validator::weights as validator_weights;

// Re-export utils with specific modules to avoid conflicts
pub use utils::{balance, crypto, encode, scale, ss58};

// Re-export decoders module
pub use utils::decoders;

// Re-export weights utils with prefix to avoid conflict
pub use utils::weights as utils_weights;

// Re-export key crypto functions for commit-reveal
pub use utils::crypto::{
    commit_hash_to_hex, generate_mechanism_commit_hash, generate_salt,
    generate_subtensor_commit_hash, hex_to_commit_hash_32, salt_u8_to_u16, verify_commit_hash,
};

// Re-export CRv4 (Commit-Reveal v4 with timelock encryption)
pub use crv4::{
    calculate_reveal_round, commit_timelocked_mechanism_weights, commit_timelocked_weights,
    encrypt_for_round, get_commit_reveal_version, get_last_drand_round, get_mechid_storage_index,
    get_reveal_period, get_tempo, prepare_and_commit_crv4_mechanism_weights,
    prepare_and_commit_crv4_weights, prepare_crv4_commit, verify_encrypted_data, Crv4CommitData,
    Crv4PersistedState, Crv4StateManager, DrandInfo, WeightsTlockPayload,
    DEFAULT_COMMIT_REVEAL_VERSION, DRAND_QUICKNET_GENESIS, DRAND_QUICKNET_PK_HEX,
    DRAND_ROUND_INTERVAL_SECS,
};

// Re-export high-level Subtensor API (like Python SDK)
pub use subtensor::{
    PendingCommit, Salt, Subtensor, SubtensorBuilder, SubtensorState, WeightResponse,
    WeightResponseData,
};

// Re-export mechanism functions from validator
pub use validator::mechanism::{
    commit_mechanism_weights, reveal_mechanism_weights, set_mechanism_weights,
};

// Re-export Dendrite HTTP client
pub use dendrite::{
    Dendrite, DendriteRequest, DendriteResponse, StreamingResponse, StreamingSynapse,
};

// Re-export Axon HTTP server
pub use axon::{
    Axon, AxonConfig as AxonServerConfig, AxonState, HandlerContext, RequestPriority,
    VerifiedRequest, AXON_VERSION,
};

// Re-export wallet module for key management
pub use wallet::{
    default_wallet_path, is_legacy_format, list_wallets, list_wallets_at, migrate_legacy_keyfile,
    wallet_path, Keyfile, KeyfileData, KeyfileError, Keypair, KeypairError, Mnemonic,
    MnemonicError, Wallet, WalletError as WalletModuleError, BITTENSOR_SS58_FORMAT, KEYFILE_VERSION,
};

// Re-export comprehensive error types
pub use errors::{
    // Unified error type and result alias
    BittensorError, BittensorResult,
    // Chain/Network Errors
    BlockNotFound, ChainConnectionError, ChainQueryError, ExtrinsicError, MetadataError,
    TransactionFailed,
    // Wallet Errors
    InvalidKeyfile, InvalidMnemonic, KeyExists, KeyfileDecryptionError, KeyfileNotFound,
    KeyfilePermissionError, WalletError,
    // Registration Errors
    AlreadyRegistered, NotRegistered, PowFailed, RegistrationFailed,
    // Stake Errors
    InsufficientBalance, InsufficientStake, StakeFailed,
    // Weights Errors
    InvalidWeights, TooManyWeights, WeightVersionMismatch, WeightsError,
    // Synapse/Communication Errors
    SerializationError, SynapseBlacklisted, SynapseError, SynapseTimeout, SynapseUnauthorized,
    // Dendrite Errors
    AxonUnreachable, DendriteError, InvalidResponse,
    // Axon Errors
    AxonConfigError, AxonError, AxonNotServing,
    // Senate/Governance Errors
    AlreadySenateMember, NotSenateMember, ProposalNotFound, VoteFailed,
};
