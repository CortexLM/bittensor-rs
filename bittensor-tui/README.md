# bittensor-tui

Terminal UI for real-time Bittensor network monitoring.

## Quick Start

```sh
# Launch the dashboard (connects to finney by default)
bittensor-tui

# Point at a specific network
bittensor-tui --network test
```

## Feature Flags

No optional features — all functionality is always enabled.

## API Overview

Built with `ratatui` + `crossterm`. Displays subnet statistics, neuron states, staking info, and block events in a real-time terminal dashboard.
