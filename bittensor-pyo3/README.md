# bittensor-pyo3

Python bindings for the bittensor-rs SDK via PyO3.

Published as `bittensor_rs` on PyPI.

## Quick Start

```python
import bittensor_rs

# Connect to chain
client = bittensor_rs.SubtensorClient(network="finney")

# Wallet management
wallet = bittensor_rs.Wallet(name="default", path="~/.bittensor/wallets")

# Query balance
balance = client.get_balance(wallet.coldkeypub_address)
print(f"Balance: {balance}")
```

## Feature Flags

No optional features — all functionality is always enabled.

## API Overview

| Module | Python Class | Purpose |
|---|---|---|
| `chain_client` | `SubtensorClient` | Chain connection and extrinsics |
| `core_types` | `Balance`, `AxonInfo`, etc. | Core type wrappers |
| `wallet` | `Wallet` | Coldkey/hotkey management |
| `synapse` | `Synapse` | Protocol type base class |
| `axon` | `Axon` | Neuron server |
| `dendrite` | `Dendrite` | Axon query client |
