# btcli-rs Command Reference

The `btcli-rs` binary is the Rust CLI for Bittensor network operations. It provides wallet management, staking, transfers, registration, subnet operations, weight setting, metagraph queries, and MEV-shielded transactions. Every subcommand matches the Python `btcli` interface, with the same workflow and flag semantics where possible.

## Installation

Install from crates.io:

```sh
cargo install bittensor-cli
```

Or with the MEV-shield feature enabled:

```sh
cargo install bittensor-cli --features mev
```

Build from source:

```sh
git clone https://github.com/opentensor/bittensor-rs.git
cd bittensor-rs
cargo build -p bittensor-cli --release
# binary at target/release/btcli-rs
```

The `mev` feature flag enables the `mev` subcommand for encrypted extrinsic submission using ML-KEM-768. Without it, the `mev` subcommand is omitted from the CLI at compile time.

## Global Flags

All subcommands accept these global flags. Each flag overrides the corresponding field in the on-disk config file (`~/.bittensor/config.yml`).

| Flag | Type | Default | Description |
|---|---|---|---|
| `--network` | string | `finney` | Network to connect to: `finney`, `test`, `local`, `archive`, `latent-lite` |
| `--wallet.name` | string | `default` | Wallet name (directory under wallet path) |
| `--wallet.path` | string | `~/.bittensor/wallets/` | Base directory for wallet storage |

### Config Resolution Order

1. Load `~/.bittensor/config.yml` if it exists
2. Apply `--network` flag (overrides `network` in config)
3. Apply `--wallet.name` flag (overrides `wallet_name` in config)
4. Apply `--wallet.path` flag (overrides `wallet_path` in config)

If no config file exists and no flags are provided, the CLI defaults to finney with a wallet named `default` stored under `~/.bittensor/wallets/`.

### Network Endpoints

| Network | WebSocket URL | Alias |
|---|---|---|
| `finney` | `wss://entrypoint-finney.opentensor.ai:443` | `mainnet` |
| `test` | `wss://test.finney.opentensor.ai:443` | `testnet` |
| `local` | `ws://127.0.0.1:9944` | - |
| `archive` | `wss://archive.finney.opentensor.ai:443` | - |
| `latent-lite` | Latent-lite endpoint | - |

### Python btcli Comparison

| btcli-rs flag | Python btcli flag | Notes |
|---|---|---|
| `--network finney` | `--subtensor.network finney` | Same effect, different flag path |
| `--wallet.name my-wallet` | `--wallet.name my-wallet` | Identical |
| `--wallet.path /tmp/wallets` | `--wallet.path /tmp/wallets` | Identical |
| (no equivalent) | `--wallet.hotkey default` | Hotkey selected via `wallet create-hotkey --hotkey` |

---

## Wallet Commands

Wallet commands manage coldkey and hotkey keyfiles on disk. Coldkeys are encrypted with NaCl secretbox. Hotkeys are stored as plaintext seed files under the `hotkeys/` subdirectory. The keyfile format is cross-compatible with the Python SDK.

### wallet create

Generate a new coldkey (encrypted) and a default hotkey.

```sh
btcli-rs wallet create [--no-password] [--password <PASSWORD>]
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `--no-password` | no | - | Skip password prompt, use empty password |
| `--password` | no | prompt | Password to encrypt the coldkey |

Output includes the wallet path, coldkey SS58 address, hotkey SS58 address, and the mnemonic phrase. The mnemonic is displayed once and must be stored offline.

```sh
btcli-rs wallet create --no-password
btcli-rs --wallet.name miner-01 wallet create --password "my-secret"
```

Python equivalent:

```sh
btcli wallet create --wallet.name miner-01
```

---

### wallet list

List all wallets found in the wallet base directory.

```sh
btcli-rs wallet list
```

Displays each wallet name with marker flags: `coldkey`, `coldkeypub`, and a count of hotkeys.

```sh
btcli-rs --wallet.path ~/.bittensor/wallets wallet list
```

Python equivalent:

```sh
btcli wallet list
```

---

### wallet show

Display wallet details: coldkeypub address and all hotkey SS58 addresses.

```sh
btcli-rs wallet show [--password <PASSWORD>]
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `--password` | no | prompt | Password to decrypt coldkey (shows full coldkey SS58) |

```sh
btcli-rs --wallet.name validator-1 wallet show
```

Python equivalent:

```sh
btcli wallet overview --wallet.name validator-1
```

---

### wallet balance

