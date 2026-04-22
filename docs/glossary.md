# Glossary

Reference for Bittensor network concepts and bittensor-rs SDK terminology. Terms are grouped by domain.

## Network and Token

### TAO

The native token of the Bittensor network. TAO is the unit users see in wallets, explorers, and CLI output. One TAO equals one billion rao. The `Balance` type handles conversion internally: `Balance::from_tao(1.0)` stores `1_000_000_000` rao.

```rust
use bittensor_core::balance::Balance;

let one_tao = Balance::from_tao(1.0);
assert_eq!(one_tao.to_rao(), 1_000_000_000);
```

### RAO

The smallest on-chain unit. 1 TAO = 10^9 rao. All chain storage and extrinsics operate in rao to avoid floating-point rounding. TAO is for display only.

```rust
let five_rao = Balance::from_rao(5);
let display = Balance::from_tao(0.000000005);
assert_eq!(five_rao, display);
```

### Subtensor

The Bittensor blockchain, built on the Substrate framework. It tracks subnet state, neuron registrations, stakes, weights, and emissions. The SDK communicates with Subtensor via WebSocket RPC using the subxt library.

### Finney

The Bittensor mainnet. Connect with `NetworkConfig::finney()`, which points to `wss://entrypoint-finney.opentensor.ai:443`. Named after Hal Finney, the first person to receive a Bitcoin transaction.

### Subnet (netuid)

A competitive market within Bittensor focused on a specific AI task. Each subnet has a unique numeric identifier called a netuid. For example, netuid 1 is the root subnet that governs cross-subnet weight allocation. Subnets define their own incentive mechanisms, registration rules, and hyperparameters.

### Neuron

A participant in a subnet. Every neuron holds a UID within that subnet and can act as a miner (providing computation or data), a validator (scoring miners), or both. Neurons are identified by their hotkey on the chain.

### UID

A unique 16-bit identifier assigned to a neuron within a subnet. UIDs range from 0 up to the subnet's maximum UID count minus one. A single hotkey can hold at most one UID per subnet.

## Keys and Wallets

### Hotkey

An sr25519 keypair used for signing messages and submitting transactions on a daily basis. The hotkey identifies a neuron on the network and signs synapse protocol requests. It is stored as a raw hex seed on disk (unencrypted, matching the Python SDK).

### Coldkey

An sr25519 keypair that holds stake and funds. The coldkey signs high-value transactions like transfers and staking operations. It is stored on disk in an encrypted NaCl secretbox file.

### Keyfile

An encrypted file that stores a coldkey keypair. Uses the NaCl secretbox format: `$NACL` prefix, 24-byte nonce, and XSalsa20-Poly1305 ciphertext. The encryption key is derived from the password using Argon2i. This format is cross-compatible with the Python SDK.

```rust
use bittensor_wallet::keyfile;

let encrypted = std::fs::read("~/.bittensor/wallets/default/coldkey")?;
assert!(keyfile::is_encrypted_nacl(&encrypted));

let decrypted = keyfile::decrypt(&encrypted, b"my-password")?;
```

### SS58

The address encoding format used by Substrate-based chains. Bittensor uses SS58 prefix 42 (the Substrate default). The `Wallet` and `Keypair` types return SS58-encoded strings from their `ss58_address()` methods.

### Keypair

An sr25519 signing keypair. In bittensor-rs, `Keypair` wraps `subxt_signer::sr25519::Keypair` and tracks the seed for serialization. It can be created from a mnemonic, a URI like `//Alice`, a hex seed, or loaded from a keyfile.

```rust
use bittensor_wallet::prelude::*;

let kp = Keypair::from_seed_hex(
    "0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7aee9f25cc4d693213c4e829"
)?;
println!("Address: {}", kp.ss58_address());
```

### Mnemonic

A BIP-39 12-word or 24-word seed phrase. The wallet crate uses `subxt_signer::bip39::Mnemonic` for generation and parsing. Seed derivation follows PBKDF2 to match the Python SDK.

