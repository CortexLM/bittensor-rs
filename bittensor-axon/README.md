# bittensor-axon

Axum-based neuron server with verification, blacklisting, and synapse routing.

## Quick Start

```rust,no_run
use bittensor_axon::prelude::*;
use bittensor_core::config::NetworkConfig;
use bittensor_chain::prelude::SubtensorClient;

#[tokio::main]
async fn main() -> Result<(), AxonError> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
    let wallet = bittensor_wallet::prelude::Wallet::new("default", "/tmp/wallets")?;

    let axon = Axon::new(client, wallet)
        .port(8091)
        .serve().await?;

    println!("Axon listening on {}", axon.external_ip());
    axon.wait_shutdown().await
}
```

## Feature Flags

No optional features — all functionality is always enabled.

## API Overview

| Module | Purpose |
|---|---|
| `axon` | `Axon` server — bind, serve, shutdown |
| `config` | `AxonConfig` — port, IP, external address |
| `middleware` | Request verification, blacklisting, body-hash, priority |
| `router` | `SynapseRegistry` — map synapse types to handler closures |
