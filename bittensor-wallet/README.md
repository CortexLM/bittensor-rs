# bittensor-wallet

Wallet, keypair, and keyfile management for the bittensor-rs SDK.

## Quick Start

```rust
use bittensor_wallet::prelude::*;

// Create a wallet from a name and path
let wallet = Wallet::new("default", "/tmp/wallets")?;

// Get SS58 address
let addr = wallet.coldkeypub_address()?;

// Load or create a hotkey
let hotkey = wallet.hotkey()?;
```

## Feature Flags

No optional features — all functionality is always enabled.

## API Overview

| Module | Purpose |
|---|---|
| `wallet` | `Wallet` struct — coldkey/hotkey management, path resolution |
| `keypair` | `Keypair` — NaCl signing keypair with sr25519 interop |
| `keyfile` | Encrypt/decrypt keyfiles (NaCl secretbox format) |
| `mnemonic` | BIP39 mnemonic generation and key derivation |
| `ss58` | SS58 encoding/decoding for Substrate addresses |
