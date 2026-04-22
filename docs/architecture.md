# SDK Architecture

This document describes the architecture of the bittensor-rs SDK: how the crates are organized, how data flows through the system, and the design principles behind each major component.

## Crate Structure

The SDK is organized into 11 crates with clear dependency boundaries. Application-level crates depend on infrastructure crates, never the reverse.

```
                    ┌─────────────────┐
                    │  bittensor-cli  │
                    │   (btcli-rs)    │
                    └────────┬────────┘
                             │
            ┌────────────────┼────────────────┐
            │                │                │
     ┌──────┴──────┐  ┌──────┴──────┐  ┌──────┴──────┐
     │ bittensor-  │  │ bittensor-  │  │ bittensor-  │
     │   wallet    │  │   tui       │  │   pyo3      │
     └──────┬──────┘  └──────┬──────┘  └──────┬──────┘
            │                │                │
            │         ┌──────┴──────┐         │
            │         │ bittensor-  │         │
            │         │ metagraph   │         │
            │         └──────┬──────┘         │
            │                │                │
            └────────────────┼────────────────┘
                             │
                      ┌──────┴──────┐
                      │ bittensor-  │
                      │   chain     │
                      └──────┬──────┘
                             │
         ┌───────────────────┼───────────────────┐
         │                   │                   │
  ┌──────┴──────┐     ┌──────┴──────┐     ┌──────┴──────┐
  │ bittensor-  │     │ bittensor-  │     │ bittensor-  │
  │   axon      │     │  dendrite   │     │   core      │
  └──────┬──────┘     └──────┬──────┘     └─────────────┘
         │                   │                   ▲
         │            ┌──────┴──────┐            │
         │            │ bittensor-  │            │
         └────────────┤  synapse   ├────────────┘
                      └─────────────┘

  ┌─────────────────────────────────────────────────────┐
  │                   bittensor-wasm                      │
  │  (standalone: reimplements core types for wasm-bindgen)│
  └─────────────────────────────────────────────────────┘

  ┌─────────────────────────────────────────────────────┐
  │                  bittensor-examples                  │
  │  (depends on all native crates for runnable samples) │
  └─────────────────────────────────────────────────────┘
```

### Dependency Graph

The dependency edges flow downward:

- **bittensor-cli**, **bittensor-tui**, and **bittensor-pyo3** sit at the top. They consume wallet, chain, metagraph, and core.
- **bittensor-chain** is the central infrastructure crate. It depends on **bittensor-core** for types and config, and uses subxt 0.50 for all chain communication.
- **bittensor-axon** and **bittensor-dendrite** depend on **bittensor-synapse** for protocol types and **bittensor-core** for shared primitives. They do not depend on **bittensor-chain** directly.
- **bittensor-synapse** depends on **bittensor-core**.
- **bittensor-core** has no SDK-internal dependencies. It defines the shared vocabulary (Balance, errors, config, types).
- **bittensor-wasm** is standalone. It reimplements a subset of core types with `wasm-bindgen` annotations instead of depending on the native crates, because `wasm-bindgen` requires specific trait implementations that conflict with the native subxt-based code.

## Crate Descriptions

### bittensor-core

Shared foundation used by every other native crate.

| Module | Contents |
|---|---|
| `balance` | `Balance` type wrapping a `u64` rao value, with checked/saturating arithmetic, TAO/rao conversion, `Display` formatting, `FromStr` parsing, serde serialization, and SCALE codec support |
| `config` | `SubtensorConfig` (wraps `subxt::config::substrate::SubstrateConfig`), `NetworkConfig` with presets for finney, test, local, archive, and latent-lite |
| `error` | `BittensorError` enum with 12 variants, `ErrorCategory` classification, `RetryConfig` for exponential backoff |
| `pow` | `PowSolution` for proof-of-work registration |
| `types` | `AxonInfo`, `NeuronInfo`, `NeuronInfoLite`, `StakeInfo`, `DelegateInfo`, `SubnetInfo`, `SubnetHyperparameters`, `ChainIdentity`, `WeightCommitInfo`, `PrometheusInfo`, `SubnetState`, `ProposalVoteData`, `MetagraphInfo`, `NeuronCertificate`, `MovingPriceInfo`, `ScheduleInfo`, `TransferInfo`, `StakeTransferInfo`, `DelegateTakeInfo`, `RegistrationInfo`, `AuditInfo` |
| `weight_utils` | Weight normalization, denormalization, and validation |