Show the TAO balance for a wallet or all wallets.

```sh
btcli-rs wallet balance [--all] [--password <PASSWORD>]
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `--all` | no | false | Show balances for every wallet in the base directory |
| `--password` | no | prompt | Password to decrypt coldkey |

```sh
btcli-rs wallet balance
btcli-rs wallet balance --all
```

Python equivalent:

```sh
btcli wallet balance --wallet.name default
btcli wallet balance --all
```

---

### wallet overview

Comprehensive wallet overview showing address, balance, and all stakes per hotkey.

```sh
btcli-rs wallet overview [--all] [--password <PASSWORD>]
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `--all` | no | false | Show overview for every wallet |
| `--password` | no | prompt | Password to decrypt coldkey |

```sh
btcli-rs wallet overview
btcli-rs wallet overview --all
```

Python equivalent:

```sh
btcli wallet overview --wallet.name default
```

---

### wallet transfer

Transfer TAO to another SS58 address.

```sh
btcli-rs wallet transfer <DEST> <AMOUNT> [--password <PASSWORD>]
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `DEST` | yes | - | Destination SS58 address |
| `AMOUNT` | yes | - | Amount in TAO (e.g. `1.5`) |
| `--password` | no | prompt | Password to decrypt coldkey |

```sh
btcli-rs wallet transfer 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY 10.0
```

Python equivalent:

```sh
btcli wallet transfer --dest 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY --amount 10.0
```

---

### wallet swap-coldkey

Initiate a coldkey swap to a new SS58 address. This submits the `swap_coldkey_announced` extrinsic.

```sh
btcli-rs wallet swap-coldkey <NEW_COLDKEY> [--password <PASSWORD>]
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `NEW_COLDKEY` | yes | - | New coldkey SS58 address |
| `--password` | no | prompt | Password to decrypt current coldkey |

```sh
btcli-rs wallet swap-coldkey 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty
```

---

### wallet inspect

Show all keys and addresses in the wallet, including hex-encoded public keys when the coldkey is decrypted.

```sh
btcli-rs wallet inspect [--password <PASSWORD>]
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `--password` | no | prompt | Password to decrypt coldkey for full details |

```sh
btcli-rs wallet inspect --password "secret"
```

---

### wallet regen-coldkey

Regenerate the coldkey from a mnemonic phrase. Overwrites the existing coldkey file.

```sh
btcli-rs wallet regen-coldkey <MNEMONIC> [--password <PASSWORD>] [--yes]
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `MNEMONIC` | yes | - | Space-separated mnemonic words |
| `--password` | no | prompt | Password for the new coldkey |
| `--yes` | no | false | Skip confirmation prompt |

Without `--yes`, the command prints a warning and waits for Enter before overwriting.

```sh
btcli-rs wallet regen-coldkey "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about" --yes
```

---

### wallet regen-coldkeypub

Regenerate the coldkeypub file from an SS58 address (no private key stored).

```sh
btcli-rs wallet regen-coldkeypub <SS58_ADDRESS>
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `SS58_ADDRESS` | yes | - | SS58 address of the coldkeypub |

```sh
btcli-rs wallet regen-coldkeypub 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
```

---

### wallet create-hotkey

Create a new hotkey under the wallet.

```sh
btcli-rs wallet create-hotkey [--hotkey <NAME>] [--password <PASSWORD>] [--seed]
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `--hotkey` | no | `default` | Name for the hotkey |
| `--password` | no | prompt | Coldkey password (for derived hotkeys) |
| `--seed` | no | false | Generate a seed-based (random) hotkey instead of derived |

Without `--seed`, the hotkey is derived from the coldkey and requires the coldkey password. With `--seed`, a random Sr25519 keypair is generated independently.

```sh
btcli-rs wallet create-hotkey --hotkey validator --seed
btcli-rs wallet create-hotkey --hotkey miner
```

---

### wallet regen-hotkey

Regenerate a hotkey from a mnemonic phrase.

```sh
btcli-rs wallet regen-hotkey <MNEMONIC> [--hotkey <NAME>]
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `MNEMONIC` | yes | - | Space-separated mnemonic words |
| `--hotkey` | no | `default` | Name for the hotkey |

```sh
btcli-rs wallet regen-hotkey "word1 word2 ... word12" --hotkey recovered-hk
```

---

## Stake Commands

Stake commands manage TAO staking, unstaking, moving, and swapping stake across hotkeys and subnets.

### stake add

Stake TAO to a hotkey on a subnet.

```sh
btcli-rs stake add --hotkey <HOTKEY_SS58> [--netuid <NETUID>] <AMOUNT> [--password <PASSWORD>]
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `--hotkey` | yes | - | Hotkey SS58 address to stake to |
| `--netuid` | no | `0` | Subnet netuid |
| `AMOUNT` | yes | - | Amount in TAO (e.g. `5.0`) |
| `--password` | no | prompt | Coldkey password |

