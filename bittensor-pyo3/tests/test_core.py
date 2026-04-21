"""Tests for bittensor_rs core types: Balance, NetworkConfig, BittensorError, and chain data models."""

import pytest
import bittensor_rs as bt


# ===========================================================================
# Balance
# ===========================================================================

class TestBalance:
    def test_from_rao(self):
        b = bt.Balance.from_rao(1_500_000_000)
        assert b.rao == 1_500_000_000
        assert abs(b.tao - 1.5) < 1e-6

    def test_from_tao(self):
        b = bt.Balance.from_tao(2.0)
        assert b.rao == 2_000_000_000
        assert abs(b.tao - 2.0) < 1e-6

    def test_zero(self):
        b = bt.Balance.zero()
        assert b.rao == 0
        assert b.tao == 0.0

    def test_one_tao(self):
        b = bt.Balance.one_tao()
        assert b.rao == 1_000_000_000
        assert abs(b.tao - 1.0) < 1e-6

    def test_str(self):
        b = bt.Balance.from_tao(1.5)
        s = str(b)
        assert '1.5' in s

    def test_repr(self):
        b = bt.Balance.from_rao(100)
        r = repr(b)
        assert 'Balance' in r

    def test_add(self):
        a = bt.Balance.from_rao(500_000_000)
        b = bt.Balance.from_rao(500_000_000)
        c = a + b
        assert c.rao == 1_000_000_000

    def test_sub(self):
        a = bt.Balance.from_rao(1_000_000_000)
        b = bt.Balance.from_rao(300_000_000)
        c = a - b
        assert c.rao == 700_000_000

    def test_mul(self):
        a = bt.Balance.from_rao(500_000_000)
        c = a * 2
        assert c.rao == 1_000_000_000

    def test_truediv(self):
        a = bt.Balance.from_rao(1_000_000_000)
        c = a / 2
        assert c.rao == 500_000_000

    def test_eq(self):
        a = bt.Balance.from_rao(100)
        b = bt.Balance.from_rao(100)
        assert a == b

    def test_lt(self):
        a = bt.Balance.from_rao(50)
        b = bt.Balance.from_rao(100)
        assert a < b

    def test_hash(self):
        a = bt.Balance.from_rao(100)
        b = bt.Balance.from_rao(100)
        assert hash(a) == hash(b)


# ===========================================================================
# NetworkConfig
# ===========================================================================

class TestNetworkConfig:
    def test_finney(self):
        cfg = bt.NetworkConfig.finney()
        assert cfg.name == 'finney'
        assert 'wss://' in cfg.ws_endpoint or 'ws://' in cfg.ws_endpoint
        assert len(cfg.ws_endpoint) > 0

    def test_test(self):
        cfg = bt.NetworkConfig.test()
        assert cfg.name == 'test'
        assert len(cfg.ws_endpoint) > 0

    def test_local(self):
        cfg = bt.NetworkConfig.local()
        assert cfg.name == 'local'
        assert '127.0.0.1' in cfg.ws_endpoint or 'localhost' in cfg.ws_endpoint

    def test_archive(self):
        cfg = bt.NetworkConfig.archive()
        assert cfg.name == 'archive'
        assert len(cfg.ws_endpoint) > 0

    def test_latent_lite(self):
        cfg = bt.NetworkConfig.latent_lite()
        assert cfg.name == 'latent-lite'
        assert len(cfg.ws_endpoint) > 0

    def test_custom_config(self):
        cfg = bt.NetworkConfig('custom', 'ws://localhost:9944')
        assert cfg.name == 'custom'
        assert cfg.ws_endpoint == 'ws://localhost:9944'

    def test_chain_id(self):
        cfg = bt.NetworkConfig.finney()
        assert isinstance(cfg.chain_id, int)

    def test_repr(self):
        cfg = bt.NetworkConfig.finney()
        r = repr(cfg)
        assert 'NetworkConfig' in r


# ===========================================================================
# BittensorError
# ===========================================================================

class TestBittensorError:
    def test_is_exception(self):
        assert issubclass(bt.BittensorError, RuntimeError)

    def test_can_raise(self):
        with pytest.raises(bt.BittensorError, match='test error'):
            raise bt.BittensorError('test error')


# ===========================================================================
# Chain data models — smoke tests for construction
# ===========================================================================

class TestAxonInfo:
    def test_repr(self):
        # AxonInfo is a pyclass — test it can be imported
        assert bt.AxonInfo is not None


class TestStakeInfo:
    def test_exists(self):
        assert bt.StakeInfo is not None


class TestDelegateInfo:
    def test_exists(self):
        assert bt.DelegateInfo is not None


class TestNeuronInfo:
    def test_exists(self):
        assert bt.NeuronInfo is not None


class TestNeuronInfoLite:
    def test_exists(self):
        assert bt.NeuronInfoLite is not None


class TestSubnetInfo:
    def test_exists(self):
        assert bt.SubnetInfo is not None


class TestSubnetHyperparameters:
    def test_exists(self):
        assert bt.SubnetHyperparameters is not None


class TestMetagraphInfo:
    def test_exists(self):
        assert bt.MetagraphInfo is not None


class TestNeuronCertificate:
    def test_exists(self):
        assert bt.NeuronCertificate is not None


class TestPrometheusInfo:
    def test_exists(self):
        assert bt.PrometheusInfo is not None
