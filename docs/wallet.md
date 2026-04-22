# Wallet API Reference

The `bittensor-wallet` crate provides wallet, keypair, keyfile, mnemonic, and SS58 utilities for the Bittensor network. It is fully compatible with the Python SDK's file layout and keyfile encryption format.

```toml
[dependencies]
bittensor-wallet = "0.1"
```

```rust
use bittensor_wallet::prelude::*;
```

---

## Wallet

Manages coldkey and hotkey pairs on disk, following the Python SDK's directory layout:

```text
~/.bittensor/wallets/<name>/
  coldkey        (encrypted NaCl)
  coldkeypub     (plaintext SS58 address)
  hotkeys/
    <hotkey_name>  (raw hex seed, unencrypted)
```

### `Wallet::new`

```rust
pub fn new(name: &str) -> Self
```

Creates a `Wallet` pointing at the default Bittensor directory (`~/.bittensor/wallets/<name>/`). The hotkey name defaults to `"default"`. Keys are lazy-loaded and cached after first access.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `name` | `&str` | Wallet name, used as the directory name under `~/.bittensor/wallets/` |

**Example**

```rust
use bittensor_wallet::prelude::*;

let mut wallet = Wallet::new("my-wallet");
println!("Wallet path: {:?}", wallet.path);
```

### `Wallet::with_path`

```rust
pub fn with_path(name: &str, path: PathBuf) -> Self
```

Creates a `Wallet` with a custom filesystem path. Useful for testing or non-default storage locations.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `name` | `&str` | Wallet name |
| `path` | `PathBuf` | Root directory for this wallet's files |

**Example**

```rust
use bittensor_wallet::prelude::*;
use std::path::PathBuf;

let mut wallet = Wallet::with_path("test", PathBuf::from("/tmp/test-wallets/test"));
```

### `Wallet::set_hotkey_name`

```rust
pub fn set_hotkey_name(&mut self, hotkey_name: &str)
```

Sets the hotkey name. Defaults to `"default"`. Must be called before `create_hotkey` or `get_hotkey_pair` if you want a name other than `"default"`.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `hotkey_name` | `&str` | Name for the hotkey file under `hotkeys/` |

**Example**

```rust
let mut wallet = Wallet::new("validator");
wallet.set_hotkey_name("validator-hotkey");
```

### `Wallet::create_coldkey`

```rust
pub fn create_coldkey(&mut self, password: &str) -> Result<bip39::Mnemonic, WalletError>
```

Generates a new 12-word mnemonic, derives a coldkey from it, encrypts the keyfile, and writes it to disk. Returns the mnemonic for backup.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `password` | `&str` | Encryption password for the coldkey file |

**Returns**

`Result<bip39::Mnemonic, WalletError>` -- The generated mnemonic phrase. Back this up.

**Errors**

- `WalletError::Io` if the directory or file cannot be created
- `WalletError::Mnemonic` if mnemonic generation fails
- `WalletError::Keyfile` if encryption fails

**Example**

```rust
let mut wallet = Wallet::with_path("demo", PathBuf::from("/tmp/wallets/demo"));
let mnemonic = wallet.create_coldkey("s3cure-p4ss")?;
println!("Back up this mnemonic: {mnemonic}");
```

### `Wallet::create_coldkey_from_mnemonic`

```rust
pub fn create_coldkey_from_mnemonic(
    &mut self,
    mnemonic: &bip39::Mnemonic,
    password: &str,
) -> Result<(), WalletError>
```

Creates a coldkey from an existing mnemonic, encrypts it, and writes to disk.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `mnemonic` | `&bip39::Mnemonic` | Existing mnemonic to derive the key from |
| `password` | `&str` | Encryption password |

**Example**

```rust
use std::str::FromStr;

let phrase = "bottom drive obey lake curtain smoke basket hold race lonely fit walk";
let mnemonic = bip39::Mnemonic::parse(phrase)?;
wallet.create_coldkey_from_mnemonic(&mnemonic, "my-password")?;
```

### `Wallet::create_hotkey`

```rust
pub fn create_hotkey(&mut self) -> Result<Keypair, WalletError>
```

Generates a new hotkey from a fresh mnemonic and writes the raw hex seed to disk (unencrypted, matching the Python SDK).

**Returns**

`Result<Keypair, WalletError>` -- The newly created hotkey keypair.

**Example**

