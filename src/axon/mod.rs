//! Axon HTTP server module for Bittensor network communication
//!
//! The Axon is an HTTP server that receives requests from Dendrites in the
//! Bittensor network. It handles:
//!
//! - Request signature verification
//! - Blacklist/whitelist enforcement
//! - Priority-based request handling
//! - Custom synapse handlers
//!
//! # Example
//!
//! ```ignore
//! use bittensor_rs::axon::{Axon, AxonConfig};
//! use bittensor_rs::wallet::Keypair;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a keypair for the axon
//!     let keypair = Keypair::from_uri("//Alice")?;
//!
//!     // Configure the axon
//!     let config = AxonConfig::new()
//!         .with_port(8091)
//!         .with_ip("0.0.0.0");
//!
//!     // Create the axon server
//!     let mut axon = Axon::new(keypair, config);
//!
//!     // Attach a handler for a specific synapse type
//!     axon.attach("MyQuery", |synapse| async move {
//!         // Process the synapse and return the response
//!         let mut response = synapse;
//!         response.set_field("result", serde_json::json!("Hello from Axon!"));
//!         response
//!     });
//!
//!     // Set a custom blacklist function
//!     axon.set_blacklist(|hotkey, synapse_name| {
//!         // Return true to blacklist, false to allow
//!         false
//!     });
//!
//!     // Set a custom priority function
//!     axon.set_priority(|hotkey, synapse_name| {
//!         // Return a priority value (higher = more priority)
//!         1.0
//!     });
//!
//!     // Start serving
//!     axon.serve().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! # Architecture
//!
//! The Axon uses the following middleware stack (in order):
//!
//! 1. **Logging** - Logs all incoming requests
//! 2. **Blacklist** - Rejects blacklisted hotkeys/IPs
//! 3. **Priority** - Assigns priority to requests
//! 4. **Verify** - Verifies request signatures
//! 5. **Timeout** - Enforces request timeouts
//! 6. **Counter** - Tracks request counts
//!
//! Each synapse type has its own route handler registered via `attach()`.

pub mod handlers;
pub mod info;
pub mod middleware;
pub mod server;

pub use handlers::{
    build_error_response, build_response_headers, build_success_response, compute_body_hash,
    extract_synapse, status_codes, status_messages, verify_request, verify_signature,
    HandlerContext, VerifiedRequest, AXON_VERSION,
};
pub use info::{AxonConfig, AxonInfo};
pub use middleware::{
    blacklist_middleware, counter_middleware, logging_middleware, priority_middleware,
    timeout_middleware, verify_middleware, RequestPriority,
};
pub use server::{Axon, AxonState, BlacklistFn, PriorityFn, SynapseHandler, VerifyFn};
