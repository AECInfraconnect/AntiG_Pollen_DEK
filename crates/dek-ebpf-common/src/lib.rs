#![no_std]

pub const LPM_MAP_CAPACITY: u32 = 4096;
pub const PORTS_MAP_CAPACITY: u32 = 4096;
pub const CGROUP_MAP_CAPACITY: u32 = 1024;

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