```rust
let mut wallet = Wallet::with_path("demo", PathBuf::from("/tmp/wallets/demo"));
std::fs::create_dir_all(&wallet.path)?;
let hotkey = wallet.create_hotkey()?;
println!("Hotkey SS58: {}", hotkey.ss58_address());
```

### `Wallet::create_hotkey_from_coldkey`

```rust
pub fn create_hotkey_from_coldkey(&mut self, password: &str) -> Result<Keypair, WalletError>
```

Derives a hotkey from the coldkey using a hard derivation path (`//<hotkey_name>`). Writes the hex seed to disk.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `password` | `&str` | Password to decrypt the coldkey |

**Example**

```rust
wallet.set_hotkey_name("derived");
let derived = wallet.create_hotkey_from_coldkey("coldkey-password")?;
```

### `Wallet::get_coldkey_pair`

```rust
pub fn get_coldkey_pair(&mut self, password: &str) -> Result<Keypair, WalletError>
```

Loads and decrypts the coldkey pair. Results are cached after the first successful load.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `password` | `&str` | Encryption password for the coldkey file |

**Errors**

- `WalletError::NotFound` if the coldkey file does not exist
- `WalletError::ColdkeyRequired` if no coldkeypub file exists and no password is available
- `WalletError::Keyfile` if decryption fails (wrong password)

**Example**

```rust
let coldkey = wallet.get_coldkey_pair("my-password")?;
println!("Coldkey: {}", coldkey.ss58_address());
```

### `Wallet::get_coldkeypub`

```rust
pub fn get_coldkeypub(&mut self) -> Result<String, WalletError>
```

Returns the coldkey public key as an SS58 address string. Reads from the `coldkeypub` plaintext file first. Falls back to requiring a password to decrypt the full coldkey if the pubkey file is missing.

**Errors**

- `WalletError::ColdkeyRequired` if no pubkey file exists and no cached coldkey is available

**Example**

```rust
let address = wallet.get_coldkeypub()?;
assert!(address.starts_with('5'));
```

### `Wallet::get_coldkeypub_pair`

```rust
pub fn get_coldkeypub_pair(&mut self, password: &str) -> Result<Keypair, WalletError>
```

Loads the coldkey public keypair (decrypts the full coldkey, caches the public key portion). Requires password.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `password` | `&str` | Encryption password |

### `Wallet::get_hotkey_pair`

```rust
pub fn get_hotkey_pair(&mut self) -> Result<Keypair, WalletError>
```

Loads the hotkey pair from the plaintext hex seed file. No password required. Results are cached.

**Errors**

- `WalletError::NotFound` if the hotkey file does not exist

**Example**

```rust
let hotkey = wallet.get_hotkey_pair()?;
println!("Hotkey: {}", hotkey.ss58_address());
```

### `Wallet::sign`

```rust
pub fn sign(&mut self, message: &[u8]) -> Result<subxt_signer::sr25519::Signature, WalletError>
```

Signs a message with the hotkey.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `message` | `&[u8]` | Message bytes to sign |

**Example**

```rust
let msg = b"hello bittensor";
let sig = wallet.sign(msg)?;
assert!(Wallet::verify(&sig, msg, &wallet.get_hotkey_pair()?.public_key()));
```

### `Wallet::sign_coldkey`

```rust
pub fn sign_coldkey(
    &mut self,
    message: &[u8],
    password: &str,
) -> Result<subxt_signer::sr25519::Signature, WalletError>
```

Signs a message with the coldkey. Requires the encryption password.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `message` | `&[u8]` | Message bytes to sign |
| `password` | `&str` | Coldkey encryption password |

### `Wallet::verify` (associated function)

```rust
pub fn verify(
    signature: &subxt_signer::sr25519::Signature,
    message: &[u8],
    public_key: &subxt_signer::sr25519::PublicKey,
) -> bool
```

Verifies an SR25519 signature against a message and public key. This is a stateless function, no `&mut self` needed.

**Example**

```rust
let valid = Wallet::verify(&sig, msg, &public_key);
```

### `Wallet::hotkey_ss58_address`

```rust
pub fn hotkey_ss58_address(&mut self) -> Result<String, WalletError>
```

Returns the SS58 address of the hotkey. Loads the key from disk if not cached.

### `Wallet::coldkey_ss58_address`

```rust
pub fn coldkey_ss58_address(&mut self, password: &str) -> Result<String, WalletError>
```

