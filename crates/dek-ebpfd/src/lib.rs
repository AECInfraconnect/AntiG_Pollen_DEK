#[cfg(target_os = "linux")]
pub mod daemon {
    use anyhow::{Context, Result};
    use aya::{
        programs::{CgroupSockAddr, SockAddrAttachType},
        Bpf,
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

        // Load BPF object (mock bytes for compilation)
        let bpf_bytes: &[u8] = &[]; // include_bytes!(...) in reality
        if bpf_bytes.is_empty() {
            info!("eBPF byte code is empty (compile time placeholder). Skipping real attach.");
            return Ok(());
        }

        let mut bpf = Bpf::load(bpf_bytes).context("Failed to load eBPF object")?;

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

        // Start Telemetry RingBuf Reader
        task::spawn(async move {
            info!("eBPFD Telemetry Sink listener started");
            // Placeholder: Use aya::maps::AsyncPerfEventArray or AsyncRingBuf to read events
            // and push them to dek-telemetry
        });

        // Start Map Compiler Loop
        task::spawn(async move {
            info!("eBPFD Policy Map Compiler started");
            // Placeholder: Read Cedar policies, resolve DNS, update pinned maps
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
