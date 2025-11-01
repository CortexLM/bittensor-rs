/// Core constants from the Bittensor protocol
/// These match the values defined in the Python implementation

/// TAO/RAO conversion factor (1 TAO = 1e9 RAO)
pub const RAOPERTAO: u64 = 1_000_000_000;

/// Global maximum subnet count
pub const GLOBAL_MAX_SUBNET_COUNT: u16 = 4096;

/// SS58 format for Bittensor addresses
pub const SS58_FORMAT: u16 = 42;

/// Block time in seconds
pub const BLOCKTIME: u64 = 12;

/// Default Axon server port
pub const DEFAULT_AXON_PORT: u16 = 8091;

/// Default Axon IP (IPv6 any)
pub const DEFAULT_AXON_IP: &str = "[::]";

/// Default max workers for Axon
pub const DEFAULT_AXON_MAX_WORKERS: usize = 10;

/// Currency symbols
pub const TAO_SYMBOL: char = '\u{03C4}'; // τ
pub const RAO_SYMBOL: char = '\u{03C1}'; // ρ

/// Network names
pub const NETWORK_FINNEY: &str = "finney";
pub const NETWORK_TEST: &str = "test";
pub const NETWORK_ARCHIVE: &str = "archive";
pub const NETWORK_LOCAL: &str = "local";

/// Default network
pub const DEFAULT_NETWORK: &str = NETWORK_FINNEY;

/// Network endpoints
pub const FINNEY_ENDPOINT: &str = "wss://entrypoint-finney.opentensor.ai:443";
pub const FINNEY_TEST_ENDPOINT: &str = "wss://test.finney.opentensor.ai:443";
pub const ARCHIVE_ENDPOINT: &str = "wss://archive.chain.opentensor.ai:443";
pub const LOCAL_ENDPOINT: &str = "ws://127.0.0.1:9944";

/// Default endpoint
pub const DEFAULT_ENDPOINT: &str = FINNEY_ENDPOINT;

/// Root TAO stake weight
pub const ROOT_TAO_STAKE_WEIGHT: f64 = 0.18;

/// Min/Max tick values for liquidity pools
pub const MIN_TICK: i32 = -887272;
pub const MAX_TICK: i32 = 887272;

/// Tick step for price calculations
pub const TICK_STEP: f64 = 1.0001;
