# bittensor-cli

Command-line interface for Bittensor network operations.

## Quick Start

```sh
# Check balance
btcli-rs balance --wallet default --network finney

# Transfer TAO
btcli-rs transfer --dest 5Grw... --amount 1.0

# Stake to a hotkey
btcli-rs stake --hotkey 5Grw... --amount 5.0 --netuid 1

# Register on a subnet
btcli-rs register --netuid 1
```

## Feature Flags

| Feature | Description |
|---|---|
| `mev` | Enable MEV-shield protected transactions |

## API Overview

Built on `clap` with subcommands matching the Python `btcli`: `balance`, `transfer`, `stake`, `unstake`, `register`, `set-weights`, `delegate`, `overview`, and more.