### bittensor-wallet

Key management and file I/O, compatible with the Python SDK's directory layout.

| Module | Contents |
|---|---|
| `wallet` | `Wallet` struct with lazy-loaded coldkey/hotkey pairs, file path resolution, coldkey creation from mnemonic, hotkey creation and derivation |
| `keypair` | `Keypair` wrapper around `subxt_signer::sr25519::Keypair`, tracks the seed for serialization, supports URI derivation, hard/soft junction derivation, encryption and decryption |
| `keyfile` | NaCl secretbox encryption/decryption (`$NACL` prefix, Argon2i key derivation, XSalsa20-Poly1305), Python SDK cross-compatible |
| `mnemonic` | BIP-39 mnemonic generation and PBKDF2-based seed derivation |
| `ss58` | SS58 encoding/decoding for Substrate addresses (prefix 42) |

### bittensor-chain

The chain interaction layer. Built on subxt 0.50 with compile-time metadata.

| Module | Contents |
|---|---|
| `client` | `SubtensorClient` with `from_config`, `from_url`, `rpc`, `at_current_block`, `get_block_hash` |
| `queries` | Read-only storage queries organized by domain: `account`, `neurons`, `subnets`, `delegates`, `stakes`, `voting`, `liquidity`, `commitments`, `metagraph_queries`, `runtime` |
| `extrinsics` | Signed transaction submission: `transfer`, `staking`, `weights`, `registration`, `serving`, `take`, `mechanism`, `root`, `children`, `senate` |
| `events` | Event monitoring, filtering, subscription, and decoding |
| `generated` | Auto-generated subxt metadata bindings (from `metadata/finney.scale`) |
| `drand` | (feature `drand`) Drand randomness beacon verification |
| `mev_shield` | (feature `mev-shield`) Post-quantum MEV protection for extrinsics |

### bittensor-synapse

Protocol types for neuron-to-neuron communication.

| Module | Contents |
|---|---|
| headers | Typed header constants and parsing for the Bittensor synapse protocol |
| hashing | Canonical request hashing for signature verification |
| signing | Synapse message signing and verification using sr25519 |
| streaming | Streaming response protocol types |

### bittensor-axon

Neuron server built on axum.

| Module | Contents |
|---|---|
| server | Axum-based HTTP server with synapse protocol middleware |
| middleware | Request verification, rate limiting, authentication |
| routing | Synapse-type routing and dispatch |

### bittensor-dendrite

Neuron client for querying other neurons.

| Module | Contents |
|---|---|
| client | reqwest-based HTTP client with synapse protocol integration |
| signing | Outbound request signing |
| streaming | Streaming response handling |

### bittensor-metagraph

Subnet graph operations.

| Module | Contents |
|---|---|
| sync | Fetch and cache subnet state from chain |
| iterate | Columnar iteration over neuron attributes |
| serialize | Graph serialization and export |

### bittensor-cli (btcli-rs)

Command-line interface matching the Python `btcli` tool.

Common commands:

```sh
# Wallet management
btcli-rs wallet create --name my-wallet
btcli-rs wallet list

# Balance queries
btcli-rs balance --name my-wallet

# Staking
btcli-rs stake add --amount 1.0 --name my-wallet

# Transfer
btcli-rs transfer --dest 5DfhGyQ... --amount 5.0
```

### bittensor-tui

Terminal dashboard for monitoring the network. Displays subnet health, neuron scores, and stake distributions in real time.

### bittensor-pyo3

Python bindings published as the `bittensor_rs` package. Uses PyO3 to expose the Rust API to Python, allowing existing Python codebases to benefit from Rust performance without rewriting.

### bittensor-wasm

Browser bindings via wasm-bindgen. Reimplements a subset of core types (Balance, NetworkConfig, SS58) with JavaScript-compatible interfaces. Does not depend on subxt, since WebSocket usage in browsers differs from the native tokio-based runtime.

