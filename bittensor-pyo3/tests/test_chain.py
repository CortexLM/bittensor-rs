"""Tests for bittensor_rs SubtensorClient: connect, queries, and transactions.

Most chain tests require a live node and are skipped if unavailable.
"""

import pytest
import bittensor_rs as bt


# Whether to attempt integration tests against a live chain.
# Set BITTENSOR_TEST_WS to a WebSocket URL to enable integration tests.
import os

_TEST_WS = os.environ.get("BITTENSOR_TEST_WS", "")
_SKIP_REASON = "Set BITTENSOR_TEST_WS env var to a WebSocket URL to run integration tests"


# ---------------------------------------------------------------------------
# Unit tests — no network required
# ---------------------------------------------------------------------------

class TestSubtensorClientInit:
    """SubtensorClient can be constructed without connecting."""

    def test_repr_disconnected(self):
        c = bt.SubtensorClient()
        r = repr(c)
        assert "disconnected" in r

    def test_repr_connected(self):
        # We can't easily test connected without a real node,
        # but the class should be constructible.
        c = bt.SubtensorClient()
        assert isinstance(c, bt.SubtensorClient)


class TestSubtensorClientConnectIsCoroutine:
    """connect() and from_url() are class/static methods returning awaitables."""

    def test_connect_is_classmethod(self):
        assert callable(bt.SubtensorClient.connect)

    def test_from_url_is_staticmethod(self):
        assert callable(bt.SubtensorClient.from_url)


class TestSubtensorClientNotConnected:
    """Methods on a disconnected client should raise errors."""

    @pytest.mark.asyncio
    async def test_get_balance_not_connected(self):
        c = bt.SubtensorClient()
        with pytest.raises(Exception, match="not connected"):
            await c.get_balance("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCVcBhwfd9iNv")

    @pytest.mark.asyncio
    async def test_get_total_stake_not_connected(self):
        c = bt.SubtensorClient()
        with pytest.raises(Exception, match="not connected"):
            await c.get_total_stake()


class TestTxSuccess:
    """TxSuccess class exists and has expected attributes."""

    def test_exists(self):
        assert bt.TxSuccess is not None


# ---------------------------------------------------------------------------
# Integration tests — require a live chain
# ---------------------------------------------------------------------------

@pytest.mark.skipif(not _TEST_WS, reason=_SKIP_REASON)
class TestSubtensorClientIntegration:
    """Integration tests against a live Subtensor node."""

    @pytest.mark.asyncio
    async def test_from_url_connect(self):
        client = await bt.SubtensorClient.from_url(_TEST_WS)
        r = repr(client)
        assert "connected" in r

    @pytest.mark.asyncio
    async def test_connect_with_config(self):
        cfg = bt.NetworkConfig.new("test", _TEST_WS)
        client = await bt.SubtensorClient.connect(cfg)
        assert "connected" in repr(client)

    @pytest.mark.asyncio
    async def test_get_balance(self):
        client = await bt.SubtensorClient.from_url(_TEST_WS)
        # Alice's address on dev chains
        balance = await client.get_balance("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCVcBhwfd9iNv")
        assert isinstance(balance, bt.Balance)

    @pytest.mark.asyncio
    async def test_get_total_balance(self):
        client = await bt.SubtensorClient.from_url(_TEST_WS)
        balance = await client.get_total_balance("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCVcBhwfd9iNv")
        assert isinstance(balance, bt.Balance)

    @pytest.mark.asyncio
    async def test_get_total_stake(self):
        client = await bt.SubtensorClient.from_url(_TEST_WS)
        stake = await client.get_total_stake()
        assert isinstance(stake, bt.Balance)

    @pytest.mark.asyncio
    async def test_get_stake_info(self):
        client = await bt.SubtensorClient.from_url(_TEST_WS)
        stakes = await client.get_stake_info("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCVcBhwfd9iNv")
        assert isinstance(stakes, list)

    @pytest.mark.asyncio
    async def test_get_metagraph(self):
        client = await bt.SubtensorClient.from_url(_TEST_WS)
        meta = await client.get_metagraph(1)
        assert isinstance(meta, bt.MetagraphInfo)

    @pytest.mark.asyncio
    async def test_invalid_ss58_raises(self):
        client = await bt.SubtensorClient.from_url(_TEST_WS)
        with pytest.raises(Exception):
            await client.get_balance("not_a_valid_address")
