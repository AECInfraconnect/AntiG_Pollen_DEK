use anyhow::Result;
use clap::Parser;
use dek_mcp_normalizer::{MessageDirection, NormalizedMcpEvent, TransportType};
use dek_policy_router::PolicyRouter;
use serde_json::{json, Value};
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    server_id: String,

    #[arg(long)]
    agent_id: String,

    #[arg(long)]
    transport: Option<String>,

    #[arg(last = true)]
    command_args: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    let args = Args::parse();

    if args.command_args.is_empty() {
        error!("Error: No command provided to wrap");
        std::process::exit(1);
    }

    info!(
        "dek-stdio-wrapper starting. Server ID: {}, Agent ID: {}",
        args.server_id, args.agent_id
    );

    // Load Bootstrap and Config
    use dek_config::{BootstrapConfig, MtlsConfig};
    let bootstrap =
        BootstrapConfig::load_or_default("bootstrap.json").unwrap_or_else(|_| BootstrapConfig {
            device_id: "local-device".into(),
            mtls: MtlsConfig {
                client_cert_path: "certs/client.crt".to_string(),
                client_key_path: "certs/client.key".to_string(),
                root_ca_path: "certs/root_ca.crt".to_string(),
            },
            pinned_bundle_public_key: "xQyzrpVpR6jeGRNbW+JoX/NIr8Y/w0qDesoSvFwfViU=".to_string(),
        });

    let mut tenant_id = "default-tenant".to_string();
    let mut spiffe_id: Option<String> = None;

    // Setup Adaptive Policy Pipeline
    let mut router = PolicyRouter::new();

    let bundle_path_str = std::env::var("DEK_BUNDLE_PATH").unwrap_or_else(|_| {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("active_bundle.json")
            .to_string_lossy()
            .into_owned()
    });
    let staged_path = std::path::Path::new(&bundle_path_str);
    if staged_path.exists() {
        if let Ok(content) = std::fs::read_to_string(staged_path) {
            if let Ok(payload) = serde_json::from_str::<Value>(&content) {
                info!("Loading dynamic policy evaluator configuration from active_bundle.json");
                dek_router_builder::load_router_config(&mut router, &payload);

                if let Some(t) = payload.get("tenant_id").and_then(|v| v.as_str()) {
                    tenant_id = t.to_string();
                }
                if let Some(s) = payload.get("spiffe_id").and_then(|v| v.as_str()) {
                    spiffe_id = Some(s.to_string());
                }
            }
        }
    }

    let router = Arc::new(RwLock::new(router));
    let mut cmd = Command::new(&args.command_args[0]);
    cmd.args(&args.command_args[1..]);
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn()?;

    let mut child_stdin = child.stdin.take().expect("Failed to open child stdin");
    let child_stdout = child.stdout.take().expect("Failed to open child stdout");
    let child_stderr = child.stderr.take().expect("Failed to open child stderr");

    // Parent streams
    let mut parent_stdin = BufReader::new(tokio::io::stdin()).lines();
    let mut parent_stdout = tokio::io::stdout();

    let (tx_out, mut rx_out) = mpsc::channel::<String>(100);

    // Task 1: Read child stderr and pipe to our stderr
    let mut child_stderr_reader = BufReader::new(child_stderr).lines();
    tokio::spawn(async move {
        while let Ok(Some(line)) = child_stderr_reader.next_line().await {
            info!("[child stderr] {}", line);
        }
    });

    // Task 2: Read child stdout and pipe to our stdout
    let mut child_stdout_reader = BufReader::new(child_stdout).lines();
    let tx_out_clone = tx_out.clone();
    tokio::spawn(async move {
        while let Ok(Some(line)) = child_stdout_reader.next_line().await {
            // Forward unmodified (for now, phase 4 handles redaction)
            let _ = tx_out_clone.send(line).await;
        }
    });

    // Task 3: Read parent stdin, intercept, and optionally pipe to child stdin
    let agent_id = args.agent_id.clone();
    let server_id = args.server_id.clone();
    tokio::spawn(async move {
        while let Ok(Some(line)) = parent_stdin.next_line().await {
            info!("[wrapper] Intercepted Request: {}", line);

            if let Ok(payload) = serde_json::from_str::<Value>(&line) {
                // Determine method for policy router
                let method = payload
                    .get("method")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");

                // Create normalized event shape
                let normalized = NormalizedMcpEvent {
                    event_id: Uuid::new_v4().to_string(),
                    transport: TransportType::Stdio,
                    direction: MessageDirection::Request,
                    request_type: method.to_string(),
                    jsonrpc_id: payload.get("id").cloned(),
                    tenant_id: tenant_id.clone(),
                    device_id: bootstrap.device_id.clone(),
                    spiffe_id: spiffe_id.clone(),
                    user_id: Some(agent_id.clone()),
                    agent_id: Some(agent_id.clone()),
                    server_id: Some(server_id.clone()),
                    tool_name: payload
                        .get("params")
                        .and_then(|p| p.get("name"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    resource_uri: None,
                    prompt_name: None,
                    payload: payload.clone(),
                    session: json!({}),
                    runtime: json!({ "os": std::env::consts::OS }),
                };

                let mut policy_input = serde_json::to_value(&normalized).unwrap_or(json!({}));
                // Mock legacy fields
                policy_input["action"] = json!(normalized
                    .tool_name
                    .unwrap_or(normalized.request_type.clone()));
                policy_input["principal"] = json!(agent_id.clone());
                policy_input["resource"] = json!(server_id.clone());

                let decision = router
                    .read()
                    .await
                    .authorize(policy_input)
                    .await
                    .unwrap_or_else(|_| dek_policy_runtime::PolicyDecision {
                        evaluator_id: "wrapper".into(),
                        evaluator_type: "wrapper".into(),
                        required: true,
                        status: "error".into(),
                        decision: "deny".into(),
                        allow: false,
                        reason: "Policy evaluation failed".into(),
                        effects: json!({}),
                        obligations: vec![],
                        metadata: json!({}),
                    });

                if !decision.allow {
                    warn!("[wrapper] Denied: {}", decision.reason);

                    let err_res = json!({
                        "jsonrpc": "2.0",
                        "id": payload.get("id").unwrap_or(&json!(null)),
                        "error": {
                            "code": -32001,
                            "message": "Pollen policy denied MCP request",
                            "data": {
                                "reason": decision.reason
                            }
                        }
                    });

                    let _ = tx_out.send(err_res.to_string()).await;
                    continue; // Skip writing to child
                }
            }

            // Allowed or unparseable JSON (let child handle errors)
            let mut l = line;
            l.push('\n');
            if child_stdin.write_all(l.as_bytes()).await.is_err() {
                break;
            }
        }
    });

    // Task 4: Write all output to parent stdout
    while let Some(mut output) = rx_out.recv().await {
        output.push('\n');
        if parent_stdout.write_all(output.as_bytes()).await.is_err() {
            break;
        }
    }

    let status = child.wait().await?;
    info!("dek-stdio-wrapper exiting with status: {}", status);

    Ok(())
}
