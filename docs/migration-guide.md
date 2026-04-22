# Migration Guide: Python bittensor SDK to bittensor-rs

This guide covers migrating from the Python `bittensor` SDK to the Rust `bittensor-rs` SDK. It includes a quick mapping table, key differences, before/after code examples for common operations, and guidance on choosing between native Rust and the Python bindings.

## Quick Mapping Table

| Python SDK | Rust SDK | Notes |
|---|---|---|
| `bittensor.subtensor(ws_url)` | `SubtensorClient::from_url(url).await?` | Rust is async everywhere |
| `bittensor.subtensor(network="finney")` | `SubtensorClient::from_config(NetworkConfig::finney()).await?` | Failover to archive included |
| `subtensor.get_balance(addr)` | `bittensor_chain::queries::account::get_balance(client.rpc(), &account_id).await?` | Returns `Balance`, not dict |
| `subtensor.get_stake(coldkey, hotkey, netuid)` | `bittensor_chain::queries::account::get_stake(client.rpc(), &coldkey, &hotkey, netuid).await?` | Same signature, typed return |
| `subtensor.transfer(wallet, dest, amount)` | `bittensor_chain::extrinsics::transfer::transfer(client.rpc(), &signer, dest, amount_rao).await?` | Amount in rao (u64) |
| `subtensor.add_stake(wallet, hotkey, amount)` | `bittensor_chain::extrinsics::staking::add_stake(client.rpc(), &signer, hotkey, netuid, amount).await?` | Requires explicit netuid |
| `subtensor.remove_stake(wallet, hotkey, amount)` | `bittensor_chain::extrinsics::staking::remove_stake(client.rpc(), &signer, hotkey, netuid, amount).await?` | Requires explicit netuid |
| `subtensor.burned_register(wallet, netuid)` | `bittensor_chain::extrinsics::registration::burned_register(client.rpc(), &signer, netuid, hotkey).await?` | POW register is separate |
| `subtensor.set_weights(wallet, netuid, uids, weights)` | `bittensor_chain::extrinsics::weights::set_weights(client.rpc(), &signer, netuid, dests, weights, version).await?` | version_key is explicit u64 |
| `bittensor.wallet(name, path)` | `Wallet::new(name)` or `Wallet::with_path(name, path)` | Lazy key loading |
| `wallet.coldkeypub.ss58_address` | `wallet.get_coldkeypub()?.ss58_address()` | Must call getter method |
| `wallet.hotkey.ss58_address` | `wallet.get_hotkey_pair()?.ss58_address()` | Must call getter method |
| `bittensor.Balance.from_tao(1.0)` | `Balance::from_tao(1.0)` | Same concept, different syntax |
| `bittensor.Balance.from_rao(10**9)` | `Balance::from_rao(1_000_000_000)` | Same concept, different syntax |
| `bittensor.AxonInfo(...)` | `bittensor_core::types::AxonInfo { ... }` | Struct literal, not keyword args |
| `bittensor.metagraph(netuid, network)` | `bittensor_metagraph::prelude::Metagraph::new(netuid, network)` | Separate crate |
| `bittensor.Synapse(...)` | `bittensor_synapse::prelude::Synapse` | Not subclassable in Rust |
| `bittensor.axon(wallet)` | `bittensor_axon::prelude::Axon::new(client, wallet)` | Requires chain client |
| `bittensor.dendrite(wallet)` | `bittensor_dendrite::prelude::Dendrite::new(wallet)` | Separate crate |

## Key Differences

### Async everywhere

The Python SDK uses synchronous calls for most chain operations, with some async support. The Rust SDK is fully async. Every chain query and extrinsic returns a `Future` that must be `.await`ed inside a `tokio` runtime.

```python
# Python: synchronous
balance = subtensor.get_balance(address)
```

```rust
// Rust: async
let balance = bittensor_chain::queries::account::get_balance(
    client.rpc(), &account_id
).await?;
```

### Result types instead of exceptions

The Python SDK raises exceptions on failure. The Rust SDK returns `Result<T, BittensorError>`. You must handle errors explicitly with `?`, `match`, or combinators.

