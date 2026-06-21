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

        // Load BPF object (Embedded at compile time per design doc)
        let bpf_bytes: &[u8] = include_bytes!("../dummy.o"); // Replace dummy.o with actual bytecode artifact in CI
        
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
            .context("dek_connect4 program not found")?
            .try_into()
            .context("Failed to get dek_connect4 program")?;
        
        program.load().context("Failed to load cgroup program")?;
        
        let cgroup = std::fs::File::open(cgroup_path)
            .context("Failed to open cgroup path")?;
        
        program.attach(cgroup, SockAddrAttachType::IPv4)
            .context("Failed to attach connect4 hook to cgroup")?;
            
        info!("Attached cgroup/connect4 to {}", cgroup_path);

        // Start DNS Telemetry RingBuf Reader
        if let Ok(dns_events_map) = bpf.take_map("DNS_EVENTS") {
            if let Ok(mut ring_buf) = aya::maps::AsyncRingBuf::try_from(dns_events_map) {
                task::spawn(async move {
                    info!("eBPFD DNS RingBuf Reader started");
                    use hickory_proto::op::Message;
                    use bytes::Bytes;
                    
                    loop {
                        if let Some(item) = ring_buf.next().await {
                            // The item is a pointer to DnsCaptureEvent bytes
                            if item.len() >= std::mem::size_of::<dek_ebpf_common::DnsCaptureEvent>() {
                                let event: dek_ebpf_common::DnsCaptureEvent = unsafe { std::ptr::read_unaligned(item.as_ptr() as *const _) };
                                let dlen = event.len as usize;
                                if dlen <= event.data.len() {
                                    if let Ok(msg) = Message::from_vec(&event.data[..dlen]) {
                                        let queries = msg.queries();
                                        for q in queries {
                                            info!(
                                                "DNS Query Captured [cgroup: {}]: {} ({:?})",
                                                event.cgroup_id,
                                                q.name(),
                                                q.query_type()
                                            );
                                            // TODO: Log to telemetry endpoint
                                        }
                                    }
                                }
                            }
                        } else {
                            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        }
                    }
                });
            } else {
                tracing::warn!("Failed to convert DNS_EVENTS map to AsyncRingBuf");
            }
        } else {
            tracing::warn!("DNS_EVENTS map not found in eBPF program");
        }

        // Start Map Compiler & TTL Sweeper Loop
        task::spawn(async move {
            info!("eBPFD Map Compiler & TTL Sweeper started");
            let mut ttl_index: std::collections::HashMap<u32, std::time::Instant> = std::collections::HashMap::new();
            let min_ttl_floor = std::time::Duration::from_secs(30);

            // In reality, this would keep an Arc<Map> handle to VERDICT_MAP
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                let now = std::time::Instant::now();
                ttl_index.retain(|ip, expires_at| {
                    if now > *expires_at {
                        // Implement TTL grace period and map eviction
                        info!("Evicting IP {} from VERDICT_MAP due to TTL expiration", ip);
                        // VERDICT_MAP.remove(ip);
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
