// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

//! ipc_client.rs — minimal client for the local DEK IPC endpoint.
//!
//! Used by the probation health-probe (and reusable by `dekctl health`).
//! Speaks the exact wire format of the IPC server in `main.rs`:
//! newline-delimited JSON of `IpcMessage<IpcRequest>` / `IpcMessage<IpcResponse>`
//! over `LinesCodec`, version string "1.x".

use anyhow::{Context, Result};
use dek_ipc::{IpcMessage, IpcRequest, IpcResponse};
use futures::{SinkExt, StreamExt};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_util::codec::{Framed, LinesCodec};

const IPC_MAX_LINE_BYTES: usize = 64 * 1024;
const IPC_TIMEOUT: Duration = Duration::from_secs(2);
const IPC_VERSION: &str = "1.0";

/// Liveness + readiness probe: connect, do a HealthCheck roundtrip, and require
/// the server to report "HEALTHY". Any failure (connect/timeout/parse/non-healthy)
/// returns `false` — callers treat that as "not yet healthy".
pub async fn health_ok(addr: &str) -> bool {
    match health_roundtrip(addr).await {
        Ok(true) => true,
        Ok(false) => {
            tracing::warn!("ipc health probe: server did not report HEALTHY");
            false
        }
        Err(e) => {
            tracing::warn!("ipc health probe failed: {e}");
            false
        }
    }
}

async fn health_roundtrip(addr: &str) -> Result<bool> {
    let stream = timeout(IPC_TIMEOUT, TcpStream::connect(addr))
        .await
        .context("ipc connect timed out")?
        .context("ipc connect failed")?;

    let mut framed = Framed::new(stream, LinesCodec::new_with_max_length(IPC_MAX_LINE_BYTES));

    let req = IpcMessage {
        version: IPC_VERSION.to_string(),
        payload: IpcRequest::HealthCheck,
    };
    let line = serde_json::to_string(&req).context("serialize ipc request")?;
    timeout(IPC_TIMEOUT, framed.send(line))
        .await
        .context("ipc send timed out")?
        .context("ipc send failed")?;

    let resp_line = timeout(IPC_TIMEOUT, framed.next())
        .await
        .context("ipc read timed out")?
        .context("ipc connection closed before response")?
        .context("ipc read error")?;

    let resp: IpcMessage<IpcResponse> =
        serde_json::from_str(&resp_line).context("parse ipc response")?;

    Ok(matches!(
        resp.payload,
        IpcResponse::HealthStatus { status, .. } if status == "HEALTHY"
    ))
}