```python
# Python: exception-based
try:
    subtensor.transfer(wallet, dest, amount)
except Exception as e:
    print(f"Failed: {e}")
```

```rust
// Rust: Result-based
match bittensor_chain::extrinsics::transfer::transfer(
    client.rpc(), &signer, dest_id, amount_rao
).await {
    Ok(tx) => println!("Block: {:?}", tx.block_hash),
    Err(e) => eprintln!("Failed: {e}"),
}
```

### RAO vs TAO: explicit units

The Python SDK sometimes accepts TAO as a float. The Rust SDK chain operations always take rao (u64). Use `Balance` for conversions.

```python
# Python: TAO float
subtensor.transfer(wallet, dest, bittensor.Balance.from_tao(1.5))
```

```rust
// Rust: rao u64
let amount = Balance::from_tao(1.5).to_rao();
bittensor_chain::extrinsics::transfer::transfer(
    client.rpc(), &signer, dest_id, amount
).await?;
```

### No global state

The Python SDK stores a global `bittensor` config and logging state. The Rust SDK has no global mutable state. You create and pass clients, wallets, and configs explicitly.

```python
# Python: global config
bittensor.logging(debug=True)
subtensor = bittensor.subtensor()
```

```rust
// Rust: explicit local state
let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
```

### Type-safe metadata, no dicts

The Python SDK returns dicts or dynamic objects for chain data. The Rust SDK returns typed structs with named fields, generated from chain metadata at compile time.

```python
# Python: dict access
neuron = subtensor.get_neuron_for_pubkey_and_subnet(hotkey, netuid)
stake = neuron.stake
```

```rust
// Rust: typed field access
let neuron = bittensor_chain::queries::neuron::get_neuron(
    client.rpc(), &hotkey_id, netuid
).await?;
let stake = neuron.stake;
```

### SS58 addresses

The Python SDK accepts both SS58 strings and `AccountId` objects. The Rust SDK uses `subxt::utils::AccountId32` for chain calls and provides SS58 encode/decode in `bittensor_wallet::ss58`.

```python
# Python: string address
addr = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
```

```rust
// Rust: AccountId32 for chain, SS58 string for display
use bittensor_wallet::prelude::*;
let addr = encode_ss58_address(&public_key, 42); // SS58 prefix 42
let account_id: subxt::utils::AccountId32 = public_key.into();
```

### Keyfile format: cross-compatible

Both SDKs use the same NaCl secretbox encrypted keyfile format. Wallets created by the Python SDK can be loaded by the Rust SDK and vice versa. The directory layout is identical:

```
~/.bittensor/wallets/<name>/
  coldkey        (encrypted NaCl)
  coldkeypub     (plaintext SS58 address)
  hotkeys/
    <hotkey_name>  (raw hex seed, unencrypted)
```

### Signing: Keypair vs wallet

The Python SDK signs through the `wallet` object. The Rust SDK accepts a `subxt_signer::sr25519::Keypair` directly in extrinsic calls. You can extract a keypair from the wallet or construct one from a mnemonic or URI.

```python
# Python: wallet signs internally
subtensor.transfer(wallet, dest, amount)
```

```rust
// Rust: signer passed explicitly
let signer = subxt_signer::sr25519::Keypair::from_uri("//Alice")?;
bittensor_chain::extrinsics::transfer::transfer(
    client.rpc(), &signer, dest_id, amount_rao
).await?;

// Or from wallet coldkey (requires password)
let coldkey_kp = wallet.get_coldkey_pair("secret")?;
let signer = subxt_signer::sr25519::Keypair::from_seed(coldkey_kp.seed());
```

## Migration Patterns

### Connecting to the network

```python
# Python
import bittensor

subtensor = bittensor.subtensor(network="finney")
print(f"Connected: {subtensor.block}")
```

```rust
// Rust
use bittensor_chain::prelude::*;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
    let block = client.at_current_block().await?;
    println!("Connected at block {:?}", block.block_hash());
    Ok(())
}
```

### Querying balance

```python
# Python
import bittensor

subtensor = bittensor.subtensor(network="finney")
balance = subtensor.get_balance("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")
print(f"Free: {balance}")
```

