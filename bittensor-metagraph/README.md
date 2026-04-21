# bittensor-metagraph

Subnet neural graph state: sync from chain, iterate neurons, serialize to disk.

## Quick Start

```rust,no_run
use bittensor_chain::prelude::SubtensorClient;
use bittensor_core::config::NetworkConfig;
use bittensor_metagraph::prelude::*;

# async fn example() -> Result<(), bittensor_core::error::BittensorError> {
let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
let metagraph = sync(&client, 1).await?;

println!("Neurons: {}", metagraph.n);

for neuron in metagraph.neurons() {
    println!("UID {} hotkey={}", neuron.uid, neuron.hotkey);
}
# Ok(())
# }
```

## Feature Flags

| Feature | Description |
|---|---|
| `ml-backend` | Enable ML-weight matrix operations on metagraph weights |

## API Overview

| Module | Purpose |
|---|---|
| `metagraph` | `Metagraph` struct — columnar neuron state for a subnet |
| `sync` | `sync()` — fetch full subnet state from chain |
| `iter` | Neuron iteration helpers |
| `serialize` | `save()` / `load()` — JSON persistence to disk |
