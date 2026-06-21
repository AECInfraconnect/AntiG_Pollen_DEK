# Pollen DEK Architecture

Pollen DEK (Distributed Enforcement Kernel) is a modular, high-performance policy evaluation and enforcement engine written in Rust. 

## Core Components

1. **dek-core**: The main supervisor and sidecar process. It handles gRPC/HTTP API requests, loads configuration, manages the SVID MTLS lifecycle, and coordinates hot-reloads via `dek-policy-syncer`.
2. **dek-policy-router**: A dynamic traffic router that evaluates input requests against multiple registered `PolicyEvaluator` implementations (e.g., Cedar, OpenFGA, OPA).
3. **dek-plugin-host & dek-plugin-sdk**: An extensible SDK to build custom policy adapters or data transform modules (like `pii-redactor`).
4. **OS Enforcement Modules**:
   - `dek-windows-wfp`: Integrates with the Windows Filtering Platform for L4 network enforcement.
   - `dek-macos-nefilter`: Integrates with macOS Network Extensions.
   - `dek-ebpfd`: Leverages eBPF on Linux to monitor and filter DNS and connection traffic.

## Data Flow

1. Application sends `DecisionRequest` to the `dek-core` Sidecar over localhost (`:43890`).
2. `dek-core` forwards the request to `dek-policy-router`.
3. `dek-policy-router` queries the active bundle configuration, invoking the specified evaluators.
4. Any registered `TransformPlugin` (like `pii-redactor`) is applied to obligations or effects.
5. The final `DecisionResponse` is returned to the application.