```rust
use bittensor_wallet::prelude::*;

let mnemonic = wallet.create_coldkey("my-password")?;
println!("Back up these words: {mnemonic}");
```

### Wallet

The top-level key management type in `bittensor-wallet`. A `Wallet` points to a directory on disk and lazily loads coldkey and hotkey pairs. It follows the same file layout as the Python SDK, so wallets created with `btcli` are readable by `btcli-rs` and vice versa.

```rust
use bittensor_wallet::prelude::*;

let mut wallet = Wallet::new("default");
let address = wallet.get_coldkeypub()?;
let hotkey = wallet.get_hotkey_pair()?;
```

## Chain Interaction

### NetworkConfig

A configuration struct that defines which Subtensor endpoint to connect to. It carries a primary WebSocket URL, an optional archive endpoint for failover, a human-readable name, and the SS58 chain ID. Preset constructors exist for finney, test, local, archive, and latent-lite.

```rust
use bittensor_core::config::NetworkConfig;

let config = NetworkConfig::finney();
// config.ws_endpoint == "wss://entrypoint-finney.opentensor.ai:443"
// config.archive_endpoint == None
```

See [Chain Client](chain-client.md) for the full `NetworkConfig` reference.

### SubtensorClient

The primary chain client in `bittensor-chain`. It wraps a subxt `OnlineClient` and exposes methods for connecting, pinning to blocks, and querying chain state. Create one with `from_config` (with failover) or `from_url` (single endpoint). It is cheaply cloneable via `Arc`.

```rust
use bittensor_chain::prelude::SubtensorClient;
use bittensor_core::config::NetworkConfig;

let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
let block = client.at_current_block().await?;
```

### subxt

The Substrate RPC client library (version 0.50) that bittensor-chain builds on. subxt handles WebSocket transport, SCALE encoding/decoding, storage queries, extrinsic submission, and event watching. The SDK embeds compiled metadata from `metadata/finney.scale` so no runtime schema fetch is needed.

### BlockAt

A snapshot of chain state pinned to a specific block hash. Obtained via `client.at_current_block()`, it guarantees that multiple storage reads all reflect the same block. The type alias is `ClientAtBlock`, which wraps subxt's block-scoped client.

### Balance

A type-safe wrapper around a `u64` rao value in `bittensor-core`. It provides checked, saturating, and panicking arithmetic; TAO/rao conversion; `Display` formatting with 9 decimal places; `FromStr` parsing; serde serialization as a string like `"1.500000000"`; and SCALE codec encoding as a raw `u64`.

```rust
use bittensor_core::balance::Balance;

let a = Balance::from_tao(1.5);
let b = Balance::from_rao(500_000_000);
let sum = a + b; // 2.0 TAO
println!("{sum}"); // "2.000000000"
```

See [Wallet](wallet.md) for `Balance` usage in Python.

### BittensorError

The unified error enum across all bittensor-rs crates. Each variant maps to an `ErrorCategory` that determines retry behavior:

| Category | Variants | Retryable |
|---|---|---|
| Transient | `Rpc`, `Network`, `Timeout` | Yes |
| RateLimit | `RateLimit` | Yes (longer backoff) |
| Auth | `Authentication` | No |
| Config | `Config` | No |
| Permanent | `Signing`, `Codec`, `Transaction`, `Wallet`, `Balance`, `Validation` | No |

```rust
use bittensor_core::error::BittensorError;

match result {
    Err(e) if e.is_retryable() => {
        let cfg = e.category().retry_config();
        println!("Retry in {} ms", cfg.base_delay_ms);
    }
    Err(e) => println!("Permanent error: {e}"),
    Ok(v) => { /* ... */ }
}
```

See [Architecture](architecture.md) for the full error handling strategy.

## Metagraph

### Metagraph

The state graph of a subnet, stored in columnar form. Each attribute (stake, rank, trust, weights, etc.) is a parallel array indexed by positional position. This layout matches the Python SDK's `bittensor.metagraph` and supports vectorized operations.

