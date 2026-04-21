# Bittensor Coldkey Encryption Protocol

This document describes the exact encryption format used by bittensor coldkey files,
compatible with both the Python SDK (PyNaCl) and the Rust SDK (sodiumoxide/btwallet).

## Wire Format

```
┌──────────────┬─────────────────┬───────────────────────────────────┐
│ $NACL        │  NONCE (24B)    │  CIPHERTEXT (16B MAC + plaintext) │
│ 5 bytes      │  24 bytes       │  variable                          │
└──────────────┴─────────────────┴───────────────────────────────────┘
```

- **Prefix**: ASCII string `$NACL` (hex: `24 4E 41 43 4C`), 5 bytes
- **Nonce**: 24 bytes, randomly generated per encryption
- **Ciphertext**: `crypto_secretbox` output — 16-byte Poly1305 MAC prepended to encrypted plaintext
- **File mode**: `0o600` (owner read/write only)
- **No base64 encoding**: the file is raw binary bytes

## Key Derivation

| Parameter | Value |
|-----------|-------|
| Algorithm | Argon2i (version 1.3) |
| ops_limit | 8 (`OPSLIMIT_SENSITIVE`) |
| mem_limit | 536870912 (`MEMLIMIT_SENSITIVE`, 512 MiB) |
| Salt | Hardcoded: `\x13q\x83\xdf\xf1Z\t\xbc\x9c\x90\xb5Q\x879\xe9\xb1` (hex: `137183dff15a09bc9c90b5518739e9b1`) |
| Output length | 32 bytes (matching `secretbox::KEYBYTES`) |

The salt is **global and hardcoded** — every coldkey uses the same salt. This matches
the reference implementation in [opentensor/btwallet](https://github.com/opentensor/btwallet/blob/main/src/keyfile.rs).

## Encryption

| Parameter | Value |
|-----------|-------|
| Algorithm | XSalsa20-Poly1305 (`crypto_secretbox`) |
| Key size | 32 bytes |
| Nonce size | 24 bytes (random, per encryption) |
| MAC size | 16 bytes (Poly1305 tag, prepended by `seal`) |

### Encrypt Procedure

1. Derive 32-byte key from password using argon2i with the hardcoded salt
2. Generate random 24-byte nonce
3. Encrypt plaintext with `crypto_secretbox_seal(plaintext, nonce, key)`
   — this returns MAC (16B) + ciphertext
4. Concatenate: `$NACL` + nonce + sealed_output

### Decrypt Procedure

1. Verify data starts with `$NACL` (5 bytes)
2. Strip prefix, extract nonce (first 24 bytes after prefix)
3. Extract ciphertext (remaining bytes after nonce)
4. Derive key from password using same argon2i parameters
5. Call `crypto_secretbox_open(ciphertext, nonce, key)`
6. Return decrypted plaintext

## Keyfile JSON Payload

The plaintext is a JSON object with these fields:

```json
{
  "accountId": "0x<hex_public_key>",
  "publicKey": "0x<hex_public_key>",
  "secretPhrase": "<mnemonic_or_null>",
  "secretSeed": "0x<hex_secret_seed>",
  "ss58Address": "<ss58_address_or_null>"
}
```

## Compatibility

This implementation has been validated in both directions:

- **Python → Rust**: Python-encrypted coldkey decrypted by Rust ✓
- **Rust → Python**: Rust-encrypted coldkey decrypted by Python ✓

### Python (PyNaCl)

```python
from nacl.pwhash import argon2i
from nacl.secret import SecretBox

NACL_SALT = b"\x13q\x83\xdf\xf1Z\t\xbc\x9c\x90\xb5Q\x879\xe9\xb1"

key = argon2i.kdf(
    SecretBox.KEY_SIZE, password.encode(), NACL_SALT,
    opslimit=argon2i.OPSLIMIT_SENSITIVE,
    memlimit=argon2i.MEMLIMIT_SENSITIVE,
)
box = SecretBox(key)
encrypted = box.encrypt(plaintext)  # nonce + MAC + ciphertext
coldkey_data = b"$NACL" + encrypted
```

### Rust (sodiumoxide)

```rust
use sodiumoxide::crypto::pwhash::argon2i13;
use sodiumoxide::crypto::secretbox;

const NACL_SALT: &[u8] = b"\x13q\x83\xdf\xf1Z\t\xbc\x9c\x90\xb5Q\x879\xe9\xb1";

fn derive_key(password: &[u8]) -> secretbox::Key {
    let salt = argon2i13::Salt::from_slice(NACL_SALT).unwrap();
    let mut key = [0u8; secretbox::KEYBYTES];
    argon2i13::derive_key(&mut key, password, &salt,
        argon2i13::OPSLIMIT_SENSITIVE,
        argon2i13::MEMLIMIT_SENSITIVE,
    ).unwrap();
    secretbox::Key(key)
}

fn encrypt(data: &[u8], password: &[u8]) -> Vec<u8> {
    let key = derive_key(password);
    let nonce = secretbox::gen_nonce();
    let ciphertext = secretbox::seal(data, &nonce, &key);
    let mut result = b"$NACL".to_vec();
    result.extend_from_slice(&nonce.0);
    result.extend_from_slice(&ciphertext);
    result
}
```

## Legacy Formats (Not Implemented)

The Python SDK also supports these legacy formats, which are **NOT** implemented here:

- **Ansible Vault**: Detected by `$ANSIBLE_VAULT` prefix
- **Fernet (legacy)**: Detected by `gAAAAA` prefix, uses PBKDF2HMAC-SHA256 with 10M iterations

## Reference Implementation

The authoritative source is the btwallet Rust crate:
https://github.com/opentensor/btwallet/blob/main/src/keyfile.rs