```sh
btcli-rs stake add --hotkey 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY --netuid 1 5.0
```

Python equivalent:

```sh
btcli stake add --hotkey_ss58 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY --amount 5.0 --netuid 1
```

---

### stake remove

Unstake TAO from a hotkey on a subnet.

```sh
btcli-rs stake remove --hotkey <HOTKEY_SS58> [--netuid <NETUID>] <AMOUNT> [--password <PASSWORD>]
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `--hotkey` | yes | - | Hotkey SS58 address to unstake from |
| `--netuid` | no | `0` | Subnet netuid |
| `AMOUNT` | yes | - | Amount in TAO |
| `--password` | no | prompt | Coldkey password |

```sh
btcli-rs stake remove --hotkey 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY 2.5
```

---

### stake move

Move stake between hotkeys (same or different subnets).

```sh
btcli-rs stake move --origin-hotkey <ORIGIN> --destination-hotkey <DEST> [--origin-netuid <N>] [--destination-netuid <N>] <AMOUNT> [--password <PASSWORD>]
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `--origin-hotkey` | yes | - | Origin hotkey SS58 |
| `--destination-hotkey` | yes | - | Destination hotkey SS58 |
| `--origin-netuid` | no | `0` | Origin subnet |
| `--destination-netuid` | no | `0` | Destination subnet |
| `AMOUNT` | yes | - | Amount in TAO |
| `--password` | no | prompt | Coldkey password |

```sh
btcli-rs stake move --origin-hotkey 5OriginHK --destination-hotkey 5DestHK --origin-netuid 1 --destination-netuid 2 10.0
```

---

### stake swap

Swap stake between subnets for the same hotkey.

```sh
btcli-rs stake swap --hotkey <HOTKEY_SS58> [--origin-netuid <N>] [--destination-netuid <N>] <AMOUNT> [--password <PASSWORD>]
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `--hotkey` | yes | - | Hotkey SS58 |
| `--origin-netuid` | no | `0` | Origin subnet |
| `--destination-netuid` | no | `0` | Destination subnet |
| `AMOUNT` | yes | - | Amount in TAO |
| `--password` | no | prompt | Coldkey password |

```sh
btcli-rs stake swap --hotkey 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY --origin-netuid 1 --destination-netuid 5 3.0
```

---

### stake list

List all stakes for the wallet's coldkey.

```sh
btcli-rs stake list
```

```sh
btcli-rs stake list
```

Python equivalent:

```sh
btcli stake list --wallet.name default
```

---

### stake get-stake

Query the stake for a specific hotkey on a subnet.

```sh
btcli-rs stake get-stake --hotkey <HOTKEY_SS58> [--netuid <NETUID>]
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `--hotkey` | yes | - | Hotkey SS58 |
| `--netuid` | no | `0` | Subnet netuid |

```sh
btcli-rs stake get-stake --hotkey 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY --netuid 1
```

---

### stake set-auto-stake

Enable or disable auto-staking for a hotkey on a subnet.

```sh
btcli-rs stake set-auto-stake --hotkey <HOTKEY_SS58> [--netuid <NETUID>] [--password <PASSWORD>]
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `--hotkey` | yes | - | Hotkey SS58 |
| `--netuid` | no | `0` | Subnet netuid |
| `--password` | no | prompt | Coldkey password |

```sh
btcli-rs stake set-auto-stake --hotkey 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY --netuid 1
```

---

## Transfer Commands

### transfer transfer

Transfer TAO to a single destination.

```sh
btcli-rs transfer transfer <DEST> <AMOUNT> [--password <PASSWORD>]
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `DEST` | yes | - | Destination SS58 address |
| `AMOUNT` | yes | - | Amount in TAO |
| `--password` | no | prompt | Coldkey password |

```sh
btcli-rs transfer transfer 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty 1.5
```

Python equivalent:

```sh
btcli transfer --dest 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty --amount 1.5
```

---

### transfer multiple

Batch transfer TAO to multiple recipients.