```rust
// Rust
use bittensor_chain::prelude::*;
use bittensor_core::config::NetworkConfig;
use bittensor_core::balance::Balance;
use subxt::utils::AccountId32;

let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

// Convert SS58 to AccountId32
let account_id: AccountId32 = bittensor_wallet::prelude::decode_ss58(
    "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
)?.try_into()?;

let balance: Balance = bittensor_chain::queries::account::get_balance(
    client.rpc(), &account_id
).await?;
println!("Free: {balance}");
```

### Transferring TAO

```python
# Python
import bittensor

wallet = bittensor.wallet(name="default")
subtensor = bittensor.subtensor(network="finney")

result = subtensor.transfer(
    wallet,
    "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
    bittensor.Balance.from_tao(1.0),
)
print(f"Success: {result}")
```

```rust
// Rust
use bittensor_chain::prelude::*;
use bittensor_core::config::NetworkConfig;
use bittensor_core::balance::Balance;
use bittensor_wallet::prelude::*;

let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
let mut wallet = Wallet::new("default");
let coldkey = wallet.get_coldkey_pair("secret")?;
let signer = subxt_signer::sr25519::Keypair::from_seed(coldkey.seed());

let dest_id: subxt::utils::AccountId32 =
    bittensor_wallet::prelude::decode_ss58(
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty"
    )?.try_into()?;

let amount_rao = Balance::from_tao(1.0).to_rao();
let tx = bittensor_chain::extrinsics::transfer::transfer(
    client.rpc(), &signer, dest_id, amount_rao
).await?;
println!("Success: block={:?}", tx.block_hash);
```

### Staking and unstaking

```python
# Python
import bittensor

wallet = bittensor.wallet(name="default")
subtensor = bittensor.subtensor(network="finney")

# Stake 5 TAO
subtensor.add_stake(
    wallet,
    hotkey_ss58="5CzR...",
    amount=bittensor.Balance.from_tao(5.0),
)

# Unstake 2 TAO
subtensor.remove_stake(
    wallet,
    hotkey_ss58="5CzR...",
    amount=bittensor.Balance.from_tao(2.0),
)
```

```rust
// Rust
use bittensor_chain::prelude::*;
use bittensor_core::config::NetworkConfig;
use bittensor_core::balance::Balance;
use bittensor_wallet::prelude::*;

let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
let mut wallet = Wallet::new("default");
let coldkey = wallet.get_coldkey_pair("secret")?;
let signer = subxt_signer::sr25519::Keypair::from_seed(coldkey.seed());

let hotkey_id: subxt::utils::AccountId32 =
    bittensor_wallet::prelude::decode_ss58("5CzR...")?.try_into()?;

// Stake 5 TAO on subnet 1
let stake_amount = Balance::from_tao(5.0).to_rao();
let tx = bittensor_chain::extrinsics::staking::add_stake(
    client.rpc(), &signer, hotkey_id.clone(), 1, stake_amount
).await?;

// Unstake 2 TAO from subnet 1
let unstake_amount = Balance::from_tao(2.0).to_rao();
let tx = bittensor_chain::extrinsics::staking::remove_stake(
    client.rpc(), &signer, hotkey_id, 1, unstake_amount
).await?;
```

### Registering on a subnet

```python
# Python
import bittensor

wallet = bittensor.wallet(name="default")
subtensor = bittensor.subtensor(network="finney")

result = subtensor.burned_register(
    wallet,
    netuid=1,
)
```

```rust
// Rust
use bittensor_chain::prelude::*;
use bittensor_core::config::NetworkConfig;
use bittensor_wallet::prelude::*;

let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
let mut wallet = Wallet::new("default");
let coldkey = wallet.get_coldkey_pair("secret")?;
let signer = subxt_signer::sr25519::Keypair::from_seed(coldkey.seed());

let hotkey_id: subxt::utils::AccountId32 =
    bittensor_wallet::prelude::decode_ss58(&wallet.get_hotkey_pair()?.ss58_address()?)?.try_into()?;

let tx = bittensor_chain::extrinsics::registration::burned_register(
    client.rpc(), &signer, 1, hotkey_id
).await?;
println!("Registered in block {:?}", tx.block_hash);
```

### Setting weights

