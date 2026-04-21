# bittensor-examples

Runnable examples demonstrating bittensor-rs SDK usage.

## Running Examples

```sh
# Build all examples
cargo build -p bittensor-examples --examples

# Run a specific example
cargo run -p bittensor-examples --example balance_arithmetic
```

## Examples

| Example | Crate | Description |
|---|---|---|
| `wallet_create` | bittensor-wallet | Create and inspect a wallet |
| `balance_arithmetic` | bittensor-core | TAO/rao conversion and arithmetic |
| `chain_query` | bittensor-chain | Query chain state (balance, neurons) |
| `chain_events` | bittensor-chain | Subscribe to real-time chain events |
| `transfer` | bittensor-chain | Submit a transfer extrinsic |
| `stake` | bittensor-chain | Add stake to a hotkey |
| `set_weights` | bittensor-chain | Set validator weights on a subnet |
| `axon_server` | bittensor-axon | Start a neuron server |
| `dendrite_client` | bittensor-dendrite | Query a remote axon |
| `metagraph_sync` | bittensor-metagraph | Sync and inspect subnet state |