```sh
btcli-rs transfer multiple --destinations <ADDRESSES> --amounts <AMOUNTS> [--password <PASSWORD>]
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `--destinations` | yes | - | Comma-separated SS58 addresses |
| `--amounts` | yes | - | Comma-separated TAO amounts (one per destination) |
| `--password` | no | prompt | Coldkey password |

The number of destinations must equal the number of amounts.

```sh
btcli-rs transfer multiple --destinations 5AddrA,5AddrB,5AddrC --amounts 1.0,2.5,0.5
```

---

## Registration Commands

### register register

Register on a subnet via Proof-of-Work. The CLI fetches the current block number, block hash, and subnet difficulty from the chain, then solves the POW locally and submits the registration extrinsic.

```sh
btcli-rs register register [--netuid <NETUID>] [--password <PASSWORD>]
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `--netuid` | no | `1` | Subnet netuid to register on |
| `--password` | no | prompt | Coldkey password |

```sh
btcli-rs register register --netuid 3
```

Python equivalent:

```sh
btcli register --netuid 3 --subtensor.network finney
```

---

### register burned-register

Register on a subnet by burning TAO.

```sh
btcli-rs register burned-register [--netuid <NETUID>] [--password <PASSWORD>]
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `--netuid` | no | `1` | Subnet netuid |
| `--password` | no | prompt | Coldkey password |

```sh
btcli-rs register burned-register --netuid 7
```

Python equivalent:

```sh
btcli register --netuid 7 --subtensor.register_burned
```

---

### register root-register

Register on the root network.

```sh
btcli-rs register root-register [--password <PASSWORD>]
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `--password` | no | prompt | Coldkey password |

```sh
btcli-rs register root-register
```

---

## Root Commands

The `root` subcommand group provides root network operations as a top-level command group, separate from the `register` group.

### root register

Register on the root network (equivalent to `register root-register`).

```sh
btcli-rs root register [--password <PASSWORD>]
```

```sh
btcli-rs root register
```

---

### root set-weights

Set weights on the root network.

```sh
btcli-rs root set-weights --netuid <NETUID> <DESTS> <WEIGHTS> [--version-key <V>] [--password <PASSWORD>]
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `--netuid` | yes | - | Netuid for the weights |
| `DESTS` | yes | - | Comma-separated destination UIDs (e.g. `1,2,3`) |
| `WEIGHTS` | yes | - | Comma-separated weight values (e.g. `100,200,300`) |
| `--version-key` | no | `0` | Version key |
| `--password` | no | prompt | Coldkey password |

```sh
btcli-rs root set-weights --netuid 1 1,2,3 100,200,300 --version-key 42
```

---

### root get-weights

Get weights set by a UID on the root network.

```sh
btcli-rs root get-weights --netuid <NETUID> <UID>
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `--netuid` | yes | - | Netuid |
| `UID` | yes | - | UID to query |

```sh
btcli-rs root get-weights --netuid 1 5
```

---

### root claim

Claim root authority for subnets.

```sh
btcli-rs root claim <SUBNETS> [--password <PASSWORD>]
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `SUBNETS` | yes | - | Comma-separated subnet IDs (e.g. `1,3,7`) |
| `--password` | no | prompt | Coldkey password |

```sh
btcli-rs root claim 1,3,7
```

---

## Subnet Commands

### subnet create

Create a new subnet.

```sh
btcli-rs subnet create [--password <PASSWORD>]
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `--password` | no | prompt | Coldkey password |

```sh
btcli-rs subnet create
```

Python equivalent:

```sh
btcli subnet create --wallet.name default
```

---

### subnet list

List all subnets on the network.

```sh
btcli-rs subnet list
```

```sh
btcli-rs --network finney subnet list
```

---

### subnet info

Show detailed information about a subnet, including owner hotkey, tempo, maximum UID, and identity details.

```sh
btcli-rs subnet info --netuid <NETUID>
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `--netuid` | yes | - | Subnet netuid |

```sh
btcli-rs subnet info --netuid 18
```

---

### subnet hyperparameters

Show all hyperparameters for a subnet: rho, kappa, difficulty, burn, immunity ratio, min/max burn, weights rate limit, weights version, max weight limit, scaling law power, subnetwork N, max N, blocks since last step, tempo, adjustment alpha, adjustment interval, bonds moving avg, alpha high, alpha low, and liquid alpha enabled.

```sh
btcli-rs subnet hyperparameters --netuid <NETUID>
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `--netuid` | yes | - | Subnet netuid |

```sh
btcli-rs subnet hyperparameters --netuid 1
```

