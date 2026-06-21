use anyhow::Result;
use dek_ipc::{IpcRequest, IpcResponse};
use tracing::{error, info};

pub async fn run(host: &str, port: u16) -> Result<()> {
    info!("Sending RotateIdentity request to DEK Core...");
    match crate::send_ipc_request(host, port, IpcRequest::RotateIdentity).await {
        Ok(IpcResponse::RotateStatus { status }) => {
            info!("DEK Core Identity Rotation Status: {}", status);
            println!("✓ Identity rotation initiated.");
        }
        Ok(IpcResponse::Error(e)) => error!("Error from DEK Core: {}", e),
        Ok(_) => error!("Unexpected response from DEK Core"),
        Err(e) => error!("IPC Request Failed: {}", e),
    }
    Ok(())
}
