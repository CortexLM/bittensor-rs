# bittensor-tui Dashboard Reference

The `bittensor-tui` binary provides a real-time terminal dashboard for monitoring the Bittensor network. It is built with `ratatui` for layout and rendering and `crossterm` for cross-platform terminal I/O. An async data fetcher polls the Subtensor chain at a configurable interval and sends updates to the UI loop over an mpsc channel, so network I/O never blocks rendering.

## Installation

Install from crates.io:

```sh
cargo install bittensor-tui
```

Build from source:

```sh
git clone https://github.com/opentensor/bittensor-rs.git
cd bittensor-rs
cargo build -p bittensor-tui --release
# binary at target/release/bittensor-tui
```

No feature flags are required. All TUI functionality is always enabled.

## Launching

```sh
bittensor-tui
```

By default, the dashboard connects to finney (mainnet) and refreshes chain data every 5 seconds.

### Command-Line Arguments

| Argument | Type | Default | Description |
|---|---|---|---|
| `--network` | string | `finney` | Network to connect to: `finney`, `test`, or `local` |
| `--refresh-rate` | u64 | `5` | Data refresh interval in seconds |

```sh
bittensor-tui --network test
bittensor-tui --network local --refresh-rate 10
bittensor-tui --network finney --refresh-rate 2
```

### Python SDK Comparison

The Python SDK does not ship a built-in TUI dashboard. Users typically rely on `btcli overview` or third-party monitoring tools. The Rust TUI provides this functionality natively as a first-class binary.

---

## Keyboard Shortcuts

| Key | Action |
|---|---|
| `Tab` | Move focus to the next panel (cyclic) |
| `Shift+Tab` | Move focus to the previous panel (cyclic) |
| `Up` | Navigate up within the active panel list |
| `Down` | Navigate down within the active panel list |
| `Enter` | Toggle expanded view for the focused panel |
| `Esc` | Collapse the expanded panel (return to grid) |
| `q` / `Q` | Quit when no panel is expanded |
| `Ctrl+C` | Force quit (always works) |

When a panel is expanded, `q` collapses it rather than quitting. This prevents accidental exits while inspecting data.

Tab cycles through panels in this order: Network Overview -> Wallet -> Subnet -> Delegate -> Neuron -> back to Network Overview.

---

## Dashboard Layout

The dashboard uses a three-section vertical layout: header, main panel area, and footer.

```
+-------------------------------------------------------------+
|  <> BITTENSOR DASHBOARD  o  Block #1234567                   |  Header
+--------------------------+----------------------------------+
|                          |                                  |
|   Network Overview       |   Subnet Explorer               |
|                          |                                  |
|                          +----------------------------------+
|--------------------------+                                  |
|                          |   Delegate / Neuron             |
|   Wallet                 |                                  |
|                          |                                  |
+--------------------------+----------------------------------+
|  Tab:Next  Up/Dn:Nav  Enter:Expand  Esc:Collapse  q:Quit  |  Footer
+-------------------------------------------------------------+
```

The main area is a 2x2 grid:

- **Top-left**: Network Overview
- **Bottom-left**: Wallet
- **Top-right**: Subnet Explorer
- **Bottom-right**: Delegate and Neuron panels (split vertically)

When a panel is expanded (Enter), that panel occupies the full main area. The header and footer remain visible.

---

## Architecture

The TUI is structured into five modules, each with a single responsibility:

```
bittensor-tui/src/
  main.rs       CLI entrypoint, argument parsing, terminal init/restore
  lib.rs        Public module declarations and prelude re-exports
  app.rs        Application state (App struct), event loop, key dispatch
  event.rs      Async keyboard/resize event handler (crossterm poll)
  network.rs    Chain data fetcher (NetworkFetcher), NetworkData snapshot
  ui.rs         Top-level draw function, layout composition, header/footer
  panels/
    mod.rs               Panel module declarations
    network_overview.rs  Network statistics panel
    wallet.rs            Wallet balance panel
    subnet.rs            Subnet list panel
    delegate.rs          Delegate monitor panel
    neuron.rs            Neuron detail panel
```

### app.rs - Application State and Event Loop

