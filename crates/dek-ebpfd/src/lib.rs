#[cfg(target_os = "linux")]
pub mod daemon {
    use anyhow::{Context, Result};
    use aya::{
        programs::{CgroupSockAddr, SockAddrAttachType},
        Ebpf,
    };
    use tracing::{error, info};
    use std::fs;
    use tokio::task;

    const BPFFS_PATH: &str = "/sys/fs/bpf/pollen-dek";

    pub async fn start_ebpfd_supervisor(cgroup_path: &str) -> Result<()> {
        info!("Starting eBPFD Supervisor (Userspace daemon for WS-D)...");

        // Ensure BPFFS directory exists
        if let Err(e) = fs::create_dir_all(BPFFS_PATH) {
            error!("Failed to create BPFFS path at {}: {}", BPFFS_PATH, e);
            // It might fail if not mounted, normally systemd mounts /sys/fs/bpf
        }

        // Ensure the supervised cgroup exists, separating it from DEK's own cgroup
        if let Err(e) = fs::create_dir_all(cgroup_path) {
            error!("Failed to create supervised cgroup {}: {}", cgroup_path, e);
        } else {
            info!("Created/Verified scoped cgroup at {}", cgroup_path);
        }

        // Load BPF object (mock bytes for compilation)
        let bpf_bytes: &[u8] = &[]; // include_bytes!(...) in reality
        if bpf_bytes.is_empty() {
            info!("eBPF byte code is empty (compile time placeholder). Skipping real attach.");
            return Ok(());
        }

        let mut bpf = Ebpf::load(bpf_bytes).context("Failed to load eBPF object")?;

        // Extract maps and pin them
        let map_names = ["VERDICT_MAP", "PORTS_MAP", "CGROUP_POLICY_MAP", "EVENTS"];
        for name in map_names {
            if let Some(map) = bpf.map_mut(name) {
                let pin_path = format!("{}/{}", BPFFS_PATH, name);
                let _ = fs::remove_file(&pin_path);
                if let Err(e) = map.pin(&pin_path) {
                    error!("Failed to pin map {} to {}: {}", name, pin_path, e);
                } else {
                    info!("Pinned map {} to {}", name, pin_path);
                }
            }
        }

        // Attach cgroup/connect4
        let program: &mut CgroupSockAddr = bpf
            .program_mut("dek_connect4")
            .unwrap()
            .try_into()
            .context("Failed to get dek_connect4 program")?;
        
        program.load().context("Failed to load cgroup program")?;
        
        let cgroup = std::fs::File::open(cgroup_path)
            .context("Failed to open cgroup path")?;
        
        program.attach(cgroup, SockAddrAttachType::IPv4)
            .context("Failed to attach connect4 hook to cgroup")?;
            
        info!("Attached cgroup/connect4 to {}", cgroup_path);

        // Start DNS Telemetry RingBuf Reader
        task::spawn(async move {
            info!("eBPFD DNS RingBuf Reader started");
            // Placeholder: Read DNS_EVENTS RingBuf using async API.
            // Parse payload with hickory-proto:
            // if let Ok(msg) = Message::from_vec(&event.data[..event.len]) { ... }
            // and log to Telemetry.
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        });

        // Start Map Compiler & TTL Sweeper Loop
        task::spawn(async move {
            info!("eBPFD Map Compiler & TTL Sweeper started");
            let mut ttl_index: std::collections::HashMap<u32, std::time::Instant> = std::collections::HashMap::new();
            let min_ttl_floor = std::time::Duration::from_secs(30);

            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                let now = std::time::Instant::now();
                ttl_index.retain(|_ip, expires_at| {
                    if now > *expires_at {
                        // In reality: bpf_map_delete_elem for VERDICT_MAP
                        // info!("Evicting IP {} from VERDICT_MAP due to TTL", ip);
                        false
                    } else {
                        true
                    }
                });
            }
        });

        Ok(())
    }
}

#[cfg(not(target_os = "linux"))]
pub mod daemon {
    pub async fn start_ebpfd_supervisor(_cgroup_path: &str) -> anyhow::Result<()> {
        tracing::warn!("eBPFD supervisor is only supported on Linux.");
        Ok(())
    }
}
