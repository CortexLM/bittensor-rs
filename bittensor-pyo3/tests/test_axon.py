"""Tests for Axon and AxonConfig Python bindings."""

import pytest


class TestAxonConfig:
    """Test AxonConfig construction and property access."""

    def test_default_construction(self):
        from bittensor_rs import AxonConfig

        cfg = AxonConfig()
        assert cfg.ip == "0.0.0.0"
        assert cfg.port == 8090
        assert cfg.max_connections == 0
        assert cfg.external_ip is None
        assert cfg.hotkey is None

    def test_construction_with_args(self):
        from bittensor_rs import AxonConfig

        cfg = AxonConfig(
            ip="127.0.0.1",
            port=3000,
            max_connections=100,
            external_ip="1.2.3.4",
            hotkey="5Hotkey123",
        )
        assert cfg.ip == "127.0.0.1"
        assert cfg.port == 3000
        assert cfg.max_connections == 100
        assert cfg.external_ip == "1.2.3.4"
        assert cfg.hotkey == "5Hotkey123"

    def test_setters(self):
        from bittensor_rs import AxonConfig

        cfg = AxonConfig()
        cfg.ip = "10.0.0.1"
        assert cfg.ip == "10.0.0.1"
        cfg.port = 9999
        assert cfg.port == 9999
        cfg.max_connections = 50
        assert cfg.max_connections == 50
        cfg.external_ip = "5.6.7.8"
        assert cfg.external_ip == "5.6.7.8"
        cfg.hotkey = "5NewHotkey"
        assert cfg.hotkey == "5NewHotkey"

    def test_repr(self):
        from bittensor_rs import AxonConfig

        cfg = AxonConfig(ip="0.0.0.0", port=8090)
        r = repr(cfg)
        assert "AxonConfig" in r


class TestAxon:
    """Test Axon server construction, attach, start/stop."""

    def test_default_construction(self):
        from bittensor_rs import Axon

        axon = Axon()
        assert "Axon" in repr(axon)

    def test_construction_with_config(self):
        from bittensor_rs import Axon, AxonConfig

        cfg = AxonConfig(port=0)
        axon = Axon(cfg)
        assert "Axon" in repr(axon)

    def test_attach_handler(self):
        from bittensor_rs import Axon, AxonConfig

        cfg = AxonConfig(port=0)
        axon = Axon(cfg)
        # Should not raise
        axon.attach("TextPrompt", lambda body: {"response": "ok"})

    @pytest.mark.asyncio
    async def test_start_stop(self):
        from bittensor_rs import Axon, AxonConfig

        cfg = AxonConfig(port=0)
        axon = Axon(cfg)
        axon.attach("TextPrompt", lambda body: {"echo": True})

        addr = await axon.start()
        assert isinstance(addr, str)
        assert ":" in addr  # Should be "ip:port" format

        # Stop should succeed
        axon.stop(addr)

    @pytest.mark.asyncio
    async def test_blacklist_unblacklist(self):
        from bittensor_rs import Axon, AxonConfig

        cfg = AxonConfig(port=0)
        axon = Axon(cfg)
        # Should not raise
        await axon.blacklist("5BadActor")
        await axon.unblacklist("5BadActor")

    @pytest.mark.asyncio
    async def test_set_priority(self):
        from bittensor_rs import Axon, AxonConfig

        cfg = AxonConfig(port=0)
        axon = Axon(cfg)
        # Should not raise
        await axon.set_priority("5Hotkey", 10)
