# WASM Bindings

The `bittensor-wasm` package provides WebAssembly bindings for Bittensor types, enabling browser-based applications to work with Bittensor data without a backend. It is compiled to `wasm32-unknown-unknown` and uses `wasm-bindgen` for JavaScript interop.

## Architecture

The WASM crate re-implements core types rather than depending on `bittensor-core` directly. This is necessary because `bittensor-core` depends on `subxt`, which pulls in `tokio` -- neither is compatible with the `wasm32-unknown-unknown` target. Serialization uses `serde` with JSON, and chain queries use lightweight JSON-RPC via `gloo-net`.

**What is included:**
- All core types (Balance, AxonInfo, NeuronInfoLite, SubnetInfo, etc.)
- JSON-RPC query functions (getBalance, getStake, getSubnetInfo, getMetagraph)
- SHA3-256 hashing
- Header prefix constants

**What is NOT included (WASM-incompatible):**
- Wallet encryption/decryption (NaCl requires libsodium)
- Extrinsic submission (signing requires platform-specific keystore)
- Full chain client (depends on tokio/subxt)

## Installation

```bash
npm install bittensor-wasm
```

The package includes pre-built `.wasm` files. No Rust toolchain is required for consumption.

## Initialization

Before using any WASM types, call `init()` to load and instantiate the WebAssembly module:

```javascript
import init, { Balance, NetworkConfig } from 'bittensor-wasm';

await init();

const bal = Balance.from_tao(1.5);
console.log(bal.display()); // "1.500000000"
```

## Quick Start

```javascript
import init, {
  Balance,
  NetworkConfig,
  AxonInfo,
  getBalance,
  getStake,
  sha3_256_hex
} from 'bittensor-wasm';

await init();

// Balance arithmetic
const a = Balance.from_tao(2.0);
const b = Balance.from_rao(500_000_000);
const total = a.add(b);
console.log(total.display()); // "2.500000000"

// Network configuration
const config = NetworkConfig.finney();
console.log(config.name());    // "finney"
console.log(config.ws_url()); // "wss://entrypoint-finney.opentensor.ai:443"

// Query balance (async, returns Promise<Number>)
const rpcUrl = "https://entrypoint-finney.opentensor.ai:443";
const balance = await getBalance(rpcUrl, "0x" + "00".repeat(32));
console.log(`Balance: ${balance} rao`);

// Hash computation
const hash = sha3_256_hex("hello world");
console.log(hash); // 64-character hex string
```

---

## Balance

Fixed-point numeric type for TAO amounts. 1 TAO = 1,000,000,000 RAO. All arithmetic uses saturating operations to prevent overflow/underflow.

### Constructors

| Method | Description |
|---|---|
| `new Balance()` | Zero balance |
| `Balance.from_tao(tao: number)` | Create from TAO (f64, rounded) |
| `Balance.from_rao(rao: number)` | Create from RAO (u64) |

```javascript
const zero = new Balance();
const one = Balance.from_tao(1.0);
const half = Balance.from_rao(500_000_000);
```

### Getters

| Method | Return Type | Description |
|---|---|---|
| `.to_tao()` | `number` | Balance in TAO (f64) |
| `.to_rao()` | `number` | Balance in RAO (u64) |
| `.display()` | `string` | Formatted string with 9 decimal places |

```javascript
const bal = Balance.from_tao(1.5);
bal.to_tao();   // 1.5
bal.to_rao();   // 1500000000
bal.display();   // "1.500000000"
```

### Arithmetic

| Method | Description |
|---|---|
| `.add(other: Balance)` | Saturating addition (clamps at u64::MAX) |
| `.sub(other: Balance)` | Saturating subtraction (clamps at 0) |

```javascript
const a = Balance.from_tao(3.0);
const b = Balance.from_tao(1.5);

const sum = a.add(b);    // 4.5 TAO
const diff = a.sub(b);   // 1.5 TAO

// Overflow protection
const max = Balance.from_rao(18446744073709551615); // u64::MAX
const one = Balance.from_tao(1.0);
max.add(one).to_rao(); // Still u64::MAX (saturated)
```

