# bittensor-metagraph

Subnet neural graph state: sync from chain, iterate neurons, serialize to disk.

## Overview

The `bittensor-metagraph` crate provides a columnar representation of a subnet's neural graph. Each neuron in a subnet has a UID, hotkey, coldkey, stake, rank, trust, consensus, incentive, dividends, emission, weights, bonds, and other fields. The metagraph stores these as parallel arrays rather than an array of structs, matching the Python SDK's `bittensor.metagraph` layout.

Columnar storage enables efficient vectorized operations on numeric fields. The `weights` and `bonds` fields store flattened n-by-n matrices in row-major order, so entire weight rows or columns can be sliced without restructuring.

The crate supports three operations: syncing state from the chain, iterating over neurons, and saving/loading to JSON files.

### Feature Flags

| Feature | Default | Description |
|---|---|---|
| `ml-backend` | no | Enable ML-based weight matrix operations via the `MlBackend` trait |

When `ml-backend` is enabled, you can implement custom tensor backends (candle, tch) instead of using the default `ndarray::Array1<f32>` storage.

### Crate

```toml
[dependencies]
bittensor-metagraph = "0.1"
# Optional: enable ML backend support
# bittensor-metagraph = { version = "0.1", features = ["ml-backend"] }
```

### Prelude

```rust
use bittensor_metagraph::prelude::*;
```

The prelude re-exports:

| Item | Source |
|---|---|
| `Metagraph` | `metagraph` module |
| `sync` | `sync` module |
| `save` | `serialize` module |
| `load` | `serialize` module |

---

## Quick Start

```rust,no_run
use bittensor_metagraph::prelude::*;
use bittensor_chain::prelude::SubtensorClient;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), bittensor_core::error::BittensorError> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
    let metagraph = sync(&client, 1).await?;
    println!("Subnet 1 has {} neurons at block {}", metagraph.n, metagraph.block);

    for neuron in metagraph.neurons() {
        println!("UID {} hotkey={} stake={:.2}", neuron.uid, neuron.hotkey, neuron.stake.to_tao());
    }

    Ok(())
}
```

---

## Protocol Flow

The metagraph is a snapshot of subnet state at a given block. It is not a live stream. The typical workflow is:

1. Connect to the chain with `SubtensorClient`.
2. Call `sync(&client, netuid)` to fetch every neuron in the subnet and build a `Metagraph`.
3. Inspect the metagraph: iterate neurons, look up by UID, read columnar arrays.
4. Optionally, save the metagraph to disk with `save()` and load it later with `load()`.

Because syncing reads N individual chain queries (one per UID), it can be slow for large subnets. Cache the result with `save()` if you need to reload it without re-querying the chain.

---

## sync Function

```rust
pub async fn sync(client: &SubtensorClient, netuid: u16) -> Result<Metagraph>
```

Fetches all neurons in the given subnet from the chain and constructs a `Metagraph`.

### Parameters

| Name | Type | Description |
|---|---|---|
| `client` | `&SubtensorClient` | An active connection to a Subtensor node |
| `netuid` | `u16` | Subnet identifier (e.g. `1` for the root subnet) |

### Returns

`Result<Metagraph>`, which is `std::result::Result<Metagraph, BittensorError>`.

Possible errors:

- `BittensorError::Rpc` if any chain query fails
- `BittensorError::Codec` if the runtime metadata is out of date

### How It Works

1. Queries the current block number from the chain.
2. Queries the neuron count for the given netuid.
3. Iterates UID 0 through (neuron_count - 1), calling `get_neuron` for each.
4. Passes the collected `NeuronInfo` values to `Metagraph::from_neurons`.

```rust,no_run
use bittensor_chain::prelude::SubtensorClient;
use bittensor_core::config::NetworkConfig;
use bittensor_metagraph::prelude::*;

let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
let metagraph = sync(&client, 1).await?;
println!("Subnet 1 has {} neurons at block {}", metagraph.n, metagraph.block);
```

---