Returns the SS58 address of the coldkey. Requires password to decrypt.

### Path accessors

```rust
pub fn coldkey_path(&self) -> PathBuf
pub fn coldkeypub_path(&self) -> PathBuf
pub fn hotkey_path(&self) -> PathBuf
```

Return filesystem paths for the coldkey, coldkeypub, and hotkey files respectively.

---

## Keypair

SR25519 signing keypair with seed tracking for cross-compatible key derivation.

```rust
use bittensor_wallet::prelude::Keypair;
```

### `Keypair::from_uri`

```rust
pub fn from_uri(uri: &SecretUri) -> Result<Self, KeypairError>
```

Creates a keypair from a [`SecretUri`]. Supports mnemonic phrases, hex seeds (`0x...`), and derivation paths (`//Alice//stash`).

**Example**

```rust
use subxt_signer::SecretUri;
use std::str::FromStr;

let uri = SecretUri::from_str("//Alice").expect("parse uri");
let kp = Keypair::from_uri(&uri)?;
assert_eq!(kp.ss58_address(), "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY");
```

### `Keypair::from_phrase`

```rust
pub fn from_phrase(
    mnemonic: &bip39::Mnemonic,
    password: Option<&str>,
) -> Result<Self, KeypairError>
```

Creates a keypair from a BIP-39 mnemonic with an optional password. Seed derivation uses PBKDF2, matching the Python SDK.

**Example**

```rust
let phrase = "bottom drive obey lake curtain smoke basket hold race lonely fit walk";
let mnemonic = bip39::Mnemonic::parse(phrase)?;
let kp = Keypair::from_phrase(&mnemonic, None)?;
assert_eq!(kp.ss58_address(), "5DfhGyQdFobKM8NsWvEeAKk5EQQgYe9AydgJ7rMB6E1EqRzV");
```

### `Keypair::from_secret_key`

```rust
pub fn from_secret_key(seed: SecretKeyBytes) -> Result<Self, KeypairError>
```

Creates a keypair from raw 32-byte secret key bytes.

### `Keypair::from_seed_hex`

```rust
pub fn from_seed_hex(hex_str: &str) -> Result<Self, KeypairError>
```

Creates a keypair from a hex-encoded seed string. Accepts both `0x`-prefixed and unprefixed hex.

**Example**

```rust
let kp = Keypair::from_seed_hex("0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7aee9f25cc4d693213c4e829")?;
```

### `Keypair::derive`

```rust
pub fn derive(&self, junctions: impl IntoIterator<Item = DeriveJunction>) -> Self
```

Derives a child keypair using hard or soft junctions.

**Example**

```rust
use subxt_signer::DeriveJunction;

let derived = kp.derive([DeriveJunction::hard("stash")]);
```

### `Keypair::public_key`

```rust
pub fn public_key(&self) -> PublicKey
```

Returns the SR25519 public key.

### `Keypair::sign`

```rust
pub fn sign(&self, message: &[u8]) -> Signature
```

Signs a message with this keypair's secret key.

### `Keypair::ss58_address`

```rust
pub fn ss58_address(&self) -> String
```

Returns the SS58-encoded address string using Bittensor format (prefix 42).

### `Keypair::seed_hex`

```rust
pub fn seed_hex(&self) -> String
```

Returns the hex-encoded seed with a `0x` prefix.

### `Keypair::seed`

```rust
pub fn seed(&self) -> &SecretKeyBytes
```

Returns a reference to the raw secret key bytes.

### `Keypair::signer`

```rust
pub fn signer(&self) -> &InnerKeypair
```

Returns a reference to the inner `subxt_signer::sr25519::Keypair` for use with subxt transaction signing.

### `Keypair::into_signer`

```rust
pub fn into_signer(self) -> InnerKeypair
```

Consumes self and returns the inner keypair. Useful when moving into async closures.

### `Keypair::from_encrypted_coldkey`

```rust
pub fn from_encrypted_coldkey(
    path: &std::path::Path,
    password: &str,
) -> Result<Self, KeypairError>
```

Loads a keypair from a NaCl-encrypted coldkey JSON file.

### `Keypair::from_hotkey_file`

```rust
pub fn from_hotkey_file(path: &std::path::Path) -> Result<Self, KeypairError>
```

Loads a keypair from a plaintext hotkey file containing a hex-encoded seed.

### `verify` (free function)

