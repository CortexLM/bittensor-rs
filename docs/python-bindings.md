# Python Bindings

The `bittensor_rs` package provides Python bindings for the bittensor-rs SDK, built with PyO3 and pyo3-async-runtimes. It exposes the same chain operations, wallet management, and protocol types as the Rust SDK, accessible from standard Python async code.

## Installation

```bash
pip install bittensor-rs
```

The wheel bundles a pre-compiled native extension. No Rust toolchain is required.

Note: the pip package name uses a hyphen (`bittensor-rs`), but the import uses an underscore:

```python
import bittensor_rs
```

## Quick Start

```python
import asyncio
import bittensor_rs as bt

async def main():
    # Connect to Finney mainnet
    client = await bt.SubtensorClient.connect(bt.NetworkConfig.finney())

    # Load a wallet
    wallet = bt.Wallet.load("default", "~/.bittensor/wallets")

    # Query balance
    balance = await client.get_balance(wallet.ss58_address)
    print(f"Balance: {balance}")

asyncio.run(main())
```

Equivalent Rust:

```rust
use bittensor_chain::prelude::*;
use bittensor_core::config::NetworkConfig;
use bittensor_wallet::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
    let wallet = Wallet::with_path("default", PathBuf::from("~/.bittensor/wallets"));
    let balance = bittensor_chain::queries::account::get_balance(
        client.rpc(), &wallet.get_coldkeypub()?.try_into()?
    ).await?;
    println!("Balance: {balance}");
    Ok(())
}
```

## Module Contents

| Python Class | Source Module | Purpose |
|---|---|---|
| `SubtensorClient` | `chain_client` | Chain connection, queries, and extrinsics |
| `Wallet` | `wallet` | Coldkey/hotkey pair management and signing |
| `Balance` | `core_types` | TAO/RAO arithmetic with full operator support |
| `NetworkConfig` | `core_types` | Network endpoint configuration |
| `AxonInfo` | `core_types` | Axon endpoint metadata |
| `PrometheusInfo` | `core_types` | Prometheus metrics endpoint metadata |
| `StakeInfo` | `core_types` | Stake record for a hotkey/coldkey pair |
| `DelegateInfo` | `core_types` | Delegate metadata including take and nominators |
| `NeuronInfo` | `core_types` | Full neuron information including weights and bonds |
| `NeuronInfoLite` | `core_types` | Lightweight neuron information |
| `SubnetInfo` | `core_types` | Subnet metadata |
| `SubnetHyperparameters` | `core_types` | Subnet incentive distribution parameters |
| `MetagraphInfo` | `core_types` | Subnet metagraph summary |
| `NeuronCertificate` | `core_types` | Neuron TLS certificate information |
| `BittensorError` | `core_types` | SDK error type |
| `TerminalInfo` | `synapse` | Synapse endpoint metadata |
| `Synapse` | `synapse` | Base synapse class (subclassable) |
| `StreamingSynapse` | `synapse` | SSE streaming synapse (subclassable) |
| `AxonConfig` | `axon` | Axon HTTP server configuration |
| `Axon` | `axon` | Neuron HTTP server with middleware |
| `DendriteConfig` | `dendrite` | Dendrite HTTP client configuration |
| `Dendrite` | `dendrite` | Signed HTTP client for Axon queries |
| `Metagraph` | `metagraph` | Subnet neural graph with sync/save/load |
| `DrandBeacon` | `drand_beacon` | DRAND randomness beacon (feature-gated) |
| `MevShield` | `mev_shield` | Post-quantum MEV protection (feature-gated) |

---

## SubtensorClient

Chain connection and extrinsic submission. All chain methods are async and return Python coroutines that must be `await`ed.

### Construction

#### `SubtensorClient()`

Creates a disconnected client instance. Call `connect()` or `from_url()` before using chain methods.

```python
client = bt.SubtensorClient()
```

#### `SubtensorClient.connect(network_config)` (classmethod)

Connects to a network defined by a `NetworkConfig`. Returns a connected `SubtensorClient` coroutine.

```python
config = bt.NetworkConfig.finney()
client = await bt.SubtensorClient.connect(config)
```

Rust equivalent:

```rust
let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
```

#### `SubtensorClient.from_url(url)` (staticmethod)

Connects to an arbitrary WebSocket URL. Returns a connected `SubtensorClient` coroutine.

```python
client = await bt.SubtensorClient.from_url("wss://entrypoint-finney.opentensor.ai:443")
```

Rust equivalent:

```rust
let client = SubtensorClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;
```

### Query Methods

#### `get_balance(address)`

Returns the free `Balance` for an SS58 address.

```python
balance = await client.get_balance("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")
print(balance.tao)   # float in TAO
print(balance.rao)   # int in RAO
```

Rust equivalent:

```rust
let balance: Balance = bittensor_chain::queries::account::get_balance(
    client.rpc(), &account_id
).await?;
```