## Metagraph Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metagraph {
    pub netuid: u16,
    pub n: usize,
    pub uids: Vec<u16>,
    pub hotkeys: Vec<String>,
    pub coldkeys: Vec<String>,
    pub stake: Array1<f32>,
    pub ranks: Array1<f32>,
    pub trust: Array1<f32>,
    pub consensus: Array1<f32>,
    pub validator_trust: Array1<f32>,
    pub incentive: Array1<f32>,
    pub dividends: Array1<f32>,
    pub emission: Array1<f32>,
    pub weights: Array1<f32>,
    pub bonds: Array1<f32>,
    pub active: Vec<bool>,
    pub axon_info: Vec<Option<AxonInfo>>,
    pub prometheus_info: Vec<Option<PrometheusInfo>>,
    pub block: u64,
}
```

### Fields

Every field is indexed by positional order. The element at index `i` corresponds to the neuron with `uids[i]`.

| Field | Type | Description |
|---|---|---|
| `netuid` | `u16` | Subnet identifier |
| `n` | `usize` | Number of registered neurons |
| `uids` | `Vec<u16>` | UID of each neuron |
| `hotkeys` | `Vec<String>` | SS58-encoded hotkey of each neuron |
| `coldkeys` | `Vec<String>` | SS58-encoded coldkey of each neuron |
| `stake` | `Array1<f32>` | Stake in TAO for each neuron |
| `ranks` | `Array1<f32>` | Rank score for each neuron |
| `trust` | `Array1<f32>` | Trust score for each neuron |
| `consensus` | `Array1<f32>` | Consensus score for each neuron |
| `validator_trust` | `Array1<f32>` | Validator trust score for each neuron |
| `incentive` | `Array1<f32>` | Incentive score for each neuron |
| `dividends` | `Array1<f32>` | Dividend earned for each neuron |
| `emission` | `Array1<f32>` | Emission in RAO per tempo for each neuron |
| `weights` | `Array1<f32>` | Flattened n*n weight matrix, row-major. `weights[i*n + j]` is the weight from neuron i to neuron j |
| `bonds` | `Array1<f32>` | Flattened n*n bond matrix, row-major. Same layout as weights |
| `active` | `Vec<bool>` | Whether each neuron is actively serving |
| `axon_info` | `Vec<Option<AxonInfo>>` | Axon serving metadata for each neuron. `None` if not serving |
| `prometheus_info` | `Vec<Option<PrometheusInfo>>` | Prometheus metrics endpoint for each neuron. `None` if not registered |
| `block` | `u64` | Block number at which this metagraph was synced |

### Weight Matrix Encoding

The `weights` array is a flattened n-by-n matrix in row-major order. The weight that the neuron at position `i` assigns to the neuron at position `j` is:

```rust
let w = metagraph.weights[i * metagraph.n + j];
```

On-chain, each neuron stores weights as a sparse vector of alternating `[uid, weight]` u16 pairs. `from_neurons` expands this sparse encoding into the full dense matrix. A value of `0.0` means no connection.

When `neuron_at` reconstructs a `NeuronInfo`, it reverses the process: it extracts the neuron's row from the flattened matrix and converts non-zero entries back to the `[uid, weight]` sparse format.

### Bond Matrix Encoding

The `bonds` array uses the same flattened n-by-n row-major layout as `weights`. The bond from neuron at position `i` to neuron at position `j` is:

```rust
let b = metagraph.bonds[i * metagraph.n + j];
```

Same sparse-to-dense expansion and dense-to-sparse reconstruction as weights.

---

## Constructors

### `Metagraph::new(netuid: u16) -> Self`

Creates an empty metagraph with zero neurons. All vector fields are empty, all `Array1` fields have length 0, and `block` is 0.

```rust
let mg = Metagraph::new(7);
assert_eq!(mg.netuid, 7);
assert_eq!(mg.n, 0);
assert!(mg.uids.is_empty());
assert!(mg.hotkeys.is_empty());
assert!(mg.coldkeys.is_empty());
assert!(mg.active.is_empty());
assert_eq!(mg.stake.len(), 0);
assert_eq!(mg.ranks.len(), 0);
assert_eq!(mg.trust.len(), 0);
assert_eq!(mg.consensus.len(), 0);
assert_eq!(mg.validator_trust.len(), 0);
assert_eq!(mg.incentive.len(), 0);
assert_eq!(mg.dividends.len(), 0);
assert_eq!(mg.emission.len(), 0);
assert_eq!(mg.weights.len(), 0);
assert_eq!(mg.bonds.len(), 0);
assert_eq!(mg.block, 0);
```

### `Metagraph::from_neurons(netuid: u16, block: u64, neurons: &[NeuronInfo]) -> Self`

Builds a metagraph from a slice of `NeuronInfo` snapshots. This is the primary construction path used by `sync`. It:

1. Extracts scalar fields into vectors.
2. Converts `Balance` stake values to f32 TAO.
3. Expands the sparse weight encoding (alternating `[uid, weight]` u16 pairs) into a full n*n matrix.
4. Does the same for bonds.

```rust
let neurons: Vec<NeuronInfo> = vec![/* ... */];
let mg = Metagraph::from_neurons(1, 500, &neurons);
```

---

## Lookup Methods

### `fn neurons(&self) -> NeuronIterator<'_>`

