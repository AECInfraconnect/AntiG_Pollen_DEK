#[cfg(target_os = "linux")]
use anyhow::{Context, Result};
#[cfg(target_os = "linux")]
use aya::{
    maps::{HashMap, MapData},
    Ebpf,
};
#[cfg(target_os = "linux")]
use byteorder::{ByteOrder, NetworkEndian};
#[cfg(target_os = "linux")]
use dek_ebpf_common::{DekDnsCacheValue, DekIp4Key, DEK_DOMAIN_HASH_LEN};
#[cfg(target_os = "linux")]
use std::net::Ipv4Addr;
#[cfg(target_os = "linux")]
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[cfg(target_os = "linux")]
fn now_ns() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_nanos() as u64
}

#[cfg(target_os = "linux")]
fn hash_domain(domain: &str) -> [u8; DEK_DOMAIN_HASH_LEN] {
    let normalized = domain.trim_end_matches('.').to_ascii_lowercase();
    *blake3::hash(normalized.as_bytes()).as_bytes()
}

#[cfg(target_os = "linux")]
pub fn update_dns_ip_cache_v4(
    ip: Ipv4Addr,
    domain: &str,
    ttl: Duration,
    policy_id: u32,
    tenant_id: u32,
) -> Result<()> {
    let pin_path = format!("{}/DNS_IP_CACHE_V4", crate::linux::BPFFS_PATH);
    let map_data = MapData::from_pin(&pin_path).context("load pinned DNS_IP_CACHE_V4")?;
    let mut map: HashMap<_, DekIp4Key, DekDnsCacheValue> =
        HashMap::try_from(aya::maps::Map::HashMap(map_data))?;

    let now = now_ns();
    let key = DekIp4Key {
        ip_be: NetworkEndian::read_u32(&ip.octets()),
        netns_cookie_lo: 0,
        netns_cookie_hi: 0,
    };

    let value = DekDnsCacheValue {
        domain_hash: hash_domain(domain),
        first_seen_ns: now,
        last_seen_ns: now,
        expires_at_ns: now.saturating_add(ttl.as_nanos() as u64),
        policy_id,
        tenant_id,
        source: 1, // DNS
        flags: 0,
    };

    map.insert(key, value, 0)
        .with_context(|| format!("failed to update DNS cache for {domain} -> {ip}"))?;

    Ok(())
}

#[cfg(target_os = "linux")]
pub fn cleanup_expired_dns_cache_v4(scan_limit: usize) -> Result<usize> {
    let pin_path = format!("{}/DNS_IP_CACHE_V4", crate::linux::BPFFS_PATH);
    let map_data = MapData::from_pin(&pin_path).context("load pinned DNS_IP_CACHE_V4")?;
    let mut map: HashMap<_, DekIp4Key, DekDnsCacheValue> =
        HashMap::try_from(aya::maps::Map::HashMap(map_data))?;

    let now = now_ns();
    let mut deleted = 0usize;
    let mut scanned = 0usize;
    let mut to_delete = Vec::new();

    for entry in map.iter() {
        let (key, value) = entry?;
        scanned += 1;

        if value.expires_at_ns != 0 && now > value.expires_at_ns {
            to_delete.push(key);
        }

        if scanned >= scan_limit {
            break;
        }
    }

    for key in to_delete {
        let _ = map.remove(&key);
        deleted += 1;
    }

    Ok(deleted)
}

#[cfg(target_os = "linux")]
pub fn estimate_map_entries_v4(bpf: &mut Ebpf, sample_limit: usize) -> Result<usize> {
    let map: HashMap<_, DekIp4Key, DekDnsCacheValue> = HashMap::try_from(
        bpf.map("DNS_IP_CACHE_V4")
            .context("DNS_IP_CACHE_V4 map not found")?,
    )?;

    let mut count = 0usize;
    for entry in map.iter() {
        let _ = entry?;
        count += 1;
        if count >= sample_limit {
            break;
        }
    }
    Ok(count)
}

#[cfg(target_os = "linux")]
pub fn on_dns_response(
    domain: &str,
    resolved_ips: &[std::net::IpAddr],
    verdict: dek_ebpf_common::PolicyVerdict,
) -> Result<()> {
    use aya::maps::lpm_trie::Key;
    use aya::maps::LpmTrie;
    let pin_path_v4 = format!("{}/VERDICT_MAP", crate::linux::BPFFS_PATH);
    let map_data_v4 = MapData::from_pin(&pin_path_v4).context("load pinned VERDICT_MAP")?;
    let mut verdict_map_v4: LpmTrie<_, u32, dek_ebpf_common::PolicyVerdict> =
        LpmTrie::try_from(aya::maps::Map::LpmTrie(map_data_v4))?;

    let pin_path_v6 = format!("{}/VERDICT_MAP_V6", crate::linux::BPFFS_PATH);
    let map_data_v6 = MapData::from_pin(&pin_path_v6).context("load pinned VERDICT_MAP_V6")?;
    let mut verdict_map_v6: LpmTrie<_, [u32; 4], dek_ebpf_common::PolicyVerdict> =
        LpmTrie::try_from(aya::maps::Map::LpmTrie(map_data_v6))?;

    for ip in resolved_ips {
        match ip {
            std::net::IpAddr::V4(ipv4) => {
                let ip_u32 = u32::from_be_bytes(ipv4.octets());
                let key = Key::new(32, ip_u32);
                verdict_map_v4.insert(&key, verdict, 0)?;
            }
            std::net::IpAddr::V6(ipv6) => {
                let octets = ipv6.octets();
                let mut ip_u32_arr = [0u32; 4];
                for i in 0..4 {
                    ip_u32_arr[i] = u32::from_be_bytes([
                        octets[i * 4],
                        octets[i * 4 + 1],
                        octets[i * 4 + 2],
                        octets[i * 4 + 3],
                    ]);
                }
                let key = Key::new(128, ip_u32_arr);
                verdict_map_v6.insert(&key, verdict, 0)?;
            }
        }
    }
    tracing::info!(domain, count = resolved_ips.len(), action = if verdict.allow == 1 { "allow" } else { "block" }, "domain policy → IP verdicts pushed");
    Ok(())
}
