"""Tests for Synapse, TerminalInfo, and StreamingSynapse Python bindings."""

import pytest


class TestTerminalInfo:
    """Test TerminalInfo construction, getters/setters, and serialization."""

    def test_default_construction(self):
        from bittensor_rs import TerminalInfo

        ti = TerminalInfo()
        assert ti.status_code is None
        assert ti.status_message is None
        assert ti.process_time is None
        assert ti.ip is None
        assert ti.port is None
        assert ti.version is None
        assert ti.nonce is None
        assert ti.uuid is None
        assert ti.hotkey is None
        assert ti.signature is None

    def test_construction_with_args(self):
        from bittensor_rs import TerminalInfo

        ti = TerminalInfo(
            status_code=200,
            status_message="OK",
            process_time=0.5,
            ip="127.0.0.1",
            port=8090,
            version=42,
            nonce=123456789,
            uuid="test-uuid",
            hotkey="5Hotkey",
            signature="0xsig",
        )
        assert ti.status_code == 200
        assert ti.status_message == "OK"
        assert ti.process_time == pytest.approx(0.5)
        assert ti.ip == "127.0.0.1"
        assert ti.port == 8090
        assert ti.version == 42
        assert ti.nonce == 123456789
        assert ti.uuid == "test-uuid"
        assert ti.hotkey == "5Hotkey"
        assert ti.signature == "0xsig"

    def test_setters(self):
        from bittensor_rs import TerminalInfo

        ti = TerminalInfo()
        ti.status_code = 404
        assert ti.status_code == 404
        ti.status_message = "Not Found"
        assert ti.status_message == "Not Found"
        ti.process_time = 1.23
        assert ti.process_time == pytest.approx(1.23)
        ti.ip = "10.0.0.1"
        assert ti.ip == "10.0.0.1"
        ti.port = 3000
        assert ti.port == 3000
        ti.version = 99
        assert ti.version == 99
        ti.nonce = 999
        assert ti.nonce == 999
        ti.uuid = "new-uuid"
        assert ti.uuid == "new-uuid"
        ti.hotkey = "5NewHotkey"
        assert ti.hotkey == "5NewHotkey"
        ti.signature = "0xnewsig"
        assert ti.signature == "0xnewsig"

    def test_to_headers_with_prefix(self):
        from bittensor_rs import TerminalInfo

        ti = TerminalInfo(status_code=200, nonce=42, hotkey="5Test")
        headers = ti.to_headers("bt_header_axon_")
        assert "bt_header_axon_status_code" in headers
        assert headers["bt_header_axon_status_code"] == "200"
        assert "bt_header_axon_nonce" in headers
        assert headers["bt_header_axon_nonce"] == "42"
        assert "bt_header_axon_hotkey" in headers
        assert headers["bt_header_axon_hotkey"] == "5Test"
        # None fields should not appear
        assert "bt_header_axon_ip" not in headers

    def test_from_headers_roundtrip(self):
        from bittensor_rs import TerminalInfo

        original = TerminalInfo(
            status_code=200,
            status_message="Success",
            process_time=0.1,
            ip="198.123.23.1",
            port=9282,
            version=111,
            nonce=111111,
            uuid="5ecbd69c-1cec-11ee-b0dc-e29ce36fec1a",
            hotkey="5EnjDGNqqWnuL2HCAdxeEtN2oqtXZw6BMBe936Kfy2PFz1J1",
            signature="0xsig",
        )
        headers = original.to_headers("bt_header_axon_")
        restored = TerminalInfo.from_headers(headers, "bt_header_axon_")
        assert restored.status_code == 200
        assert restored.status_message == "Success"
        assert restored.process_time == pytest.approx(0.1)
        assert restored.ip == "198.123.23.1"
        assert restored.port == 9282
        assert restored.version == 111
        assert restored.nonce == 111111
        assert restored.uuid == "5ecbd69c-1cec-11ee-b0dc-e29ce36fec1a"
        assert restored.hotkey == "5EnjDGNqqWnuL2HCAdxeEtN2oqtXZw6BMBe936Kfy2PFz1J1"
        assert restored.signature == "0xsig"

    def test_repr(self):
        from bittensor_rs import TerminalInfo

        ti = TerminalInfo(status_code=200)
        r = repr(ti)
        assert "TerminalInfo" in r