### Comparison

Balance implements `PartialEq`, `Eq`, `PartialOrd`, `Ord`, and `Hash`. In JavaScript, use the getters for comparison:

```javascript
const a = Balance.from_tao(1.0);
const b = Balance.from_tao(2.0);

a.to_rao() < b.to_rao();  // true
a.to_rao() === b.to_rao(); // false
```

---

## NetworkConfig

Defines the WebSocket endpoint and chain identity for connecting to a Subtensor node.

### Static Constructors

| Method | Description |
|---|---|
| `NetworkConfig.finney()` | Finney mainnet |
| `NetworkConfig.test()` | Testnet |
| `NetworkConfig.local()` | Local development node |

```javascript
const finney = NetworkConfig.finney();
const testnet = NetworkConfig.test();
const local = NetworkConfig.local();
```

### Getters

| Method | Return Type | Description |
|---|---|---|
| `.name()` | `string` | Network name identifier |
| `.ws_url()` | `string` | WebSocket RPC endpoint URL |
| `.chain_id()` | `number` | Chain identifier (SS58 prefix) |

### Predefined Endpoints

| Network | WebSocket URL |
|---|---|
| Finney | `wss://entrypoint-finney.opentensor.ai:443` |
| Test | `wss://test.finney.opentensor.ai:443` |
| Local | `ws://127.0.0.1:9944` |

---

## AxonInfo

Metadata describing a neuron's Axon endpoint.

### Constructor

```javascript
const axon = new AxonInfo(
  2130706433,  // ip: u64 (127.0.0.1 as packed integer)
  8090,        // port: u16
  4,           // ip_type: u8 (4=IPv4, 6=IPv6)
  1,           // protocol: u8 (0=HTTP, 1=HTTPS)
  4,           // version: u32
  "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", // hotkey
  "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty"  // coldkey
);
```

### Getters

| Method | Return Type | Description |
|---|---|---|
| `.ip()` | `number` | Encoded IP address as u64 |
| `.port()` | `number` | TCP port number |
| `.ip_type()` | `number` | IP version (4 or 6) |
| `.protocol()` | `number` | Transport protocol (0=HTTP, 1=HTTPS) |
| `.version()` | `number` | Bittensor node version |
| `.hotkey()` | `string` | SS58-encoded hotkey |
| `.coldkey()` | `string` | SS58-encoded coldkey |

### Serialization

```javascript
// Serialize to JSON
const json = axon.to_json();
// {"ip":2130706433,"port":8090,"ipType":4,"protocol":1,"version":4,"hotkey":"5Grw...","coldkey":"5Fhn..."}

// Deserialize from JSON
const restored = AxonInfo.from_json(json);
console.log(restored.hotkey()); // "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
```

---

## RegistrationInfo

Neuron registration record.

### Getters

| Method | Return Type | Description |
|---|---|---|
| `.netuid()` | `number` | Subnet where registered |
| `.hotkey()` | `string` | Registered hotkey |
| `.block()` | `number` | Block number of registration |
| `.burn_rao()` | `number` | Burn cost in RAO |
| `.burn_tao()` | `number` | Burn cost in TAO (f64) |

### Serialization

```javascript
const json = regInfo.to_json();
const restored = RegistrationInfo.from_json(json);
```

---

## TerminalInfo

Endpoint metadata attached to Synapse headers during transmission. Wraps `bittensor_synapse::TerminalInfo`.

### Constructor

```javascript
const ti = new TerminalInfo();
```

Creates an empty instance with all fields set to `null`.

### Getters