Returns an iterator that yields a `NeuronInfo` for each neuron. Each iteration reconstructs a `NeuronInfo` from the columnar storage, matching the Python SDK's `metagraph.neurons()` pattern.

The iterator implements `ExactSizeIterator`, so you can call `.len()` to get the remaining count.

```rust
for neuron in metagraph.neurons() {
    println!(
        "UID {} hotkey={} stake={:.2}",
        neuron.uid, neuron.hotkey, neuron.stake.to_tao()
    );
}
```

### `fn neuron_by_uid(&self, uid: u16) -> Option<NeuronInfo>`

Looks up a neuron by UID value. Returns `None` if the UID is not present in the metagraph.

This performs a linear scan of the `uids` vector to find the positional index, then calls `neuron_at`. For large subnets with frequent UID lookups, consider building a `HashMap<u16, usize>` index yourself.

```rust
if let Some(neuron) = metagraph.neuron_by_uid(42) {
    println!("Found UID 42: hotkey={}", neuron.hotkey);
}
```

### `fn neuron_at(&self, pos: usize) -> NeuronInfo`

Reconstructs a `NeuronInfo` at the given positional index. If `pos` is out of bounds, returns a default `NeuronInfo` with `active: false`, `uid: 0`, and zero values.

The reconstruction extracts the neuron's row from the flattened weight and bond matrices, converts it back to the sparse `[uid, weight, uid, weight, ...]` encoding, and packs all scalar fields.

```rust
let first = metagraph.neuron_at(0);
println!("First neuron UID: {}", first.uid);
```

---

## Index Trait

```rust
impl Index<u16> for Metagraph {
    type Output = ();
    fn index(&self, uid: u16) -> &Self::Output;
}
```

Validates that a UID exists in the metagraph. Panics if the UID is not found, matching the Python SDK's `metagraph[uid]` behavior. Because the `Index` trait cannot return allocated values, this is primarily useful for assertion-style checks. For actual data retrieval, use `neuron_by_uid` or `neuron_at`.

```rust
let _ = &metagraph[0];  // OK if UID 0 exists
// &metagraph[999];     // panics if UID 999 not found
```

---

## IntoIterator

A reference to a `Metagraph` implements `IntoIterator`, yielding `NeuronInfo` values:

```rust
for neuron in &metagraph {
    println!("UID {}", neuron.uid);
}
```

This is equivalent to calling `metagraph.neurons()`.

---

## NeuronIterator

```rust
pub struct NeuronIterator<'a> {
    pub metagraph: &'a Metagraph,
    pub index: usize,
}

impl<'a> Iterator for NeuronIterator<'a> {
    type Item = NeuronInfo;
    // ...
}

impl<'a> ExactSizeIterator for NeuronIterator<'a> {}

impl<'a> IntoIterator for &'a Metagraph {
    type Item = NeuronInfo;
    type IntoIter = NeuronIterator<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.neurons()
    }
}
```

Yields reconstructed `NeuronInfo` values by calling `metagraph.neuron_at(index)` for each position from 0 to `n - 1`. Implements `ExactSizeIterator` for efficient `len()` calls.

---

## Serialization: save and load

### `fn save(metagraph: &Metagraph, path: &Path) -> Result<()>`

Serializes the metagraph to a pretty-printed JSON file. Creates parent directories if they do not exist.

#### Parameters

| Name | Type | Description |
|---|---|---|
| `metagraph` | `&Metagraph` | The metagraph to persist |
| `path` | `&Path` | Destination file path |

#### Returns

`Ok(())` on success, or a `BittensorError` variant:

- `BittensorError::Validation` if the path has no parent directory or file I/O fails
- `BittensorError::Codec` if JSON serialization fails

```rust
use std::path::Path;

save(&metagraph, Path::new("/tmp/metagraphs/subnet_1.json"))?;
```

### `fn load(path: &Path) -> Result<Metagraph>`

Deserializes a metagraph from a JSON file.

#### Parameters

