# Identity Queries

Module: `bittensor_chain::queries::identity`

Identity registrations, subnet identities, neuron certificates, axon endpoints, and Prometheus telemetry data.

```rust
use bittensor_chain::queries::identity;
use bittensor_chain::prelude::*;
```

All functions take `&OnlineClient<SubtensorConfig>` as the first argument and return `Result<T, BittensorError>`.

---

## `get_identity`

```rust
pub async fn get_identity(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
) -> Result<Option<subtensor::runtime_types::pallet_registry::types::Registration<u64>>>
```

Fetches the identity registration for a given hotkey from the `registry` pallet. The `Registration` struct contains identity info (display name, legal name, web, etc.) and judgement data. Returns `None` if the hotkey has no registered identity.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `hotkey` | `&AccountId32` | Hotkey account ID |

**Returns**

`Option<Registration<u64>>` -- The identity registration, or `None` if not set.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::identity;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let hotkey = subxt::utils::AccountId32::from([0u8; 32]);
    if let Some(reg) = identity::get_identity(client.rpc(), &hotkey).await? {
        println!("Identity registered for hotkey");
    } else {
        println!("No identity found");
    }

    Ok(())
}
```

---

## `get_identities_v2`

```rust
pub async fn get_identities_v2(
    client: &OnlineClient<SubtensorConfig>,
    hotkey: &subxt::utils::AccountId32,
) -> Result<Option<subtensor::runtime_types::pallet_subtensor::pallet::ChainIdentityV2>>
```

Fetches the `ChainIdentityV2` for a given hotkey from the `subtensor_module`. This is the newer identity format that stores identity data directly in the subtensor pallet rather than the registry pallet. Returns `None` if no identity is found.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `hotkey` | `&AccountId32` | Hotkey account ID |

**Returns**

`Option<ChainIdentityV2>` -- The V2 chain identity, or `None` if not set.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::identity;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let hotkey = subxt::utils::AccountId32::from([0u8; 32]);
    if let Some(id) = identity::get_identities_v2(client.rpc(), &hotkey).await? {
        println!("ChainIdentityV2 found for hotkey");
    } else {
        println!("No V2 identity");
    }

    Ok(())
}
```

---

## `get_subnet_identities_v3`

```rust
pub async fn get_subnet_identities_v3(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
) -> Result<Option<subtensor::runtime_types::pallet_subtensor::pallet::SubnetIdentityV3>>
```

Fetches the `SubnetIdentityV3` for a given subnet from the `subtensor_module`. This contains the subnet-level identity data such as the subnet name, description, and GitHub repository URL. Returns `None` if the subnet has no identity registered.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet unique identifier |

**Returns**

`Option<SubnetIdentityV3>` -- The V3 subnet identity, or `None` if not set.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::identity;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    if let Some(subnet_id) = identity::get_subnet_identities_v3(client.rpc(), 1).await? {
        println!("Subnet 1 has identity data");
    } else {
        println!("Subnet 1 has no identity");
    }

    Ok(())
}
```

---

## `get_neuron_certificates`

```rust
pub async fn get_neuron_certificates(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    hotkey: &subxt::utils::AccountId32,
) -> Result<Option<subtensor::runtime_types::pallet_subtensor::pallet::NeuronCertificate>>
```

Fetches the TLS certificate for a neuron (hotkey) in a subnet. Neurons register DER-encoded certificates on-chain to enable TLS-encrypted communication with other neurons. Returns `None` if no certificate is registered.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet unique identifier |
| `hotkey` | `&AccountId32` | Hotkey account ID |

**Returns**

`Option<NeuronCertificate>` -- The neuron certificate, or `None` if not set.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::identity;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let hotkey = subxt::utils::AccountId32::from([0u8; 32]);
    if let Some(cert) = identity::get_neuron_certificates(client.rpc(), 1, &hotkey).await? {
        println!("Neuron certificate found for hotkey in subnet 1");
    } else {
        println!("No certificate registered");
    }

    Ok(())
}
```

---

## `get_axons`

```rust
pub async fn get_axons(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    hotkey: &subxt::utils::AccountId32,
) -> Result<Option<subtensor::runtime_types::pallet_subtensor::pallet::AxonInfo>>
```

Fetches the axon endpoint information for a neuron (hotkey) in a subnet. The `AxonInfo` struct contains the IP address, port, protocol version, and other metadata needed to connect to a neuron's serving endpoint. Returns `None` if no axon is registered.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet unique identifier |
| `hotkey` | `&AccountId32` | Hotkey account ID |

**Returns**

`Option<AxonInfo>` -- The axon info struct, or `None` if not set.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::identity;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let hotkey = subxt::utils::AccountId32::from([0u8; 32]);
    if let Some(axon) = identity::get_axons(client.rpc(), 1, &hotkey).await? {
        println!("Axon registered for hotkey in subnet 1");
    } else {
        println!("No axon found");
    }

    Ok(())
}
```

---

## `get_prometheus`

```rust
pub async fn get_prometheus(
    client: &OnlineClient<SubtensorConfig>,
    netuid: u16,
    hotkey: &subxt::utils::AccountId32,
) -> Result<Option<subtensor::runtime_types::pallet_subtensor::pallet::PrometheusInfo>>
```

Fetches the Prometheus telemetry endpoint information for a neuron (hotkey) in a subnet. The `PrometheusInfo` struct contains the IP address and port where the neuron exposes Prometheus metrics for monitoring. Returns `None` if no Prometheus endpoint is registered.

**Parameters**

| Name | Type | Description |
|------|------|-------------|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected subxt client |
| `netuid` | `u16` | Subnet unique identifier |
| `hotkey` | `&AccountId32` | Hotkey account ID |

**Returns**

`Option<PrometheusInfo>` -- The Prometheus info struct, or `None` if not set.

**Example**

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::identity;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;

    // Requires live node
    let hotkey = subxt::utils::AccountId32::from([0u8; 32]);
    if let Some(prom) = identity::get_prometheus(client.rpc(), 1, &hotkey).await? {
        println!("Prometheus endpoint found for hotkey in subnet 1");
    } else {
        println!("No Prometheus endpoint registered");
    }

    Ok(())
}
```

---

## Full Example

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::queries::identity;
use bittensor_core::config::NetworkConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SubtensorClient::from_config(NetworkConfig::finney()).await?;
    let rpc = client.rpc();

    // Requires live node
    let hotkey = subxt::utils::AccountId32::from([0u8; 32]);

    // Registry identity
    if let Some(reg) = identity::get_identity(rpc, &hotkey).await? {
        println!("Registry identity found");
    }

    // Chain identity V2
    if let Some(id) = identity::get_identities_v2(rpc, &hotkey).await? {
        println!("ChainIdentityV2 found");
    }

    // Subnet identity V3
    if let Some(subnet_id) = identity::get_subnet_identities_v3(rpc, 1).await? {
        println!("Subnet 1 has identity data");
    }

    // Neuron certificate
    if let Some(cert) = identity::get_neuron_certificates(rpc, 1, &hotkey).await? {
        println!("Neuron certificate found");
    }

    // Axon info
    if let Some(axon) = identity::get_axons(rpc, 1, &hotkey).await? {
        println!("Axon endpoint found");
    }

    // Prometheus
    if let Some(prom) = identity::get_prometheus(rpc, 1, &hotkey).await? {
        println!("Prometheus endpoint found");
    }

    Ok(())
}
```