The `App` struct holds all mutable dashboard state:

| Field | Type | Description |
|---|---|---|
| `active_panel` | `Panel` | Which panel is currently focused |
| `should_quit` | `bool` | Whether the event loop should exit |
| `network_data` | `NetworkData` | Latest snapshot from the chain fetcher |
| `selected_index` | `usize` | Selected row in the active list panel |
| `expanded` | `bool` | Whether the active panel is in expanded view |
| `term_size` | `(u16, u16)` | Terminal dimensions at last render |

The `Panel` enum defines the five panels in navigation order:

| Variant | Position in grid |
|---|---|
| `NetworkOverview` | Top-left |
| `Wallet` | Bottom-left |
| `Subnet` | Top-right |
| `Delegate` | Bottom-right (top half) |
| `Neuron` | Bottom-right (bottom half) |

The main event loop in `App::run`:

1. Spawns `NetworkFetcher` as a Tokio task, receiving `NetworkData` over an mpsc channel
2. Spawns `EventHandler` to poll crossterm at 100ms intervals
3. On each iteration: drain pending network data, render the frame, handle the next input event
4. Exits cleanly when `should_quit` becomes true

### event.rs - Keyboard and Terminal Event Handler

The `EventHandler` struct wraps an mpsc unbounded receiver. A spawned Tokio task polls crossterm for events at a configurable tick rate (default 100ms) and forwards them as `Event` variants:

| Variant | Trigger |
|---|---|
| `Key(KeyEvent)` | Any key press (only `KeyEventKind::Press`) |
| `Resize(u16, u16)` | Terminal window resize |
| `Quit` | Ctrl+C, or crossterm read error |

The handler uses `spawn_blocking` for the crossterm poll call so it does not block the Tokio runtime.

### network.rs - Chain Data Fetcher

The `NetworkFetcher` struct connects to the Subtensor chain on startup and polls it at a configurable interval. It sends `NetworkData` snapshots over an mpsc channel.

`NetworkData` fields:

| Field | Type | Chain query | Description |
|---|---|---|---|
| `block_height` | `u64` | `get_network_block` | Current chain block number |
| `total_stake` | `Balance` | `get_total_network_stake` | Total TAO staked across all subnets |
| `total_issuance` | `Balance` | `get_total_issuance` | Total TAO issued by the chain |
| `network_hash_rate` | `u64` | placeholder | Reserved for future chain support |
| `connected` | `bool` | internal | Whether the WebSocket is active |
| `subnet_ids` | `Vec<u16>` | on-demand | List of subnet netuids |
| `last_error` | `Option<String>` | internal | Most recent connection or fetch error |

If the chain connection drops, `connected` is set to `false` and `last_error` records the error message. The fetcher continues retrying on each tick interval.

### ui.rs - Layout and Rendering

The `draw` function composes the full dashboard. It splits the terminal into three vertical sections (header, main, footer) using `ratatui::layout::Layout`. The main section is either a single expanded panel or a 2x2 grid with the layout described above.

The header displays the title "BITTENSOR DASHBOARD", a connection indicator (solid green circle when connected, hollow red circle when disconnected), and the current block height.

The footer shows key binding hints with highlighted key names.

### panels/ - Individual Dashboard Panels

Each panel module exports a `render` function that takes a `Frame`, area `Rect`, its data, and a `focused: bool` flag. When `focused` is true, the panel border uses the teal highlight color; otherwise it uses a dim border color.

Brand colors used across all panels:

| Constant | RGB Value | Usage |
|---|---|---|
| `COLOR_NAVY` | `(10, 14, 39)` | Reserved |
| `COLOR_TEAL` | `(0, 212, 170)` | Focused borders, highlight values |
| `COLOR_GOLD` | `(255, 215, 0)` | Stake and balance values |
| `COLOR_DIM` | `(80, 90, 120)` | Labels and secondary text |
| `COLOR_BG` | `(12, 16, 42)` | Panel background |
| `COLOR_BORDER` | `(30, 40, 80)` | Unfocused panel borders |

---

## Panels

### Network Overview

Displays high-level network statistics fetched from the chain:

