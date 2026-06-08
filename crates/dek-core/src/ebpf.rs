use tracing::{info, warn};

#[cfg(target_os = "linux")]
fn has_bpf_caps() -> bool {
    if let Ok(content) = std::fs::read_to_string("/proc/self/status") {
        for line in content.lines() {
            if line.starts_with("CapEff:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() == 2 {
                    if let Ok(cap) = u64::from_str_radix(parts[1], 16) {
                        // CAP_SYS_ADMIN is bit 21, CAP_BPF is bit 39
                        return (cap & (1 << 39)) != 0 || (cap & (1 << 21)) != 0;
                    }
                }
            }
        }
    }
    false
}

#[cfg(target_os = "linux")]
pub fn probe_ebpf_support() -> bool {
    // Basic checks for eBPF support on Linux:
    // 1. Check for CAP_BPF or CAP_SYS_ADMIN capabilities (Least-Privilege)
    // 2. Check for BTF support
    let has_caps = has_bpf_caps();
    let has_btf = std::path::Path::new("/sys/kernel/btf/vmlinux").exists();

    if !has_caps {
        warn!("Missing CAP_BPF or CAP_SYS_ADMIN. Falling back to App-Layer-Only.");
    }
    if !has_btf {
        warn!("Kernel BTF (/sys/kernel/btf/vmlinux) not found. Falling back to App-Layer-Only.");
    }

    has_caps && has_btf
}

#[cfg(target_os = "linux")]
pub async fn load_and_attach(
    obs_tx: Option<tokio::sync::mpsc::Sender<dek_ebpfd::DnsObservation>>,
) -> Option<dek_ebpfd::EbpfHandle> {
    if !probe_ebpf_support() {
        warn!("eBPF unsupported; degrading to app-layer only.");
        return None;
    }
    let cgroup = "/sys/fs/cgroup/pollen-dek-supervised";
    match dek_ebpfd::start_ebpfd_supervisor(cgroup, obs_tx).await {
        Ok(handle) => {
            info!("eBPF Control Point active.");
            Some(handle)
        }
        Err(e) => {
            tracing::error!("eBPFD failed: {e}");
            None
        }
    }
}

#[cfg(not(target_os = "linux"))]
pub async fn load_and_attach(
    _obs_tx: Option<tokio::sync::mpsc::Sender<dek_ebpfd::DnsObservation>>,
) -> Option<dek_ebpfd::EbpfHandle> {
    info!("Layer 2 eBPF WS-D guardrails are skipped on non-Linux platforms.");
    warn!("Platform relies solely on App-layer MCP and opt-in proxy redirect.");
    Some(dek_ebpfd::EbpfHandle)
}
