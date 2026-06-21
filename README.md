# Pollen DEK - Open Source Desktop Enforcement Kit

Pollen DEK is an Apache-2.0 open-source runtime for enforcing and observing AI agent, MCP, API, and tool-call activity at the desktop/edge.

It can run fully locally with the Local Admin Dashboard, or connect to Pollen Cloud for managed enterprise policy, observability, compliance workflows, and support.

## Quickstart

### Local-only mode

```bash
pollen-dek local-admin start
pollen-dek service start --control-plane http://127.0.0.1:8787
pollen-dek mcp-proxy start
```

### Pollen Cloud mode

```bash
pollen-dek enroll --cloud https://cloud.pollen.ai --tenant <tenant>
pollen-dek service start
```

## Download

See GitHub Releases.

### Install on Linux
```bash
curl -L https://github.com/AECInfraconnect/AntiG_Pollen_DEK/releases/latest/download/pollen-dek-linux-amd64.tar.gz -o pollen-dek.tar.gz
tar -xzf pollen-dek.tar.gz
sudo ./pollen-dek/install.sh
```

### Install on Windows
```powershell
Invoke-WebRequest -Uri "https://github.com/AECInfraconnect/AntiG_Pollen_DEK/releases/latest/download/pollen-dek-windows-amd64.msi" -OutFile "pollen-dek.msi"
Start-Process msiexec.exe -Wait -ArgumentList '/i pollen-dek.msi /qn'
```

### Install on macOS
```bash
curl -L https://github.com/AECInfraconnect/AntiG_Pollen_DEK/releases/latest/download/pollen-dek-macos-universal.pkg -o pollen-dek.pkg
sudo installer -pkg pollen-dek.pkg -target /
```

## Plugin Architecture

- Policy evaluators: OPA, Cedar, OpenFGA
- Transform plugins: PII Redactor
- Telemetry sinks
- Enforcement providers

## License

Pollen DEK is Apache-2.0. Pollen Cloud is commercial.
