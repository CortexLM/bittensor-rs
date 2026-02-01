//! Dendrite HTTP client module for Bittensor network communication
//!
//! The Dendrite is responsible for making HTTP requests to Axon servers
//! in the Bittensor network. It handles request signing, response parsing,
//! and supports both standard and streaming communication patterns.
//!
//! # Example
//!
//! ```ignore
//! use bittensor_rs::dendrite::Dendrite;
//! use bittensor_rs::types::{AxonInfo, Synapse};
//!
//! let dendrite = Dendrite::new(None);
//! let axon = // ... get axon info from metagraph
//! let synapse = Synapse::new().with_name("MyQuery");
//!
//! let response = dendrite.call(&axon, synapse).await?;
//! ```

pub mod client;
pub mod request;
pub mod response;
pub mod streaming;

pub use client::Dendrite;
pub use request::{headers_to_synapse, synapse_to_headers, DendriteRequest};
pub use response::DendriteResponse;
pub use streaming::{StreamingResponse, StreamingSynapse};
