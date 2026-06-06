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
pub fn load_and_attach() -> anyhow::Result<()> {
    info!("Probing eBPF support...");
    if !probe_ebpf_support() {
        warn!("eBPF support check failed. Gracefully degrading to App-Layer-Only (Layer 3/7).");
        return Ok(());
    }

    info!("Initializing WS-D eBPFD Subsystem...");
    
    // eBPFD is spawned asynchronously to manage BPF maps and ringbuf
    // Passing the cgroup path of the supervised processes
    let cgroup_path = "/sys/fs/cgroup/pollen-dek-supervised";
    
    // Start supervisor logic
    tokio::spawn(async move {
        if let Err(e) = dek_ebpfd::daemon::start_ebpfd_supervisor(cgroup_path).await {
            tracing::error!("eBPFD Supervisor failed: {}", e);
        }
    });

    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn load_and_attach() -> anyhow::Result<()> {
    info!("Layer 2 eBPF WS-D guardrails are skipped on non-Linux platforms.");
    warn!("Platform relies solely on App-layer MCP and opt-in proxy redirect.");
    Ok(())
}
