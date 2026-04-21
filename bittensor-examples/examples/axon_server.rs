//! Example: Start an Axon server and attach a synapse handler.
//!
//! Run with: cargo run -p bittensor-examples --example axon_server

use bittensor_axon::prelude::{Axon, AxonConfig};

#[tokio::main]
async fn main() {
    let config = AxonConfig { port: 8090, ..Default::default() };

    let mut axon = Axon::new(config).attach("TextPrompt", || async { "hello from bittensor-rs" });

    let addr = axon.start().await.expect("failed to start axon");
    println!("Axon serving on {addr}");

    axon.blacklist("5BadActor").await;
    axon.set_priority("5GoodValidator", 100).await;

    tokio::signal::ctrl_c().await.ok();
    axon.stop().expect("shutdown");
}
