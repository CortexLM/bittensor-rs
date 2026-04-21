# Integration Tests — bittensor-chain

Integration tests that run against a **local Subtensor devnet**. These tests connect to a live node and verify real RPC queries, extrinsic submissions, event subscriptions, and block subscriptions.

## Prerequisites

- Docker and Docker Compose installed
- The `bittensor-chain` crate built with the `integration-tests` feature

## Start the Devnet

From the workspace root (`llm-proxy/`):

```bash
# Start the devnet in the background
./scripts/devnet.sh start

# Wait for the node to be ready (health check passes)
./scripts/devnet.sh wait
```

The devnet exposes:
- **WebSocket RPC**: `ws://localhost:31444`
- **HTTP RPC**: `http://localhost:31333`

The node runs in `--dev` mode with:
- Instant block sealing
- Pre-funded dev accounts (Alice, Bob, Charlie, Dave, Eve)
- Ephemeral storage (`--tmp`)

## Run the Integration Tests

```bash
# From the workspace root
cargo test --features integration-tests --test integration -- --ignored
```

> **Note:** Integration tests are marked `#[ignore]` so they don't run during normal `cargo test`. The `--ignored` flag is required to execute them.

### Run a specific test

```bash
cargo test --features integration-tests --test integration query_get_balance_alice_has_funds -- --ignored
```

### Run only query tests

```bash
cargo test --features integration-tests --test integration query_ -- --ignored
```

## Stop the Devnet

```bash
./scripts/devnet.sh stop
```

## Test Categories

### Query Tests (`query_*`)

Test storage reads against the chain:
- `get_balance` — verify dev accounts have funds, unknown accounts return zero
- `get_stake` — verify the stub returns without error
- `get_total_network_stake` — verify chain returns total stake
- `get_neuron_count` — verify neuron count query succeeds
- `get_metagraph` — verify metagraph query returns correct netuid

### Extrinsic Tests (`extrinsic_*`)

Test transaction submission and finalization:
- `transfer` — Alice sends TAO to Bob, verify balance increases
- `add_stake` — submit a stake call (may be rejected by chain on empty devnet)
- `set_weights` — submit a weights call (may be rejected by chain on empty devnet)

> Extrinsic tests that interact with Bittensor-specific pallets (`add_stake`, `set_weights`) may fail with chain-side errors if the devnet doesn't have the expected subnet structure. This is expected — the test validates that the submission path works, not that the chain state supports the operation.

### Event Tests (`event_*`)

Test event subscriptions:
- `subscribe_events` — verify events are received after triggering a transfer
- `verify_transfer_event_emission` — verify a `Balances::Transfer` event is emitted

### Subscription Tests (`subscription_*`)

Test block subscriptions:
- `subscribe_blocks_increasing` — verify block numbers increase over time
- `blocks_have_valid_hash` — verify block hashes are non-zero

### Flow Tests (`flow_*`)

End-to-end flows combining multiple operations:
- `transfer_then_query_balance` — transfer then verify balance updated
- `multiple_transfers_succeed` — submit several transfers sequentially

## Architecture

All tests:
- Connect to `ws://localhost:31444` (the devnet WS endpoint)
- Use `subxt_signer::dev` accounts (Alice, Bob, Charlie) for signing
- Are feature-gated with `#[cfg(feature = "integration-tests")]`
- Are marked `#[ignore]` to prevent accidental execution without devnet
- Use `#[tokio::test]` for async execution

## Troubleshooting

### "failed to connect to devnet"

The devnet is not running or not ready. Start it with `./scripts/devnet.sh start` and wait for readiness.

### Tests timeout

Devnet block production may be slow under load. The default timeout is 60 seconds for extrinsic finalization and 30 seconds for event/block subscription waits. If tests timeout, try reducing parallelism:

```bash
cargo test --features integration-tests --test integration -- --ignored --test-threads=1
```

### "add_stake" or "set_weights" rejected by chain

This is expected on a fresh devnet without registered subnets. The tests handle this gracefully by logging the error and passing.
