use crate::model::PolicySuggestion;
use anyhow::Result;
use dek_agent_observer::model::AgentObservationEvent;

pub trait SuggestionRule: Send + Sync {
    fn evaluate(&self, events: &[AgentObservationEvent]) -> Result<Vec<PolicySuggestion>>;
}

pub struct RuleEngine {
    rules: Vec<Box<dyn SuggestionRule + Send + Sync>>,
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleEngine {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add_rule(&mut self, rule: Box<dyn SuggestionRule + Send + Sync>) {
        self.rules.push(rule);
    }

    pub fn evaluate_all(&self, events: &[AgentObservationEvent]) -> Result<Vec<PolicySuggestion>> {
        let mut all_suggestions = Vec::new();
        for rule in &self.rules {
            let suggestions = rule.evaluate(events)?;
            all_suggestions.extend(suggestions);
        }
        Ok(all_suggestions)
    }
}

pub struct ShadowAgentDetectionRule;

impl SuggestionRule for ShadowAgentDetectionRule {
    fn evaluate(&self, events: &[AgentObservationEvent]) -> Result<Vec<PolicySuggestion>> {
        let mut suggestions = Vec::new();
        // Detect shadow agents that appeared more than once
        let mut shadow_counts = std::collections::HashMap::new();
        for event in events {
            if let Some(shadow_id) = &event.shadow_candidate_id {
                *shadow_counts.entry(shadow_id.clone()).or_insert(0) += 1;
            }
        }

        for (shadow_id, count) in shadow_counts {
            if count >= 3 {
                let suggestion = PolicySuggestion {
                    suggestion_id: format!("shadow-detect-{}", shadow_id),
                    tenant_id: "default".into(),
                    target_agent_id: Some(shadow_id.clone()),
                    target_resource_id: None,
                    target_tool_id: None,
                    suggestion_type: crate::model::SuggestionType::RegisterShadowAgent,
                    title: format!("Unregistered Agent Detected: {}", shadow_id),
                    summary: format!("An unregistered agent (ID: {}) was detected performing {} actions. Consider registering it or blocking it.", shadow_id, count),
                    severity: crate::model::SuggestionSeverity::High,
                    confidence: 0.9,
                    recommended_policy_type: crate::model::SuggestedPolicyLanguage::Cedar,
                    recommended_pep_type: "mcp_proxy".into(),
                    artifacts: vec![],
                    status: crate::model::SuggestionStatus::Draft,
                    created_at: chrono::Utc::now().to_rfc3339(),
                };
                suggestions.push(suggestion);
            }
        }
        Ok(suggestions)
    }
}

pub struct HighRiskResourceRule;

impl SuggestionRule for HighRiskResourceRule {
    fn evaluate(&self, events: &[AgentObservationEvent]) -> Result<Vec<PolicySuggestion>> {
        let mut suggestions = Vec::new();
        for event in events {
            if let Some(risk) = &event.risk_level {
                if risk == "high" || risk == "critical" {
                    if let (Some(agent_id), Some(resource_id)) = (&event.agent_id, &event.resource_id) {
                        let suggestion = PolicySuggestion {
                            suggestion_id: format!("high-risk-{}-{}", agent_id, resource_id),
                            tenant_id: event.tenant_id.clone(),
                            target_agent_id: Some(agent_id.clone()),
                            target_resource_id: Some(resource_id.clone()),
                            target_tool_id: event.tool_id.clone(),
                            suggestion_type: crate::model::SuggestionType::RequireApprovalForSensitiveResource,
                            title: "High Risk Resource Access Detected".into(),
                            summary: format!("Agent '{}' accessed high-risk resource '{}'. Consider requiring explicit approval.", agent_id, resource_id),
                            severity: crate::model::SuggestionSeverity::Critical,
                            confidence: 0.95,
                            recommended_policy_type: crate::model::SuggestedPolicyLanguage::Cedar,
                            recommended_pep_type: "stdio_wrapper".into(),
                            artifacts: vec![],
                            status: crate::model::SuggestionStatus::Draft,
                            created_at: chrono::Utc::now().to_rfc3339(),
                        };
                        suggestions.push(suggestion);
                        break; // Only suggest once per evaluate call to avoid spam
                    }
                }
            }
        }
        Ok(suggestions)
    }
}
