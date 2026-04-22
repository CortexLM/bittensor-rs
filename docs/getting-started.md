# Getting Started

This guide walks through installing the bittensor-rs SDK, setting up prerequisites, and writing your first programs that connect to the Bittensor network.

## Prerequisites

### Rust Toolchain

bittensor-rs requires Rust 1.85 or later with the 2024 edition. Install or update Rust using rustup:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable
rustup default stable
```

Verify your installation:

```sh
rustc --version
# Expected: rustc 1.85.0 or later
```

### System Libraries

The wallet crate uses libsodium for NaCl secretbox encryption (Argon2i key derivation, XSalsa20-Poly1305 authenticated encryption). Install it through your package manager:

```sh
# Ubuntu / Debian
apt install libsodium-dev

# macOS
brew install libsodium

# Fedora
dnf install libsodium-devel
```

### Network Access

You need access to a Subtensor node. The public endpoints are:

| Network | WebSocket URL |
|---|---|
| Finney (mainnet) | `wss://entrypoint-finney.opentensor.ai:443` |
| Testnet | `wss://test.finney.opentensor.ai:443` |
| Local | `ws://127.0.0.1:9944` |

For local development, run a Subtensor node:

```sh
git clone https://github.com/opentensor/subtensor.git
cd subtensor
cargo run --release -- --dev
```

## Installation

### Add to an Existing Project

Add the core crates to your `Cargo.toml`:

```toml
[dependencies]
bittensor-core = "0.1"
bittensor-chain = "0.1"
bittensor-wallet = "0.1"
```

Or use cargo-add from the command line:

```sh
cargo add bittensor-core bittensor-chain bittensor-wallet
```

### Add Optional Crates

Include additional crates as your project needs them:

```sh
# For running a neuron server
cargo add bittensor-axon

# For querying other neurons
cargo add bittensor-dendrite

# For protocol types and signing
cargo add bittensor-synapse

# For subnet graph operations
cargo add bittensor-metagraph
```

### Feature Flags

Some crates have optional features you can enable:

```toml
[dependencies]
bittensor-chain = { version = "0.1", features = ["drand", "mev-shield"] }
bittensor-metagraph = { version = "0.1", features = ["ml-backend"] }
bittensor-cli = { version = "0.1", features = ["mev"] }
```

The default features for bittensor-chain include `storage-subscriptions`, which enables real-time storage change notifications. You can disable defaults if you want a minimal build:

```toml
bittensor-chain = { version = "0.1", default-features = false }
```

### Start a New Project

Create a fresh project from scratch:

```sh
cargo new my-bittensor-app
cd my-bittensor-app
cargo add bittensor-core bittensor-chain bittensor-wallet tokio --features tokio/full
```

## Connecting to the Network

### Using NetworkConfig Presets

The `NetworkConfig` type provides preset constructors for each network:

```rust,no_run
use bittensor_chain::prelude::SubtensorClient;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to Finney mainnet
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Or connect to testnet
    // let client = SubtensorClient::from_config(NetworkConfig::test()).await?;

    // Or connect to a local node
    // let client = SubtensorClient::from_config(NetworkConfig::local()).await?;

    // Or connect to an archive node
    // let client = SubtensorClient::from_config(NetworkConfig::archive()).await?;

    println!("Connected to Subtensor");
    Ok(())
}
```

### Connecting with a Raw URL

For custom or non-standard endpoints, use `from_url`:

```rust,no_run
use bittensor_chain::prelude::SubtensorClient;

let client = SubtensorClient::from_url("wss://my-custom-node.example.com:443").await?;
```

### Failover Behavior

When you use `from_config` with a `NetworkConfig` that has an `archive_endpoint`, the client attempts the archive endpoint first, then falls back to the primary `ws_endpoint` if the archive connection fails. This gives you automatic resilience when archive nodes are available:

```rust,no_run
use bittensor_core::config::NetworkConfig;

let config = NetworkConfig::archive();
// Attempts wss://archive.finney.opentensor.ai:443 first,
// then falls back to the primary endpoint if archive is unreachable
let client = SubtensorClient::from_config(config).await?;
```

### Verifying the Connection

Once connected, you can verify the connection by checking the current block:

```rust,no_run
use bittensor_chain::prelude::SubtensorClient;
use bittensor_core::config::NetworkConfig;

let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
let block = client.at_current_block().await?;
println!("Connected at block hash: {:?}", block.block_hash());

// Look up a specific block number
let hash = client.get_block_hash(1000000).await?;
println!("Block 1000000 hash: {:?}", hash);
```

## Querying Balance

The Balance type represents on-chain value in rao, the smallest unit (1 TAO = 10^9 rao). All chain storage and extrinsics operate in rao; TAO is used for display only.

