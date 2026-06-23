# Mock-Cloud Sandbox Quickstart

The Pollek Local Enforcement Kit Mock-Cloud provides a local, self-contained environment to test Local Enforcement Kit integration without a real backend.

## Quickstart Steps

Follow these 6 steps to complete the sandbox scenario:

### 1. Start Mock Cloud

```bash
Pollek-mock-cloud --seed sandbox
```

### 2. Enroll Local Enforcement Kit

```bash
Pollek-dekctl enroll --cloud-url https://127.0.0.1:43892
```

### 3. Start Local Enforcement Kit

```bash
Pollek-Local Enforcement Kit --config ~/.Pollek/Local Enforcement Kit/bootstrap.json
```

### 4. Start MCP Proxy

```bash
Pollek-mcp-proxy --listen 127.0.0.1:8787
```

### 5. Run Test Client

```bash
Pollek-dekctl sandbox run mcp-allow-deny
```

### 6. View Telemetry

Access the dashboard at: `https://127.0.0.1:43892/admin/dashboard` (or `http://127.0.0.1:43892` based on your mock-cloud setup).
From here, you can:

- Trigger bundle updates and policy tampering.
- Trigger network outages and chaos events.
- Rotate keys and revoke devices.
- Review recent decisions and trace logs.
