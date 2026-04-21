# Decisions — bittensor-rs

## 2026-04-19 Planning
- Multi-crate mono-repo (11 crates) — chosen over monolith for modularity
- Greenfield — existing Rust SDKs too immature to fork
- subxt 0.50.0 — chosen for historic block support + auto runtime upgrades
- NaCl keyfile compat — validated as CRITICAL before any wallet code (Task 5)
- WASM excludes wallet encryption (libsodium unavailable in browser)
- MEV Shield + DRAND feature-gated (not default)

## Task 23: bittensor-tui architecture decisions

- **Polling over EventStream**: Used `spawn_blocking` + `event::poll()` instead of `EventStream` to avoid needing the `event-stream` feature flag on crossterm (which requires `futures-core`)
- **mpsc channels for async→sync**: Network fetcher sends `NetworkData` via `std::sync::mpsc` to the sync render loop; event handler uses `tokio::sync::mpsc::unbounded_channel` for async event production
- **2×2 grid layout**: Network Overview (top-left), Wallet (bottom-left), Subnet (top-right), Delegate+Neuron split (bottom-right) — gives good visual hierarchy
- **Panel expansion**: Enter toggles a panel to fill the main area; Esc collapses back to grid
- **V1 keyboard-only**: No mouse support as specified; Tab cycles panels, ↑↓ navigates lists
- **Dark theme with brand colors**: Deep navy background, teal for active/focused borders, gold for stake/balance values
