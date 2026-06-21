# Pollen DEK (Data Enrichment Kit)

Pollen DEK is an extensible, highly concurrent and robust edge computing proxy node written in Rust. It serves as a secure, local interception point for traffic and events routing between applications and the Pollen Cloud.

## Architecture

- **dek-core**: Supervisor service managing the device lifecycle, configuration bootstrapping via mTLS, telemetry emission, and bundle synchronization.
- **dek-policy-router**: Acts as an API proxy forwarding payloads to WebAssembly components based on dynamic policies.
- **dek-policy-runtime**: WASI-based runtime embedded via Wasmtime for executing isolated, dynamic WebAssembly modules.
- **dek-telemetry**: Streams telemetry and Prometheus metrics back to Pollen Cloud.
- **dek-openfga** and **dek-cedar**: Integrations for fine-grained authorization with external stores (OpenFGA) and Cedar policy engine.
- **mock-cloud**: A mock Pollen Cloud for development and testing.

## License

This project is licensed under the Apache License 2.0. See the [LICENSE](LICENSE) file for more information.
