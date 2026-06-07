# Pollen DEK Capability Matrix

The following table explicitly defines the support status of various capabilities across platforms.

| Capability                           | Linux        | Windows       | macOS         | Status          |
|--------------------------------------|--------------|---------------|---------------|-----------------|
| MCP HTTP PEP                         | Supported    | Supported     | Supported     | Target GA       |
| MCP stdio PEP                        | Supported    | Supported     | Supported     | Target GA       |
| Network egress eBPF                  | Supported    | Not supported | Not supported | Linux only      |
| System-wide transparent interception | Limited      | Not supported | Not supported | Do not claim    |
| Opt-in proxy redirect                | Supported    | Supported     | Supported     | Target GA       |
| Envoy/Istio ext_authz                | Supported    | N/A           | N/A           | Container/K8s   |
