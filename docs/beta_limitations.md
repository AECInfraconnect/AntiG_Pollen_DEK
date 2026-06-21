# Pollen DEK v1.0.0-beta Known Limitations

Before evaluating or deploying Pollen DEK `v1.0.0-beta`, please be aware of the following known limitations. This release is intended as a **developer sandbox / beta** for testing AI Agent and MCP policy enforcement, rather than an enterprise production-ready security agent.

1. **Mock-Cloud Backend**: Pollen Cloud production integration is not yet finalized. This beta uses `mock-cloud` as the control plane for configuration, policy distribution, and telemetry.
2. **Opt-in Network Egress on Windows/macOS**: Egress network enforcement on Windows and macOS is currently limited to opt-in traffic redirection (e.g., configuring proxy settings or injecting environment variables like `HTTP_PROXY`). Transparent kernel-level interception is not implemented in this phase to maintain system stability.
3. **Experimental eBPF (Linux)**: The eBPF guardrail implementation for Linux network egress filtering is included but considered experimental. It requires root privileges (`CAP_BPF`, `CAP_NET_ADMIN`) and specific kernel versions.
4. **Auto-Update Features**: The automatic update functionality (`pollen-dekctl update`) is in beta and opt-in.
5. **Scale and High Availability**: Enterprise multi-tenant scaling, high availability of the control plane, and large-scale fleet management have not been fully validated in this beta release.
6. **Future Roadmap**: Deep kernel-level enforcement for Windows (WFP) and macOS (Network Extensions) are planned for future roadmap milestones.
7. **Cloud Repository Availability**: The official Pollen Cloud repository (`pollenwithclaw`) was not publicly accessible during the final beta review. Final API contract alignment must be verified once it becomes available.
