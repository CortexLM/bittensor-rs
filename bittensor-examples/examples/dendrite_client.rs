//! Example: Query a remote Axon using the Dendrite HTTP client.
//!
//! Run with: cargo run -p bittensor-examples --example dendrite_client
//!
//! Requires a running Axon at 127.0.0.1:8090 with a TextPrompt handler.

use bittensor_core::types::AxonInfo;
use bittensor_dendrite::prelude::{Dendrite, DendriteConfig};

/// A minimal Synapse implementation for the TextPrompt route.
#[derive(serde::Serialize, serde::Deserialize)]
struct TextPrompt {
    prompt: String,
    timeout_val: f64,
    dendrite_info: bittensor_synapse::TerminalInfo,
    axon_info: bittensor_synapse::TerminalInfo,
    computed_hash: String,
    total: u64,
    header: u64,
}

impl bittensor_synapse::Synapse for TextPrompt {
    type Output = String;

    fn name(&self) -> &str {
        "TextPrompt"
    }
    fn timeout(&self) -> f64 {
        self.timeout_val
    }
    fn set_timeout(&mut self, t: f64) {
        self.timeout_val = t;
    }
    fn dendrite(&self) -> &bittensor_synapse::TerminalInfo {
        &self.dendrite_info
    }
    fn set_dendrite(&mut self, info: bittensor_synapse::TerminalInfo) {
        self.dendrite_info = info;
    }
    fn axon(&self) -> &bittensor_synapse::TerminalInfo {
        &self.axon_info
    }
    fn set_axon(&mut self, info: bittensor_synapse::TerminalInfo) {
        self.axon_info = info;
    }
    fn computed_body_hash(&self) -> &str {
        &self.computed_hash
    }
    fn set_computed_body_hash(&mut self, h: String) {
        self.computed_hash = h;
    }
    fn total_size(&self) -> u64 {
        self.total
    }
    fn set_total_size(&mut self, s: u64) {
        self.total = s;
    }
    fn header_size(&self) -> u64 {
        self.header
    }
    fn set_header_size(&mut self, s: u64) {
        self.header = s;
    }
}

#[tokio::main]
async fn main() {
    let dendrite = Dendrite::new(DendriteConfig::default()).expect("failed to build dendrite");

    // Point at a known axon (example: localhost miner)
    let axon_info = AxonInfo {
        ip: 2130706433, // 127.0.0.1 packed as u64
        port: 8090,
        ip_type: 4,
        protocol: 0,
        version: 1,
        hotkey: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".into(),
        coldkey: "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty".into(),
    };

    let synapse = TextPrompt {
        prompt: "What is Bittensor?".into(),
        timeout_val: 12.0,
        dendrite_info: bittensor_synapse::TerminalInfo::default(),
        axon_info: bittensor_synapse::TerminalInfo::default(),
        computed_hash: String::new(),
        total: 0,
        header: 0,
    };

    match dendrite.query(synapse, &axon_info).await {
        Ok(resp) => println!("Response: {:?}", resp.axon_info.status_message),
        Err(e) => eprintln!("Query failed: {e}"),
    }
}
