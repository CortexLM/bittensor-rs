# Frequently Asked Questions

Common questions about the bittensor-rs SDK, organized by topic.

## General

### What is bittensor-rs?

bittensor-rs is a Rust SDK for the Bittensor decentralized AI network. It provides wallet management, chain interaction, neuron serving (axon), neuron querying (dendrite), and subnet monitoring (metagraph). The SDK is split into multiple crates so you only pull in what you need.

It differs from the Python SDK in a few key ways:
- Compiled to a native binary instead of running through an interpreter
- Uses subxt 0.50 for typed chain communication instead of `substrate-interface`
- No GIL; full async concurrency with tokio
- SS58, SCALE, and keyfile formats are cross-compatible with the Python SDK

See [Architecture](architecture.md) for the full crate breakdown.

### What Rust version is required?

Rust 1.85 or later with Edition 2024. Verify with `rustc --version`. The SDK uses Edition 2024 features that are not available on older compilers.

```sh
rustc --version
# Must show 1.85.0 or later
```

### Which platforms are supported?

Linux (x86_64, aarch64), macOS (x86_64, Apple Silicon), and Windows (x86_64). The `bittensor-wasm` crate also targets WASM for browser usage via wasm-bindgen.

### Can I use the Rust SDK alongside the Python SDK?

Yes. Keyfiles use the same NaCl secretbox format, so a coldkey created with Python's `btcli` can be decrypted by bittensor-rs and vice versa. The wallet directory layout (`~/.bittensor/wallets/<name>/`) is identical. You can run both SDKs against the same Finney endpoints at the same time.

```rust
// Read a coldkey file created by Python btcli
use bittensor_wallet::keyfile;

let encrypted = std::fs::read("~/.bittensor/wallets/default/coldkey")?;
assert!(keyfile::is_encrypted_nacl(&encrypted));
let decrypted = keyfile::decrypt(&encrypted, b"my-password")?;
```

See [Wallet](wallet.md) for more on cross-compatibility.

## Chain Interaction

### How do I connect to the Bittensor network?

Use `SubtensorClient::from_config` with a `NetworkConfig` preset:

```rust
use bittensor_chain::prelude::SubtensorClient;
use bittensor_core::config::NetworkConfig;

let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
```

For custom endpoints, use `from_url`:

```rust
let client = SubtensorClient::from_url("wss://my-node.example.com:443").await?;
```

See [Getting Started](getting-started.md) for the full connection walkthrough.

### What are the network endpoints?

| Network | WebSocket URL | Constructor |
|---|---|---|
| Finney (mainnet) | `wss://entrypoint-finney.opentensor.ai:443` | `NetworkConfig::finney()` |
| Testnet | `wss://test.finney.opentensor.ai:443` | `NetworkConfig::test()` |
| Local | `ws://127.0.0.1:9944` | `NetworkConfig::local()` |
| Archive | `wss://archive.finney.opentensor.ai:443` | `NetworkConfig::archive()` |
| Latent Lite | `wss://lite.finney.opentensor.ai:443` | `NetworkConfig::latent_lite()` |

The archive config also enables failover: if the archive endpoint is unreachable, the client falls back to the primary endpoint.

### Why are all amounts in RAO? How do I convert?

The chain stores all balances and stakes as 64-bit integers in rao. Floating-point arithmetic on financial values causes rounding errors, so the SDK avoids it internally. Use the `Balance` type for conversion:

```rust
use bittensor_core::balance::Balance;

let amount = Balance::from_tao(5.0);
println!("RAO: {}", amount.to_rao());  // 5000000000
println!("TAO: {:.9}", amount.to_tao()); // 5.000000000
println!("Display: {amount}");          // 5.000000000
```

When passing amounts to extrinsics, convert to rao:

```rust
let amount_rao = Balance::from_tao(1.0).to_rao();
bittensor_chain::extrinsics::transfer::transfer(
    client.rpc(), &signer, &dest, amount_rao
).await?;
```

### How do I query balance, stake, or neurons?

All chain reads go through the `queries` module. Pass `client.rpc()` as the first argument:

```rust
use bittensor_chain::queries;
use bittensor_core::balance::Balance;

// Balance
let balance: Balance = queries::account::get_balance(client.rpc(), &account_id).await?;

// Total stake for a coldkey
let stakes: Vec<StakeInfo> = queries::stakes::get_stake_info_for_coldkey(client.rpc(), &coldkey_id).await?;

// All neurons in a subnet
let neurons: Vec<NeuronInfo> = queries::neurons::get_all_neurons(client.rpc(), netuid).await?;
```

For consistent reads across multiple queries, pin to a block:

```rust
let block = client.at_current_block().await?;
// All queries using block.storage() will read from this exact block
```

### How do I submit extrinsics (transfer, stake, register, set_weights)?

Extrinsics require a signing keypair. The `extrinsics` module builds the call, signs it, submits it, and waits for finalization:

```rust
use bittensor_chain::extrinsics;
use bittensor_core::balance::Balance;

// Transfer
let amount = Balance::from_tao(1.0).to_rao();
extrinsics::transfer::transfer(client.rpc(), &signer, &dest, amount).await?;

// Add stake
extrinsics::staking::add_stake(client.rpc(), &signer, &hotkey, netuid, amount).await?;

// Register (burned)
extrinsics::registration::register(client.rpc(), &signer, netuid).await?;

// Set weights
let uids = vec![0, 1, 2];
let weights = vec![30000, 20000, 10000]; // u16 values
extrinsics::weights::set_weights(client.rpc(), &signer, netuid, &uids, &weights).await?;
```

### What is TxSuccess and what does it contain?

`TxSuccess` is returned by every extrinsic function on success. It confirms that the transaction was included in a block and finalized:

```rust
pub struct TxSuccess {
    pub block_hash: subxt::utils::H256,
    pub extrinsic_hash: subxt::utils::H256,
}
```

Both hashes are 32 bytes, displayed as `0x`-prefixed hex strings (66 characters). In Python bindings, the same fields are exposed as `block_hash` and `extrinsic_hash` string properties.

If the extrinsic fails (insufficient balance, invalid state, etc.), the function returns `BittensorError::Transaction`.

## Wallet

### How do I create a wallet?

```rust
use bittensor_wallet::prelude::*;

let mut wallet = Wallet::new("my-wallet");

// Generate coldkey (encrypted). Back up the returned mnemonic.
let mnemonic = wallet.create_coldkey("strong-password")?;
println!("Mnemonic: {mnemonic}");

// Generate hotkey (unencrypted, no password needed).
let hotkey = wallet.create_hotkey()?;
println!("Hotkey: {}", hotkey.ss58_address());
```

See [Wallet](wallet.md) for the full API.

### Are keyfiles compatible with the Python SDK?

Yes. Both SDKs use NaCl secretbox with the same hardcoded salt, Argon2i key derivation parameters, and `$NACL` file format. A coldkey created with `btcli` can be decrypted by bittensor-rs, and a coldkey created with `btcli-rs` can be decrypted by Python's `btwallet`. Wrong passwords produce a `DecryptionFailed` error rather than garbled output because Poly1305 authentication detects corruption before returning plaintext.

### How do I recover a wallet from a mnemonic?

```rust
use bittensor_wallet::prelude::*;
use subxt_signer::bip39;

let phrase = "bottom drive obey lake curtain smoke basket hold race lonely fit walk";
let mnemonic = bip39::Mnemonic::parse(phrase)?;

let mut wallet = Wallet::new("recovered");
wallet.create_coldkey_from_mnemonic(&mnemonic, "my-password")?;

let address = wallet.get_coldkeypub()?;
println!("Recovered address: {address}");
```

### What is the difference between hotkey and coldkey?

| Property | Hotkey | Coldkey |
|---|---|---|
| Purpose | Signing messages, submitting transactions | Holding stake and funds |
| Storage | Raw hex seed, unencrypted | NaCl secretbox, encrypted |
| Password | Not required | Required to decrypt |
| Frequency | Used frequently for synapse signing | Used rarely for transfers and staking |
| Risk exposure | Higher (key is on disk unencrypted) | Lower (encrypted at rest) |