```rust,no_run
use bittensor_chain::prelude::*;
use bittensor_core::config::NetworkConfig;
use bittensor_core::balance::Balance;
use bittensor_core::types::AccountId32;

let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

// Query balance for an account
let account_id = AccountId32::from([0u8; 32]); // replace with real account
let balance: Balance = bittensor_chain::queries::account::get_balance(
    client.rpc(), &account_id
).await?;

// Display in TAO (9 decimal places)
println!("Balance: {balance}");

// Or access the raw rao value
println!("Balance in rao: {}", balance.to_rao());
```

### Balance Arithmetic

The `Balance` type supports checked, saturating, and panicking arithmetic:

```rust
use bittensor_core::balance::Balance;

// Create balances
let one_tao = Balance::from_tao(1.0);
let half_tao = Balance::from_tao(0.5);
let five_rao = Balance::from_rao(5);

// Panicking arithmetic (panics on overflow/underflow)
let sum = one_tao + half_tao; // 1.5 TAO
let diff = one_tao - half_tao; // 0.5 TAO
let triple = one_tao * 3;     // 3.0 TAO
let ratio = one_tao / half_tao; // 2 (integer division)

// Checked arithmetic (returns Option)
if let Some(safe_sum) = one_tao.checked_add(half_tao) {
    println!("Safe sum: {safe_sum}");
}

// Saturating arithmetic (clamps instead of panicking)
let max_balance = Balance::from_rao(u64::MAX);
let saturated = max_balance.saturating_add(Balance::from_rao(1));
// saturated == max_balance (clamped at u64::MAX)
```

### Parsing Balance from Strings

```rust
use bittensor_core::balance::Balance;

let b: Balance = "1.5".parse()?;
assert_eq!(b, Balance::from_tao(1.5));
```

## Transferring TAO

Transfers require a signing keypair. You can sign with development keys for local testing, or with wallet keypairs for mainnet.

### Transfer with Development Keys (Local/Test)

```rust,no_run
use bittensor_chain::prelude::*;
use bittensor_core::config::NetworkConfig;
use bittensor_core::balance::Balance;

let client = SubtensorClient::from_config(NetworkConfig::local()).await?;

// Development keypairs (only valid on local/test chains)
let signer = subxt_signer::sr25519::Keypair::from_uri("//Alice")?;
let dest = subxt_signer::sr25519::PublicKey::from_uri("//Bob")?;

// Transfer 1.0 TAO
let amount_rao = Balance::from_tao(1.0).to_rao();
bittensor_chain::extrinsics::transfer::transfer(
    client.rpc(), &signer, &dest, amount_rao
).await?;

println!("Transfer submitted");
```

### Transfer with Wallet Keys (Mainnet)

```rust,no_run
use bittensor_chain::prelude::*;
use bittensor_core::config::NetworkConfig;
use bittensor_core::balance::Balance;
use bittensor_wallet::prelude::*;

let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

// Load a wallet and decrypt the coldkey
let mut wallet = Wallet::new("my-wallet");
let coldkey = wallet.get_coldkey_pair("my-password")?;
let signer = coldkey.into_signer();

// Build the destination from an SS58 address
let dest_public = subxt_signer::sr25519::PublicKey::from_ss58_address(
    "5DfhGyQdFobKM8NsWvEeAKk5EQQgYe9AydgJ7rMB6E1EqRzV"
)?;

let amount_rao = Balance::from_tao(5.0).to_rao();
bittensor_chain::extrinsics::transfer::transfer(
    client.rpc(), &signer, &dest_public, amount_rao
).await?;
```

## Creating a Wallet

The wallet crate manages coldkey and hotkey pairs using the same file layout as the Python SDK, so wallets created with `btcli` are readable by `btcli-rs` and vice versa.

### Directory Layout

```
~/.bittensor/wallets/<name>/
  coldkey        # Encrypted with NaCl secretbox (Argon2i + XSalsa20-Poly1305)
  coldkeypub     # Plaintext SS58 address
  hotkeys/
    default      # Raw hex seed (unencrypted, matches Python behavior)
```

### Create a New Wallet

```rust
use bittensor_wallet::prelude::*;

// Create a wallet with default name "default"
let mut wallet = Wallet::new("default");

// Generate a coldkey. The mnemonic is returned so you can back it up.
let mnemonic = wallet.create_coldkey("strong-password")?;
println!("Write down this mnemonic: {mnemonic}");

// Read the public address without needing the password again
let address = wallet.get_coldkeypub()?;
println!("Coldkey address: {address}");

// Generate a hotkey
let hotkey = wallet.create_hotkey()?;
println!("Hotkey address: {}", hotkey.ss58_address());
```

### Create a Wallet from an Existing Mnemonic

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

### Load an Existing Wallet

```rust
use bittensor_wallet::prelude::*;

let mut wallet = Wallet::new("my-wallet");

// The coldkeypub file is plaintext, so reading the address needs no password
let address = wallet.get_coldkeypub()?;

// Decrypting the coldkey requires the password
let coldkey = wallet.get_coldkey_pair("my-password")?;
println!("Coldkey SS58: {}", coldkey.ss58_address());

// Hotkeys are stored as raw hex seeds and need no password
let hotkey = wallet.get_hotkey_pair()?;
println!("Hotkey SS58: {}", hotkey.ss58_address());
```

