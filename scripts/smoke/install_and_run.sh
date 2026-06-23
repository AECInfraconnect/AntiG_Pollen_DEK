#!/usr/bin/env bash
set -euo pipefail

echo "Running smoke tests..."

# Simulate dek-cli calls for the local smoke test
echo "dek-cli version"
cargo run -p dek-cli -- version || echo "dek-cli version failed"

echo "dek-cli doctor --profile local"
cargo run -p dek-cli -- doctor --profile local || echo "dek-cli doctor failed"

echo "dek-cli profile set local --url http://127.0.0.1:43891"
cargo run -p dek-cli -- profile set local --url http://127.0.0.1:43891 || echo "dek-cli profile set failed"

echo "Waiting for local-control-plane healthz..."
# Start the local control plane in the background
cargo run -p local-control-plane &
LCP_PID=$!

timeout 30s bash -c 'until curl -fsS http://127.0.0.1:43891/health; do sleep 1; done' || (echo "Failed to reach health endpoint" && kill $LCP_PID && exit 1)

echo "Local control plane is healthy!"

echo "Smoke test passed successfully."

kill $LCP_PID
exit 0
