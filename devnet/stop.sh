#!/usr/bin/env bash
# Stop local Subtensor devnet node
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPOSE_FILE="$SCRIPT_DIR/docker-compose.yml"

echo "=== Stopping Subtensor Devnet ==="

if ! docker compose -f "$COMPOSE_FILE" ps -q subtensor-node 2>/dev/null | grep -q .; then
    echo "No running devnet node found."
    exit 0
fi

docker compose -f "$COMPOSE_FILE" down --timeout 30

echo "✓ Devnet node stopped."
