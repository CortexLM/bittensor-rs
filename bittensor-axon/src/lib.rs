//! bittensor-axon — part of the bittensor-rs SDK

pub mod axon;
pub mod config;
pub mod middleware;
pub mod router;

pub mod prelude {
    pub use crate::axon::{Axon, AxonError};
    pub use crate::config::AxonConfig;
    pub use crate::middleware::{
        MiddlewareState, RequestPriority, blacklist_middleware, body_hash_middleware,
        priority_middleware, verification_middleware,
    };
    pub use crate::router::{SynapseRegistry, register_synapse_route};
}
