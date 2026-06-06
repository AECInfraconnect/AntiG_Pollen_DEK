use dek_ipc::{IpcMessage, IpcRequest, IpcResponse};
use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{debug, error};

/// Sends a health check request to the local IPC endpoint using LinesCodec.
pub async fn ipc_health_ok(addr: &str) -> bool {
    let connect_timeout = Duration::from_secs(2);
    let stream = match timeout(connect_timeout, TcpStream::connect(addr)).await {
        Ok(Ok(s)) => s,
        _ => {
            debug!("Failed to connect to IPC endpoint at {}", addr);
            return false;
        }
    };

    let mut framed = Framed::new(stream, LinesCodec::new_with_max_length(64 * 1024));

    let req = IpcMessage {
        version: "1.0".to_string(),
        payload: IpcRequest::HealthCheck,
    };

    let req_json = match serde_json::to_string(&req) {
        Ok(json) => json,
        Err(e) => {
            error!("Failed to serialize health check request: {}", e);
            return false;
        }
    };

    if let Err(e) = framed.send(req_json).await {
        debug!("Failed to send health check to IPC: {}", e);
        return false;
    }

    match timeout(Duration::from_secs(3), framed.next()).await {
        Ok(Some(Ok(line))) => {
            if let Ok(res_msg) = serde_json::from_str::<IpcMessage<IpcResponse>>(&line) {
                if let IpcResponse::HealthStatus { status, .. } = res_msg.payload {
                    return status == "HEALTHY";
                }
            }
        }
        Ok(Some(Err(e))) => debug!("Error reading IPC response: {}", e),
        Ok(None) => debug!("IPC stream closed before response"),
        Err(_) => debug!("IPC response timed out"),
    }

    false
}