class TestSynapse:
    """Test Synapse construction, getters/setters, and serialization."""

    def test_default_construction(self):
        from bittensor_rs import Synapse

        s = Synapse()
        assert s.name == "Synapse"
        assert s.timeout == pytest.approx(12.0)
        assert s.computed_body_hash == ""
        assert s.total_size == 0
        assert s.header_size == 0

    def test_construction_with_args(self):
        from bittensor_rs import Synapse

        s = Synapse(name="TextPrompt", timeout=30.0)
        assert s.name == "TextPrompt"
        assert s.timeout == pytest.approx(30.0)

    def test_setters(self):
        from bittensor_rs import Synapse

        s = Synapse()
        s.name = "TextEncoding"
        assert s.name == "TextEncoding"
        s.timeout = 60.0
        assert s.timeout == pytest.approx(60.0)
        s.computed_body_hash = "abc123"
        assert s.computed_body_hash == "abc123"
        s.total_size = 1024
        assert s.total_size == 1024
        s.header_size = 256
        assert s.header_size == 256

    def test_dendrite_axon_getters(self):
        from bittensor_rs import Synapse, TerminalInfo

        s = Synapse()
        # Default: empty TerminalInfo
        assert isinstance(s.dendrite, TerminalInfo)
        assert isinstance(s.axon, TerminalInfo)

        # Set new TerminalInfo
        ti = TerminalInfo(status_code=200, hotkey="5TestHotkey")
        s.axon = ti
        assert s.axon.status_code == 200
        assert s.axon.hotkey == "5TestHotkey"

    def test_body_hash(self):
        from bittensor_rs import Synapse

        # SHA3-256 of empty bytes
        result = Synapse.body_hash(b"")
        assert result == "a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a"

        # SHA3-256 of "hello"
        result = Synapse.body_hash(b"hello")
        assert len(result) == 64  # hex-encoded 32 bytes

    def test_to_headers(self):
        from bittensor_rs import Synapse, TerminalInfo

        s = Synapse(name="TextPrompt", timeout=12.0)
        s.computed_body_hash = "abc123"
        headers = s.to_headers()
        assert "name" in headers
        assert headers["name"] == "TextPrompt"
        assert "timeout" in headers
        assert "computed_body_hash" in headers
        assert headers["computed_body_hash"] == "abc123"

    def test_from_headers(self):
        from bittensor_rs import Synapse

        headers = {
            "name": "MySynapse",
            "timeout": "5.0",
            "computed_body_hash": "deadbeef",
            "total_size": "999",
        }
        s = Synapse.from_headers(headers)
        assert s.name == "MySynapse"
        assert s.timeout == pytest.approx(5.0)
        assert s.computed_body_hash == "deadbeef"
        assert s.total_size == 999

    def test_repr(self):
        from bittensor_rs import Synapse

        s = Synapse(name="TextPrompt", timeout=12.0)
        r = repr(s)
        assert "Synapse" in r
        assert "TextPrompt" in r


class TestStreamingSynapse:
    """Test StreamingSynapse construction and basic behavior."""

    def test_default_construction(self):
        from bittensor_rs import StreamingSynapse

        ss = StreamingSynapse()
        assert ss.name == "StreamingSynapse"
        assert ss.timeout == pytest.approx(12.0)

    def test_construction_with_args(self):
        from bittensor_rs import StreamingSynapse

        ss = StreamingSynapse(name="TextStreaming", timeout=30.0)
        assert ss.name == "TextStreaming"
        assert ss.timeout == pytest.approx(30.0)

    def test_setters(self):
        from bittensor_rs import StreamingSynapse

        ss = StreamingSynapse()
        ss.name = "NewName"
        assert ss.name == "NewName"
        ss.timeout = 60.0
        assert ss.timeout == pytest.approx(60.0)

    def test_process_chunk(self):
        from bittensor_rs import StreamingSynapse

        ss = StreamingSynapse()
        # Default process_chunk returns UTF-8 string
        result = ss.process_chunk(b"hello")
        assert result == "hello"

    def test_to_headers(self):
        from bittensor_rs import StreamingSynapse

        ss = StreamingSynapse(name="TestStream", timeout=5.0)
        headers = ss.to_headers()
        assert headers["name"] == "TestStream"

    def test_repr(self):
        from bittensor_rs import StreamingSynapse

        ss = StreamingSynapse(name="TestStream")
        r = repr(ss)
        assert "StreamingSynapse" in r
