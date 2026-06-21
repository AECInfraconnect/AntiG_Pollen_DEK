$ErrorActionPreference = "Stop"

cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p dek-control-plane-api
cargo test -p local-control-plane

$env:DEK_LCP_AUTH_DISABLE="1"
$env:DEK_LCP_DB="sqlite://./target/e2e/pollen-local.db?mode=rwc"
$env:DEK_LCP_DATA="./target/e2e/pollen-local-data"

$proc = Start-Process cargo -ArgumentList "run -p local-control-plane" -PassThru
Start-Sleep -Seconds 5

try {
  Invoke-RestMethod http://127.0.0.1:3000/health
  cargo test -p local-control-plane --test e2e_registry
  cargo test -p local-control-plane --test e2e_policy_publish

  Push-Location apps/local-admin-dashboard
  npm ci
  npm run build
  npx playwright test
  Pop-Location
}
finally {
  Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
}
