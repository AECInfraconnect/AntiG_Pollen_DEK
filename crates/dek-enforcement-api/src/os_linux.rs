#![allow(unsafe_code)]
#![allow(unused)]
use crate::control_method::TelemetrySink;
use crate::resource_observer::{AgentPidMap, ResourceObserver};
use async_trait::async_trait;
use std::collections::HashMap;

pub struct LinuxEbpfVfsObserver;

#[async_trait]
impl ResourceObserver for LinuxEbpfVfsObserver {
    fn id(&self) -> &str {
        "linux_ebpf_vfs"
    }
    async fn observe(&self, _agents: AgentPidMap, _sink: TelemetrySink) -> anyhow::Result<()> {
        // Actual FFI bindings to fanotify / eBPF
        // Requires root/CAP_SYS_ADMIN. Here we provide the FFI definitions.
        Ok(())
    }
}

#[cfg(target_os = "linux")]
pub mod ffi {
    use libc::{c_int, c_uint};

    pub const FAN_CLASS_NOTIF: c_uint = 0x0000_0000;
    pub const FAN_CLASS_CONTENT: c_uint = 0x0000_0004;
    pub const FAN_CLASS_PRE_CONTENT: c_uint = 0x0000_0008;
    pub const FAN_MARK_ADD: c_uint = 0x0000_0001;
    pub const FAN_MARK_MOUNT: c_uint = 0x0000_0010;
    pub const FAN_ACCESS: u64 = 0x0000_0001;
    pub const FAN_MODIFY: u64 = 0x0000_0002;
    pub const FAN_CLOSE_WRITE: u64 = 0x0000_0008;
    pub const FAN_OPEN: u64 = 0x0000_0020;

    extern "C" {
        pub fn fanotify_init(flags: c_uint, event_f_flags: c_uint) -> c_int;
        pub fn fanotify_mark(
            fanotify_fd: c_int,
            flags: c_uint,
            mask: u64,
            dirfd: c_int,
            pathname: *const libc::c_char,
        ) -> c_int;
    }
}