| Field | Source | Description |
|---|---|---|
| Status | internal | Connection state: "Connected" (green) or "Disconnected" (red) |
| Block Height | `get_network_block` | Current chain block number |
| Total Stake | `get_total_network_stake` | Total TAO staked across all subnets |
| Issuance | `get_total_issuance` | Total TAO issued by the chain |
| Hash Rate | placeholder | Reserved for future chain support |

The connection indicator in the header mirrors this status: solid green circle when connected, hollow red circle when disconnected. Error messages from the fetcher appear when disconnected.

---

### Wallet Panel

Shows wallet balance and delegation information:

| Field | Description |
|---|---|
| Address | Coldkeypub SS58 address (truncated with ellipsis if longer than 20 characters) |
| Free | Free TAO balance |
| Staked | Staked TAO balance |
| Delegations | Up to 5 delegation entries showing amount and delegate SS58 |

Wallet data is populated from the on-chain account state using the same query paths as `btcli-rs wallet balance` and `btcli-rs wallet overview`.

---

### Subnet Explorer

Lists all subnets with their metadata:

| Column | Description |
|---|---|
| Netuid | Subnet unique identifier |
| Name | Subnet name from on-chain identity |
| Total Stake | Total TAO staked on this subnet |
| Neurons | Active neuron count |

Navigate with Up/Down arrows. Press Enter to expand for detailed subnet information including hyperparameters (rho, kappa, difficulty, burn, immunity ratio, min/max burn, weights rate limit, etc.).

---

### Delegate Monitor

Lists delegate validators and their delegation state:

| Column | Description |
|---|---|
| Rank | Position in the list (1-indexed) |
| Hotkey | Delegate hotkey SS58 (truncated at 12 characters) |
| Total Stake | Total TAO delegated to this validator |
| Take | Delegate take as percentage of 65535 basis points |
| Nominators | Count of nominating coldkeys |

The panel uses a `List` widget with highlight selection and a "▶ " marker for the currently selected row.

---

### Neuron Panel

Shows individual neuron details for a selected subnet:

| Field | Description |
|---|---|
| UID | Neuron UID within the subnet |
| Subnet | Subnet netuid |
| Active | Whether the neuron is currently active (green if active, red if inactive) |
| Stake | Stake in TAO |
| Rank | Rank value (0-65535) |
| Trust | Trust value (0-65535) |
| Consensus | Consensus value (0-65535) |
| Incentive | Incentive value (0-65535) |
| Dividend | Dividend value (0-65535) |
| Emission | Emission in rao |
| V.Trust | Validator trust value (0-65535) |

The `NeuronDisplay` struct converts from `NeuronInfo` (from `bittensor_core::types`) for rendering.

---

## Data Refresh Architecture

The TUI uses an async fetcher pattern to keep the UI responsive:

```
+----------------+      mpsc::channel      +----------------+
|  Network       | --- NetworkData -------> |  App           |
|  Fetcher       |                          |  (UI loop)     |
|  (tokio task)  |                          |  (main thread) |
+----------------+                          +----------------+
       ^                                           ^
       |                                           |
  Subtensor RPC                            crossterm events
  (async, every Ns)                       (100ms poll via EventHandler)
```

1. On startup, `NetworkFetcher` is spawned as a Tokio task
2. It connects to the Subtensor chain via `SubtensorClient::from_config`
3. On a configurable interval (default 5 seconds), it fetches block height, total stake, and total issuance
4. Updates are sent over an `mpsc` channel to the main event loop
5. The event loop drains all pending updates before each render

The event handler polls crossterm at 100ms intervals for keyboard and resize events, keeping the UI responsive without blocking on network I/O.

If the chain connection drops, the fetcher sets `connected: false` and records the error in `last_error`. The UI displays the disconnected state and error message. The fetcher continues retrying on each tick interval.

---

## Terminal Requirements

- A terminal emulator supporting ANSI escape sequences and true color (crossterm handles detection automatically)
- Minimum recommended size: 80x24 columns
- The TUI restores the terminal to its original state on exit, even if the process panics (via `ratatui::restore()`)

---

## Feature Flags

No optional features. All TUI functionality is always enabled when the crate is compiled.
