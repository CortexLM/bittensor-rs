#!/usr/bin/env python3
"""Verify that a Rust-encrypted coldkey can be decrypted by Python.

This validates the Rust → Python direction of NaCl compatibility.
Uses the same parameters as btwallet: argon2i SENSITIVE, hardcoded salt, XSalsa20-Poly1305.
"""

import json
import sys
import os

from nacl.pwhash import argon2i
from nacl.secret import SecretBox


NACL_SALT = b"\x13q\x83\xdf\xf1Z\t\xbc\x9c\x90\xb5Q\x879\xe9\xb1"


def decrypt_coldkey(data: bytes, password: str) -> bytes:
    assert data[:5] == b"$NACL", f"Not a NaCl coldkey: {data[:10]!r}"
    key = argon2i.kdf(
        SecretBox.KEY_SIZE,
        password.encode("utf-8"),
        NACL_SALT,
        opslimit=argon2i.OPSLIMIT_SENSITIVE,
        memlimit=argon2i.MEMLIMIT_SENSITIVE,
    )
    box = SecretBox(key)
    return box.decrypt(data[5:])


def main():
    script_dir = os.path.dirname(os.path.abspath(__file__))
    meta_path = os.path.join(script_dir, "rust_coldkey_meta.json")

    if not os.path.exists(meta_path):
        print("ERROR: rust_coldkey_meta.json not found.")
        print("Run `cargo run -p bittensor-wallet --example create_test_coldkey` first.")
        sys.exit(1)

    with open(meta_path) as f:
        meta = json.load(f)

    coldkey_path = meta["coldkey_path"]
    password = meta["password"]
    expected_secret_hex = meta["secret_key_hex"]

    print("=== Verifying Rust-Created Coldkey in Python ===\n")
    print(f"Coldkey path: {coldkey_path}")
    print(f"Password:     {password}")
    print(f"Expected key: {expected_secret_hex}\n")

    with open(coldkey_path, "rb") as f:
        encrypted = f.read()

    print(f"Encrypted data ({len(encrypted)} bytes), prefix: {encrypted[:5]!r}")

    if encrypted[:5] != b"$NACL":
        print("ERROR: Data does not have $NACL prefix!")
        sys.exit(1)
    print("✓ $NACL prefix confirmed\n")

    decrypted = decrypt_coldkey(encrypted, password)
    decrypted_str = decrypted.decode("utf-8")
    print(f"Decrypted JSON ({len(decrypted)} bytes):")
    print(f"  {decrypted_str}\n")

    if expected_secret_hex in decrypted_str:
        print(f"✓ Secret key match CONFIRMED: {expected_secret_hex}")
        print("\n=== Rust → Python NaCl compatibility: PASSED ===")
    else:
        print("✗ Secret key NOT found in decrypted data!")
        sys.exit(1)


if __name__ == "__main__":
    main()
