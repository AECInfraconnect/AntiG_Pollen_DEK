use crate::state::AppState;
use dek_domain_schema::{ControlMode, AgentStatus};
use serde_json::json;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

pub async fn start_anomaly_detector(state: Arc<AppState>) {
    info!("Starting P2 Anomaly Detector...");
    loop {
        sleep(Duration::from_secs(30)).await;

        let tenant_id = "local"; // local tenant

        // 1. Fetch recent telemetry (e.g. decision logs and resource accesses)
        let evs = match state.telemetry_store.list_telemetry(tenant_id, "decision").await {
            Ok(v) => v,
            Err(_) => continue,
        };

        // 2. Compute failure rates per agent
        let mut fail_counts: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
        
        for ev in evs {
            if let (Some(agent_id), Some(allow)) = (
                ev.get("subject").and_then(|s| s.get("id")).and_then(|id| id.as_str()),
                ev.get("allow").and_then(|a| a.as_bool()),
            ) {
                if !allow {
                    *fail_counts.entry(agent_id.to_string()).or_insert(0) += 1;
                }
            }
        }

        // 3. Update trust score and trigger playbooks
        if let Ok(mut agents) = state.registry_store.list_agents(tenant_id).await {
            for mut agent in agents {
                let fails = fail_counts.get(&agent.id).copied().unwrap_or(0);
                
                if fails > 5 {
                    let old_score = agent.trust_score;
                    agent.trust_score = (agent.trust_score - fails * 5).max(0);
                    
                    if old_score != agent.trust_score {
                        warn!(
                            "AnomalyDetector: Agent {} trust score degraded {} -> {} due to {} policy violations.",
                            agent.id, old_score, agent.trust_score, fails
                        );

                        // Auto-remediation playbook
                        if agent.trust_score < 50 && agent.status == AgentStatus::Controlled {
                            warn!("AnomalyDetector: Triggering Auto-Kill Switch playbook for agent {}! Trust score critically low.", agent.id);
                            
                            // Emulate playbook mutating the agent's policy deployment to StrictDeny
                            if let Ok(mut deployments) = state.policy_store.list_deployments(tenant_id).await {
                                for mut dep in deployments {
                                    if dep.control_bindings.iter().any(|b| b.agent_id == agent.id) {
                                        dep.control_mode = ControlMode::StrictDeny;
                                        let _ = state.policy_store.put_deployment(&dep).await;
                                        info!("AnomalyDetector: Mutated deployment {} to StrictDeny.", dep.deployment_id);
                                    }
                                }
                            }

                            // Emit security event telemetry
                            let sec_event = json!({
                                "type": "security_event",
                                "schema_version": "pollen.telemetry.v2",
                                "tenant_id": tenant_id,
                                "event_id": uuid::Uuid::new_v4().to_string(),
                                "timestamp": chrono::Utc::now().to_rfc3339(),
                                "severity": "critical",
                                "name": "auto_kill_switch_triggered",
                                "agent_id": agent.id,
                                "trust_score": agent.trust_score,
                                "trigger": format!("Failed {} policies in short time window", fails)
                            });
                            let _ = state.telemetry_store.put_telemetry("security_event", &sec_event.to_string()).await;
                        }
                        
                        let _ = state.registry_store.put_agent(tenant_id, &agent).await;
                    }
                }
            }
        }
    }
}