```python
# Python
import bittensor

wallet = bittensor.wallet(name="default")
subtensor = bittensor.subtensor(network="finney")

uids = [0, 1, 2]
weights = [0.3, 0.5, 0.2]

result = subtensor.set_weights(
    wallet,
    netuid=1,
    uids=uids,
    weights=weights,
)
```

```rust
// Rust
use bittensor_chain::prelude::*;
use bittensor_core::config::NetworkConfig;
use bittensor_wallet::prelude::*;

let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
let mut wallet = Wallet::new("default");
let coldkey = wallet.get_coldkey_pair("secret")?;
let signer = subxt_signer::sr25519::Keypair::from_seed(coldkey.seed());

let dests: Vec<u16> = vec![0, 1, 2];
// Weights on-chain are u16. Scale by 65535 if the Python SDK would use 0.0--1.0.
// Exact scaling depends on subnet parameters. Common pattern:
let weights: Vec<u16> = vec![19660, 32767, 13107]; // ~0.3, 0.5, 0.2 of u16::MAX
let version_key: u64 = 1;

let tx = bittensor_chain::extrinsics::weights::set_weights(
    client.rpc(), &signer, 1, dests, weights, version_key
).await?;
```

### Creating a wallet

```python
# Python
import bittensor

wallet = bittensor.wallet.create("miner", "~/.bittensor/wallets", password="secret")
print(f"Coldkey: {wallet.coldkeypub.ss58_address}")
print(f"Hotkey: {wallet.hotkey.ss58_address}")
```

```rust
// Rust
use bittensor_wallet::prelude::*;

let mut wallet = Wallet::with_path("miner", std::path::PathBuf::from("~/.bittensor/wallets"));
let mnemonic = wallet.create_coldkey("secret")?;
println!("Back up this mnemonic: {mnemonic}");
wallet.create_hotkey()?;

println!("Coldkey: {}", wallet.get_coldkeypub()?);
println!("Hotkey: {}", wallet.get_hotkey_pair()?.ss58_address()?);
```

### Running an Axon

```python
# Python
import bittensor

wallet = bittensor.wallet(name="default")
subtensor = bittensor.subtensor(network="finney")
axon = bittensor.axon(wallet=wallet, port=8090)

@axon.forward(text_prompt)
def handle_prompt(synapse):
    synapse.completion = "Hello from Rust SDK"
    return synapse

axon.start()
```

```rust
// Rust
use bittensor_axon::prelude::*;
use bittensor_chain::prelude::*;
use bittensor_core::config::NetworkConfig;
use bittensor_wallet::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
    let mut wallet = Wallet::new("default");

    let axon = Axon::new(client, wallet)
        .port(8090)
        .serve().await?;

    println!("Axon listening on {}", axon.external_ip());
    axon.wait_shutdown().await?;
    Ok(())
}
```

### Querying the metagraph

```python
# Python
import bittensor

subtensor = bittensor.subtensor(network="finney")
metagraph = bittensor.metagraph(netuid=1, network="finney")
metagraph.sync()
print(f"Subnet 1: {metagraph.n.item()} neurons at block {metagraph.block.item()}")

for uid in range(metagraph.n.item()):
    print(f"  UID {uid}: stake={metagraph.S[uid]}, incentive={metagraph.I[uid]}")
```

```rust
// Rust
use bittensor_chain::prelude::*;
use bittensor_core::config::NetworkConfig;
use bittensor_core::balance::Balance;

let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
let meta = bittensor_chain::queries::metagraph::get_metagraph(
    client.rpc(), 1
).await?;

println!("Subnet 1: {} neurons at block {}", meta.n, meta.block);
for neuron in &meta.neurons {
    println!(
        "  UID {}: stake={}, incentive={}",
        neuron.uid,
        Balance::from_rao(neuron.stake),
        neuron.incentive
    );
}
```

### Getting stake info

```python
# Python
import bittensor

subtensor = bittensor.subtensor(network="finney")
stakes = subtensor.get_stake_info_for_coldkey("5Grw...")

for s in stakes:
    print(f"Hotkey: {s.hotkey_ss58}, Stake: {s.stake.tao} TAO")
```