```rust
pub fn verify(
    signature: &Signature,
    message: &[u8],
    public_key: &PublicKey,
) -> bool
```

Verifies an SR25519 signature against a message and public key.

---

## Keyfile

NaCl secretbox encryption and decryption for coldkey files. Fully compatible with the Python SDK's `$NACL` format.

```rust
use bittensor_wallet::prelude::{encrypt, decrypt, is_encrypted_nacl};
```

### `encrypt`

```rust
pub fn encrypt(data: &[u8], password: &[u8]) -> Result<Vec<u8>, KeyfileError>
```

Encrypts data using NaCl secretbox (XSalsa20-Poly1305). The key is derived from the password using Argon2i with the same salt as the Python SDK. Output format: `$NACL` + 24-byte nonce + ciphertext.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `data` | `&[u8]` | Plaintext to encrypt |
| `password` | `&[u8]` | Encryption password (passed through Argon2i KDF) |

**Example**

```rust
let plaintext = b"my secret data";
let encrypted = encrypt(plaintext, b"password123")?;
assert!(is_encrypted_nacl(&encrypted));
```

### `decrypt`

```rust
pub fn decrypt(encrypted: &[u8], password: &[u8]) -> Result<Vec<u8>, KeyfileError>
```

Decrypts a `$NACL`-prefixed ciphertext. Compatible with keyfiles created by the Python SDK.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `encrypted` | `&[u8]` | Encrypted data including the `$NACL` prefix |
| `password` | `&[u8]` | Decryption password |

**Errors**

- `KeyfileError::InvalidEncryption` if the data lacks the `$NACL` prefix or has an invalid nonce
- `KeyfileError::DecryptionFailed` if the password is wrong or data is corrupted

**Example**

```rust
let decrypted = decrypt(&encrypted, b"password123")?;
assert_eq!(decrypted, b"my secret data");
```

### `is_encrypted_nacl`

```rust
pub fn is_encrypted_nacl(data: &[u8]) -> bool
```

Returns `true` if the byte slice starts with the `$NACL` prefix, indicating NaCl secretbox encryption.

---

## Mnemonic

BIP-39 mnemonic generation and key derivation.

```rust
use bittensor_wallet::prelude::{WordCount, generate_mnemonic, parse_mnemonic, keypair_from_mnemonic};
```

### `WordCount`

```rust
pub enum WordCount {
    Words12,
    Words24,
}
```

Supported mnemonic word counts. 12 words produce 128 bits of entropy; 24 words produce 256 bits.

### `generate_mnemonic`

```rust
pub fn generate_mnemonic(word_count: WordCount) -> Result<bip39::Mnemonic, MnemonicError>
```

Generates a random BIP-39 mnemonic with the given word count.

**Example**

```rust
let mnemonic = generate_mnemonic(WordCount::Words12)?;
let words: Vec<&str> = mnemonic.words().collect();
assert_eq!(words.len(), 12);
```

### `parse_mnemonic`

```rust
pub fn parse_mnemonic(phrase: &str) -> Result<bip39::Mnemonic, MnemonicError>
```

Parses a mnemonic phrase string into a `Mnemonic`. Validates checksum.

**Example**

```rust
let mnemonic = parse_mnemonic("bottom drive obey lake curtain smoke basket hold race lonely fit walk")?;
```

### `keypair_from_mnemonic`

```rust
pub fn keypair_from_mnemonic(
    mnemonic: &bip39::Mnemonic,
    password: Option<&str>,
) -> Result<Keypair, MnemonicError>
```

Derives a `Keypair` from a mnemonic with an optional password.

**Example**

```rust
let kp = keypair_from_mnemonic(&mnemonic, Some("extra-password"))?;
println!("Address: {}", kp.ss58_address());
```

---

## SS58

Substrate address encoding and decoding.

```rust
use bittensor_wallet::prelude::{encode_ss58, encode_ss58_address, decode_ss58};
```

### `encode_ss58`

```rust
pub fn encode_ss58(public_key: &[u8; 32], format: u8) -> String
```

Encodes a 32-byte public key as an SS58 address with the specified format byte. Bittensor uses format 42 (Substrate default).

**Example**

```rust
let key = [42u8; 32];
let address = encode_ss58(&key, 42);
```

### `encode_ss58_address`

```rust
pub fn encode_ss58_address(public_key: &[u8; 32]) -> String
```

