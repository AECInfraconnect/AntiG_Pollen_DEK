// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

#![no_std]

pub const LPM_MAP_CAPACITY: u32 = 4096;
pub const PORTS_MAP_CAPACITY: u32 = 4096;
pub const CGROUP_MAP_CAPACITY: u32 = 1024;

pub const DEK_DOMAIN_HASH_LEN: usize = 32;
pub const DEK_DNS_CACHE_MAX_ENTRIES: u32 = 262_144;
pub const DEK_CONN_CACHE_MAX_ENTRIES: u32 = 524_288;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Ipv4LpmKey {
    pub prefix_len: u32,
    pub ip: u32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Ipv6LpmKey {
    pub prefix_len: u32,
    pub ip: [u8; 16],
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct DekIp4Key {
    pub ip_be: u32,
    pub netns_cookie_lo: u32,
    pub netns_cookie_hi: u32,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct DekIp6Key {
    pub ip6: [u8; 16],
    pub netns_cookie_lo: u32,
    pub netns_cookie_hi: u32,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct DekDnsCacheValue {
    pub domain_hash: [u8; DEK_DOMAIN_HASH_LEN],
    pub first_seen_ns: u64,
    pub last_seen_ns: u64,
    pub expires_at_ns: u64,
    pub policy_id: u32,
    pub tenant_id: u32,
    pub source: u32,
    pub flags: u32,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct DekConn4Key {
    pub saddr_be: u32,
    pub daddr_be: u32,
    pub sport_be: u16,
    pub dport_be: u16,
    pub proto: u8,
    pub pad: [u8; 3],
    pub netns_cookie_lo: u32,
    pub netns_cookie_hi: u32,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct DekConnDecisionValue {
    pub created_at_ns: u64,
    pub expires_at_ns: u64,
    pub decision: u32, // 0 unknown, 1 allow, 2 deny, 3 observe
    pub policy_id: u32,
    pub reason_code: u32,
    pub flags: u32,
}

#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
pub struct DekMetrics {
    pub dns_cache_hit: u64,
    pub dns_cache_miss: u64,
    pub dns_cache_expired: u64,
    pub conn_cache_hit: u64,
    pub conn_cache_miss: u64,
    pub fallback_allow: u64,
    pub fallback_deny: u64,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PolicyVerdict {
    pub allow: u8, // 1 = allow, 0 = deny
    pub log_event: u8,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct EgressEvent {
    pub pid: u32,
    pub cgroup_id: u64,
    pub dest_ip: u32,
    pub dest_port: u16,
    pub action_taken: u8, // 1 = allowed, 0 = denied
}

pub const DNS_PAYLOAD_MAX: usize = 512;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DnsCaptureEvent {
    pub cgroup_id: u64,
    pub len: u16,
    pub data: [u8; DNS_PAYLOAD_MAX],
}

#[allow(unsafe_code)]
#[cfg(all(feature = "user", target_os = "linux"))]
unsafe impl aya::Pod for Ipv4LpmKey {}
#[allow(unsafe_code)]
#[cfg(all(feature = "user", target_os = "linux"))]
unsafe impl aya::Pod for Ipv6LpmKey {}
#[allow(unsafe_code)]
#[cfg(all(feature = "user", target_os = "linux"))]
unsafe impl aya::Pod for DekIp4Key {}
#[allow(unsafe_code)]
#[cfg(all(feature = "user", target_os = "linux"))]
unsafe impl aya::Pod for DekIp6Key {}
#[allow(unsafe_code)]
#[cfg(all(feature = "user", target_os = "linux"))]
unsafe impl aya::Pod for DekDnsCacheValue {}
#[allow(unsafe_code)]
#[cfg(all(feature = "user", target_os = "linux"))]
unsafe impl aya::Pod for DekConn4Key {}
#[allow(unsafe_code)]
#[cfg(all(feature = "user", target_os = "linux"))]
unsafe impl aya::Pod for DekConnDecisionValue {}
#[allow(unsafe_code)]
#[cfg(all(feature = "user", target_os = "linux"))]
unsafe impl aya::Pod for DekMetrics {}
#[allow(unsafe_code)]
#[cfg(all(feature = "user", target_os = "linux"))]
unsafe impl aya::Pod for PolicyVerdict {}
#[allow(unsafe_code)]
#[cfg(all(feature = "user", target_os = "linux"))]
unsafe impl aya::Pod for EgressEvent {}
#[allow(unsafe_code)]
#[cfg(all(feature = "user", target_os = "linux"))]
unsafe impl aya::Pod for DnsCaptureEvent {}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct DekRuntimeMode {
    pub default_action: u32, // 1 = allow (observe-only), 0 = deny (fail-closed)
}

#[allow(unsafe_code)]
#[cfg(all(feature = "user", target_os = "linux"))]
unsafe impl aya::Pod for DekRuntimeMode {}