### bittensor-examples

Runnable code samples. Each example demonstrates a specific capability: connecting to the network, querying a balance, submitting a transfer, running an axon, querying a dendrite, or syncing a metagraph.

## Query Flow

Reading data from the chain follows a typed pipeline from client to subxt to the WebSocket connection and back.

```
SubtensorClient
      │
      ▼
 client.rpc()  ──►  OnlineClient<SubtensorConfig>
      │
      ▼
 .at_current_block()  ──►  ClientAtBlock
      │
      ▼
 queries::account::get_balance(rpc, &account_id)
      │
      ▼
 subxt storage query  ──►  SCALE-encoded bytes
      │
      ▼
 subxt auto-decode via generated metadata  ──►  Rust type
      │
      ▼
 SDK-level conversion  ──►  Balance / NeuronInfo / etc.
```

Step by step:

1. **Client initialization.** `SubtensorClient::from_config` or `from_url` creates an `OnlineClient<SubtensorConfig>` by connecting to the WebSocket endpoint. The compiled metadata (`metadata/finney.scale`) is embedded in the binary, so the client knows the runtime API at compile time.

2. **Block pinning.** Calling `at_current_block()` returns a `ClientAtBlock` that pins all subsequent storage reads to the same block hash. This guarantees consistent reads across multiple queries.

3. **Storage query.** Functions in `bittensor_chain::queries` build typed storage access paths using the generated metadata bindings. For example, `get_balance` queries the `System.Account` storage map.

4. **SCALE decoding.** subxt decodes the SCALE-encoded response bytes into the generated Rust type automatically, using the type information from the metadata.

5. **SDK conversion.** The query functions convert from the generated types into the SDK's public types (`Balance`, `NeuronInfo`, etc.), isolating consumers from subxt internals.

### Bulk Queries

When fetching multiple items, the SDK uses `FuturesUnordered` for concurrent requests rather than sequential awaits:

```
queries::neurons::get_all_neurons(rpc, netuid)
      │
      ├──► fetch neuron UIDs (single query)
      │
      ├──► spawn FuturesUnordered for per-UID queries
      │      ├──► neuron 0: AxonInfo + PrometheusInfo
      │      ├──► neuron 1: AxonInfo + PrometheusInfo
      │      ├──► neuron 2: AxonInfo + PrometheusInfo
      │      └──► ...
      │
      └──► collect and merge into Vec<NeuronInfo>
```

This pattern appears in:

- Per-neuron axon and Prometheus data
- Stake distribution lookups across hotkey/coldkey pairs
- Delegate nominator enumeration

## Transaction Flow

Submitting an extrinsic (a signed chain transaction) follows a multi-stage pipeline from call construction through finalization.

```
Build call
    │
    ▼
 Sign with Keypair
    │
    ▼
 Submit (broadcast)
    │
    ▼
 Watch for events
    │
    ├──► InBlock (included in a block)
    │
    └──► Finalized (irreversible)
```

Step by step:

1. **Build the call.** The extrinsics module constructs a typed subxt call using the generated metadata. For example, `transfer::transfer` builds a `Balances.transfer` call with the destination and amount.

2. **Sign.** The call is signed using a `subxt_signer::sr25519::Keypair`. The signer can come from the wallet crate (via `Keypair::into_signer()`) or from a development URI like `//Alice`.

3. **Submit and watch.** `submit_and_watch` sends the signed extrinsic to the node and returns an event stream. This is preferred over fire-and-forget `submit` because it lets you track inclusion and finality.

4. **InBlock.** The transaction has been included in a block. Events at this stage reflect the immediate outcome (transfer succeeded, staking completed, etc.).

5. **Finalized.** The block containing the transaction has been finalized by the consensus mechanism. At this point the result is irreversible.

### Example: Transfer Flow

```rust,no_run
use bittensor_chain::prelude::*;
use bittensor_core::config::NetworkConfig;
use bittensor_core::balance::Balance;

let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
let signer = subxt_signer::sr25519::Keypair::from_uri("//Alice")?;
let dest = subxt_signer::sr25519::PublicKey::from_uri("//Bob")?;

// 1. Build the call internally (done by the extrinsics function)
// 2. Sign with the keypair
// 3. Submit and watch
// 4. Wait for finalization
let amount = Balance::from_tao(1.0).to_rao();
bittensor_chain::extrinsics::transfer::transfer(
    client.rpc(), &signer, &dest, amount
).await?;
// At this point, the extrinsic has been finalized on chain
```

