# Type Definitions

This document provides an overview of all core data structures used in the Bittensor Rust SDK.

## Neuron Types

### NeuronInfo

Complete neuron information with all fields.

```rust
pub struct NeuronInfo {
    pub uid: u64,
    pub netuid: u16,
    pub hotkey: AccountId32,
    pub coldkey: AccountId32,
    pub stake: u128,
    pub stake_dict: HashMap<AccountId32, u128>,
    pub total_stake: u128,
    pub root_stake: u128,
    pub rank: f64,
    pub trust: f64,
    pub consensus: f64,
    pub validator_trust: f64,
    pub incentive: f64,
    pub emission: f64,
    pub dividends: f64,
    pub active: bool,
    pub last_update: u64,
    pub validator_permit: bool,
    pub version: u64,
    pub weights: Vec<(u64, u64)>,
    pub bonds: Vec<Vec<u64>>,
    pub pruning_score: u64,
    pub prometheus_info: Option<PrometheusInfo>,
    pub axon_info: Option<AxonInfo>,
    pub is_null: bool,
}
```

**Fields:**
- `uid`: Unique identifier within the subnet
- `netuid`: Network unique identifier
- `hotkey`: Hotkey account (SS58 address)
- `coldkey`: Coldkey account (SS58 address)
- `stake`: Total stake on this neuron
- `stake_dict`: Mapping of coldkey to stake amount
- `total_stake`: Total stake on the subnet
- `root_stake`: Total stake on root subnet (RAO)
- `rank`: Normalized rank score
- `trust`: Normalized trust score
- `consensus`: Normalized consensus score
- `validator_trust`: Validator trust score
- `incentive`: Normalized incentive score
- `emission`: Emission amount in RAO
- `dividends`: Dividends received
- `active`: Whether the neuron is active
- `last_update`: Last update block number
- `validator_permit`: Whether the neuron has validator permit
- `version`: Version key
- `weights`: List of weight assignments [(uid, weight)]
- `bonds`: List of bond assignments
- `pruning_score`: Pruning score
- `prometheus_info`: Optional Prometheus metrics information
- `axon_info`: Optional axon network information
- `is_null`: Whether this is a null neuron

### NeuronInfoLite

Lightweight neuron information with essential fields only.

```rust
pub struct NeuronInfoLite {
    pub uid: u64,
    pub netuid: u16,
    pub hotkey: AccountId32,
    pub coldkey: AccountId32,
    pub stake: u128,
    pub rank: f64,
    pub trust: f64,
    pub consensus: f64,
    pub incentive: f64,
    pub emission: f64,
    pub active: bool,
    pub validator_permit: bool,
    pub prometheus_info: Option<PrometheusInfo>,
    pub axon_info: Option<AxonInfo>,
    pub is_null: bool,
}
```

## Subnet Types

### SubnetInfo

Complete subnet information.

```rust
pub struct SubnetInfo {
    pub netuid: u16,
    pub rho: u16,
    pub kappa: u16,
    pub difficulty: u64,
    pub immunity_period: u64,
    pub min_allowed_weights: u64,
    pub max_allowed_weights: u64,
    pub max_weight_limit: u64,
    pub scaling_law_power: u64,
    pub subnetwork_n: u16,
    pub max_n: u16,
    pub blocks_since_epoch: u64,
    pub tempo: u64,
    pub modality: u8,
    pub connected_uids: Vec<u64>,
    pub emission_values: Vec<u64>,
    pub burn: u64,
    pub owner: Option<AccountId32>,
}
```

### SubnetConfigInfo

Subnet configuration parameters.

```rust
pub struct SubnetConfigInfo {
    pub min_allowed_weights: u64,
    pub max_weight_limit: u64,
    pub weights_version: u64,
    pub tempo: u64,
    pub max_allowed_uids: u64,
    pub min_stake: u128,
    pub immunity_period: u64,
    pub min_burn: u128,
    pub max_burn: u128,
    pub adjustment_alpha: u64,
    pub target_regs_per_interval: u64,
}
```

### SubnetIdentity

Subnet identity information.

```rust
pub struct SubnetIdentity {
    pub coldkey: AccountId32,
    pub hotkey: AccountId32,
}
```

## Delegate Types

### DelegateInfo

Complete delegate information.

```rust
pub struct DelegateInfo {
    pub hotkey_ss58: String,
    pub total_delegated: u128,
    pub nominators: u64,
    pub owner_ss58: String,
    pub take: u16,
    pub validator_permits: Vec<u16>,
    pub registrations: Vec<u16>,
    pub return_per_1000: u64,
    pub total_daily_return: u128,
}
```

### DelegatedInfo

Information about a delegation.

```rust
pub struct DelegatedInfo {
    pub hotkey_ss58: String,
    pub coldkey_ss58: String,
    pub stake: u128,
}
```

## Network Types

### AxonInfo

Axon network connection information.

```rust
pub struct AxonInfo {
    pub version: u32,
    pub hotkey: Option<String>,
    pub block: u64,
    pub ip: IpAddr,
    pub port: u16,
    pub ip_type: u8,
    pub protocol: u8,
    pub placeholder1: u8,
    pub placeholder2: u8,
}
```

**Fields:**
- `hotkey`: Optional axon hotkey (SS58)
- `block`: Registration block
- `version`: Protocol version
- `ip`: IP address
- `port`: Port number
- `ip_type`: IP type (4 for IPv4, 6 for IPv6)
- `protocol`: Protocol type (TCP, UDP, etc.)

### PrometheusInfo

Prometheus metrics information.

```rust
pub struct PrometheusInfo {
    pub version: u32,
    pub ip: u128,
    pub port: u16,
    pub ip_type: u8,
    pub protocol: u8,
    pub placeholder1: u8,
    pub placeholder2: u8,
}
```

## Commitment Types

### WeightCommitInfo

Weight commitment information for commit-reveal schemes.

```rust
pub struct WeightCommitInfo {
    pub commitment: [u8; 32],
    pub salt: Vec<u16>,
    pub block_number: u64,
}
```

## Liquidity Types

### LiquidityPosition

Liquidity pool position information.

```rust
pub struct LiquidityPosition {
    pub tau_balance: u128,
    pub tao_balance: u128,
    pub lp_balance: u128,
    pub liquidity: u128,
}
```

## Governance Types

### ProposalVoteData

Vote data for governance proposals.

```rust
pub struct ProposalVoteData {
    pub proposal_id: u64,
    pub votes: u64,
    pub nay: u64,
    pub aye: u64,
}
```

## Chain Types

### ChainIdentity

Chain identity information stored as key-value pairs.

```rust
pub struct ChainIdentity {
    pub fields: HashMap<String, String>,
}
```

## Account Types

All account addresses use the `AccountId32` type from `sp_core::crypto`, which represents a 32-byte SS58-encoded account identifier compatible with Substrate-based chains.

## Serialization

All types implement `Serialize` and `Deserialize` from the `serde` crate, allowing easy conversion to and from JSON and other formats.

## Usage

Import types from the main crate:

```rust
use bittensor_rs::types::{
    NeuronInfo, NeuronInfoLite, SubnetInfo, DelegateInfo,
    AxonInfo, PrometheusInfo, WeightCommitInfo
};
```