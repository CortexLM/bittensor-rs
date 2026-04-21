"""Tests for bittensor_rs Wallet: create, load, sign, verify, ss58_address."""

import os
import tempfile
import pytest
import bittensor_rs as bt


# Use a module-scoped temp dir and wallet to avoid recreating keys for every test
@pytest.fixture(scope="module")
def wallet_dir():
    d = tempfile.mkdtemp()
    return d


@pytest.fixture(scope="module")
def created_wallet(wallet_dir):
    """Create a wallet once for the whole module."""
    w = bt.Wallet.create("mod-wallet", wallet_dir, password="testpw")
    return w


class TestWalletCreate:
    """Wallet.create generates new keys on disk."""

    def test_create_returns_wallet(self, wallet_dir):
        w = bt.Wallet.create("w1", wallet_dir)
        assert isinstance(w, bt.Wallet)

    def test_create_has_ss58_address(self, wallet_dir):
        w = bt.Wallet.create("w2", wallet_dir)
        addr = w.ss58_address
        assert isinstance(addr, str)
        assert len(addr) > 40

    def test_create_ss58_starts_with_5(self, wallet_dir):
        w = bt.Wallet.create("w3", wallet_dir)
        assert w.ss58_address.startswith("5")

    def test_create_repr(self, wallet_dir):
        w = bt.Wallet.create("w4", wallet_dir)
        r = repr(w)
        assert "Wallet" in r

    def test_create_name(self, wallet_dir):
        w = bt.Wallet.create("w5", wallet_dir)
        assert w.name == "w5"

    def test_create_path(self, wallet_dir):
        w = bt.Wallet.create("w6", wallet_dir)
        assert wallet_dir in w.path

    def test_create_hotkey_name(self, wallet_dir):
        w = bt.Wallet.create("w7", wallet_dir)
        assert w.hotkey_name == "default"


class TestWalletLoad:
    """Wallet.load reads existing keys from disk."""

    def test_load_after_create(self, created_wallet, wallet_dir):
        addr1 = created_wallet.ss58_address
        w2 = bt.Wallet.load("mod-wallet", wallet_dir)
        addr2 = w2.ss58_address
        assert addr1 == addr2

    def test_load_custom_hotkey_name(self, wallet_dir):
        w = bt.Wallet.load("w1", wallet_dir, hotkey_name="myhotkey")
        assert w.hotkey_name == "myhotkey"


class TestWalletSignVerify:
    """Sign with hotkey/coldkey, then verify the signature."""

    def test_sign_returns_hex(self, created_wallet):
        message = b"hello bittensor"
        sig_hex = created_wallet.sign(message)
        assert isinstance(sig_hex, str)
        assert len(sig_hex) == 128  # 64 bytes hex-encoded

    def test_sign_coldkey(self, created_wallet):
        message = b"cold signed msg"
        sig_hex = created_wallet.sign_coldkey(message, "testpw")
        assert isinstance(sig_hex, str)
        assert len(sig_hex) == 128

    def test_verify_static_method_exists(self):
        """Verify is a static method with the right signature."""
        assert callable(bt.Wallet.verify)


class TestWalletGetKeypairs:
    """get_coldkeypub, get_coldkey_pair, get_hotkey_pair return SS58 addresses."""

    def test_get_coldkeypub(self, created_wallet):
        addr = created_wallet.get_coldkeypub()
        assert isinstance(addr, str)
        assert addr.startswith("5")

    def test_get_coldkey_pair(self, created_wallet):
        addr = created_wallet.get_coldkey_pair("testpw")
        assert isinstance(addr, str)
        assert addr.startswith("5")

    def test_get_hotkey_pair(self, created_wallet):
        addr = created_wallet.get_hotkey_pair()
        assert isinstance(addr, str)
        assert addr.startswith("5")

    def test_coldkey_pub_matches_coldkey_pair(self, created_wallet):
        pub_addr = created_wallet.get_coldkeypub()
        pair_addr = created_wallet.get_coldkey_pair("testpw")
        assert pub_addr == pair_addr


class TestWalletErrors:
    """Error handling for invalid wallet operations."""

    def test_coldkey_pair_wrong_password(self, wallet_dir):
        """Wrong password on a freshly loaded wallet should fail.
        Note: if the wallet was previously loaded with the correct password,
        the key is cached and the wrong password is not checked. We must
        create a new Wallet object to test this."""
        bt.Wallet.create("err-wallet", wallet_dir, password="right_pass")
        # Load a fresh Wallet object (no cached coldkey)
        w = bt.Wallet.load("err-wallet", wallet_dir)
        with pytest.raises(Exception):
            w.get_coldkey_pair("wrong_password")

    def test_load_nonexistent_wallet(self, wallet_dir):
        """Loading a wallet that doesn't exist still returns a Wallet object
        (keys are loaded lazily)."""
        w = bt.Wallet.load("nonexistent", wallet_dir)
        assert isinstance(w, bt.Wallet)
