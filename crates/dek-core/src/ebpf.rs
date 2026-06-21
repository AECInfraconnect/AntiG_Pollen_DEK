use tracing::{info, warn};

#[cfg(target_os = "linux")]
pub fn load_and_attach() -> anyhow::Result<()> {
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
