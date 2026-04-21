//! bittensor-metagraph — Subnet neural graph state, sync, and serialization.
//!
//! The metagraph is the columnar representation of a subnet's neurons,
//! matching the Python SDK's `bittensor.metagraph` fields exactly.
//!
//! # Quick start
//!
//! ```no_run
//! use bittensor_chain::prelude::SubtensorClient;
//! use bittensor_core::config::NetworkConfig;
//! use bittensor_metagraph::prelude::sync;
//!
//! # async fn example() -> Result<(), bittensor_core::error::BittensorError> {
//! let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
//! let metagraph = sync(&client, 1).await?;
//! println!("Subnet 1 has {} neurons", metagraph.n);
//! # Ok(())
//! # }
//! ```

pub mod iter;
pub mod metagraph;
pub mod serialize;
pub mod sync;

pub use metagraph::Metagraph;
pub use serialize::{load, save};
pub use sync::sync;

pub mod prelude {
    pub use crate::metagraph::Metagraph;
    pub use crate::serialize::{load, save};
    pub use crate::sync::sync;
}
