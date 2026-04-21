#!/usr/bin/env python3
"""Create a NaCl-encrypted coldkey file matching the bittensor-wallet format.

This replicates the exact encryption used by opentensor/btwallet (Rust)
and the legacy bittensor Python SDK (PyNaCl).

Format: b"$NACL" + nonce(24B) + ciphertext(16B MAC + encrypted_data)

Key derivation: argon2i, OPSLIMIT_SENSITIVE=8, MEMLIMIT_SENSITIVE=1073741824
Salt: hardcoded b"\\x13q\\x83\\xdf\\xf1Z\\t\\xbc\\x9c\\x90\\xb5Q\\x879\\xe9\\xb1"
Encryption: crypto_secretbox (XSalsa20-Poly1305)
"""

import json
import os
import sys

from nacl.public import PrivateKey
from nacl.pwhash import argon2i
from nacl.secret import SecretBox
from nacl.utils import random as random_bytes


# Hardcoded salt from bittensor-wallet (identical in Rust and Python)
NACL_SALT = b"\x13q\x83\xdf\xf1Z\t\xbc\x9c\x90\xb5Q\x879\xe9\xb1"

# libsodium argon2i SENSITIVE parameters
OPSLIMIT_SENSITIVE = argon2i.OPSLIMIT_SENSITIVE  # 8
MEMLIMIT_SENSITIVE = argon2i.MEMLIMIT_SENSITIVE  # 1073741824


def derive_key(password: bytes) -> bytes:
    """Derive a 32-byte encryption key from password using argon2i."""
    return argon2i.kdf(
        SecretBox.KEY_SIZE,
        password,
        NACL_SALT,
        opslimit=OPSLIMIT_SENSITIVE,
        memlimit=MEMLIMIT_SENSITIVE,
    )


def encrypt_keyfile_data(keyfile_data: bytes, password: str) -> bytes:
    """Encrypt keyfile data in NaCl format: $NACL + nonce + ciphertext."""
    key = derive_key(password.encode("utf-8"))
    box = SecretBox(key)
    # SecretBox.encrypt generates a random nonce and returns nonce + MAC + ciphertext
    encrypted = box.encrypt(keyfile_data)
    return b"$NACL" + encrypted


def decrypt_keyfile_data(keyfile_data: bytes, password: str) -> bytes:
    """Decrypt NaCl-encrypted keyfile data."""
    assert keyfile_data[:5] == b"$NACL", f"Not a NaCl keyfile, starts with {keyfile_data[:10]!r}"
    key = derive_key(password.encode("utf-8"))
    box = SecretBox(key)
    # After stripping $NACL prefix, remaining data is nonce(24B) + MAC(16B) + ciphertext
    return box.decrypt(keyfile_data[5:])


def main():
    password = "testpassword123"
    
    # Known secret key (32 bytes, all zeros except last byte = 1)
    secret_key_hex = "0000000000000000000000000000000000000000000000000000000000000001"
    
    # Create keyfile JSON payload (matches bittensor serialization format)
    keyfile_json = json.dumps({
        "accountId": "0x" + secret_key_hex,
        "publicKey": "0x" + secret_key_hex,
        "secretPhrase": None,
        "secretSeed": "0x" + secret_key_hex,
        "ss58Address": None,
    }).encode("utf-8")
    
    print(f"Plaintext keyfile JSON ({len(keyfile_json)} bytes):")
    print(f"  {keyfile_json.decode()}")
    print()
    
    # Encrypt
    encrypted = encrypt_keyfile_data(keyfile_json, password)
    print(f"Encrypted coldkey ({len(encrypted)} bytes):")
    print(f"  Prefix: {encrypted[:5]}")
    print(f"  Full hex: {encrypted.hex()}")
    print()
    
    # Verify round-trip
    decrypted = decrypt_keyfile_data(encrypted, password)
    assert decrypted == keyfile_json, "Round-trip failed!"
    print("Round-trip verification: PASSED")
    print()
    
    # Write coldkey file
    output_path = os.path.join(os.path.dirname(__file__), "test_coldkey")
    with open(output_path, "wb") as f:
        f.write(encrypted)
    os.chmod(output_path, 0o600)
    print(f"Written coldkey to: {output_path}")
    print(f"Password: {password}")
    print(f"Secret key hex: {secret_key_hex}")
    
    # Also write a metadata file for the Rust test to read
    meta_path = os.path.join(os.path.dirname(__file__), "test_coldkey_meta.json")
    with open(meta_path, "w") as f:
        json.dump({
            "password": password,
            "secret_key_hex": secret_key_hex,
            "coldkey_path": output_path,
            "plaintext_hex": keyfile_json.hex(),
        }, f, indent=2)
    print(f"Written metadata to: {meta_path}")


if __name__ == "__main__":
    main()
