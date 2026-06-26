#![allow(unsafe_code)]
#![allow(unused)]
use crate::control_method::TelemetrySink;
use crate::resource_observer::{AgentPidMap, ResourceObserver};
use async_trait::async_trait;
use std::collections::HashMap;

pub struct MacosEndpointSecurityObserver;

#[async_trait]
impl ResourceObserver for MacosEndpointSecurityObserver {
    fn id(&self) -> &str {
        "macos_endpoint_security"
    }
    async fn observe(&self, _agents: AgentPidMap, _sink: TelemetrySink) -> anyhow::Result<()> {
        // Actual FFI bindings to macOS Endpoint Security
        // Requires root and Apple Developer Endpoint Security entitlement.
        Ok(())
    }
}

#[cfg(target_os = "macos")]
pub mod ffi {
    use libc::{c_int, c_void, size_t, uint32_t};

    pub type es_client_t = *mut c_void;
    pub type es_message_t = *mut c_void;

    pub const ES_NEW_CLIENT_RESULT_SUCCESS: u32 = 0;
    pub const ES_NEW_CLIENT_RESULT_ERR_NOT_ENTITLED: u32 = 1;
    pub const ES_NEW_CLIENT_RESULT_ERR_NOT_PERMITTED: u32 = 2;
    pub const ES_NEW_CLIENT_RESULT_ERR_NOT_PRIVILEGED: u32 = 3;

    pub const ES_EVENT_TYPE_NOTIFY_OPEN: u32 = 14;
    pub const ES_EVENT_TYPE_NOTIFY_CLOSE: u32 = 15;
    pub const ES_EVENT_TYPE_NOTIFY_WRITE: u32 = 16;
    pub const ES_EVENT_TYPE_NOTIFY_EXEC: u32 = 25;

    // A C block type for the callback. Rust doesn't natively support Objective-C blocks perfectly
    // without `block` crate, but we declare the opaque pointer.
    pub type es_handler_block_t = *mut c_void;

    extern "C" {
        pub fn es_new_client(client: *mut es_client_t, handler: es_handler_block_t) -> u32;

        pub fn es_subscribe(client: es_client_t, events: *const u32, event_count: u32) -> u32;

        pub fn es_unsubscribe_all(client: es_client_t) -> u32;

        pub fn es_delete_client(client: es_client_t) -> u32;
    }
}