#### `get_total_balance(address)`

Returns the total `Balance` (free + reserved) for an SS58 address.

```python
total = await client.get_total_balance("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")
```

Rust equivalent:

```rust
let total: Balance = bittensor_chain::queries::account::get_total_balance(
    client.rpc(), &account_id
).await?;
```

#### `get_total_stake()`

Returns the global total staked `Balance` across all subnets.

```python
total_stake = await client.get_total_stake()
```

#### `get_stake_info(coldkey_address)`

Returns a list of `StakeInfo` records for all hotkeys delegated to the given coldkey.

```python
stakes = await client.get_stake_info("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")
for s in stakes:
    print(f"Hotkey: {s.hotkey}, Stake: {s.stake.tao}")
```

#### `get_metagraph(netuid)`

Returns a `MetagraphInfo` summary for the specified subnet.

```python
meta = await client.get_metagraph(1)
print(f"Subnet 1: n={meta.n}, block={meta.block}, stake={meta.stake.tao}")
```

### Extrinsic Methods

All extrinsic methods accept a `signer` parameter that can be either:
- A 64-character hex-encoded secret seed (with or without `0x` prefix)
- A BIP-39 mnemonic phrase (detected by word count >= 12)

The optional `password` parameter is used when the signer is derived from an encrypted keyfile.

#### `add_stake(hotkey, netuid, amount, signer, password=None)`

Stake RAO to a hotkey on a subnet. `amount` is in RAO (u64). Returns `TxSuccess` on success.

```python
result = await client.add_stake(
    hotkey="5CzR6NjA5V6Nq2k6U6iU8V6L2r2F2p2n2v2b2m2s2t2u2w2y",
    netuid=1,
    amount=5_000_000_000,  # 5 TAO in RAO
    signer="word1 word2 word3 ... word12",
)
print(f"Block: {result.block_hash}")
print(f"Extrinsic: {result.extrinsic_hash}")
```

Rust equivalent:

```rust
let signer = subxt_signer::sr25519::Keypair::from_uri("//Alice")?;
bittensor_chain::extrinsics::staking::add_stake(
    client.rpc(), &signer, hotkey_id, netuid, amount_rao
).await?;
```

#### `remove_stake(hotkey, netuid, amount, signer, password=None)`

Unstake RAO from a hotkey on a subnet. `amount` is in RAO (u64).

```python
result = await client.remove_stake(
    hotkey="5CzR6NjA5V6Nq2k6U6iU8V6L2r2F2p2n2v2b2m2s2t2u2w2y",
    netuid=1,
    amount=2_000_000_000,
    signer="0x0000000000000000000000000000000000000000000000000000000000000001",
)
```

#### `transfer(dest, amount, signer, password=None)`

Transfer RAO to a destination address. `amount` is in RAO (u64).

```python
result = await client.transfer(
    dest="5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
    amount=1_000_000_000,  # 1 TAO in RAO
    signer="word1 word2 word3 ... word12",
)
```

Rust equivalent:

```rust
let dest = subxt_signer::sr25519::PublicKey::from_uri("//Bob")?;
bittensor_chain::extrinsics::transfer::transfer(
    client.rpc(), &signer, &dest, amount_rao
).await?;
```

#### `register(netuid, hotkey, signer, password=None)`

Register a hotkey on a subnet via burned registration. The registration cost is deducted from the signer's balance.

```python
result = await client.register(
    netuid=1,
    hotkey="5CzR6NjA5V6Nq2k6U6iU8V6L2r2F2p2n2v2b2m2s2t2u2w2y",
    signer="word1 word2 word3 ... word12",
)
```

### TxSuccess

Returned by all extrinsic methods on success.

| Property | Type | Description |
|---|---|---|
| `block_hash` | `str` | Block hash as `0x`-prefixed hex (66 characters) |
| `extrinsic_hash` | `str` | Extrinsic hash as `0x`-prefixed hex (66 characters) |

```python
print(repr(result))
# TxSuccess(block_hash='0x...', extrinsic_hash='0x...')
```

---

## Wallet

Manages coldkey/hotkey pairs stored on disk in the NaCl secretbox format (cross-compatible with the Python SDK).

### Construction

#### `Wallet.create(name, path, password="")` (classmethod)

Generates a new coldkey, saves it to disk, creates a default hotkey, and returns a `Wallet` instance.

```python
wallet = bt.Wallet.create("miner", "~/.bittensor/wallets", password="secret")
```

Rust equivalent:

```rust
let mut wallet = Wallet::with_path("miner", PathBuf::from("~/.bittensor/wallets"));
let mnemonic = wallet.create_coldkey("secret")?;
wallet.create_hotkey()?;
println!("Back up this mnemonic: {mnemonic}");
```

#### `Wallet.load(name, path, hotkey_name="default")` (classmethod)

Loads an existing wallet from disk. Reads the coldkeypub and the specified hotkey.