| Name | Type | Description |
|---|---|---|
| `path` | `&Path` | Source file path |

#### Returns

`Ok(Metagraph)` on success, or a `BittensorError` variant:

- `BittensorError::Validation` if the file cannot be read
- `BittensorError::Codec` if the JSON is malformed

```rust
let loaded = load(Path::new("/tmp/metagraphs/subnet_1.json"))?;
assert_eq!(loaded.netuid, 1);
```

---

## Feature: ml-backend

When the `ml-backend` feature is enabled, the crate exposes the `MlBackend` trait:

```rust
#[cfg(feature = "ml-backend")]
pub trait MlBackend: Clone {
    type Tensor: Clone + Send + Sync;

    fn zeros(len: usize) -> Self::Tensor;
    fn from_vec(data: Vec<f32>) -> Self::Tensor;
    fn get(tensor: &Self::Tensor, index: usize) -> f32;
    fn set(tensor: &mut Self::Tensor, index: usize, value: f32);
}
```

This trait defines the interface that any ML tensor backend must satisfy. It abstracts over the storage and access patterns of 1-D float tensors, letting you swap `ndarray` for `candle` or `tch` without changing the metagraph logic.

### NdarrayBackend

A concrete implementation using `ndarray::Array1<f32>` is always available when `ml-backend` is enabled:

```rust
#[cfg(feature = "ml-backend")]
impl MlBackend for NdarrayBackend {
    type Tensor = Array1<f32>;

    fn zeros(len: usize) -> Self::Tensor { Array1::zeros(len) }
    fn from_vec(data: Vec<f32>) -> Self::Tensor { Array1::from_vec(data) }
    fn get(tensor: &Self::Tensor, index: usize) -> f32 { tensor[index] }
    fn set(tensor: &mut Self::Tensor, index: usize, value: f32) { tensor[index] = value; }
}
```

To implement a custom backend:

```rust,ignore
#[cfg(feature = "ml-backend")]
use bittensor_metagraph::metagraph::MlBackend;

#[derive(Clone)]
struct CandleBackend;

impl MlBackend for CandleBackend {
    type Tensor = candle_core::Tensor;

    fn zeros(len: usize) -> Self::Tensor {
        candle_core::Tensor::zeros(len, candle_core::DType::F32, &candle_core::Device::Cpu).unwrap()
    }

    fn from_vec(data: Vec<f32>) -> Self::Tensor {
        candle_core::Tensor::from_vec(data, candle_core::DType::F32, &candle_core::Device::Cpu).unwrap()
    }

    fn get(tensor: &Self::Tensor, index: usize) -> f32 {
        tensor.get(index).unwrap().to_scalar::<f32>().unwrap()
    }

    fn set(tensor: &mut Self::Tensor, index: usize, value: f32) {
        // candle tensors are immutable; consider a mutable wrapper or side buffer
        unimplemented!()
    }
}
```

---

## Code Examples

### Sync and Print

```rust,no_run
use bittensor_metagraph::prelude::*;
use bittensor_chain::prelude::SubtensorClient;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), bittensor_core::error::BittensorError> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
    let metagraph = sync(&client, 1).await?;
    println!("Synced subnet {} at block {}", metagraph.netuid, metagraph.block);
    println!("Total neurons: {}", metagraph.n);
    Ok(())
}
```

### Iterate Neurons

```rust,no_run
for neuron in metagraph.neurons() {
    println!(
        "  UID {:3} | hotkey={} | stake={:.4} TAO | active={}",
        neuron.uid, neuron.hotkey, neuron.stake.to_tao(), neuron.active
    );
}

// Count active neurons
let active_count = metagraph.active.iter().filter(|&&a| a).count();
println!("Active neurons: {active_count} / {}", metagraph.n);
```

### Lookup by UID

```rust,no_run
// Find a specific UID
if let Some(neuron) = metagraph.neuron_by_uid(0) {
    println!("UID 0 hotkey: {}", neuron.hotkey);
}

// Assertion-style check (panics if missing)
let _ = &metagraph[0];
```

### Save and Load

```rust,no_run
use std::path::Path;

// Save to disk
let save_path = Path::new("/tmp/bittensor/metagraph_subnet_1.json");
save(&metagraph, save_path)?;
println!("Saved to {}", save_path.display());

// Load from disk (no chain query needed)
let loaded = load(save_path)?;
assert_eq!(loaded.netuid, metagraph.netuid);
assert_eq!(loaded.n, metagraph.n);
assert_eq!(loaded.block, metagraph.block);
```

