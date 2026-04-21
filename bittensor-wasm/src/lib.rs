//! bittensor-wasm — WASM-compatible subset of the bittensor-rs SDK.
//!
//! This crate provides browser/edge-compatible types and async query functions
//! for the Bittensor network, compiled to `wasm32-unknown-unknown`.
//!
//! # Architecture
//!
//! - **Types**: Re-implemented with `#[wasm_bindgen]` attributes to be
//!   JavaScript-accessible. The original `bittensor-core` crate depends on
//!   `subxt` (which pulls in `tokio`), so we cannot depend on it directly.
//! - **Queries**: Lightweight JSON-RPC wrappers using `gloo-net` HTTP POST.
//!   These connect to Subtensor RPC endpoints directly, bypassing the
//!   heavy `subxt`/`tokio` stack.
//! - **Synapse types**: Wrapped from `bittensor-synapse` (which has no
//!   `tokio` dependencies).
//!
//! # Excluded (not WASM-compatible)
//!
//! - Wallet encryption/decryption (NaCl needs libsodium)
//! - Extrinsic submission (signing needs platform-specific keystore)
//! - Full chain client (depends on tokio/subxt)

use wasm_bindgen::prelude::*;

pub mod prelude {
    //! Re-exports of commonly used types.
    pub use crate::types::{
        AxonInfo, Balance, DelegateInfo, NetworkConfig, NeuronInfoLite, RegistrationInfo,
        StakeInfo, SubnetHyperparams, SubnetInfo, TerminalInfo,
    };
}

pub mod queries;
pub mod types;

// ---------------------------------------------------------------------------
// Re-exported constants and utilities from bittensor-synapse
// ---------------------------------------------------------------------------

/// Returns the axon header prefix string (`"bt_header_axon_"`).
#[wasm_bindgen(js_name = axonPrefix)]
pub fn axon_prefix() -> String {
    bittensor_synapse::header::keys::AXON_PREFIX.to_string()
}

/// Returns the dendrite header prefix string (`"bt_header_dendrite_"`).
#[wasm_bindgen(js_name = dendritePrefix)]
pub fn dendrite_prefix() -> String {
    bittensor_synapse::header::keys::DENDRITE_PREFIX.to_string()
}

/// Returns the input-obj header prefix string (`"bt_header_input_obj_"`).
#[wasm_bindgen(js_name = inputObjPrefix)]
pub fn input_obj_prefix() -> String {
    bittensor_synapse::header::keys::INPUT_OBJ_PREFIX.to_string()
}

/// Compute SHA3-256 hex digest — re-exported from bittensor-synapse.
#[wasm_bindgen(js_name = sha3_256_hex)]
pub fn sha3_256_hex_wasm(input: &str) -> String {
    bittensor_synapse::sha3_256_hex(input.as_bytes())
}

// ---------------------------------------------------------------------------
// WASM smoke tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod wasm_tests {
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn balance_works_in_wasm() {
        let b = super::types::Balance::from_tao(1.0);
        assert_eq!(b.to_rao(), 1_000_000_000);
    }

    #[wasm_bindgen_test]
    fn network_config_finney_works_in_wasm() {
        let cfg = super::types::NetworkConfig::finney();
        assert_eq!(cfg.name(), "finney");
    }

    #[wasm_bindgen_test]
    fn prefix_constants_work_in_wasm() {
        assert_eq!(super::axon_prefix(), "bt_header_axon_");
        assert_eq!(super::dendrite_prefix(), "bt_header_dendrite_");
    }
}