```python
wallet = bt.Wallet.load("default", "~/.bittensor/wallets", hotkey_name="default")
```

### Properties and Methods

#### `ss58_address` (property)

The SS58-encoded coldkeypub address.

```python
print(wallet.ss58_address)  # '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY'
```

#### `get_coldkeypub()`

Returns the coldkeypub SS58 address string (same as `ss58_address`).

```python
addr = wallet.get_coldkeypub()
```

#### `get_coldkey_pair(password)`

Decrypts and returns the coldkey SS58 address. Requires the password used during creation.

```python
coldkey_ss58 = wallet.get_coldkey_pair("secret")
```

#### `get_hotkey_pair()`

Returns the hotkey SS58 address string.

```python
hotkey_ss58 = wallet.get_hotkey_pair()
```

#### `sign(message)`

Signs a message with the hotkey. Returns the signature as a hex-encoded string.

```python
sig = wallet.sign(b"hello world")
print(sig)  # 'a3f2...'
```

Rust equivalent:

```rust
let signature = wallet.sign(b"hello world")?;
```

#### `sign_coldkey(message, password)`

Signs a message with the coldkey. Requires the decryption password.

```python
sig = wallet.sign_coldkey(b"hello world", "secret")
```

#### `Wallet.verify(message, signature_hex, public_key_hex)` (staticmethod)

Verifies a signature against a public key. Returns `True` if valid.

```python
valid = bt.Wallet.verify(
    b"hello world",
    "a3f2...",
    "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"
)
```

Rust equivalent:

```rust
let valid = bittensor_wallet::keypair::verify(&signature, message, &public_key);
```

#### `name` (property), `path` (property), `hotkey_name` (property)

Read-only access to wallet metadata.

```python
print(wallet.name)        # 'default'
print(wallet.path)        # '/home/user/.bittensor/wallets/default'
print(wallet.hotkey_name) # 'default'
```

---

## Balance

A fixed-point numeric type representing TAO on the Bittensor network. 1 TAO = 1,000,000,000 RAO. The `Balance` class supports Python arithmetic operators and comparisons.

### Construction

| Constructor | Description |
|---|---|
| `Balance(rao=0)` | Create from RAO value (u64) |
| `Balance.from_tao(tao)` | Create from TAO value (float) |
| `Balance.from_rao(rao)` | Create from RAO value (int) |
| `Balance.zero()` | Zero balance |
| `Balance.one_tao()` | Exactly 1.0 TAO |

```python
b1 = bt.Balance(rao=1_000_000_000)   # 1 TAO
b2 = bt.Balance.from_tao(1.0)        # 1 TAO
b3 = bt.Balance.from_rao(500_000_000) # 0.5 TAO
b4 = bt.Balance.zero()               # 0
b5 = bt.Balance.one_tao()             # 1 TAO
```

Rust equivalent:

```rust
use bittensor_core::balance::Balance;

let b1 = Balance::from_rao(1_000_000_000);
let b2 = Balance::from_tao(1.0);
let b3 = Balance::from_rao(500_000_000);
let b4 = Balance::ZERO;
let b5 = Balance::ONE_TAO;
```

### Properties

| Property | Type | Description |
|---|---|---|
| `rao` | `int` | Balance in RAO (u64) |
| `tao` | `float` | Balance in TAO (f64) |

### Operators

| Operator | Operand Type | Result |
|---|---|---|
| `a + b` | `Balance` | Balance addition |
| `a - b` | `Balance` | Balance subtraction |
| `a * n` | `int` | Scalar multiplication |
| `a / b` | `Balance` | Division returns float ratio |
| `a / n` | `int` | Integer division returns Balance |
| `a == b` | `Balance` | Equality |
| `a != b` | `Balance` | Inequality |
| `a < b` | `Balance` | Less than |
| `a <= b` | `Balance` | Less than or equal |
| `a > b` | `Balance` | Greater than |
| `a >= b` | `Balance` | Greater than or equal |
| `hash(a)` | | Hash by RAO value (usable in sets/dicts) |

```python
a = bt.Balance.from_tao(2.0)
b = bt.Balance.from_tao(1.0)

assert (a + b).tao == 3.0
assert (a - b).tao == 1.0
assert (a * 3).tao == 6.0
assert a > b
assert a != b
assert hash(a) == a.rao
```

Rust equivalent:

```rust
let a = Balance::from_tao(2.0);
let b = Balance::from_tao(1.0);

assert_eq!((a + b).to_tao(), 3.0);
assert_eq!((a - b).to_tao(), 1.0);
assert!(a > b);
```

### String Representations

```python
b = bt.Balance.from_tao(1.5)
str(b)    # Human-readable TAO string: "1.500000000"
repr(b)   # 'Balance(rao=1500000000)'
```

---

## NetworkConfig

Defines the WebSocket endpoint and chain identity for connecting to a Subtensor node.

### Construction