### Access Weight Matrix

```rust,no_run
// Total stake in the subnet
let total_stake: f32 = metagraph.stake.sum();
println!("Total stake: {total_stake:.2} TAO");

// Average incentive
let avg_incentive: f32 = metagraph.incentive.sum() / metagraph.n as f32;
println!("Average incentive: {avg_incentive:.4}");

// Read the weight from neuron at position 3 to neuron at position 7
let w = metagraph.weights[3 * metagraph.n + 7];
println!("Weight from 3->7: {w}");

// Print all outgoing weights for neuron at position 0
let row_start = 0;
let row_end = row_start + metagraph.n;
let weight_row = &metagraph.weights.slice(ndarray::s![row_start..row_end]);
for (j, &w) in weight_row.iter().enumerate() {
    if w > 0.0 {
        println!("Neuron 0 -> Neuron {j}: weight = {w}");
    }
}
```

---

## Error Types

Metagraph operations use `bittensor_core::error::BittensorError` as their error type. The specific variants you may encounter:

| Variant | Source | Description |
|---|---|---|
| `BittensorError::Rpc(msg)` | `sync` | A chain query returned an error or the RPC endpoint is unreachable |
| `BittensorError::Codec(msg)` | `sync`, `save`, `load` | JSON serialization/deserialization failed |
| `BittensorError::Validation(msg)` | `save`, `load` | File I/O error, invalid path, or missing parent directory |

The `Result` type alias used internally:

```rust
pub type Result<T> = std::result::Result<T, BittensorError>;
```

---

## Caching Strategy

Because `sync` is expensive (one RPC call per neuron), the recommended pattern for long-running applications is to sync once, cache to disk, and only re-sync when the block has advanced:

```rust,no_run
use std::path::Path;

const CACHE_PATH: &str = "/tmp/bittensor/metagraph_cache.json";

async fn get_metagraph(
    client: &SubtensorClient,
    netuid: u16,
) -> Result<Metagraph, bittensor_core::error::BittensorError> {
    let path = Path::new(CACHE_PATH);

    // Try loading from cache
    if path.exists() {
        if let Ok(cached) = load(path) {
            let current_block = bittensor_chain::queries::get_network_block(client.rpc()).await?;
            if cached.block >= current_block {
                return Ok(cached);
            }
        }
    }

    // Cache is stale or missing: re-sync
    let metagraph = sync(client, netuid).await?;
    let _ = save(&metagraph, path);
    Ok(metagraph)
}
```

---

## Comparison with Python SDK

| Feature | Python `bittensor.metagraph` | Rust `bittensor-metagraph` |
|---|---|---|
| Storage | Columnar `torch.Tensor` / `np.ndarray` | Columnar `ndarray::Array1<f32>` |
| Field names | Identical | Identical (`uids`, `hotkeys`, `stake`, `ranks`, etc.) |
| Weight matrix | Flattened n*n, row-major | Flattened n*n, row-major (identical layout) |
| Bond matrix | Flattened n*n, row-major | Flattened n*n, row-major (identical layout) |
| `neurons()` | Returns list of `NeuronInfo` | Returns `NeuronIterator` yielding `NeuronInfo` |
| `neuron_by_uid()` | Not built-in (manual dict) | Built-in, returns `Option<NeuronInfo>` |
| `metagraph[uid]` | Validates UID, raises `KeyError` | Validates UID, panics (matches Python behavior) |
| Persistence | `torch.save()` / `torch.load()` (pickle) | `save()` / `load()` (JSON) |
| Sync | `metagraph.sync()` (method on object) | `sync(&client, netuid)` (free function) |
| Thread safety | N/A (GIL) | `Send + Sync` (safe across tokio tasks) |
| Feature flags | None | `ml-backend` (swap tensor backend) |
| `from_neurons()` | Internal during sync | Public, can build from any `&[NeuronInfo]` |
| `neuron_at()` | Not available | Available, returns `NeuronInfo` (out-of-bounds returns default) |

The Rust crate produces structurally equivalent data. The columnar field layout matches the Python SDK exactly, so any analysis pipeline that reads `metagraph.ranks[5]` or `metagraph.weights[i*n + j]` works identically in both SDKs. The key difference is that the Rust crate uses JSON for persistence (cross-language readable) instead of Python's pickle format.