```rust
// Rust
use bittensor_chain::prelude::*;
use bittensor_core::config::NetworkConfig;
use subxt::utils::AccountId32;

let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
let coldkey_id: AccountId32 =
    bittensor_wallet::prelude::decode_ss58("5Grw...")?.try_into()?;

let stakes = bittensor_chain::queries::account::get_stake_info_for_coldkey(
    client.rpc(), &coldkey_id
).await?;

for s in &stakes {
    println!("Hotkey: {}, Stake: {} TAO", s.hotkey, Balance::from_rao(s.stake));
}
```

## Python Bindings Alternative

If a full rewrite to Rust is not feasible, the `bittensor-pyo3` crate provides Python bindings via PyO3. The published package is `bittensor-rs` on PyPI (hyphen in the package name, underscore in the import):

```bash
pip install bittensor-rs
```

```python
import asyncio
import bittensor_rs as bt

async def main():
    client = await bt.SubtensorClient.connect(bt.NetworkConfig.finney())
    balance = await client.get_balance("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")
    print(f"Balance: {balance.tao} TAO")

asyncio.run(main())
```

The Python bindings expose the same classes as the Rust SDK: `SubtensorClient`, `Wallet`, `Balance`, `Axon`, `Dendrite`, `Metagraph`, `Synapse`, `StreamingSynapse`. See [Python Bindings](python-bindings.md) for full API documentation.

### When to use Python bindings vs Rust directly

| Criterion | Python bindings | Rust SDK directly |
|---|---|---|
| Existing Python codebase | Yes | No |
| ML framework integration (PyTorch) | Yes | No |
| Gradual migration from Python SDK | Yes | No |
| Rapid prototyping and Jupyter | Yes | No |
| Maximum throughput | No | Yes |
| Production validator/miner latency | No | Yes |
| WASM or embedded targets | No | Yes |
| Compile-time type safety | No | Yes |
| Minimal memory footprint | No | Yes |

## Common Pitfalls

### Forgetting to await

In Rust, every chain call is a future. Calling a query without `.await` stores the future object, not the result. The compiler catches this with a warning, but it can be confusing at first.

```rust
// Wrong: stores the Future, never executes
let balance = bittensor_chain::queries::account::get_balance(client.rpc(), &id);

// Correct: awaits the Future
let balance = bittensor_chain::queries::account::get_balance(
    client.rpc(), &id
).await?;
```

### Using TAO where RAO is required

All extrinsic functions in the Rust SDK accept amounts in rao (u64). Passing a TAO float directly is a type error.

```rust
// Wrong: TAO float where u64 expected (will not compile)
bittensor_chain::extrinsics::transfer::transfer(
    client.rpc(), &signer, &dest, 1.5
).await?;

// Correct: convert to rao first
let amount_rao = Balance::from_tao(1.5).to_rao();
bittensor_chain::extrinsics::transfer::transfer(
    client.rpc(), &signer, &dest, amount_rao
).await?;
```

### Not handling errors

The `?` operator propagates errors up the call stack. If you ignore the `Result`, your code will not compile. This is intentional: the Rust SDK forces explicit error handling.

```rust
// This will not compile - Result must be used
let balance = bittensor_chain::queries::account::get_balance(
    client.rpc(), &id
).await;

// Correct: propagate with ?
let balance = bittensor_chain::queries::account::get_balance(
    client.rpc(), &id
).await?;

// Or handle explicitly
match bittensor_chain::queries::account::get_balance(
    client.rpc(), &id
).await {
    Ok(b) => println!("Balance: {b}"),
    Err(e) => eprintln!("Query failed: {e}"),
}
```

### SS58 address encoding

The Python SDK accepts SS58 strings directly in most functions. The Rust SDK chain queries need `AccountId32`. Use `decode_ss58` from the wallet crate to convert.

```rust
use bittensor_wallet::prelude::*;

let (public_key, prefix) = decode_ss58("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")?;
let account_id: subxt::utils::AccountId32 = public_key.into();
```

### Netuid is required for staking

The Python SDK sometimes infers the netuid or defaults to 0. The Rust SDK requires an explicit netuid parameter in all staking extrinsics.

