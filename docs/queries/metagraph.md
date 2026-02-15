# Metagraph Queries

Metagraph queries provide a full snapshot of a subnet's neuron state, including
axon info and fast-access UID-indexed arrays for hotkeys, coldkeys, and activity
flags.

## Overview

Use metagraph queries when you need a consistent snapshot of a subnet. The
`Metagraph` structure includes:

- `neurons`: `HashMap<u16, NeuronInfo>` keyed by UID
- `axons`: `HashMap<u16, AxonInfo>` keyed by UID
- `hotkeys`: `Vec<AccountId32>` indexed by UID
- `coldkeys`: `Vec<AccountId32>` indexed by UID
- `validator_permit`: `Vec<bool>` indexed by UID
- `active`: `Vec<bool>` indexed by UID

All arrays are populated after sorting neurons by UID, matching Python SDK
behavior where UID index aligns with vector indices.

## Query Functions

### get_metagraph_info

Fetch a metagraph snapshot for a single subnet.

```rust
use bittensor_rs::queries::get_metagraph_info;

let metagraph = get_metagraph_info(&client, 1).await?;
println!("Total neurons: {}", metagraph.n);
println!("Hotkey for UID 0: {}", metagraph.hotkeys[0]);
```

### get_all_metagraphs_info

Fetch metagraph snapshots for all active subnets.

```rust
use bittensor_rs::queries::get_all_metagraphs_info;

let metagraphs = get_all_metagraphs_info(&client).await?;
for graph in metagraphs {
    println!("Subnet {} has {} neurons", graph.netuid, graph.n);
}
```

## Notes

- Metagraph values are derived from on-chain storage via `queries::neurons` and
  `queries::subnets` utilities.
- All RAO/TAO conversions remain the caller's responsibility. The metagraph
  stores on-chain values (RAO).