The recommended pattern is to keep the coldkey offline or in a hardware device, and only load it when you need to sign a transfer or staking transaction.

## Axon and Dendrite

### How do I run a neuron server (axon)?

```rust
use bittensor_axon::prelude::*;

let config = AxonConfig {
    port: 8091,
    hotkey: Some("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".to_string()),
    ..Default::default()
};

let mut axon = Axon::new(config)
    .attach("TextPrompt", text_handler)
    .attach("Embedding", embedding_handler);

let addr = axon.start().await?;
println!("Axon listening on {addr}");
```

The axon runs a middleware stack: verification, blacklist, priority, and body hash checks. You can manage the blacklist and priority map at runtime:

```rust
axon.blacklist("5BadActor").await;
axon.set_priority("5VIPClient", 10).await;
```

See [Axon](axon.md) for the full server API.

### How do I query other axons (dendrite)?

```rust
use bittensor_dendrite::prelude::*;
use bittensor_core::types::AxonInfo;
use subxt_signer::sr25519::dev::alice;

let config = DendriteConfig::new()
    .with_timeout_secs(30)
    .with_hotkey(alice());
let dendrite = Dendrite::new(config)?;

let axon = AxonInfo { ip: 2130706433, port: 8091, ip_type: 4, protocol: 0, version: 1, hotkey: "5Target".into(), coldkey: "5TargetCold".into() };

let response = dendrite.query(my_synapse, &axon).await?;
println!("Axon responded with status: {:?}", response.axon().status_code);
```

For streaming responses:

```rust
let chunk = dendrite.call_stream(streaming_synapse, &axon).await?;
```

See [Dendrite](dendrite.md) for the full client API.

### What is a Synapse and how do I implement one?

A synapse is the protocol type for neuron-to-neuron communication. To create one, implement the `Synapse` trait on a struct. The trait requires you to define the route name, timeout, body hash, and two `TerminalInfo` fields (dendrite and axon). It also requires an `Output` associated type for deserializing the response body.

```rust
use bittensor_synapse::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TextPrompt {
    name_val: String,
    timeout_val: f64,
    dendrite_info: TerminalInfo,
    axon_info: TerminalInfo,
    computed_hash: String,
    total_bytes: u64,
    header_bytes: u64,
    pub prompt: String,
    pub completion: String,
}

impl Synapse for TextPrompt {
    type Output = TextOutput;

    fn name(&self) -> &str { &self.name_val }
    fn timeout(&self) -> f64 { self.timeout_val }
    fn set_timeout(&mut self, t: f64) { self.timeout_val = t; }
    fn dendrite(&self) -> &TerminalInfo { &self.dendrite_info }
    fn set_dendrite(&mut self, info: TerminalInfo) { self.dendrite_info = info; }
    fn axon(&self) -> &TerminalInfo { &self.axon_info }
    fn set_axon(&mut self, info: TerminalInfo) { self.axon_info = info; }
    fn computed_body_hash(&self) -> &str { &self.computed_hash }
    fn set_computed_body_hash(&mut self, h: String) { self.computed_hash = h; }
    fn total_size(&self) -> u64 { self.total_bytes }
    fn set_total_size(&mut self, s: u64) { self.total_bytes = s; }
    fn header_size(&self) -> u64 { self.header_bytes }
    fn set_header_size(&mut self, s: u64) { self.header_bytes = s; }

    fn from_headers(headers: &HashMap<String, String>) -> Result<Self, SynapseError> {
        // reconstruct from HTTP headers
        todo!()
    }
}
```

For streaming, also implement `StreamingSynapse`:

```rust
impl StreamingSynapse for TextPrompt {
    type StreamItem = String;
    fn process_chunk(chunk: &[u8]) -> Result<String, SynapseError> {
        String::from_utf8(chunk.to_vec())
            .map_err(|e| SynapseError::DeserializationFailed(e.to_string()))
    }
}
```

See [Synapse](synapse.md) for the complete trait reference.

