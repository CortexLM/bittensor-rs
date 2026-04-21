# bittensor-dendrite

HTTP client for querying axons with request signing and streaming support.

## Quick Start

```rust,no_run
use bittensor_dendrite::prelude::*;
use bittensor_chain::prelude::SubtensorClient;
use bittensor_wallet::prelude::Wallet;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
    let wallet = Wallet::new("default", "/tmp/wallets")?;

    let dendrite = Dendrite::new(client);

    // Query a remote axon (requires a Synapse impl)
    let response = dendrite.query(&wallet, "5Grw...", &my_synapse).await?;
    Ok(())
}
```

## Feature Flags

No optional features — all functionality is always enabled.

## API Overview

| Module | Purpose |
|---|---|
| `dendrite` | `Dendrite` — query axons, stream responses |
| `config` | `DendriteConfig` — timeouts, retry behavior |
| `signing` | `SignedRequest` — attach hotkey signatures to outbound requests |
