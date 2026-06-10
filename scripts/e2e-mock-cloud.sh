#!/usr/bin/env bash
set -euo pipefail

export MOCK_CLOUD_TENANT='acme-dev'
export MOCK_CLOUD_AUTH_DISABLE=1

cargo run -p mock-cloud -- --listen 127.0.0.1:8787 &
MOCK_PID=$!
trap 'kill $MOCK_PID || true' EXIT
sleep 5

curl -fsS http://127.0.0.1:8787/health

curl -fsS -X POST http://127.0.0.1:8787/v1/tenants/acme-dev/registry/agents \
  -H 'Content-Type: application/json' \
  -H 'x-mock-role: admin' \
  --data @examples/e2e/agent.json

curl -fsS http://127.0.0.1:8787/v1/tenants/acme-dev/registry/agents