## Columnar Metagraph Design

The metagraph crate models a subnet as a columnar dataset rather than a collection of row-oriented neuron structs. Each attribute (stake, rank, trust, etc.) is stored in its own vector, enabling vectorized iteration and efficient serialization.

```
Metagraph {
    netuid: u16,
    block: u64,
    n: u16,

    // Column vectors (one entry per neuron)
    uids:           Vec<u16>,
    hotkeys:        Vec<String>,
    coldkeys:       Vec<String>,
    active:         Vec<bool>,
    stake:          Vec<Balance>,
    rank:           Vec<u16>,
    trust:          Vec<u16>,
    consensus:      Vec<u16>,
    incentive:      Vec<u16>,
    dividend:       Vec<u16>,
    emission:       Vec<u64>,
    last_update:    Vec<u64>,
    validator_trust: Vec<u16>,
    weights:        Vec<Vec<u16>>,
    bonds:          Vec<Vec<u16>>,
}
```

This layout makes it straightforward to:

- Iterate over a single attribute for all neurons without touching unrelated data
- Compute aggregate statistics (total stake, average rank) with simple vector operations
- Export to NumPy/polars-compatible formats via the column-oriented serialization
- Sync incrementally by replacing individual columns when the chain state changes

The `ml-backend` feature flag enables ML-based scoring backends that operate on these column vectors directly, feeding computed scores back into weight-setting transactions.

## Synapse Protocol

Neurons communicate using the synapse protocol, which adds typed headers and cryptographic signatures to standard HTTP requests.

### Header Structure

Every synapse request and response carries these headers:

| Header | Purpose |
|---|---|
| `bt-header-signature` | sr25519 signature of the canonical request hash |
| `bt-header-hash` | SHA-256 hash of the request body |
| `bt-header-nonce` | Monotonic nonce to prevent replay attacks |
| `bt-header-timestamp` | Unix timestamp for liveness checks |
| `bt-header-version` | Protocol version for compatibility |
| `bt-header-hotkey` | SS58 address of the signing hotkey |
| `bt-header-coldkey` | SS58 address of the owning coldkey |

### Request Signing Flow

```
1. Serialize request body
2. Compute SHA-256 hash of body
3. Construct canonical string: method + path + headers-sorted + body-hash
4. Sign canonical string with sr25519 private key
5. Attach signature, hash, and identity headers
6. Send HTTP request
```

### Response Verification Flow

```
1. Receive HTTP response
2. Read bt-header-hash, compare against SHA-256 of response body
3. Read bt-header-signature, verify against signer's public key
4. Read bt-header-nonce, ensure it is greater than the last seen nonce
5. Read bt-header-timestamp, reject stale responses
```

### Streaming

For large responses (e.g., model inference), the synapse protocol supports chunked transfer with per-chunk hashing. Each chunk is signed independently, allowing the receiver to verify partial results before the full response completes.

## SubtensorConfig and Metadata

The `SubtensorConfig` type wraps `subxt::config::substrate::SubstrateConfig` to provide a distinct Bittensor-specific config while inheriting standard Substrate primitives:

- Blake2-256 hashing
- 32-byte account IDs (`AccountId32`)
- sr25519 signatures (via `MultiSignature`)
- Standard Substrate extrinsic parameters

Metadata is compiled into the binary from `metadata/finney.scale`. This means:

- No runtime metadata fetch is needed on startup (faster cold connect).
- The SDK is pinned to a specific runtime version. When Finney upgrades, you must regenerate the metadata.
- Type mismatches between the compiled metadata and the live chain surface as `Codec` errors from subxt.

### Refreshing Metadata

```sh
cargo install subxt-cli@0.50.0 --locked
subxt metadata --url wss://entrypoint-finney.opentensor.ai:443 -f bytes > metadata/finney.scale
cargo check -p bittensor-chain
```