Encodes a 32-byte public key as an SS58 address using Bittensor default format (42). Shorthand for `encode_ss58(key, 42)`.

### `decode_ss58`

```rust
pub fn decode_ss58(address: &str) -> Result<(u8, [u8; 32]), Ss58Error>
```

Decodes an SS58 address string into its format byte and 32-byte public key. Verifies the Blake2b-512 checksum.

**Errors**

- `Ss58Error::InvalidAddress` if the address string is malformed
- `Ss58Error::LengthMismatch` if the decoded bytes are the wrong length
- `Ss58Error::InvalidChecksum` if the checksum does not match
- `Ss58Error::BadPrefix` if the prefix byte is unrecognized
- `Ss58Error::Base58` if base58 decoding fails

**Example**

```rust
let (format, pubkey) = decode_ss58("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")?;
assert_eq!(format, 42);
```

---

## Error Types

### `WalletError`

```rust
pub enum WalletError {
    Keypair(KeypairError),
    Mnemonic(MnemonicError),
    Ss58(Ss58Error),
    Keyfile(KeyfileError),
    Io(std::io::Error),
    NotFound(String),
    ColdkeyRequired(String),
    Serialization(serde_json::Error),
}
```

| Variant | Description |
|---------|-------------|
| `Keypair` | Underlying keypair operation failed |
| `Mnemonic` | Mnemonic generation or parsing failed |
| `Ss58` | SS58 encoding or decoding failed |
| `Keyfile` | Keyfile encryption or decryption failed |
| `Io` | Filesystem I/O error |
| `NotFound` | Key file not found on disk |
| `ColdkeyRequired` | Operation needs the coldkey but only the public key was available |
| `Serialization` | JSON serialization error |

### `KeypairError`

```rust
pub enum KeypairError {
    InvalidMnemonic(String),
    InvalidSecretUri(String),
    InvalidSeedHex(String),
    SeedDerivationFailed,
    Keyfile(KeyfileError),
    Ss58(Ss58Error),
    Io(std::io::Error),
    Signer(sr25519::Error),
    Schnorrkel(String),
}
```

### `KeyfileError`

```rust
pub enum KeyfileError {
    InvalidEncryption(String),
    DecryptionFailed(String),
    KeyDerivationFailed,
}
```

### `MnemonicError`

```rust
pub enum MnemonicError {
    Parse(bip39::Error),
    Keypair(KeypairError),
}
```

### `Ss58Error`

```rust
pub enum Ss58Error {
    InvalidAddress(String),
    BadPrefix(u8),
    LengthMismatch { expected: usize, actual: usize },
    InvalidChecksum,
    Base58(bs58::decode::Error),
}
```

---

## Full Example: Create and Sign

```rust
use bittensor_wallet::prelude::*;
use std::path::PathBuf;

fn main() -> Result<(), WalletError> {
    let dir = PathBuf::from("/tmp/example-wallet");
    let mut wallet = Wallet::with_path("example", dir);

    // Create coldkey, get the mnemonic back
    let mnemonic = wallet.create_coldkey("strong-password")?;
    println!("Write down this mnemonic: {mnemonic}");

    // Create hotkey
    let hotkey = wallet.create_hotkey()?;
    println!("Hotkey: {}", hotkey.ss58_address());

    // Read coldkey address without password (from coldkeypub file)
    let coldkey_addr = wallet.get_coldkeypub()?;
    println!("Coldkey: {coldkey_addr}");

    // Sign with hotkey
    let message = b"test message";
    let signature = wallet.sign(message)?;
    let public_key = wallet.get_hotkey_pair()?.public_key();
    assert!(Wallet::verify(&signature, message, &public_key));

    Ok(())
}
```

## Full Example: Load Existing Wallet

```rust
use bittensor_wallet::prelude::*;

fn main() -> Result<(), WalletError> {
    let mut wallet = Wallet::new("my-existing-wallet");

    // Load coldkey (needs password)
    let coldkey = wallet.get_coldkey_pair("my-password")?;
    println!("Coldkey address: {}", coldkey.ss58_address());

    // Load hotkey (no password needed)
    let hotkey = wallet.get_hotkey_pair()?;
    println!("Hotkey address: {}", hotkey.ss58_address());

    // Sign a message with the coldkey
    let sig = wallet.sign_coldkey(b"important data", "my-password")?;
    println!("Signature created");

    Ok(())
}
```