| Constructor | Description |
|---|---|
| `NetworkConfig(name, ws_endpoint, archive_endpoint, chain_id)` | Custom configuration |
| `NetworkConfig.finney()` | Finney mainnet |
| `NetworkConfig.test()` | Testnet |
| `NetworkConfig.local()` | Local development node |
| `NetworkConfig.archive()` | Archive node |
| `NetworkConfig.latent_lite()` | Latent lite node |

```python
config = bt.NetworkConfig.finney()
print(config.name)          # 'finney'
print(config.ws_endpoint)   # 'wss://entrypoint-finney.opentensor.ai:443'
print(config.chain_id)      # 42
```

Rust equivalent:

```rust
let config = NetworkConfig::finney();
println!("{}", config.name);
println!("{}", config.ws_endpoint);
```

### Properties

| Property | Type | Description |
|---|---|---|
| `name` | `str` | Network name identifier |
| `ws_endpoint` | `str` | WebSocket RPC endpoint URL |
| `archive_endpoint` | `str or None` | Archive node endpoint (if available) |
| `chain_id` | `int` | Chain identifier (SS58 prefix) |

### Predefined Endpoints

| Network | WebSocket URL |
|---|---|
| Finney | `wss://entrypoint-finney.opentensor.ai:443` |
| Test | `wss://test.finney.opentensor.ai:443` |
| Local | `ws://127.0.0.1:9944` |
| Archive | `wss://archive.finney.opentensor.ai:443` |

---

## AxonInfo

Metadata describing a neuron's Axon endpoint.

### Construction

```python
axon = bt.AxonInfo(
    ip=3232235521,       # u64 IP as integer
    port=8090,           # u16
    ip_type=4,           # u8 (4 for IPv4, 6 for IPv6)
    protocol=1,          # u8 (0 = HTTP, 1 = HTTPS)
    version=4,           # u32
    hotkey="5CzR...",    # str
    coldkey="5Grw..."    # str
)
```

All parameters have defaults: `ip=0`, `port=8090`, `ip_type=4`, `protocol=0`, `version=0`, `hotkey=""`, `coldkey=""`.

### Properties (read-only)

| Property | Type | Description |
|---|---|---|
| `ip` | `int` | IP address as packed u64 |
| `port` | `int` | TCP port number |
| `ip_type` | `int` | IP version (4 or 6) |
| `protocol` | `int` | Protocol identifier |
| `version` | `int` | Protocol version |
| `hotkey` | `str` | Hotkey SS58 address |
| `coldkey` | `str` | Coldkey SS58 address |

---

## PrometheusInfo

Metadata for a neuron's Prometheus metrics endpoint.

### Properties (read-only)

| Property | Type | Description |
|---|---|---|
| `ip` | `int` | IP address as packed u64 |
| `port` | `int` | Metrics port (typically 9100) |
| `version` | `int` | Version identifier |
| `block` | `int` | Block at which this info was registered |

---

## StakeInfo

Stake record for a single hotkey/coldkey pair.

### Properties (read-only)

| Property | Type | Description |
|---|---|---|
| `hotkey` | `str` | Hotkey SS58 address |
| `coldkey` | `str` | Coldkey SS58 address |
| `stake` | `Balance` | Staked amount |

```python
stakes = await client.get_stake_info(coldkey_addr)
for s in stakes:
    print(f"{s.hotkey}: {s.stake.tao} TAO")
```

---

## DelegateInfo

Delegate metadata including take percentage, nominators, and subnet registrations.

### Properties (read-only)

| Property | Type | Description |
|---|---|---|
| `delegate_ss58` | `str` | Delegate SS58 address |
| `delegate_hotkey` | `str` | Delegate hotkey |
| `total_stake` | `Balance` | Total stake under this delegate |
| `owner_hotkey` | `str` | Owner hotkey |
| `take` | `int` | Take percentage (basis points, e.g. 1800 = 18.00%) |
| `owner_ss58` | `str` | Owner SS58 address |
| `registrations` | `list[int]` | List of netuids where delegate is registered |
| `validator_permits` | `list[int]` | List of netuids where delegate has validator permit |
| `nominators` | `list[tuple[str, Balance]]` | List of (SS58 address, stake) tuples |

```python
delegates = await client.get_delegates()
for d in delegates:
    print(f"{d.delegate_hotkey}: take={d.take}, total_stake={d.total_stake.tao}")
    for addr, stake in d.nominators:
        print(f"  Nominator {addr}: {stake.tao} TAO")
```

---

## NeuronInfo

Full neuron information including incentive scores, weights, and bonds.

### Properties (read-only)

| Property | Type | Description |
|---|---|---|
| `uid` | `int` | Neuron UID within the subnet |
| `netuid` | `int` | Subnet identifier |
| `active` | `bool` | Whether the neuron is active |
| `stake` | `Balance` | Total stake |
| `rank` | `int` | Rank score |
| `trust` | `int` | Trust score |
| `consensus` | `int` | Consensus score |
| `incentive` | `int` | Incentive score |
| `dividend` | `int` | Dividend score |
| `emission` | `int` | Emission in RAO per block |
| `hotkey` | `str` | Hotkey SS58 address |
| `coldkey` | `str` | Coldkey SS58 address |
| `last_update` | `int` | Block number of last weight update |
| `validator_trust` | `int` | Validator trust score |