| Method | Return Type | Description |
|---|---|---|
| `.status_code()` | `number or null` | HTTP status code |
| `.status_message()` | `string or null` | Status message |
| `.process_time()` | `number or null` | Processing time in seconds |
| `.ip()` | `string or null` | IP address string |
| `.port()` | `number or null` | Port number |
| `.version()` | `number or null` | Protocol version |
| `.nonce()` | `number or null` | Request nonce |
| `.uuid()` | `string or null` | Request UUID |
| `.hotkey()` | `string or null` | Signer hotkey |
| `.signature()` | `string or null` | Hex-encoded Sr25519 signature |

### Serialization

```javascript
const json = ti.to_json();
const restored = TerminalInfo.from_json(json);
```

---

## NeuronInfoLite

Lightweight neuron information without weight/bond vectors.

### Getters

| Method | Return Type | Description |
|---|---|---|
| `.uid()` | `number` | Neuron UID within subnet |
| `.hotkey()` | `string` | SS58-encoded hotkey |
| `.coldkey()` | `string` | SS58-encoded coldkey |
| `.active()` | `boolean` | Whether neuron is active |
| `.incentive()` | `number` | Incentive value (0--65535) |
| `.stake_rao()` | `number` | Stake in RAO |
| `.stake_tao()` | `number` | Stake in TAO (f64) |

### Serialization

```javascript
const json = neuronLite.to_json();
const restored = NeuronInfoLite.from_json(json);
```

---

## SubnetInfo

Subnet metadata.

### Getters

| Method | Return Type | Description |
|---|---|---|
| `.netuid()` | `number` | Subnet identifier |
| `.name()` | `string` | Human-readable subnet name |
| `.owner_hotkey()` | `string` | SS58-encoded owner hotkey |
| `.tempo()` | `number` | Blocks per tempo period |
| `.maximum_uid()` | `number` | Maximum UID count |
| `.modality()` | `number` | Subnet modality (0=text, 1=image, 2=audio) |
| `.network_uid()` | `number` | Network-level UID |

### Serialization

```javascript
const json = subnet.to_json();
const restored = SubnetInfo.from_json(json);
```

---

## SubnetHyperparams

Tunable parameters controlling incentive distribution within a subnet.

### Getters

| Method | Return Type | Description |
|---|---|---|
| `.rho()` | `number` | Trust ratio denominator (u16) |
| `.kappa()` | `number` | Trust ratio numerator (u16) |
| `.difficulty()` | `number` | POW registration difficulty (u64) |
| `.burn()` | `number` | Current burn cost in RAO (u64) |
| `.immunity_ratio()` | `number` | Immunity period percentage (u16) |
| `.min_burn()` | `number` | Minimum burn in RAO (u64) |
| `.max_burn()` | `number` | Maximum burn in RAO (u64) |
| `.weights_rate_limit()` | `number` | Min blocks between weight sets (u64) |
| `.weights_version()` | `number` | Weights version key (u64) |
| `.liquid_alpha_enabled()` | `boolean` | Whether liquid alpha is active |
| `.tempo()` | `number` | Blocks per tempo period (u16) |

### Serialization

```javascript
const json = hyperparams.to_json();
const restored = SubnetHyperparams.from_json(json);
```

---

## StakeInfo

Stake record linking a hotkey/coldkey pair to a stake amount.

### Getters

| Method | Return Type | Description |
|---|---|---|
| `.hotkey()` | `string` | SS58-encoded hotkey |
| `.coldkey()` | `string` | SS58-encoded coldkey |
| `.stake_rao()` | `number` | Stake in RAO |
| `.stake_tao()` | `number` | Stake in TAO (f64) |

### Serialization

```javascript
const json = stake.to_json();
const restored = StakeInfo.from_json(json);
```

---

## DelegateInfo

Delegate metadata including take percentage, nominators, and subnet registrations.

### Getters