---

### subnet set-identity

Set the identity metadata for a subnet.

```sh
btcli-rs subnet set-identity --netuid <NETUID> --name <NAME> --github-repo <REPO> --contact <CONTACT> --url <URL> --discord <DISCORD> --description <DESC> [--password <PASSWORD>]
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `--netuid` | yes | - | Subnet netuid |
| `--name` | yes | - | Subnet name |
| `--github-repo` | yes | - | GitHub repository URL |
| `--contact` | yes | - | Contact information |
| `--url` | yes | - | Subnet URL |
| `--discord` | yes | - | Discord link |
| `--description` | yes | - | Subnet description |
| `--password` | no | prompt | Coldkey password |

```sh
btcli-rs subnet set-identity \
  --netuid 5 \
  --name "my-subnet" \
  --github-repo "https://github.com/example/repo" \
  --contact "admin@example.com" \
  --url "https://example.com" \
  --discord "https://discord.gg/example" \
  --description "A test subnet"
```

---

## Delegate Commands

### delegate add

Stake TAO to a delegate (hotkey).

```sh
btcli-rs delegate add --hotkey <HOTKEY_SS58> [--netuid <NETUID>] <AMOUNT> [--password <PASSWORD>]
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `--hotkey` | yes | - | Delegate hotkey SS58 address |
| `--netuid` | no | `0` | Subnet netuid |
| `AMOUNT` | yes | - | Amount in TAO |
| `--password` | no | prompt | Coldkey password |

```sh
btcli-rs delegate add --hotkey 5HK123 --netuid 1 10.0
```

---

### delegate remove

Remove stake from a delegate.

```sh
btcli-rs delegate remove --hotkey <HOTKEY_SS58> [--netuid <NETUID>] <AMOUNT> [--password <PASSWORD>]
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `--hotkey` | yes | - | Delegate hotkey SS58 address |
| `--netuid` | no | `0` | Subnet netuid |
| `AMOUNT` | yes | - | Amount in TAO |
| `--password` | no | prompt | Coldkey password |

```sh
btcli-rs delegate remove --hotkey 5HK456 --netuid 1 5.0
```

---

### delegate list

List all delegates on the network.

```sh
btcli-rs delegate list
```

```sh
btcli-rs --network finney delegate list
```

---

### delegate take

Set the delegate take percentage for a hotkey. The CLI queries the current on-chain take and automatically calls `increase_take` or `decrease_take` based on whether the new value is higher or lower.

```sh
btcli-rs delegate take --hotkey <HOTKEY_SS58> <TAKE> [--password <PASSWORD>]
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `--hotkey` | yes | - | Delegate hotkey SS58 address |
| `TAKE` | yes | - | Take value (u16, basis points out of 65535) |
| `--password` | no | prompt | Coldkey password |

```sh
btcli-rs delegate take --hotkey 5HK789 18
```

---

### delegate my-delegates

Show delegations from the wallet's coldkey.

```sh
btcli-rs delegate my-delegates [--password <PASSWORD>]
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `--password` | no | prompt | Coldkey password |

```sh
btcli-rs delegate my-delegates
```

---

## Weights Commands

### weights set-weights

Set weights on a subnet. This is a dangerous operation that requires the `--yes` flag to skip the interactive confirmation prompt.

```sh
btcli-rs weights set-weights --netuid <NETUID> <DESTS> <WEIGHTS> [--version-key <V>] [--wallet-name <NAME>] [--wallet-path <PATH>] --yes [--password <PASSWORD>]
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `--netuid` | yes | - | Subnet netuid |
| `DESTS` | yes | - | Comma-separated destination UIDs (e.g. `1,2,3`) |
| `WEIGHTS` | yes | - | Comma-separated weight values (e.g. `100,200,300`) |
| `--version-key` | no | `0` | Version key |
| `--wallet-name` | no | global | Override wallet name |
| `--wallet-path` | no | global | Override wallet path |
| `--yes` | yes | false | Skip confirmation prompt (required) |
| `--password` | no | prompt | Coldkey password |

The number of destination UIDs must match the number of weight values.

```sh
btcli-rs weights set-weights --netuid 3 1,2,3 100,200,300 --yes
```

Python equivalent:

```sh
btcli weights set-weights --netuid 3 --uids 1,2,3 --weights 100,200,300
```

---

### weights get-weights

Get weights set by a specific UID (or all UIDs) on a subnet.

