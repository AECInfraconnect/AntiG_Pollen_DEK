use std::time::Duration;
use tracing::{info, warn};
use dek_policy_suggester::rules::{RuleEngine, LowTrustRule};
use crate::state::AppState;

pub async fn start_governance_loop(state: AppState) -> anyhow::Result<()> {
    tokio::spawn(async move {
        let mut suggester = RuleEngine::new();
        suggester.add_rule(Box::new(LowTrustRule { threshold: 60 }));

        loop {
            // 1) OBSERVE: ดึง observation ล่าสุดจาก observer store (สมมติว่าเป็นทุก tenant หรือ default)
            // ในที่นี้จำลองใช้ tenant_id = "default"
            let tenant_id = "default";
            let events_result = state.observability_store.list_observation_events(tenant_id).await;
            
            match events_result {
                Ok(events) => {
                    // 2) SUGGEST: rule engine
                    match suggester.evaluate_all(&events) {
                        Ok(suggestions) => {
                            for s in suggestions {
                                info!("Governance Loop: Generated suggestion {}", s.suggestion_id);
                                // 3) สร้าง draft policy (รอ admin approve)
                                if let Err(e) = state.observability_store.upsert_policy_suggestion(&s).await {
                                    warn!("Governance Loop: Failed to upsert suggestion: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Governance Loop: Failed to evaluate suggestions: {}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Governance Loop: Failed to fetch recent events: {}", e);
                }
            }

            // วนลูปทำงานทุก ๆ 300 วินาที
            tokio::time::sleep(Duration::from_secs(300)).await;
        }
    });

    Ok(())
}
