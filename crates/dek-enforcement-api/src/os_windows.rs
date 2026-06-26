#![allow(unsafe_code)]
#![allow(unused)]
use crate::control_method::TelemetrySink;
use crate::resource_observer::{AgentPidMap, ResourceObserver};
use async_trait::async_trait;
use std::collections::HashMap;

pub struct WindowsEtwObserver;

#[async_trait]
impl ResourceObserver for WindowsEtwObserver {
    fn id(&self) -> &str {
        "windows_etw"
    }
    async fn observe(&self, _agents: AgentPidMap, _sink: TelemetrySink) -> anyhow::Result<()> {
        Ok(())
    }
}

#[cfg(target_os = "windows")]
pub mod ffi {
    use windows_sys::Win32::System::Diagnostics::Etw::{
        StartTraceW, CONTROLTRACE_HANDLE, EVENT_TRACE_PROPERTIES,
    };

    // FFI Definitions for ETW (Event Tracing for Windows) mapping
    pub const EVENT_TRACE_CONTROL_QUERY: u32 = 0;
    pub const EVENT_TRACE_CONTROL_STOP: u32 = 1;
    pub const EVENT_TRACE_CONTROL_UPDATE: u32 = 2;
    pub const EVENT_TRACE_CONTROL_FLUSH: u32 = 3;

    pub const PROCESS_TRACE_MODE_REAL_TIME: u32 = 0x00000100;
    pub const PROCESS_TRACE_MODE_EVENT_RECORD: u32 = 0x10000000;

    #[repr(C)]
    pub struct EtwTraceContext {
        pub handle: u64,
        pub props: *mut EVENT_TRACE_PROPERTIES,
    }

    /// # Safety
    ///
    /// `session_name` must be a valid null-terminated UTF-16 pointer and
    /// `properties` must point to a valid writable `EVENT_TRACE_PROPERTIES`
    /// buffer for the lifetime of the call.
    pub unsafe fn start_etw_trace(
        session_name: *const u16,
        properties: *mut EVENT_TRACE_PROPERTIES,
    ) -> u32 {
        let mut handle = CONTROLTRACE_HANDLE { Value: 0 };
        unsafe { StartTraceW(&mut handle, session_name, properties) }
    }
}
