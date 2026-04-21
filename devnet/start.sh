#!/usr/bin/env bash
# Start local Subtensor devnet node
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPOSE_FILE="$SCRIPT_DIR/docker-compose.yml"

echo "=== Subtensor Devnet Starter ==="

# Check Docker availability
if ! command -v docker &>/dev/null; then
    echo "ERROR: docker not found. Please install Docker first." >&2
    exit 1
fi

if ! docker compose version &>/dev/null; then
    echo "ERROR: docker compose not available. Please install Docker Compose." >&2
    exit 1
fi

# Pull latest image
echo "[1/3] Pulling subtensor image..."
docker compose -f "$COMPOSE_FILE" pull

# Start node
echo "[2/3] Starting subtensor devnet node..."
docker compose -f "$COMPOSE_FILE" up -d

# Wait for readiness
echo "[3/3] Waiting for node to produce blocks..."
MAX_WAIT=30
ELAPSED=0
while [ $ELAPSED -lt $MAX_WAIT ]; do
    # Try WebSocket health via HTTP RPC endpoint
    RESPONSE=$(curl -s -X POST http://localhost:31333 \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"system_health","params":[],"id":1}' 2>/dev/null || true)

    if echo "$RESPONSE" | grep -q '"isSyncing":false\|"isSyncing": false'; then
        echo ""
        echo "✓ Node is ready and producing blocks!"
        echo ""
        echo "  WebSocket RPC:  ws://localhost:31444"
        echo "  HTTP RPC:       http://localhost:31333"
        echo "  P2P:            localhost:31033"
        echo ""
        exit 0
    fi

    sleep 1
    ELAPSED=$((ELAPSED + 1))
    printf "."
done

echo ""
echo "WARNING: Node did not report healthy within ${MAX_WAIT}s."
echo "Check logs: docker compose -f $COMPOSE_FILE logs subtensor-node"
exit 1