```rust
use bittensor_metagraph::prelude::*;

let metagraph = sync(&client, 1).await?;
println!("{} neurons at block {}", metagraph.n, metagraph.block);

for neuron in metagraph.neurons() {
    println!("UID {} stake={:.2}", neuron.uid, neuron.stake.to_tao());
}
```

See [Metagraph](metagraph-lib.md) for the full API.

### Axon

A neuron's server endpoint for receiving synapse requests. On the chain, axon info is stored as `AxonInfo` (IP, port, protocol, hotkey, coldkey). In `bittensor-axon`, the `Axon` type runs an axum HTTP server with a middleware stack for verification, blacklisting, priority routing, and body hash checking.

```rust
use bittensor_axon::prelude::*;

let mut axon = Axon::new(AxonConfig::default())
    .attach("TextPrompt", my_handler);
let addr = axon.start().await?;
```

See [Axon](axon.md) for the full server API.

### Dendrite

The client that queries axon endpoints. `bittensor-dendrite` provides a `Dendrite` struct that constructs signed HTTP POST requests from a synapse, sends them to the target axon, and populates the response metadata. It supports single-shot queries and SSE streaming.

```rust
use bittensor_dendrite::prelude::*;

let dendrite = Dendrite::new(DendriteConfig::new().with_hotkey(keypair))?;
let response = dendrite.query(synapse, &axon_info).await?;
```

See [Dendrite](dendrite.md) for the full client API.

### Synapse

The protocol type for neuron-to-neuron communication. Any struct that implements the `Synapse` trait can be serialized into HTTP headers and a JSON body, signed by the sender, and verified by the receiver. The trait defines how to read and write the protocol fields: name, timeout, body hash, and two `TerminalInfo` structs (dendrite and axon).

```rust
use bittensor_synapse::prelude::*;

impl Synapse for MySynapse {
    type Output = MyOutput;
    fn name(&self) -> &str { "MySynapse" }
    // ... remaining trait methods
}
```

See [Synapse](synapse.md) for the full trait reference and example implementations.

### StreamingSynapse

An extension of the `Synapse` trait for Server-Sent Events responses. Instead of waiting for the full body, a streaming synapse processes incremental chunks. Each chunk is parsed by the `process_chunk` associated function.

```rust
impl StreamingSynapse for MyStreamSynapse {
    type StreamItem = String;
    fn process_chunk(chunk: &[u8]) -> Result<String, SynapseError> {
        String::from_utf8(chunk.to_vec())
            .map_err(|e| SynapseError::DeserializationFailed(e.to_string()))
    }
}
```

### TerminalInfo

Metadata attached to synapse headers, carrying identity and connection details for one endpoint of the communication. A synapse has two `TerminalInfo` fields: `dendrite` (the requester) and `axon` (the responder). Each field is optional: `hotkey`, `nonce`, `uuid`, `signature`, `status_code`, `status_message`, `process_time`, `ip`, `port`, and `version`.

```rust
let info = TerminalInfo {
    hotkey: Some("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".into()),
    nonce: Some(1700000000000),
    ..Default::default()
};
let headers = info.to_headers_with_prefix("bt_header_axon_");
```

## Incentive Mechanics

### Stake

TAO locked to a hotkey to participate in a subnet. Stake determines a neuron's influence on consensus and its share of emissions. Validators need stake to weight miners; miners need stake to register and stay active.

### Weights

Inter-neuron scoring submitted by validators. Each validator sets a weight vector over the UIDs in the subnet, reflecting how it scores each miner. These weights determine how incentive is distributed. Weights are stored on-chain as sparse `Vec<u16>` pairs (uid, weight) and expanded into a dense n-by-n matrix in the metagraph.

### Bonds

Validator-to-miner investment tracking. Bonds record how much stake a validator has directed toward each miner over time. Like weights, bonds are stored as sparse `Vec<u16>` pairs on-chain and expanded into a dense matrix in the metagraph.

### Emission

TAO released per block to subnet participants. Emission is computed from weights and stake. Miners with higher incentive scores receive more emission; validators earn emission proportional to their dividend scores.