If the runtime has changed significantly, the generated code in `bittensor-chain/src/generated.rs` will fail to compile. Fix any breaking changes, then rebuild.

## Keyfile Encryption

Coldkey files use NaCl secretbox encryption to match the Python SDK exactly.

```
Password ──► Argon2i (OPSLIMIT_SENSITIVE, MEMLIMIT_SENSITIVE) ──► 32-byte key
                                                              │
JSON payload ──► XSalsa20-Poly1305 (key, random nonce) ──► ciphertext
                                                              │
Output: "$NACL" + nonce (24 bytes) + ciphertext
```

The salt is hardcoded to match the Python SDK's `btwallet` implementation. This cross-compatibility means:

- A coldkey created with `btcli` can be decrypted by bittensor-rs.
- A coldkey created with `btcli-rs` can be decrypted by the Python `btwallet`.
- Wrong passwords produce a `DecryptionFailed` error rather than garbled output, because Poly1305 authentication detects corruption before returning plaintext.

## Balance Type Design

The `Balance` type is a fixed-point wrapper around `u64` representing rao:

```
Balance { rao: u64 }
```

Key design decisions:

- **Internal unit is rao** (10^-9 TAO). All chain operations use integers, avoiding floating-point rounding issues.
- **TAO is for display only.** `from_tao(f64)` converts to rao with rounding; `to_tao()` converts back to f64 for display. The 9-decimal `Display` implementation always shows the full precision.
- **Arithmetic follows Rust conventions.** `Add`/`Sub`/`Mul<u64>`/`Div` panic on overflow/underflow/division-by-zero, matching standard integer behavior. `checked_*` and `saturating_*` variants provide safe alternatives.
- **Serde serialization uses the string form** (`"1.500000000"`), not a raw integer, so JSON output matches the Python SDK's `Balance.__str__` format.
- **SCALE codec uses the raw `u64`**, matching the on-chain representation.

## SS58 Address Encoding

Bittensor uses SS58 prefix 42 (the Substrate default). The encoding process:

1. Prepend the format byte (42) to the 32-byte public key.
2. Compute BLAKE2-256 of the 33-byte concatenation.
3. Append the first 2 bytes of the hash as a checksum.
4. Base58-encode the 35-byte result.

The wallet crate's `ss58` module provides both `encode_ss58` and `decode_ss58` for round-trip conversions.

## Error Handling Strategy

The SDK uses `BittensorError` as the unified error type across all crates:

```
BittensorError
├── Rpc(String)           ── WebSocket/HTTP RPC failure
├── Signing(String)       ── Signature creation/verification failure
├── Codec(String)         ── SCALE or JSON serialization failure
├── Transaction(String)   ── Extrinsic submission/finalization failure
├── Wallet(String)        ── Wallet file I/O or decryption failure
├── Network(String)       ── Connectivity or DNS failure
├── Config(String)        ── Invalid configuration
├── Balance(String)       ── Overflow, underflow, or invalid conversion
├── Timeout(String)       ── Operation exceeded deadline
├── RateLimit(String)     ── Server rate limit hit
├── Authentication(String)─ Auth/authorization failure
└── Validation(String)    ── Input validation failure
```

Each variant maps to an `ErrorCategory` that determines retry behavior:

| Category | Variants | max_retries | base_delay_ms | backoff |
|---|---|---|---|---|
| Transient | Rpc, Network, Timeout | 3 | 1000 | 2x |
| RateLimit | RateLimit | 5 | 5000 | 2x |
| Auth | Authentication | 0 | 0 | none |
| Config | Config | 0 | 0 | none |
| Permanent | Signing, Codec, Transaction, Wallet, Balance, Validation | 0 | 0 | none |

## Design Principles

### Performance

- Bulk storage queries fetch multiple items in a single round trip.
- `FuturesUnordered` enables concurrent requests where safe.
- SCALE encoding avoids reflection-based serialization overhead.
- Compile-time metadata eliminates runtime schema lookups.

### Type Safety

- All chain data is strongly typed through subxt's generated bindings.
- `AccountId32` is used consistently for account identification.
- The `Balance` type prevents accidental mixing of rao and TAO values.
- `BittensorError` classifies every failure mode.

