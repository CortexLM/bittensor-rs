# Subtensor Devnet

Local single-node Subtensor chain for development and testing.

## Quick Start

```bash
# Start the node
./start.sh

# Stop the node
./stop.sh

# Check dev account info
./fund_test_accounts.sh
```

## Endpoints

| Service       | URL                       |
|---------------|---------------------------|
| WebSocket RPC | ws://localhost:31444      |
| HTTP RPC      | http://localhost:31333    |
| P2P           | localhost:31033           |

## Dev Accounts

On `--dev` chains, these accounts are pre-funded with ample balances:

| Name    | SS58 Address                                        |
|---------|-----------------------------------------------------|
| Alice   | `5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY` |
| Bob     | `5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92Byj8Fd6J4Q` |
| Charlie | `5DAAnrj7VHTznn2AWBemMq4jHMBKqGK8ANB2T20nFz8eKkKv` |
| Dave    | `5GNJqTPyYxP9G6dX1J6oE1R5yVCN4RqWnNJHdCSPQUKAFdGi` |
| Eve     | `5HGjWAeFDfFCWPsjFQmSdodT5dhr6N3gW3iEAM4MAFCb3p3A` |

Alice has sudo access on the dev chain.

## Connecting with subxt

```rust
use subxt::OnlineClient;

let client = OnlineClient::<subxt::PolkadotConfig>::from_url("ws://localhost:31444").await?;
let block = client.blocks().at_latest().await?;
println!("Block #{}", block.number());
```

## Connecting with polkadot-js

Open [polkadot.js.org/apps](https://polkadot.js.org/apps/?rpc=ws://localhost:31444) and set the custom endpoint to `ws://localhost:31444`.

## Details

- **Image**: `ghcr.io/opentensor/subtensor:latest`
- **Chain mode**: `--dev` (single validator, instant seal)
- **Storage**: `--tmp` (ephemeral, no persistent data)
- **Ports**: All externally-exposed ports are in the 3100-3199 range per project constraints
