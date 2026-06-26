use crate::control_method::TelemetrySink;
use async_trait::async_trait;

/// Core interface for capturing network egress events and piping them to the telemetry sink.
#[async_trait]
pub trait EgressEventSource: Send + Sync {
    /// Returns the unique identifier for this egress observer (e.g. "windows_wfp_egress").
    fn id(&self) -> &str;

    /// Starts observing network connections and pushes `ResourceAccessPayload` to the sink.
    async fn start_observing(&self, sink: TelemetrySink) -> anyhow::Result<()>;
}

// -----------------------------------------------------------------------------
// FFI Stubs for Future Implementation (PR-RT-Network)
// -----------------------------------------------------------------------------

pub struct WindowsWfpEgressSource;

#[async_trait]
impl EgressEventSource for WindowsWfpEgressSource {
    fn id(&self) -> &str {
        "windows_wfp_egress"
    }

    async fn start_observing(&self, _sink: TelemetrySink) -> anyhow::Result<()> {
        anyhow::bail!(
            "windows_wfp_egress is not active: install and approve the Pollek WFP service before real network observation or blocking"
        )
    }
}

pub struct MacNetworkExtensionEgressSource;

#[async_trait]
impl EgressEventSource for MacNetworkExtensionEgressSource {
    fn id(&self) -> &str {
        "macos_network_extension_egress"
    }

    async fn start_observing(&self, _sink: TelemetrySink) -> anyhow::Result<()> {
        anyhow::bail!(
            "macos_network_extension_egress is not active: install and approve the NetworkExtension system extension before real network observation or blocking"
        )
    }
}

pub struct LinuxEbpfEgressSource;

#[async_trait]
impl EgressEventSource for LinuxEbpfEgressSource {
    fn id(&self) -> &str {
        "linux_ebpf_egress"
    }

    async fn start_observing(&self, _sink: TelemetrySink) -> anyhow::Result<()> {
        anyhow::bail!(
            "linux_ebpf_egress is not active: run with CAP_BPF/CAP_NET_ADMIN and load the Pollek eBPF program before real network observation or blocking"
        )
    }
}
