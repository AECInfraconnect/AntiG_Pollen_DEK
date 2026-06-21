#!/usr/bin/env bash
set -euo pipefail

cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p dek-control-plane-api
cargo test -p local-control-plane

export DEK_LCP_AUTH_DISABLE=1
export DEK_LCP_DB='sqlite://./target/e2e/pollen-local.db?mode=rwc'
export DEK_LCP_DATA='./target/e2e/pollen-local-data'

cargo run -p local-control-plane &
LCP_PID=$!
trap 'kill $LCP_PID || true' EXIT
sleep 5

curl -fsS http://127.0.0.1:3000/health
cargo test -p local-control-plane --test e2e_registry
cargo test -p local-control-plane --test e2e_policy_publish

pushd apps/local-admin-dashboard
npm ci
npm run build
npx playwright test
popd