---

## NeuronInfoLite

Lightweight neuron information without weights and bonds arrays.

### Properties (read-only)

Same as `NeuronInfo` except it does not include `emission`, `last_update`, `validator_trust`, or the weights/bonds vectors. Properties: `uid`, `hotkey`, `coldkey`, `active`, `stake`, `rank`, `trust`, `consensus`, `incentive`.

---

## SubnetInfo

Subnet metadata.

### Properties (read-only)

| Property | Type | Description |
|---|---|---|
| `netuid` | `int` | Subnet identifier |
| `name` | `str` | Subnet name |
| `owner_hotkey` | `str` | Owner hotkey SS58 |
| `tempo` | `int` | Blocks per tempo period |
| `maximum_uid` | `int` | Maximum UID count |
| `modality` | `int` | Subnet modality type |
| `network_uid` | `int` | Network-level UID |

---

## SubnetHyperparameters

Parameters controlling incentive distribution within a subnet.

### Properties (read-only)

| Property | Type | Description |
|---|---|---|
| `rho` | `int` | Rho parameter (u16) |
| `kappa` | `int` | Kappa parameter (u16) |
| `difficulty` | `int` | POW difficulty (u32) |
| `burn` | `int` | Current burn cost in RAO (u64) |
| `immunity_ratio` | `int` | Immunity period ratio (u16) |
| `min_burn` | `int` | Minimum burn cost in RAO (u64) |
| `max_burn` | `int` | Maximum burn cost in RAO (u64) |
| `weights_rate_limit` | `int` | Min blocks between weight sets (u64) |
| `weights_version` | `int` | Weights version key (u16) |
| `max_weight_limit` | `int` | Maximum weight limit (u16) |
| `scaling_law_power` | `int` | Scaling law exponent (u16) |
| `subnetwork_n` | `int` | Current subnetwork size (u16) |
| `max_n` | `int` | Maximum subnetwork size (u16) |
| `tempo` | `int` | Blocks per tempo period (u16) |
| `liquid_alpha_enabled` | `bool` | Whether liquid alpha is active |

---

## MetagraphInfo

Summary of a subnet's metagraph state.

### Properties (read-only)

| Property | Type | Description |
|---|---|---|
| `netuid` | `int` | Subnet identifier |
| `block` | `int` | Block number at sync time |
| `n` | `int` | Number of neurons |
| `stake` | `Balance` | Total stake in the subnet |
| `total_issuance` | `Balance` | Total issuance for the subnet |

---

## NeuronCertificate

TLS certificate information for a neuron.

### Properties (read-only)

| Property | Type | Description |
|---|---|---|
| `hotkey` | `str` | Hotkey SS58 address |
| `certificate` | `bytes` | Raw certificate bytes |
| `block` | `int` | Block at which the certificate was set |

---

## TerminalInfo

Endpoint metadata attached to Synapse headers during transmission. All fields are optional and mutable.

### Construction

```python
ti = bt.TerminalInfo(
    status_code=200,
    status_message="OK",
    process_time=0.42,
    ip="10.0.0.1",
    port=8090,
    version=4,
    nonce=12345,
    uuid="abc-def",
    hotkey="5CzR...",
    signature="0xa1b2..."
)
```

All parameters default to `None`.

### Properties (gettable and settable)

| Property | Type | Description |
|---|---|---|
| `status_code` | `int or None` | HTTP status code |
| `status_message` | `str or None` | Status message string |
| `process_time` | `float or None` | Processing time in seconds |
| `ip` | `str or None` | IP address string |
| `port` | `int or None` | TCP port |
| `version` | `int or None` | Protocol version |
| `nonce` | `int or None` | Request nonce |
| `uuid` | `str or None` | Request UUID |
| `hotkey` | `str or None` | Signer hotkey |
| `signature` | `str or None` | Request signature |

### Serialization

#### `to_headers(prefix)`

Serialize non-None fields into a header dictionary with the given prefix.

```python
headers = ti.to_headers("bt_header_axon_")
# {"bt_header_axon_status_code": "200", "bt_header_axon_hotkey": "5CzR...", ...}
```

#### `TerminalInfo.from_headers(headers, prefix)` (classmethod)

Deserialize from a header dictionary.

```python
ti = bt.TerminalInfo.from_headers(response_headers, "bt_header_dendrite_")
```

---

## Synapse

Base class for Bittensor protocol serialization. Python users should subclass `bt.Synapse` to define custom synapse types.

### Construction

```python
syn = bt.Synapse(name="TextPrompt", timeout=12.0)
```

