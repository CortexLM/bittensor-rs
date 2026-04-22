# Serving Extrinsics

Module path: `bittensor_chain::extrinsics::serving`

Serving extrinsics let neurons advertise their network endpoints on the Bittensor chain. Validators and miners use these endpoints to discover and connect with each other. There are two variants: one for plain TCP connections and one for TLS-secured connections.

## Transaction Result

All serving functions return `Result<TxSuccess>`. The `TxSuccess` struct:

```rust
pub struct TxSuccess {
    pub block_hash: H256,
    pub extrinsic_hash: H256,
}
```

- **`block_hash`**: The hash of the block that included the transaction.
- **`extrinsic_hash`**: The hash of the extrinsic, used for transaction tracking.

---

## serve_axon

Advertise a plain TCP endpoint for a neuron on a subnet. After calling this, other neurons on the subnet can discover the advertised IP and port from chain metadata and connect directly.

### Signature

```rust
pub async fn serve_axon(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    netuid: u16,
    version: u32,
    ip: u128,
    port: u16,
    ip_type: u8,
) -> Result<TxSuccess>
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected Subtensor client |
| `signer` | `&Keypair` | Hotkey of the neuron serving the endpoint |
| `netuid` | `u16` | Subnet ID where the neuron is registered |
| `version` | `u32` | Protocol version of the serving neuron. Must match the subnet's expected version |
| `ip` | `u128` | IP address of the endpoint, encoded as a u128 |
| `port` | `u16` | TCP port number |
| `ip_type` | `u8` | IP protocol version: `4` for IPv4, `6` for IPv6 |

### Returns

`Result<TxSuccess>` containing the block hash and extrinsic hash on success.

### IP Address Encoding

IPv4 and IPv6 addresses must be encoded as `u128` values. For IPv4, the address is stored in the lower 32 bits. For IPv6, the full 128 bits are used.

```rust
use std::net::{Ipv4Addr, Ipv6Addr};

fn ipv4_to_u128(addr: &str) -> u128 {
    let ip: Ipv4Addr = addr.parse().unwrap();
    u32::from(ip) as u128
}

fn ipv6_to_u128(addr: &str) -> u128 {
    let ip: Ipv6Addr = addr.parse().unwrap();
    u128::from(ip)
}

// IPv4 example: 192.168.1.100
let ip = ipv4_to_u128("192.168.1.100");  // ip_type = 4

// IPv6 example: 2001:db8::1
let ip = ipv6_to_u128("2001:db8::1");    // ip_type = 6
```

### Example

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::extrinsics::serving;
use std::net::Ipv4Addr;

async fn serve_my_axon() -> Result<()> {
    let client = OnlineClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;
    let signer = subxt_signer::sr25519::Keypair::from_uri("//MyMinerHotkey")?;

    let netuid: u16 = 1;
    let version: u32 = 24; // Bittensor protocol version
    let ip: u128 = u32::from("192.168.1.100".parse::<Ipv4Addr>()?) as u128;
    let port: u16 = 8091;
    let ip_type: u8 = 4;

    let result = serving::serve_axon(
        &client,
        &signer,
        netuid,
        version,
        ip,
        port,
        ip_type,
    ).await?;

    println!(
        "Axon serving at 192.168.1.100:8091 on subnet {}, block: {}",
        netuid,
        result.block_hash
    );

    Ok(())
}
```

---

## serve_axon_tls

Advertise a TLS-secured endpoint for a neuron on a subnet. This is functionally identical to `serve_axon` but signals to connecting peers that the endpoint supports TLS encryption. The chain does not verify TLS certificates; it simply records that the endpoint advertises TLS support.

### Signature

```rust
pub async fn serve_axon_tls(
    client: &OnlineClient<SubtensorConfig>,
    signer: &subxt_signer::sr25519::Keypair,
    netuid: u16,
    version: u32,
    ip: u128,
    port: u16,
    ip_type: u8,
) -> Result<TxSuccess>
```

### Parameters

| Parameter | Type | Description |
|---|---|---|
| `client` | `&OnlineClient<SubtensorConfig>` | Connected Subtensor client |
| `signer` | `&Keypair` | Hotkey of the neuron serving the endpoint |
| `netuid` | `u16` | Subnet ID where the neuron is registered |
| `version` | `u32` | Protocol version of the serving neuron |
| `ip` | `u128` | IP address of the endpoint, encoded as a u128 |
| `port` | `u16` | TLS port number |
| `ip_type` | `u8` | IP protocol version: `4` for IPv4, `6` for IPv6 |

### Returns

`Result<TxSuccess>` containing the block hash and extrinsic hash on success.

### Example

```rust
use bittensor_chain::prelude::*;
use bittensor_chain::extrinsics::serving;
use std::net::Ipv4Addr;

async fn serve_my_tls_axon() -> Result<()> {
    let client = OnlineClient::from_url("wss://entrypoint-finney.opentensor.ai:443").await?;
    let signer = subxt_signer::sr25519::Keypair::from_uri("//MyMinerHotkey")?;

    let netuid: u16 = 1;
    let version: u32 = 24;
    let ip: u128 = u32::from("10.0.0.50".parse::<Ipv4Addr>()?) as u128;
    let port: u16 = 443;
    let ip_type: u8 = 4;

    let result = serving::serve_axon_tls(
        &client,
        &signer,
        netuid,
        version,
        ip,
        port,
        ip_type,
    ).await?;

    println!(
        "TLS axon serving at 10.0.0.50:443 on subnet {}, block: {}",
        netuid,
        result.block_hash
    );

    Ok(())
}
```

### Choosing Between serve_axon and serve_axon_tls

Use `serve_axon_tls` when your endpoint terminates TLS connections. This is recommended for production deployments where data travels over public networks. Use `serve_axon` for local development, testing, or endpoints behind a TLS-terminating reverse proxy.

---

## Important Notes

### Registration Required

A neuron must be registered on the target subnet before it can serve an axon endpoint. Calling `serve_axon` or `serve_axon_tls` for an unregistered hotkey on the target subnet will result in a failed transaction.

### Endpoint Overwrite

Calling `serve_axon` or `serve_axon_tls` replaces any previously advertised endpoint for that hotkey on the given subnet. There is no need to "unserve" before changing an endpoint. The new call simply updates the on-chain metadata.

### Version Field

The `version` parameter should match the Bittensor protocol version that the neuron software implements. Mismatched versions may cause other neurons to reject connections or ignore the endpoint. Check the current Bittensor release notes for the expected version value.

### Rate Limiting

Subnets may impose rate limits on how frequently a hotkey can update its serving metadata. If you update your axon endpoint too rapidly, the transaction may be rejected. The typical limit is one update per epoch per subnet.

### Internal vs. External IPs

The IP you advertise on chain is what other neurons use to connect. If your node sits behind a NAT or load balancer, advertise the external-facing IP, not the internal one. Advertising a private IP (like 192.168.x.x or 10.x.x.x) will make your node unreachable from the public internet.
