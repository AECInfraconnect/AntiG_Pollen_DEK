#![no_std]
#![no_main]

use aya_bpf::{
    macros::{cgroup_sock_addr, map},
    maps::{HashMap, LpmTrie, RingBuf},
    programs::SockAddrContext,
};
use dek_ebpf_common::{
    EgressEvent, Ipv4LpmKey, PolicyVerdict, CGROUP_MAP_CAPACITY, LPM_MAP_CAPACITY,
    PORTS_MAP_CAPACITY,
};

#[map]
static VERDICT_MAP: LpmTrie<Ipv4LpmKey, PolicyVerdict> =
    LpmTrie::with_max_entries(LPM_MAP_CAPACITY, 0);

#[map]
static PORTS_MAP: HashMap<u16, PolicyVerdict> = HashMap::with_max_entries(PORTS_MAP_CAPACITY, 0);

#[map]
static CGROUP_POLICY_MAP: HashMap<u64, PolicyVerdict> =
    HashMap::with_max_entries(CGROUP_MAP_CAPACITY, 0);

#[map]
static EVENTS: RingBuf = RingBuf::with_byte_size(256 * 1024, 0);

#[cgroup_sock_addr(connect4)]
pub fn dek_connect4(ctx: SockAddrContext) -> i32 {
    match try_dek_connect4(ctx) {
        Ok(ret) => ret,
        Err(_) => 1, // Default allow on error
    }
}

fn try_dek_connect4(ctx: SockAddrContext) -> Result<i32, ()> {
    let dest_ip = u32::from_be(ctx.user_ip4());
    let dest_port = u16::from_be(ctx.user_port() as u16);
    let cgroup_id = ctx.cgroup_id();
    let pid = ctx.pid() as u32;

    let mut verdict = PolicyVerdict { allow: 1, log_event: 0 };

    // 1. Check cgroup specific policy
    if let Some(v) = unsafe { CGROUP_POLICY_MAP.get(&cgroup_id) } {
        verdict = *v;
    } else {
        // 2. Check LPM Trie for IP CIDR
        let key = Ipv4LpmKey {
            prefix_len: 32,
            ip: dest_ip,
        };
        if let Some(v) = unsafe { VERDICT_MAP.get(&key) } {
            verdict = *v;
        } else {
            // 3. Check ports map
            if let Some(v) = unsafe { PORTS_MAP.get(&dest_port) } {
                verdict = *v;
            }
        }
    }

    if verdict.log_event != 0 {
        if let Some(mut buf) = EVENTS.reserve::<EgressEvent>(0) {
            let event = EgressEvent {
                pid,
                cgroup_id,
                dest_ip,
                dest_port,
                action_taken: verdict.allow,
            };
            unsafe {
                core::ptr::write_unaligned(buf.as_mut_ptr() as *mut EgressEvent, event);
            }
            buf.submit(0);
        }
    }

    Ok(verdict.allow as i32)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