```sh
btcli-rs weights get-weights --netuid <NETUID> [--uid <UID>]
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `--netuid` | yes | - | Subnet netuid |
| `--uid` | no | all | Specific UID to query |

```sh
btcli-rs weights get-weights --netuid 1
btcli-rs weights get-weights --netuid 1 --uid 5
```

---

## Metagraph Commands

### metagraph show

Display metagraph information for a subnet. Output can be a table (default) or JSON.

```sh
btcli-rs metagraph show --netuid <NETUID> [--json] [--no-prompt]
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `--netuid` | yes | - | Subnet netuid |
| `--json` | no | false | Output as JSON instead of table |
| `--no-prompt` | no | false | Skip interactive prompts |

Table output shows neuron UID, hotkey (truncated), stake, rank, and trust columns.

```sh
btcli-rs metagraph show --netuid 1
btcli-rs metagraph show --netuid 1 --json
```

---

### metagraph sync

Sync metagraph from the chain and optionally save to file.

```sh
btcli-rs metagraph sync --netuid <NETUID> [--output <PATH>]
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `--netuid` | yes | - | Subnet netuid |
| `--output` | no | - | File path for saving the synced metagraph as JSON |

```sh
btcli-rs metagraph sync --netuid 1
btcli-rs metagraph sync --netuid 1 --output /tmp/mg.json
```

---

## MEV Shield Commands

Requires the `mev` feature flag at compile time. Without it, the `mev` subcommand is not registered.

### mev submit-encrypted

Submit an encrypted extrinsic via MEV Shield using ML-KEM-768.

```sh
btcli-rs mev submit-encrypted <EXTRINSIC_HEX> [--password <PASSWORD>]
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `EXTRINSIC_HEX` | yes | - | Hex-encoded extrinsic payload (with or without `0x` prefix) |
| `--password` | no | prompt | Coldkey password |

```sh
btcli-rs mev submit-encrypted 0x1234abcd
```

The MEV Shield workflow:
1. Fetch the on-chain NextKey (ML-KEM-768 public key)
2. Encrypt the extrinsic bytes using `MevShieldSubmit::encrypt_extrinsic`
3. SCALE-encode the payload using `MevShieldSubmit::scale_encode_payload`
4. Submit via the `submit_encrypted_extrinsic` RPC call

---

## Safety: Confirmation Prompts

Dangerous operations that modify on-chain state require an interactive confirmation prompt before submission. This protects against accidental transfers, stake changes, and weight updates.

| Flag | Behavior |
|---|---|
| `--yes` | Skip the confirmation prompt and execute immediately |

Commands that require `--yes` (or will prompt without it):
- `weights set-weights` - requires `--yes` to skip the "Proceed?" dialog
- `wallet regen-coldkey` - requires `--yes` to skip the overwrite warning

Other commands that submit extrinsics (transfers, staking, registration) will prompt for the coldkey password but do not require a separate `--yes` confirmation.

---

## Output Format

The `metagraph show` command supports `--json` for JSON output instead of the default table format. Other commands produce plain-text output with block hash and extrinsic hash on success:

```
Transfer submitted successfully.
  Block hash:      0xabc123...
  Extrinsic hash:  0xdef456...
```

The `metagraph sync --output` flag writes the full metagraph as pretty-printed JSON to the specified file path.

---

## Common Workflows

### Create a wallet and register on subnet 3

```sh
btcli-rs wallet create --no-password
btcli-rs register register --netuid 3
```

### Stake TAO and set weights

```sh
btcli-rs stake add --hotkey 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY --netuid 1 5.0
btcli-rs weights set-weights --netuid 1 1,2,3 100,200,300 --yes
```

### Check balance and overview

```sh
btcli-rs wallet balance
btcli-rs wallet overview
```

### Transfer TAO

```sh
btcli-rs transfer transfer 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty 1.5
```

### Delegate stake to a validator

```sh
btcli-rs delegate add --hotkey 5DelegateHK --netuid 1 10.0
```

### Inspect subnet state

```sh
btcli-rs metagraph show --netuid 1
btcli-rs subnet hyperparameters --netuid 1
btcli-rs weights get-weights --netuid 1
```

### Move stake between hotkeys

```sh
btcli-rs stake move --origin-hotkey 5OriginHK --destination-hotkey 5DestHK --origin-netuid 1 --destination-netuid 2 10.0
```

### Regenerate a coldkey from mnemonic

```sh
btcli-rs wallet regen-coldkey "word1 word2 ... word12" --yes --password "new-password"
```
