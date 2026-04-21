//! Example: Subscribe to chain events and decode them.
//!
//! Run with: cargo run -p bittensor-examples --example chain_events

use bittensor_chain::prelude::{SubtensorClient, subscribe_events};
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await.expect("connect");

    println!("Subscribing to chain events...");

    match subscribe_events(client.rpc()).await {
        Ok(mut rx) => {
            while let Some(event) = rx.recv().await {
                println!(
                    "[{}] {}.{} (block {})",
                    event.pallet_name(),
                    event.pallet_name(),
                    event.event_name(),
                    event.block_number(),
                );
            }
        }
        Err(e) => eprintln!("Subscription failed: {e}"),
    }
}
