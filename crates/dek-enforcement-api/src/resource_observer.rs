use crate::control_method::TelemetrySink;
use async_trait::async_trait;
use std::collections::HashMap;

pub type AgentPidMap = HashMap<u32, String>;

#[async_trait]
pub trait ResourceObserver: Send + Sync {
    fn id(&self) -> &str;
    async fn observe(&self, agents: AgentPidMap, sink: TelemetrySink) -> anyhow::Result<()>;
}

#[cfg(target_os = "linux")]
#[path = "os_linux.rs"]
pub mod os_linux;
#[cfg(target_os = "linux")]
pub use os_linux::LinuxEbpfVfsObserver;

#[cfg(not(target_os = "linux"))]
pub struct LinuxEbpfVfsObserver;
#[cfg(not(target_os = "linux"))]
#[async_trait]
impl ResourceObserver for LinuxEbpfVfsObserver {
    fn id(&self) -> &str {
        "linux_ebpf_vfs"
    }
    async fn observe(&self, _agents: AgentPidMap, _sink: TelemetrySink) -> anyhow::Result<()> {
        Ok(())
    }
}

#[cfg(target_os = "windows")]
#[path = "os_windows.rs"]
pub mod os_windows;
#[cfg(target_os = "windows")]
pub use os_windows::WindowsEtwObserver;

#[cfg(not(target_os = "windows"))]
pub struct WindowsEtwObserver;
#[cfg(not(target_os = "windows"))]
#[async_trait]
impl ResourceObserver for WindowsEtwObserver {
    fn id(&self) -> &str {
        "windows_etw"
    }
    async fn observe(&self, _agents: AgentPidMap, _sink: TelemetrySink) -> anyhow::Result<()> {
        Ok(())
    }
}

#[cfg(target_os = "macos")]
#[path = "os_macos.rs"]
pub mod os_macos;
#[cfg(target_os = "macos")]
pub use os_macos::MacosEndpointSecurityObserver;

#[cfg(not(target_os = "macos"))]
pub struct MacosEndpointSecurityObserver;
#[cfg(not(target_os = "macos"))]
#[async_trait]
impl ResourceObserver for MacosEndpointSecurityObserver {
    fn id(&self) -> &str {
        "macos_endpoint_security"
    }
    async fn observe(&self, _agents: AgentPidMap, _sink: TelemetrySink) -> anyhow::Result<()> {
        Ok(())
    }
}