### Incentive

The reward score assigned to miners based on weights. Higher incentive means more emission per block. The incentive vector is derived from the weight matrix and stake distribution through Yuma consensus.

### Trust

Accumulated signal of neuron reliability. Trust measures how much the network relies on a miner's output. It is computed from the weight matrix, weighted by validator stake.

### Consensus

Agreement metric across validators. Consensus reflects how aligned a validator's weights are with the rest of the validator set. Validators with high consensus scores have more influence over incentive distribution.

### Dividends

Returns paid to delegators and stakers. Validators receive dividends proportional to their consensus scores, which they share with nominators who have staked to their hotkey.

### Delegate

A neuron that accepts external stake from other coldkeys. Delegates set a "take" percentage (in basis points) that determines the split of dividends between the delegate and nominators. Delegates appear in the chain's delegate registry.

### Registration

The process of joining a subnet. There are two registration methods:

- **Burned registration**: Pay a variable amount of TAO (the "burn") to register. The burn cost fluctuates based on demand.
- **POW registration**: Solve a proof-of-work puzzle. The difficulty adjusts based on subnet parameters.

Both methods require a hotkey that is not already registered in the target subnet.

## Additional SDK Types

### PowSolution

Proof-of-work solution for POW registration. Contains the nonce, block number, difficulty, and the resulting hash that meets the difficulty target.

### SubtensorConfig

The subxt `Config` implementation for Bittensor's chain. Inherits Blake2-256 hashing, 32-byte account IDs, sr25519 signatures, and standard Substrate extrinsic parameters from `subxt::config::substrate::SubstrateConfig`.

### NeuronInfo

Full neuron information returned by chain queries. Includes UID, netuid, hotkey, coldkey, stake, rank, trust, consensus, incentive, dividend, emission, validator trust, weights, bonds, and more. The metagraph reconstructs `NeuronInfo` values from its columnar storage when you iterate.

### AxonInfo

Axon endpoint metadata stored on-chain. Contains IP (packed u64), port, IP type, protocol, version, hotkey, and coldkey. The dendrite converts `AxonInfo` into a URL to reach the axon.

### TxSuccess

The result type returned by extrinsic methods in `bittensor-chain`. It contains the block hash and extrinsic hash, confirming that the transaction was included and finalized on-chain.

```rust
// Rust return type
pub struct TxSuccess {
    pub block_hash: subxt::utils::H256,
    pub extrinsic_hash: subxt::utils::H256,
}
```

In Python bindings, the same data is returned as `TxSuccessPy` with `block_hash` and `extrinsic_hash` string properties.

### PrometheusInfo

Metadata for a neuron's Prometheus metrics endpoint. Contains IP (packed u64), port, version identifier, and the block number at which the endpoint was registered. Validators and subnet owners use Prometheus endpoints to monitor miner uptime and performance.

### StakeInfo

A record linking a hotkey/coldkey pair to a specific stake amount. Returned by `get_stake_info` queries, each `StakeInfo` contains the hotkey SS58 address, coldkey SS58 address, and staked `Balance`.

### DelegateInfo

On-chain delegate metadata. Contains the delegate SS58 address, delegate hotkey, total stake, owner hotkey, take percentage (basis points), owner SS58, list of registered netuids, validator permits, and a vector of nominator (address, stake) pairs.

### SubnetInfo

Subnet descriptor stored on-chain. Contains netuid, human-readable name, owner hotkey, tempo (blocks per weight-setting round), maximum UID count, modality type (text, image, audio, etc.), and the network-level UID.

### SubnetHyperparameters

Tunable parameters that control incentive distribution within a subnet. Includes rho, kappa, difficulty, burn, immunity_ratio, min_burn, max_burn, weights_rate_limit, weights_version, max_weight_limit, scaling_law_power, subnetwork_n, max_n, tempo, and liquid_alpha_enabled. These are set by the subnet owner and can be updated via the `set_hyperparameters` extrinsic.

### MetagraphInfo

