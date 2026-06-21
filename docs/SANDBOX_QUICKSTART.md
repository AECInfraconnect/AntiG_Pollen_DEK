# Mock-Cloud Sandbox Quickstart

The Pollen DEK Mock-Cloud provides a local, self-contained environment to test DEK integration without a real backend.

## Starting Mock-Cloud
```bash
cargo run -p mock-cloud
```

## Admin Dashboard
Access the dashboard at: `http://127.0.0.1:43892/admin/dashboard`
From here, you can:
- Trigger bundle updates and policy tampering.
- Trigger network outages and chaos events.
- Rotate keys and revoke devices.

## Connecting DEK Core
Run `dek-core` with environment variables pointing to Mock-Cloud:
```bash
export DEK_API_URL=https://127.0.0.1:43891
export DEK_ENROLL_URL=http://127.0.0.1:43892
cargo run -p dek-core
```