| Parameter | Default | Description |
|---|---|---|
| `name` | `"Synapse"` | Route name (used as URL path) |
| `timeout` | `12.0` | Query timeout in seconds |

### Properties (gettable and settable)

| Property | Type | Description |
|---|---|---|
| `name` | `str` | Synapse route name |
| `timeout` | `float` | Query timeout (seconds) |
| `dendrite` | `TerminalInfo` | Dendrite-side endpoint metadata |
| `axon` | `TerminalInfo` | Axon-side endpoint metadata |
| `computed_body_hash` | `str` | SHA3-256 hash of request body |
| `total_size` | `int` | Total request body size in bytes |
| `header_size` | `int` | Header size in bytes |

### Methods

#### `Synapse.body_hash(body)` (staticmethod)

Compute SHA3-256 hex digest of the body bytes.

```python
h = bt.Synapse.body_hash(b'{"prompt": "hello"}')
print(h)  # 64-character hex string
```

#### `to_headers()`

Serialize synapse metadata into an HTTP header dictionary.

```python
headers = syn.to_headers()
```

#### `Synapse.from_headers(headers)` (classmethod)

Reconstruct a Synapse from HTTP response headers.

```python
syn = bt.Synapse.from_headers(response_headers)
```

### Subclassing

```python
class TextPrompt(bt.Synapse):
    def __init__(self, prompt="", completion="", *args, **kwargs):
        super().__init__(name="TextPrompt", *args, **kwargs)
        self.prompt = prompt
        self.completion = completion

syn = TextPrompt(prompt="What is TAO?")
```

---

## StreamingSynapse

SSE streaming variant of Synapse. Subclass and override `process_chunk` to define chunk parsing behavior.

### Construction

```python
ss = bt.StreamingSynapse(name="StreamingTextPrompt", timeout=60.0)
```

### Methods

#### `process_chunk(chunk)`

Process a single SSE data chunk. Override in subclasses. Default implementation decodes UTF-8.

```python
class StreamText(bt.StreamingSynapse):
    def process_chunk(self, chunk: bytes) -> str:
        return chunk.decode("utf-8")
```

#### `to_headers()` / `from_headers(headers)`

Same behavior as `Synapse`.

---

## AxonConfig

Configuration for the Axon HTTP server.

### Construction

```python
config = bt.AxonConfig(
    ip="0.0.0.0",
    port=8090,
    max_connections=0,        # 0 = unlimited
    external_ip="1.2.3.4",   # None = auto-detect
    hotkey="5CzR..."          # None = no hotkey binding
)
```

| Parameter | Default | Description |
|---|---|---|
| `ip` | `"0.0.0.0"` | Bind address |
| `port` | `8090` | Listen port |
| `max_connections` | `0` | Max concurrent connections (0 = unlimited) |
| `external_ip` | `None` | Advertised external IP |
| `hotkey` | `None` | Hotkey for request verification |

### Properties (gettable and settable)

All constructor parameters are exposed as properties with getters and setters.

---

## Axon

Neuron HTTP server built on Axum with middleware chain for blacklist enforcement and priority routing.

### Construction

```python
axon = bt.Axon(config)
```

If `config` is omitted, defaults to `AxonConfig()`.

### Methods

#### `attach(synapse_type, handler)`

Register a Python callable as a handler for a synapse route. The handler receives a `dict` of the parsed JSON body and must return a `dict` or `str`.

```python
def handle_prompt(body: dict) -> dict:
    prompt = body.get("prompt", "")
    return {"completion": f"Echo: {prompt}"}

axon.attach("TextPrompt", handle_prompt)
```

#### `start()`

Start the Axon server. Returns a coroutine that resolves to the bound address string.

```python
addr = await axon.start()
print(f"Axon listening on {addr}")
```

#### `stop(addr)`

Stop the Axon server at the given address.

```python
axon.stop(addr)
```

#### `blacklist(key)` / `unblacklist(key)`

Manage the hotkey blacklist. Blacklisted hotkeys receive `403 Forbidden`.

```python
await axon.blacklist("5BadActor...")
await axon.unblacklist("5Reformed...")
```

#### `set_priority(key, priority)`

Set the request priority for a hotkey (higher = served first).

```python
await axon.set_priority("5VIPClient...", 10)
```

---

## DendriteConfig

Configuration for the Dendrite HTTP client.

### Construction

```python
config = bt.DendriteConfig(
    timeout_secs=12,
    max_connections=100,
    hotkey_seed="0x0000...0001"  # None = unsigned requests
)
```

| Parameter | Default | Description |
|---|---|---|
| `timeout_secs` | `12` | Request timeout in seconds |
| `max_connections` | `100` | Max idle connections per host |
| `hotkey_seed` | `None` | Hex-encoded 32-byte secret key for signing |

### Properties (gettable and settable)

All constructor parameters are exposed as properties with getters and setters.

---

## Dendrite

HTTP client for sending signed Synapse requests to Axons.