A summary structure for a subnet's metagraph state. Contains netuid, block number, neuron count (n), total stake, and total issuance. Returned by `get_metagraph` queries.

### NeuronCertificate

TLS certificate information associated with a neuron's hotkey. Contains the hotkey SS58 address, raw certificate bytes, and the block at which the certificate was registered. Used for establishing secure Axon-Dendrite connections.

## Protocol and Security

### Request Signing

The process by which a Dendrite attaches a Sr25519 signature to outgoing Synapse requests. The signature covers the request body, the target axon's hotkey, and a millisecond-precision nonce. The Axon verifies this signature before processing the request. In bittensor-rs, `sign_request` from `bittensor-dendrite` handles this automatically.

### Body Hash

The SHA3-256 hex digest of the Synapse request body. Both Dendrite and Axon compute this hash independently; if the hashes do not match, the request is rejected. The `computed_body_hash` field on the Synapse carries this value. In bittensor-rs, `sha3_256_hex` computes the digest.

### Blacklist

A set of hotkeys that an Axon refuses to serve. When a blacklisted hotkey sends a request, the Axon middleware returns `403 Forbidden` before reaching the handler. Managed via `axon.blacklist(key)` and `axon.unblacklist(key)`.

### Priority Routing

A mechanism by which an Axon serves higher-priority requests before lower-priority ones. Each hotkey is assigned a numeric priority value; the middleware sorts incoming requests accordingly. Managed via `axon.set_priority(key, value)`.

### Nonce

A monotonically increasing number included in each signed Synapse request for replay protection. The Dendrite sets the nonce to the current Unix timestamp in milliseconds. The Axon may reject requests with stale or duplicate nonces.

## Architecture

### SCALE Codec

Simple Concatenated Aggregate Little-endian encoding. The binary serialization format used by Substrate for all on-chain data. subxt generates typed SCALE decoders from the compiled metadata at `metadata/finney.scale`. bittensor-rs does not require users to interact with SCALE directly; the subxt layer handles encoding and decoding transparently.

### Feature Flags

Conditional compilation flags that gate optional functionality in bittensor-rs crates. For example, `bittensor-chain` supports `storage-subscriptions` (on by default), `drand`, `mev-shield`, and `integration-tests`. The `bittensor-cli` crate exposes a `mev` feature for MEV-protected extrinsics.

### Saturating Arithmetic

Overflow- and underflow-safe arithmetic used in Balance operations. Addition saturates at `u64::MAX` (18,446,744,073,709,551,615). Subtraction saturates at 0. Both the WASM `Balance` type and the Python `Balance` type use saturating `add` and `sub` methods to prevent panics.

### PyO3

The Rust framework used to build the `bittensor_rs` Python package. PyO3 provides Rust-to-Python type bridging, GIL management, and async runtime integration via `pyo3-async-runtimes::tokio`. All chain methods return Python coroutines that must be `await`ed.

### wasm-bindgen

The tool and library that generates JavaScript bindings from Rust code compiled to `wasm32-unknown-unknown`. The `bittensor-wasm` crate uses `wasm-bindgen` to expose types like `Balance`, `AxonInfo`, and `NeuronInfoLite` to browser JavaScript.

### gloo-net

The HTTP client library used by `bittensor-wasm` for JSON-RPC queries in the browser. It replaces reqwest (which requires tokio) with a WASM-compatible fetch-based HTTP client.

### Columnar Storage

The metagraph stores neuron attributes as parallel arrays (stake, rank, trust, etc.) indexed by positional UID, rather than as an array of neuron structs. This layout enables vectorized operations and matches the Python SDK's `metagraph.S`, `metagraph.R`, etc.

### Metadata

Pre-compiled Substrate runtime metadata stored at `metadata/finney.scale`. subxt uses this to generate typed storage and extrinsic interfaces at compile time. When the Finney runtime upgrades, regenerate the metadata:

```bash
subxt metadata --url wss://entrypoint-finney.opentensor.ai:443 -f bytes > metadata/finney.scale
cargo check -p bittensor-chain
```