```rust
// Python: subtensor.add_stake(wallet, hotkey, amount)
// Rust: must specify netuid
bittensor_chain::extrinsics::staking::add_stake(
    client.rpc(), &signer, hotkey_id, 1, // netuid = 1
    amount_rao
).await?;
```

### Weight integer scaling

The Python SDK accepts float weights in the 0.0 to 1.0 range and auto-scales them. The Rust SDK takes `Vec<u16>` weights. The on-chain representation is u16. You must scale manually: multiply by 65535 and round, or use whatever scale the subnet expects.

```rust
// Python: weights = [0.3, 0.5, 0.2]
// Rust: scale to u16 range
let weights: Vec<u16> = vec![19660, 32767, 13107]; // approx 0.3, 0.5, 0.2 of u16::MAX
```

## Crate Dependency Reference

Add the crates you need to your `Cargo.toml`:

```toml
[dependencies]
bittensor-core = "0.1"   # Balance, NetworkConfig, BittensorError, types
bittensor-chain = "0.1"  # SubtensorClient, queries, extrinsics, events
bittensor-wallet = "0.1" # Wallet, Keypair, SS58, keyfile
bittensor-synapse = "0.1" # Synapse, TerminalInfo, headers, hashing
bittensor-axon = "0.1"   # Axon server, middleware, routing
bittensor-dendrite = "0.1" # Dendrite client, request signing
bittensor-metagraph = "0.1" # Metagraph sync, serialization
subxt = "0.50"            # Low-level chain access (optional)
subxt-signer = "0.50"     # sr25519 Keypair for signing extrinsics
tokio = { version = "1", features = ["full"] } # Async runtime
```

Most applications only need `bittensor-core`, `bittensor-chain`, and `bittensor-wallet`. The axon, dendrite, metagraph, and synapse crates are for neuron operators.

### Import patterns

```rust
// Common imports for chain operations
use bittensor_chain::prelude::*;
use bittensor_core::config::NetworkConfig;
use bittensor_core::balance::Balance;
use bittensor_core::error::BittensorError;
use bittensor_wallet::prelude::*;

// Axon server
use bittensor_axon::prelude::*;

// Dendrite client
use bittensor_dendrite::prelude::*;

// Metagraph
use bittensor_metagraph::prelude::*;

// Synapse types
use bittensor_synapse::prelude::*;
```

## Network Endpoints

| Network | Python SDK | Rust SDK |
|---|---|---|
| Finney | `bittensor.subtensor(network="finney")` | `NetworkConfig::finney()` |
| Test | `bittensor.subtensor(network="test")` | `NetworkConfig::test()` |
| Local | `bittensor.subtensor(network="local")` | `NetworkConfig::local()` |
| Archive | `bittensor.subtensor(network="archive")` | `NetworkConfig::archive()` |
| Custom URL | `bittensor.subtensor("wss://...")` | `SubtensorClient::from_url("wss://...").await?` |

The Rust `SubtensorClient::from_config` includes automatic failover: if the primary WebSocket endpoint is unreachable, it falls back to the archive endpoint (if configured). The Python SDK does not have built-in failover.

## Migration Checklist

1. Replace `import bittensor as bt` with `import bittensor_rs as bt` for Python bindings, or add Rust crate dependencies for native code.
2. Replace `bt.subtensor()` with `SubtensorClient::from_config()` or `bt.SubtensorClient.connect()`.
3. Replace float weight values (0.0--1.0) with integer weight values (0--65535).
4. Replace `wallet` objects in extrinsic calls with explicit `signer` parameters (hex seed or mnemonic).
5. Replace `bool` return values from extrinsics with `TxSuccess` objects containing `block_hash` and `extrinsic_hash`.
6. Replace `bt.metagraph().sync()` with the Rust sync pattern (returns the metagraph directly).
7. Update error handling from try/except to Result-based patterns or `bt.BittensorError`.
8. Replace `btcli` with `btcli-rs` in shell scripts.
9. Keyfiles are cross-compatible. No migration needed for wallet storage.
10. Replace `bt.axon(wallet=w)` with explicit `AxonConfig` construction.
11. Add explicit `netuid` parameters to all staking extrinsics.
12. Convert SS58 addresses to `AccountId32` before passing to chain queries.