### Construction

```python
dendrite = bt.Dendrite(config)
```

If `config` is omitted, defaults to `DendriteConfig()`.

### Methods

#### `query(synapse, axon_info)`

Send a signed synapse query to an Axon. Returns the synapse with response metadata populated.

```python
syn = bt.Synapse(name="TextPrompt")
result = await dendrite.query(syn, axon_info)
print(result.axon.status_code)
```

Rust equivalent:

```rust
let response = dendrite.query(&synapse, &axon_info).await?;
```

#### `forward(synapse, axon_info)`

Alias for `query`.

#### `call(synapse, axon_info)`

Alias for `query`.

#### `call_stream(synapse, axon_info)`

Send a signed synapse request and return an async generator yielding SSE data chunks as `str`.

```python
syn = bt.StreamingSynapse(name="StreamingTextPrompt")
async for chunk in dendrite.call_stream(syn, axon_info):
    print(chunk, end="", flush=True)
```

---

## Metagraph

Subnet neural graph with chain sync, serialization, and neuron access.

### Construction

```python
mg = bt.Metagraph(network="finney", netuid=1)
```

| Parameter | Default | Description |
|---|---|---|
| `network` | `"finney"` | Network name (`"finney"`, `"test"`, `"local"`, `"archive"`, `"latent-lite"`) |
| `netuid` | `1` | Subnet identifier |

### Methods

#### `sync()`

Fetch all neuron data from the chain and populate columnar fields. Must be called before accessing neurons.

```python
await mg.sync()
```

#### `save(path)`

Serialize the metagraph to a JSON file.

```python
mg.save("~/metagraph_1.json")
```

#### `Metagraph.load(path)` (staticmethod)

Deserialize a metagraph from a JSON file.

```python
mg = bt.Metagraph.load("~/metagraph_1.json")
```

#### `neurons()`

Return a list of dicts, each containing: `uid`, `netuid`, `active`, `hotkey`, `coldkey`, `stake`, `rank`, `trust`, `consensus`, `incentive`, `dividend`, `emission`, `validator_trust`.

```python
for n in mg.neurons():
    print(f"UID {n['uid']}: stake={n['stake']} TAO, incentive={n['incentive']}")
```

### Properties

| Property | Type | Description |
|---|---|---|
| `netuid` | `int` | Subnet identifier (requires sync) |
| `block` | `int` | Block number at sync time (requires sync) |

### Indexing

Access a neuron by positional index:

```python
neuron = mg[0]  # Returns dict for neuron at position 0
```

### Length

```python
len(mg)  # Number of neurons (requires sync)
```

---

## BittensorError

Exception type raised by SDK operations. Subclasses Python's built-in `RuntimeError`.

```python
try:
    result = await client.transfer(dest, amount, signer)
except bt.BittensorError as e:
    print(f"Transfer failed: {e}")
```

All methods in the SDK return Result types internally and never panic. Errors are always propagated as `BittensorError` exceptions to Python code.

---

## Feature-Gated Classes

Some Python classes are only available when the corresponding Cargo feature is enabled during the wheel build.

### DrandBeacon (drand feature)

DRAND randomness beacon client with BLS12-381 signature verification. Fetches and verifies DRAND rounds from the Quicknet HTTP API and caches recent rounds in an LRU cache.

```python
# Only available if the wheel was built with --features drand
beacon = bt.DrandBeacon()
round_info = await beacon.get_round(123)
valid = beacon.verify(round_info)
print(beacon.chain_hash)  # mainnet chain hash (64 hex chars)
```

| Method | Returns | Description |
|---|---|---|
| `get_latest()` | `dict` (round, randomness, signature) | Fetch and verify the latest DRAND round |
| `get_round(n)` | `dict` (round, randomness, signature) | Fetch and verify a specific round |
| `verify(round_dict)` | `bool` | Verify the BLS12-381 signature of a round |
| `chain_hash` | `str` | The chain hash this beacon is configured for |

### MevShield (mev-shield feature)

Post-quantum encrypted extrinsic submission using ML-KEM-768. Encrypts extrinsic payloads with an on-chain ML-KEM-768 public key and formats them for submission via `submit_encrypted_extrinsic`.

```python
# Only available if the wheel was built with --features mev-shield
shield = bt.MevShield()
# encrypt_extrinsic and submit_encrypted require chain client context
# See SubtensorClient for integrated MEV-shield transaction submission
print(repr(shield))  # "MevShield()"
```

| Method | Returns | Description |
|---|---|---|
| `encrypt_extrinsic(extrinsic_hex, password)` | `dict` | Encrypt an extrinsic payload (requires on-chain NextKey) |
| `submit_encrypted(encrypted_hex)` | `None` | Submit an encrypted payload (requires chain client) |

---

## Error Handling

All methods return Result types internally and never panic. On the Python side, any failure is raised as a `BittensorError` exception. This applies to:

