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

        // Start DNS Telemetry RingBuf Reader and Map Updater
        let bpf_fs_path = BPFFS_PATH.to_string();
        if let Ok(dns_events_map) = bpf.take_map("DNS_EVENTS") {
            if let Ok(mut ring_buf) = aya::maps::AsyncRingBuf::try_from(dns_events_map) {
                task::spawn(async move {
                    info!("eBPFD DNS RingBuf Reader started");
                    use hickory_proto::op::Message;
                    use hickory_proto::rr::RecordType;
                    use aya::maps::MapData;
                    
                    // We open VERDICT_MAP from the pinned path to insert IPs dynamically
                    let verdict_map_path = format!("{}/VERDICT_MAP", bpf_fs_path);
                    
                    loop {
                        if let Some(item) = ring_buf.next().await {
                            if item.len() >= std::mem::size_of::<dek_ebpf_common::DnsCaptureEvent>() {
                                let event: dek_ebpf_common::DnsCaptureEvent = unsafe { std::ptr::read_unaligned(item.as_ptr() as *const _) };
                                let dlen = event.len as usize;
                                if dlen <= event.data.len() {
                                    if let Ok(msg) = Message::from_vec(&event.data[..dlen]) {
                                        let queries = msg.queries();
                                        for q in queries {
                                            info!("DNS Query Captured: {} ({:?})", q.name(), q.query_type());
                                        }
                                        
                                        // Attempt to open the map and inject IPs
                                        if let Ok(aya::maps::Map::LpmTrie(mut map)) = aya::maps::Map::open(&verdict_map_path, aya::maps::MapData::from_fd(0)) {
                                            for answer in msg.answers() {
                                                if answer.record_type() == RecordType::A {
                                                    if let Some(data) = answer.data() {
                                                        if let Some(ip) = data.as_a() {
                                                            let ip_u32 = u32::from_be_bytes(ip.0);
                                                            let key = dek_ebpf_common::Ipv4LpmKey { prefix_len: 32, ip: ip_u32 };
                                                            let verdict = dek_ebpf_common::PolicyVerdict { allow: 1, log_event: 1 };
                                                            if let Err(e) = map.insert(&key, &verdict, 0) {
                                                                tracing::error!("Failed to insert IP into VERDICT_MAP: {}", e);
                                                            } else {
                                                                info!("Added IP {} to VERDICT_MAP (DNS-observe)", ip);
                                                                // Future: Also send this to TTL sweeper channel
                                                            }
                                                        }
                                                    }
                                                }
                                            }
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
        let bpf_fs_path2 = BPFFS_PATH.to_string();
        task::spawn(async move {
            info!("eBPFD Map Compiler & TTL Sweeper started");
            let mut ttl_index: std::collections::HashMap<u32, std::time::Instant> = std::collections::HashMap::new();
            // Note: In a complete implementation, the DNS reader task would send newly observed IPs 
            // over an mpsc::channel to this TTL sweeper loop to populate `ttl_index`.

            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                let now = std::time::Instant::now();
                let verdict_map_path = format!("{}/VERDICT_MAP", bpf_fs_path2);
                
                if let Ok(aya::maps::Map::LpmTrie(mut map)) = aya::maps::Map::open(&verdict_map_path, aya::maps::MapData::from_fd(0)) {
                    ttl_index.retain(|ip, expires_at| {
                        if now > *expires_at {
                            info!("Evicting IP {} from VERDICT_MAP due to TTL expiration", ip);
                            let key = dek_ebpf_common::Ipv4LpmKey { prefix_len: 32, ip: *ip };
                            let _ = map.remove(&key);
                            false
                        } else {
                            true
                        }
                    });
                }
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
