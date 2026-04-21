//! bittensor-dendrite — HTTP client with signing and streaming for the Bittensor network.

pub mod config;
pub mod dendrite;
pub mod signing;

pub mod prelude {
    pub use crate::config::DendriteConfig;
    pub use crate::dendrite::Dendrite;
    pub use crate::signing::SignedRequest;
}