### Sign and Verify Messages

```rust
use bittensor_wallet::prelude::*;

let mut wallet = Wallet::new("signer");
wallet.create_coldkey("pass")?;
wallet.create_hotkey()?;

// Sign with the hotkey (no password needed)
let message = b"hello bittensor";
let signature = wallet.sign(message)?;

// Verify
let hotkey = wallet.get_hotkey_pair()?;
assert!(Wallet::verify(&signature, message, &hotkey.public_key()));

// Sign with the coldkey (password required)
let coldkey_sig = wallet.sign_coldkey(message, "pass")?;
let coldkey = wallet.get_coldkey_pair("pass")?;
assert!(Wallet::verify(&coldkey_sig, message, &coldkey.public_key()));
```

## Keyfile Encryption

Coldkey files are encrypted using NaCl secretbox, which is the same format the Python SDK uses. This means keyfiles created by the Python `btcli` can be decrypted by bittensor-rs, and vice versa.

### How It Works

1. A 16-byte salt is hardcoded (matching the Python SDK's `btwallet` salt).
2. The password is stretched using Argon2i with `OPSLIMIT_SENSITIVE` and `MEMLIMIT_SENSITIVE` (512 MB, 8 passes).
3. The derived key encrypts the JSON payload with XSalsa20-Poly1305.
4. The output is `$NACL` + 24-byte nonce + ciphertext.

### Cross-Compatibility Check

```rust
use bittensor_wallet::keyfile;

// Read a coldkey file created by Python btcli
let encrypted = std::fs::read("~/.bittensor/wallets/default/coldkey")?;
assert!(keyfile::is_encrypted_nacl(&encrypted), "Not a NaCl file");

let decrypted = keyfile::decrypt(&encrypted, b"my-password")?;
let json: serde_json::Value = serde_json::from_slice(&decrypted)?;
assert!(json["ss58Address"].is_str());
```

## SS58 Address Encoding

Bittensor uses SS58 encoding with prefix 42 (the Substrate default). The wallet crate handles encoding and decoding:

```rust
use bittensor_wallet::prelude::*;

// Encoding is done automatically by Keypair::ss58_address()
let keypair = Keypair::from_uri(
    &subxt_signer::SecretUri::from_str("//Alice")?
)?;
let addr = keypair.ss58_address();
assert!(addr.starts_with('5'));
```

## Error Handling

All fallible operations return `Result<T, BittensorError>`. The error type classifies failures into retry-oriented categories:

```rust
use bittensor_core::error::BittensorError;

match result {
    Ok(value) => { /* handle success */ },
    Err(e) => {
        if e.is_retryable() {
            // Transient or rate-limited: retry with backoff
            let config = e.category().retry_config();
            println!("Retryable error: {e}, max retries: {}", config.max_retries);
        } else {
            // Permanent: bad config, auth failure, codec error, etc.
            println!("Permanent error: {e}");
        }
    }
}
```

### Error Categories

| Category | Variants | Retryable? |
|---|---|---|
| Transient | `Rpc`, `Network`, `Timeout` | Yes |
| RateLimit | `RateLimit` | Yes (longer backoff) |
| Auth | `Authentication` | No |
| Config | `Config` | No |
| Permanent | `Signing`, `Codec`, `Transaction`, `Wallet`, `Balance`, `Validation` | No |

## Your First Complete Program

This program connects to Finney, queries the balance of an account, and prints it:

```rust
use bittensor_chain::prelude::SubtensorClient;
use bittensor_core::config::NetworkConfig;
use bittensor_core::balance::Balance;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to Finney mainnet
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
    println!("Connected to Finney");

    // Check current block
    let block = client.at_current_block().await?;
    println!("Current block hash: {:?}", block.block_hash());

    // Query a balance (replace with a real account ID)
    let account_bytes: [u8; 32] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    let account_id = subxt::utils::AccountId32::from(account_bytes);
    let balance: Balance = bittensor_chain::queries::account::get_balance(
        client.rpc(), &account_id
    ).await?;

    println!("Account balance: {balance}");
    println!("In rao: {}", balance.to_rao());
    println!("In TAO: {:.9}", balance.to_tao());

    Ok(())
}
```

Save this as `src/main.rs` in your project, add the dependencies, and run:

```sh
cargo run
```

## Next Steps

- [Architecture Overview](architecture.md) -- Understand the crate structure, query flow, transaction pipeline, and synapse protocol
- [Chain Queries](queries.md) -- Full reference for read-only chain queries
- [Extrinsics](extrinsics/) -- Full reference for signed transactions
- [Types](types.md) -- All data structures with field descriptions
- [Validator Operations](validator.md) -- Staking, weights, registration, serving
