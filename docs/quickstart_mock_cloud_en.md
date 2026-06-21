# Mock-Cloud Sandbox Quickstart

The Pollen DEK Mock-Cloud provides a local, self-contained environment to test DEK integration without a real backend.

## Quickstart Steps

Follow these 6 steps to complete the sandbox scenario:

### 1. Start Mock Cloud

```bash
pollen-mock-cloud --seed sandbox
```

### 2. Enroll DEK

```bash
pollen-dekctl enroll --cloud-url https://127.0.0.1:43892
```

### 3. Start DEK

```bash
pollen-dek --config ~/.pollen/dek/bootstrap.json
```

### 4. Start MCP Proxy

```bash
pollen-mcp-proxy --listen 127.0.0.1:8787
```

### 5. Run Test Client

```bash
pollen-dekctl sandbox run mcp-allow-deny
```

### 6. View Telemetry

Access the dashboard at: `https://127.0.0.1:43892/admin/dashboard` (or `http://127.0.0.1:43892` based on your mock-cloud setup).
From here, you can:

- Trigger bundle updates and policy tampering.
- Trigger network outages and chaos events.
- Rotate keys and revoke devices.
- Review recent decisions and trace logs.
