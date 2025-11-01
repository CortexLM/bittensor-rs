pub mod chain;
pub mod core;
pub mod metagraph;
pub mod queries;
pub mod types;
pub mod utils;
pub mod validator;

pub use chain::{BittensorClient, Error as ChainError};
pub use metagraph::Metagraph;

// Re-export types first (includes liquidity types)
pub use types::*;

// Re-export queries with specific naming to avoid conflicts
pub use queries::{
    // All query functions except liquidity module
    neurons::*, subnets::*, stakes::*, balances::*, delegates::*, 
    metagraph_queries::*, chain_info::*, identity::*, neurons_storage::*
};
// Re-export liquidity queries with module prefix to avoid conflict
pub use queries::liquidity as liquidity_queries;

// Re-export validator functions (except weights to avoid conflict)
pub use validator::{
    registration::*, staking::*, serving::*, transfer::*, root::*,
    children::*, liquidity::*, mechanism::*, take::*
};
// Re-export weights module with prefix to avoid conflict
pub use validator::weights as validator_weights;

// Re-export utils with specific modules to avoid conflicts
pub use utils::{crypto, ss58, balance, scale, encode};
// Re-export value_decode module
pub use utils::value_decode;
// Re-export scale_decode module separately (has some duplicate names with value_decode)
pub use utils::scale_decode;
// Re-export weights utils with prefix to avoid conflict
pub use utils::weights as utils_weights;