- Connection failures (WebSocket handshake, DNS resolution)
- RPC errors (chain node returns an error)
- Extrinsic failures (transaction rejected or not finalized)
- Validation errors (invalid SS58 address, malformed hex seed)
- Timeout errors (request exceeded configured timeout)
- Wallet errors (decryption failure, file I/O)

```python
try:
    balance = await client.get_balance("invalid_address")
except bt.BittensorError:
    print("Could not fetch balance")
```

---

## Code Examples

### Connect and Query Balance

```python
import asyncio
import bittensor_rs as bt

async def main():
    client = await bt.SubtensorClient.connect(bt.NetworkConfig.finney())
    balance = await client.get_balance("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")
    print(f"Balance: {balance.tao} TAO ({balance.rao} rao)")

asyncio.run(main())
```

### Transfer TAO

```python
import asyncio
import bittensor_rs as bt

async def main():
    client = await bt.SubtensorClient.connect(bt.NetworkConfig.finney())
    result = await client.transfer(
        dest="5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        amount=1_000_000_000,  # 1 TAO in RAO
        signer="your twelve word mnemonic phrase goes here",
    )
    print(f"Transfer included in block {result.block_hash}")

asyncio.run(main())
```

### Create a Wallet

```python
import bittensor_rs as bt

wallet = bt.Wallet.create("validator", "~/.bittensor/wallets", password="secret")
print(f"Coldkey: {wallet.ss58_address}")
print(f"Hotkey: {wallet.get_hotkey_pair()}")
```

### Staking and Unstaking

```python
import asyncio
import bittensor_rs as bt

async def main():
    client = await bt.SubtensorClient.connect(bt.NetworkConfig.finney())

    # Stake 5 TAO to a hotkey on subnet 1
    result = await client.add_stake(
        hotkey="5CzR6NjA5V6Nq2k6U6iU8V6L2r2F2p2n2v2b2m2s2t2u2w2y",
        netuid=1,
        amount=5_000_000_000,
        signer="word1 word2 word3 ... word12",
    )
    print(f"Staked in block {result.block_hash}")

    # Unstake 2 TAO
    result = await client.remove_stake(
        hotkey="5CzR6NjA5V6Nq2k6U6iU8V6L2r2F2p2n2v2b2m2s2t2u2w2y",
        netuid=1,
        amount=2_000_000_000,
        signer="word1 word2 word3 ... word12",
    )

asyncio.run(main())
```

### Query the Metagraph

```python
import asyncio
import bittensor_rs as bt

async def main():
    mg = bt.Metagraph(network="finney", netuid=1)
    await mg.sync()
    print(f"Subnet 1 has {len(mg)} neurons at block {mg.block}")

    for neuron in mg.neurons()[:5]:
        print(f"  UID {neuron['uid']}: stake={neuron['stake']:.4f}, incentive={neuron['incentive']}")

    # Save for offline analysis
    mg.save("metagraph_1.json")

asyncio.run(main())
```

---

## When to Use Python Bindings vs Rust SDK Directly

### Use Python bindings when:

- Your existing codebase is in Python
- You need integration with Python ML frameworks (PyTorch, transformers)
- You want a gradual migration path from the Python bittensor SDK
- Rapid prototyping and scripting are priorities
- You need Jupyter notebook interactivity

### Use the Rust SDK directly when:

- You need maximum throughput for high-frequency chain operations
- Building production validator/miner servers with strict latency requirements
- Running in WASM or embedded environments where a Python runtime is unavailable
- You want compile-time type checking and zero-cost abstractions
- Memory usage must be minimal (no Python interpreter overhead)
- You need features not yet exposed in the Python bindings

---

## Performance: Python Bindings vs Pure Python SDK

The `bittensor_rs` Python bindings outperform the pure Python `bittensor` SDK because all compute-intensive operations run as compiled native code. The Python layer is a thin async wrapper around the Rust implementation.

| Operation | Python SDK | bittensor_rs |
|---|---|---|
| Balance query | ~200ms (substrate-interface) | ~50ms (subxt + native WS) |
| Transfer extrinsic | ~3s (sign + submit + finalize) | ~1.5s (native sr25519 signing) |
| Metagraph sync (256 neurons) | ~8s | ~2s |
| Wallet key generation | ~50ms (NaCl via PyNaCl) | ~5ms (native Ed25519/Sr25519) |
| Memory usage (idle client) | ~80MB (Python + substrate-interface) | ~15MB (native Rust, no GC) |

Key factors in the performance advantage:

- **subxt vs substrate-interface**: The Rust chain client uses subxt 0.50 with compile-time metadata bindings, eliminating runtime scale decoding overhead.
- **Native sr25519 signing**: Signature operations run in compiled Rust without Python GIL contention.
- **No GIL for async operations**: The tokio runtime handles all async I/O outside the Python GIL, allowing true concurrent chain queries.
- **Compact memory**: No Python GC pressure, no substrate-interface object graph overhead.