### How does request signing work?

When a dendrite has a hotkey configured, every outbound request goes through this process:

1. Serialize the synapse body to JSON.
2. Compute the SHA3-256 hash of the body.
3. Generate a monotonic nonce (Unix timestamp in milliseconds, incrementing for concurrent requests).
4. Construct the signing message: `"{nonce}.{dendrite_hotkey}.{axon_hotkey}.{uuid}.{body_hash}"`.
5. Sign the message bytes with the sr25519 keypair.
6. Attach `bt-*` headers to the HTTP request.

The axon's `VerificationMiddleware` checks that the `bt_header_dendrite_signature` header is present and that the signing fields parse correctly. The dendrite and axon produce wire-compatible requests, so cross-SDK signature verification works.

## Metagraph

### How do I sync the metagraph for a subnet?

```rust
use bittensor_metagraph::prelude::*;
use bittensor_chain::prelude::SubtensorClient;
use bittensor_core::config::NetworkConfig;

let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
let metagraph = sync(&client, 1).await?;
println!("Subnet 1: {} neurons at block {}", metagraph.n, metagraph.block);
```

Syncing queries the chain for every neuron in the subnet, so it can be slow on large subnets. Cache the result with `save()` if you plan to reload it.

### How do I iterate over neurons?

```rust
for neuron in metagraph.neurons() {
    println!(
        "UID {} | hotkey={} | stake={:.4} | incentive={:.4}",
        neuron.uid, neuron.hotkey, neuron.stake.to_tao(), neuron.incentive
    );
}
```

You can also iterate by reference:

```rust
for neuron in &metagraph {
    println!("UID {}", neuron.uid);
}
```

Or look up a specific UID:

```rust
if let Some(neuron) = metagraph.neuron_by_uid(42) {
    println!("UID 42 hotkey: {}", neuron.hotkey);
}
```

### How do I access the weight matrix?

The `weights` field is a flattened n-by-n array in row-major order. The weight that the neuron at position `i` assigns to the neuron at position `j` is:

```rust
let w = metagraph.weights[i * metagraph.n + j];
```

On-chain, weights are stored as sparse vectors of alternating `[uid, weight]` u16 pairs. The metagraph expands this into a full dense matrix during `sync`. A value of `0.0` means no connection.

The bond matrix follows the same layout:

```rust
let b = metagraph.bonds[i * metagraph.n + j];
```

### Can I save/load the metagraph to disk?

Yes. The `save` and `load` functions serialize to pretty-printed JSON:

```rust
use std::path::Path;

save(&metagraph, Path::new("/tmp/metagraph_subnet_1.json"))?;

let loaded = load(Path::new("/tmp/metagraph_subnet_1.json"))?;
assert_eq!(loaded.netuid, metagraph.netuid);
assert_eq!(loaded.n, metagraph.n);
```

This is useful for caching. Sync is expensive (one RPC call per neuron), so you can save the result and only re-sync when the block advances:

```rust
if path.exists() {
    if let Ok(cached) = load(path) {
        let current_block = queries::runtime::get_network_block(client.rpc()).await?;
        if cached.block >= current_block {
            return Ok(cached);
        }
    }
}
let metagraph = sync(client, netuid).await?;
save(&metagraph, path)?;
```

See [Metagraph](metagraph-lib.md) for the full API.

## Python and WASM Bindings

### How do I use the Python bindings?

Install the package and import it:

```bash
pip install bittensor-rs
```

```python
import asyncio
import bittensor_rs as bt

async def main():
    client = await bt.SubtensorClient.connect(bt.NetworkConfig.finney())
    balance = await client.get_balance("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")
    print(f"Balance: {balance}")

asyncio.run(main())
```

The Python module exposes `SubtensorClient`, `Wallet`, `Balance`, `NetworkConfig`, `AxonInfo`, `Dendrite`, `Metagraph`, and all the core types. See [Python Bindings](python-bindings.md) for the full reference.

### What is available in the WASM build?

