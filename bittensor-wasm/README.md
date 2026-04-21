# bittensor-wasm

WebAssembly bindings for Bittensor types — use Bittensor in the browser.

## Quick Start

```js
import init, { Balance, AxonInfo } from 'bittensor-wasm';

await init();

const bal = Balance.from_tao("1.5");
console.log(bal.display()); // "1.500000000 TAO"

const info = AxonInfo.from_json('{"ip":"1.2.3.4","port":8091,...}');
console.log(info.hotkey());
```

## Feature Flags

No optional features — all functionality is always enabled.

## API Overview

Exports WASM-wrapped types: `Balance`, `AxonInfo`, `NeuronInfoLite`, `SubnetInfo`, `SubnetHyperparams`, `StakeInfo`, `DelegateInfo`, `TerminalInfo`. Each type exposes getters for all fields plus `to_json()`/`from_json()` serialization.