### Python SDK Parity

- Wallet directory layout matches Python's `~/.bittensor/wallets/` structure.
- Keyfile encryption uses the same NaCl secretbox parameters.
- Balance display formatting matches Python's 9-decimal precision.
- Commit-rereveal weight-setting semantics follow the same versioning logic (CRv4 when `CommitRevealVersion >= 4`).
- RAO/TAO conventions match: on-chain calls use rao; TAO is formatting only.

### Compatibility

- SS58 prefix 42 matches Substrate's default, same as the Python SDK.
- SCALE encoding ensures all extrinsics match the Subtensor runtime's expected format.
- Weight vectors use `Vec<u16>` scaled by `u16::MAX`, matching the on-chain representation.
- IP encoding uses packed `u64` for IPv4 and `u128` for IPv6, matching the runtime's storage format.
- Commit-reveal indices use `NetUidStorageIndex` (`u16`), computed as `mechanism_id * 4096 + netuid`.

## Configuration Reference

### NetworkConfig Fields

| Field | Type | Description |
|---|---|---|
| `name` | `String` | Human-readable network name |
| `ws_endpoint` | `String` | WebSocket endpoint URL |
| `archive_endpoint` | `Option<String>` | Archive node endpoint, used for failover |
| `chain_id` | `u16` | SS58 prefix / chain identifier |

### NetworkConfig Constructors

| Constructor | Endpoint | Archive |
|---|---|---|
| `NetworkConfig::finney()` | `wss://entrypoint-finney.opentensor.ai:443` | None |
| `NetworkConfig::test()` | `wss://test.finney.opentensor.ai:443` | None |
| `NetworkConfig::local()` | `ws://127.0.0.1:9944` | None |
| `NetworkConfig::archive()` | `wss://archive.finney.opentensor.ai:443` | Same as endpoint |
| `NetworkConfig::latent_lite()` | `wss://lite.finney.opentensor.ai:443` | None |

### SubtensorClient Constructors

| Method | Description |
|---|---|
| `from_config(config)` | Connect via NetworkConfig, with archive failover |
| `from_url(url)` | Connect to a single URL, no failover |

### SubtensorClient Methods

| Method | Return Type | Description |
|---|---|---|
| `rpc()` | `&OnlineClient<SubtensorConfig>` | Access the underlying subxt client |
| `at_current_block()` | `ClientAtBlock` | Pin queries to the current best block |
| `get_block_hash(n)` | `Option<H256>` | Look up the hash for block number `n` |

## Extension Points

The architecture supports extension in several ways:

- **Custom queries.** Add new functions in `bittensor_chain::queries` following the existing pattern: build a storage accessor from the generated metadata, query it, and convert the result to an SDK type.
- **Custom types.** Add new structs in `bittensor_core::types`. Implement `Encode`, `Decode`, `Serialize`, and `Deserialize` as needed.
- **Custom extrinsics.** Add new functions in `bittensor_chain::extrinsics`. Build the call from generated metadata, sign it, and use `submit_and_watch`.
- **Custom middleware.** The axon crate supports axum middleware layers. Add authentication, rate limiting, or logging as needed.
- **WASM bindings.** Extend `bittensor-wasm` with new types and functions, ensuring all exposed types implement `wasm-bindgen` traits.

## Best Practices

1. **Reuse client instances.** `SubtensorClient` wraps a single WebSocket connection. Create it once and share it across your application via `Arc` or by cloning (the inner `OnlineClient` is cheaply cloneable).
2. **Pin blocks for multi-query consistency.** Always use `at_current_block()` when you need a consistent snapshot across multiple storage reads.
3. **Handle errors by category.** Use `error.is_retryable()` and `error.category().retry_config()` to implement structured retries instead of ad-hoc retry loops.
4. **Use checked arithmetic for user-supplied amounts.** When computing transfer or stake amounts from user input, prefer `Balance::checked_add` and `checked_sub` to avoid panics.
5. **Refresh metadata after runtime upgrades.** If you encounter `Codec` errors that previously worked, the runtime has likely changed. Regenerate `metadata/finney.scale` and rebuild.