The `bittensor-wasm` crate reimplements a subset of core types for the browser: `Balance`, `NetworkConfig`, SS58 encoding/decoding, and basic type constructors. These work in any JavaScript environment via wasm-bindgen.

```javascript
import { Balance, NetworkConfig } from "bittensor-wasm";

const b = Balance.from_tao(5.0);
console.log(b.rao);   // 5000000000
console.log(b.tao);   // 5.0

const config = NetworkConfig.finney();
console.log(config.ws_endpoint); // "wss://entrypoint-finney.opentensor.ai:443"
```

### What is NOT available in WASM?

The WASM build cannot include:

- **Wallet encryption/decryption** (requires libsodium/Argon2i, which does not compile to WASM)
- **Extrinsic submission** (requires subxt WebSocket client, which needs tokio)
- **Full `SubtensorClient`** (same reason: subxt requires a native async runtime)
- **Axon and Dendrite** (require axum/reqwest with native TLS)

For full chain interaction in the browser, call a backend server that uses the native Rust SDK, or use the Python bindings server-side.

## Troubleshooting

### Connection timeout errors

If `SubtensorClient::from_config` times out:

1. Check that you can reach the endpoint: `curl -i wss://entrypoint-finney.opentensor.ai:443`
2. Try the archive endpoint instead: `NetworkConfig::archive()`
3. If behind a firewall, confirm port 443 outbound is open for WebSocket traffic
4. Increase the connection timeout by using `from_url` with an endpoint closer to your region

For local development, make sure your Subtensor node is running on `ws://127.0.0.1:9943`:

```sh
cd subtensor && cargo run --release -- --dev
```

### "Insufficient balance" when staking

This error occurs when the free balance is less than the amount you are trying to stake. Common causes:

- **Forgetting about existential deposit**: Substrate requires a minimum balance (existential deposit) to keep an account alive. You cannot stake your entire free balance.
- **Pending transactions**: If you have a pending transfer or staking transaction, those funds are reserved and not available for a new transaction.
- **Wrong account**: Make sure the signer corresponds to the coldkey that actually holds the funds.

Check your actual free balance:

```rust
let balance: Balance = queries::account::get_balance(client.rpc(), &account_id).await?;
println!("Free: {balance}");
```

### Keyfile decryption failures

If `keyfile::decrypt` returns `KeyfileError::DecryptionFailed`:

- **Wrong password**: This is the most common cause. Check for typos, extra whitespace, or encoding issues.
- **Corrupted file**: Compare the file size with a known good coldkey file. The `$NACL` prefix should be exactly 5 bytes.
- **Wrong SDK version**: Very old Python SDK versions used a different salt. The current bittensor-rs matches the modern Python `btwallet` salt.

You can verify the file format:

```rust
let data = std::fs::read(&path)?;
assert!(keyfile::is_encrypted_nacl(&data), "Not a valid NaCl file");
```

### Build errors on Windows

On Windows, the wallet crate needs libsodium. The `libsodium-sys` build script looks for the library via:

1. The `SODIUM_LIB_DIR` environment variable
2. vcpkg (if `VCPKG_ROOT` is set)
3. The default system paths

Install libsodium with vcpkg:

```powershell
git clone https://github.com/microsoft/vcpkg
cd vcpkg
bootstrap-vcpkg.bat
vcpkg install libsodium:x64-windows
set VCPKG_ROOT=C:\path\to\vcpkg
cargo build
```

Or use the prebuilt binary from https://download.libsodium.org/libsodium/releases/.

### WASM build failures

If `cargo build --target wasm32-unknown-unknown` fails:

1. Make sure you are building `bittensor-wasm`, not the full workspace. Many crates do not compile to WASM.
2. Install the wasm32 target: `rustup target add wasm32-unknown-unknown`
3. The WASM crate does not depend on `bittensor-core` or `bittensor-chain`. It reimplements the subset of types it needs with `wasm-bindgen` annotations. If you see subxt or tokio errors, you are pulling in a native crate.
4. For `wasm-pack` builds, use `wasm-pack build --target web`.

```sh
cd bittensor-wasm
wasm-pack build --target web
```
