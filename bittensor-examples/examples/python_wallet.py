"""
Example: Wallet creation, balance query, and transfer from Python.

This example requires: pip install bittensor_rs

Usage:
    python python_wallet.py
"""

import asyncio
from bittensor_rs import Wallet, SubtensorClient


async def main():
    # Create a wallet (keys are generated on disk if they don't exist)
    wallet = Wallet.create("my-wallet", "/tmp/bt-wallets", password="secret")
    print(f"Coldkey address: {wallet.ss58_address}")

    # Connect to the Finney network
    client = await SubtensorClient.from_url("wss://entrypoint-finney.opentensor.ai:443")

    # Query free balance
    balance = await client.get_balance(wallet.ss58_address)
    print(f"Free balance: {balance}")

    # Transfer example (commented out — requires real funds)
    # result = await client.transfer(
    #     dest="5DestAddress...",
    #     amount=1_000_000_000,  # 1 TAO in rao
    #     signer="word1 word2 word3 ... word12",
    #     password=None,
    # )
    # print(f"Transfer result: {result}")


if __name__ == "__main__":
    asyncio.run(main())