| Method | Return Type | Description |
|---|---|---|
| `.delegate_ss58()` | `string` | SS58-encoded delegate address |
| `.delegate_hotkey()` | `string` | Delegate hotkey |
| `.total_stake_rao()` | `number` | Total delegated stake in RAO |
| `.total_stake_tao()` | `number` | Total delegated stake in TAO (f64) |
| `.owner_hotkey()` | `string` | Owner hotkey |
| `.take()` | `number` | Take percentage in basis points (0--10000) |
| `.owner_ss58()` | `string` | SS58-encoded owner address |
| `.nominator_count()` | `number` | Number of nominators |

### Serialization

```javascript
const json = delegate.to_json();
const restored = DelegateInfo.from_json(json);
```

---

## Query Functions

These async functions perform JSON-RPC queries against a Subtensor HTTP RPC endpoint. They return JavaScript `Promise` values.

### `getBalance(rpcUrl, address)`

Get the free balance of an address. Returns the balance in RAO as a `Number`.

| Parameter | Type | Description |
|---|---|---|
| `rpcUrl` | `string` | HTTP RPC endpoint URL |
| `address` | `string` | `0x`-prefixed hex-encoded 32-byte account ID |

```javascript
const balance = await getBalance(
  "https://entrypoint-finney.opentensor.ai:443",
  "0x" + "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"
);
console.log(`Free balance: ${balance} rao`);
```

### `getStake(rpcUrl, hotkey, netuid)`

Get the stake of a hotkey on a subnet. Returns the stake in RAO as a `Number`.

| Parameter | Type | Description |
|---|---|---|
| `rpcUrl` | `string` | HTTP RPC endpoint URL |
| `hotkey` | `string` | `0x`-prefixed hex-encoded 32-byte hotkey account ID |
| `netuid` | `number` | Subnet identifier |

```javascript
const stake = await getStake(
  "https://entrypoint-finney.opentensor.ai:443",
  "0x" + "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d",
  1
);
console.log(`Stake on subnet 1: ${stake} rao`);
```

### `getSubnetInfo(rpcUrl, netuid)`

Get subnet information. Returns a `SubnetInfo` object.

| Parameter | Type | Description |
|---|---|---|
| `rpcUrl` | `string` | HTTP RPC endpoint URL |
| `netuid` | `number` | Subnet identifier |

```javascript
const info = await getSubnetInfo(
  "https://entrypoint-finney.opentensor.ai:443",
  1
);
console.log(`Subnet name: ${info.name()}, owner: ${info.owner_hotkey()}`);
```

### `getMetagraph(rpcUrl, netuid)`

Get the metagraph for a subnet. Returns a JSON string containing block number, neuron count, total stake, total issuance, total weight, and total bonds.

| Parameter | Type | Description |
|---|---|---|
| `rpcUrl` | `string` | HTTP RPC endpoint URL |
| `netuid` | `number` | Subnet identifier |

```javascript
const metagraphJson = await getMetagraph(
  "https://entrypoint-finney.opentensor.ai:443",
  1
);
const meta = JSON.parse(metagraphJson);
console.log(`Block: ${meta.block}, Neurons: ${meta.n}, Stake: ${meta.stake}`);
```

---

## Utility Functions

### `sha3_256_hex(input)`

Compute SHA3-256 hex digest of a string. Re-exported from `bittensor-synapse`.

```javascript
const hash = sha3_256_hex("hello world");
console.log(hash); // 64-character hex string
```

### `axonPrefix()`

Returns the axon header prefix string: `"bt_header_axon_"`.

```javascript
console.log(axonPrefix()); // "bt_header_axon_"
```

### `dendritePrefix()`

Returns the dendrite header prefix string: `"bt_header_dendrite_"`.

```javascript
console.log(dendritePrefix()); // "bt_header_dendrite_"
```

### `inputObjPrefix()`

Returns the input-obj header prefix string: `"bt_header_input_obj_"`.

```javascript
console.log(inputObjPrefix()); // "bt_header_input_obj_"
```

---

## Browser Build Setup

For bundlers that support WebAssembly (webpack 5+, Vite, esbuild):

