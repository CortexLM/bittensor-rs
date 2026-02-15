# Runtime Queries

Runtime queries expose low-level Subtensor storage values that are commonly
needed for parity with the Python SDK. Use these for version checks, ownership
lookups, and commit-reveal configuration.

## Query Functions

### get_weights_version_key

Returns the current `WeightsVersion` from `SubtensorModule`.

```rust
use bittensor_rs::queries::get_weights_version_key;

let version_key = get_weights_version_key(&client).await?;
```

### commit_reveal_enabled

Checks whether commit-reveal is enabled globally (`SubtensorModule.CommitRevealEnabled`).
Defaults to `true` if the storage item is absent.

```rust
use bittensor_rs::queries::commit_reveal_enabled;

if commit_reveal_enabled(&client).await? {
    println!("Commit-reveal is enabled");
}
```

### get_tempo

Fetches the tempo (block step) for a subnet.

```rust
use bittensor_rs::queries::get_tempo;

let tempo = get_tempo(&client, 1).await?;
println!("Subnet 1 tempo: {}", tempo);
```

### get_hotkey_owner

Finds the coldkey owner for a hotkey. Returns `None` if the hotkey is not
registered.

```rust
use bittensor_rs::queries::get_hotkey_owner;
use sp_core::crypto::AccountId32;

let owner = get_hotkey_owner(&client, &hotkey).await?;
if let Some(owner) = owner {
    println!("Owner: {}", owner);
}
```

## Notes

- These helpers use dynamic storage queries and return raw on-chain values.
- All RAO/TAO conversions remain the caller's responsibility.