```javascript
// vite.config.js
import { defineConfig } from 'vite';

export default defineConfig({
  optimizeDeps: {
    exclude: ['bittensor-wasm']
  }
});
```

For webpack 5:

```javascript
// webpack.config.js
module.exports = {
  experiments: {
    asyncWebAssembly: true
  }
};
```

---

## TypeScript Types

All WASM types can be typed in TypeScript:

```typescript
interface Balance {
  new (): Balance;
  from_tao(tao: number): Balance;
  from_rao(rao: number): Balance;
  to_tao(): number;
  to_rao(): number;
  display(): string;
  add(other: Balance): Balance;
  sub(other: Balance): Balance;
}

interface NetworkConfig {
  finney(): NetworkConfig;
  test(): NetworkConfig;
  local(): NetworkConfig;
  name(): string;
  ws_url(): string;
  chain_id(): number;
}

interface AxonInfo {
  new (ip: number, port: number, ip_type: number, protocol: number,
       version: number, hotkey: string, coldkey: string): AxonInfo;
  ip(): number;
  port(): number;
  ip_type(): number;
  protocol(): number;
  version(): number;
  hotkey(): string;
  coldkey(): string;
  to_json(): string;
  from_json(json: string): AxonInfo;
}

interface NeuronInfoLite {
  uid(): number;
  hotkey(): string;
  coldkey(): string;
  active(): boolean;
  incentive(): number;
  stake_rao(): number;
  stake_tao(): number;
  to_json(): string;
  from_json(json: string): NeuronInfoLite;
}

interface SubnetInfo {
  netuid(): number;
  name(): string;
  owner_hotkey(): string;
  tempo(): number;
  maximum_uid(): number;
  modality(): number;
  network_uid(): number;
  to_json(): string;
  from_json(json: string): SubnetInfo;
}

interface SubnetHyperparams {
  rho(): number;
  kappa(): number;
  difficulty(): number;
  burn(): number;
  immunity_ratio(): number;
  min_burn(): number;
  max_burn(): number;
  weights_rate_limit(): number;
  weights_version(): number;
  liquid_alpha_enabled(): boolean;
  tempo(): number;
  to_json(): string;
  from_json(json: string): SubnetHyperparams;
}

interface StakeInfo {
  hotkey(): string;
  coldkey(): string;
  stake_rao(): number;
  stake_tao(): number;
  to_json(): string;
  from_json(json: string): StakeInfo;
}

interface DelegateInfo {
  delegate_ss58(): string;
  delegate_hotkey(): string;
  total_stake_rao(): number;
  total_stake_tao(): number;
  owner_hotkey(): string;
  take(): number;
  owner_ss58(): string;
  nominator_count(): number;
  to_json(): string;
  from_json(json: string): DelegateInfo;
}

interface TerminalInfo {
  new (): TerminalInfo;
  status_code(): number | null;
  status_message(): string | null;
  process_time(): number | null;
  ip(): string | null;
  port(): number | null;
  version(): number | null;
  nonce(): number | null;
  uuid(): string | null;
  hotkey(): string | null;
  signature(): string | null;
  to_json(): string;
  from_json(json: string): TerminalInfo;
}

interface RegistrationInfo {
  netuid(): number;
  hotkey(): string;
  block(): number;
  burn_rao(): number;
  burn_tao(): number;
  to_json(): string;
  from_json(json: string): RegistrationInfo;
}

// Query functions
declare function getBalance(rpcUrl: string, address: string): Promise<number>;
declare function getStake(rpcUrl: string, hotkey: string, netuid: number): Promise<number>;
declare function getSubnetInfo(rpcUrl: string, netuid: number): Promise<SubnetInfo>;
declare function getMetagraph(rpcUrl: string, netuid: number): Promise<string>;

// Utility functions
declare function sha3_256_hex(input: string): string;
declare function axonPrefix(): string;
declare function dendritePrefix(): string;
declare function inputObjPrefix(): string;
```
