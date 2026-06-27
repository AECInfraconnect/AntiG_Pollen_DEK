// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use crate::{GuardAction, GuardOutcome};
use dek_plugin_sdk::RedactionFinding;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GuardFindingSummary {
    pub kind: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GuardRemediation {
    pub user_message: String,
    pub recommended_actions: Vec<String>,
    pub doc_url: Option<String>,
    pub can_override: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GuardEvent {
    pub event_id: String,
    pub ts: String,
    pub tenant_id: Option<String>,
    pub agent_id: Option<String>,
    pub direction: String,
    pub action: GuardAction,
    pub categories: Vec<String>,
    pub injection_score: f32,
    pub findings_summary: Vec<GuardFindingSummary>,
    pub severity: String,
    pub remediation: GuardRemediation,
    pub redaction_applied: bool,
}

impl GuardEvent {
    pub fn from_outcome(
        event_id: impl Into<String>,
        ts: impl Into<String>,
        tenant_id: Option<String>,
        agent_id: Option<String>,
        direction: impl Into<String>,
        outcome: &GuardOutcome,
        redaction_applied: bool,
    ) -> Self {
        let direction = direction.into();
        let categories = outcome.categories.clone();
        Self {
            event_id: event_id.into(),
            ts: ts.into(),
            tenant_id,
            agent_id,
            direction,
            action: outcome.action,
            categories: categories.clone(),
            injection_score: outcome.injection_score,
            findings_summary: summarize_findings(&outcome.findings),
            severity: severity_for(outcome.action, &categories),
            remediation: remediation_for(outcome.action, &categories),
            redaction_applied,
        }
    }
}

pub fn summarize_findings(findings: &[RedactionFinding]) -> Vec<GuardFindingSummary> {
    let mut counts: BTreeMap<String, u32> = BTreeMap::new();
    for finding in findings {
        let entry = counts.entry(finding.kind.clone()).or_insert(0);
        *entry = entry.saturating_add(1);
    }
    counts
        .into_iter()
        .map(|(kind, count)| GuardFindingSummary { kind, count })
        .collect()
}

pub fn severity_for(action: GuardAction, categories: &[String]) -> String {
    if action == GuardAction::Deny
        && categories.iter().any(|category| {
            category == "llm01_prompt_injection" || category == "llm07_system_prompt_leakage"
        })
    {
        "critical".to_string()
    } else if action == GuardAction::Deny || action == GuardAction::Redact {
        "warn".to_string()
    } else {
        "info".to_string()
    }
}

pub fn remediation_for(action: GuardAction, categories: &[String]) -> GuardRemediation {
    if categories
        .iter()
        .any(|category| category == "llm01_prompt_injection")
    {
        return GuardRemediation {
            user_message: "คำขอถูกบล็อกเพราะตรวจพบรูปแบบ prompt injection หรือคำสั่งแฝง".to_string(),
            recommended_actions: vec![
                "ตรวจสอบข้อความหรือเอกสารต้นทางที่ agent กำลังใช้".to_string(),
                "ลบคำสั่งแฝงที่พยายามเปลี่ยน system/developer instruction".to_string(),
                "ขออนุมัติผ่าน human-in-the-loop ถ้าเป็นงานที่ตั้งใจทำจริง".to_string(),
            ],
            doc_url: Some("https://pollek.ai/docs/guard/prompt-injection".to_string()),
            can_override: true,
        };
    }

    if categories
        .iter()
        .any(|category| category == "llm07_system_prompt_leakage")
    {
        return GuardRemediation {
            user_message: "ผลลัพธ์ถูกบล็อกเพราะมีสัญญาณว่า system prompt หรือ canary token อาจรั่วไหล"
                .to_string(),
            recommended_actions: vec![
                "อย่าส่งผลลัพธ์นี้กลับไปให้ agent หรือผู้ใช้ปลายทาง".to_string(),
                "หมุนเวียน canary/secret ที่เกี่ยวข้องถ้าพบการรั่วไหลจริง".to_string(),
                "ตรวจสอบ tool output และ conversation ก่อนหน้า".to_string(),
            ],
            doc_url: Some("https://pollek.ai/docs/guard/system-prompt-leak".to_string()),
            can_override: false,
        };
    }

    if categories
        .iter()
        .any(|category| category == "llm02_sensitive_information_disclosure")
    {
        return GuardRemediation {
            user_message: "ข้อมูลอ่อนไหวถูกปกปิดก่อนส่งต่อ".to_string(),
            recommended_actions: vec![
                "ตรวจสอบว่าข้อมูลที่ถูกปกปิดจำเป็นต่อคำขอหรือไม่".to_string(),
                "ใช้ data minimization หรือเลือก resource ที่แคบลง".to_string(),
                "เปิด Third Party NER ใน Enterprise Cloud ถ้าต้องการจับชื่อ/ที่อยู่หลายภาษาให้ละเอียดขึ้น"
                    .to_string(),
            ],
            doc_url: Some("https://pollek.ai/docs/guard/pii-redaction".to_string()),
            can_override: false,
        };
    }

    if categories
        .iter()
        .any(|category| category == "llm05_improper_output_handling")
    {
        return GuardRemediation {
            user_message: "ผลลัพธ์ถูกปกปิดเพราะพบ secret หรือ markup ที่เสี่ยงก่อนส่งกลับ agent".to_string(),
            recommended_actions: vec![
                "ตรวจสอบ tool หรือ API ที่ส่งค่า secret กลับมา".to_string(),
                "ปิดการ echo token/key ใน log และ response".to_string(),
                "ส่งออกเฉพาะข้อความที่ผ่าน filter response แล้ว".to_string(),
            ],
            doc_url: Some("https://pollek.ai/docs/guard/output-handling".to_string()),
            can_override: false,
        };
    }

    if action == GuardAction::Redact {
        GuardRemediation {
            user_message: "ระบบปกปิดข้อมูลบางส่วนก่อนส่งต่อ".to_string(),
            recommended_actions: vec!["ตรวจสอบรายละเอียดใน Guard Incident Feed".to_string()],
            doc_url: Some("https://pollek.ai/docs/guard".to_string()),
            can_override: false,
        }
    } else {
        GuardRemediation {
            user_message: "ไม่พบเหตุการณ์ที่ต้องบล็อกหรือปกปิด".to_string(),
            recommended_actions: Vec::new(),
            doc_url: Some("https://pollek.ai/docs/guard".to_string()),
            can_override: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde_json::json;

    const GUARD_EVENT_CORPUS: &str = include_str!("../tests/corpus/guard_event.jsonl");

    #[derive(Debug, Deserialize)]
    struct GuardEventCorpusCase {
        id: String,
        category: String,
        finding_kind: String,
        raw_path: String,
        raw_value: String,
        status: String,
    }

    fn finding(kind: &str, path: &str) -> RedactionFinding {
        RedactionFinding {
            kind: kind.to_string(),
            confidence: 0.95,
            path: path.to_string(),
            replacement: format!("[REDACTED_{kind}]"),
        }
    }

    #[test]
    fn guard_event_summarizes_findings_without_raw_pii() {
        let outcome = GuardOutcome {
            action: GuardAction::Redact,
            injection_score: 0.0,
            categories: vec!["llm02_sensitive_information_disclosure".to_string()],
            findings: vec![
                finding("EMAIL", "$.content.alice@example.com"),
                finding("EMAIL", "$.body.bob@example.com"),
                finding("THAI_NATIONAL_ID", "$.profile.1101700207030"),
            ],
            redacted_payload: Some(json!({"content": "[REDACTED_EMAIL]"})),
            normalization_steps: Vec::new(),
            confidence: 0.95,
        };

        let event = GuardEvent::from_outcome(
            "ge_test",
            "2026-06-27T00:00:00Z",
            Some("tenant-a".to_string()),
            Some("agent-a".to_string()),
            "response",
            &outcome,
            true,
        );
        let rendered = serde_json::to_string(&event).unwrap_or_default();

        assert_eq!(event.findings_summary.len(), 2);
        assert!(event
            .findings_summary
            .iter()
            .any(|finding| finding.kind == "EMAIL" && finding.count == 2));
        assert!(event.redaction_applied);
        assert!(event.remediation.user_message.contains("ข้อมูลอ่อนไหว"));
        assert!(!rendered.contains("alice@example.com"));
        assert!(!rendered.contains("1101700207030"));
        assert!(!rendered.contains("$.profile"));
    }

    #[test]
    fn prompt_injection_remediation_is_actionable_in_thai() {
        let categories = vec!["llm01_prompt_injection".to_string()];
        let remediation = remediation_for(GuardAction::Deny, &categories);

        assert!(remediation.user_message.contains("คำขอถูกบล็อก"));
        assert!(remediation.can_override);
        assert!(remediation
            .recommended_actions
            .iter()
            .any(|action| action.contains("human-in-the-loop")));
        assert_eq!(severity_for(GuardAction::Deny, &categories), "critical");
    }

    #[test]
    fn guard_event_golden_corpus_is_summary_only() -> Result<(), serde_json::Error> {
        for line in GUARD_EVENT_CORPUS
            .lines()
            .filter(|line| !line.trim().is_empty())
        {
            let case: GuardEventCorpusCase = serde_json::from_str(line)?;
            if case.status != "active" {
                continue;
            }
            let outcome = GuardOutcome {
                action: GuardAction::Redact,
                injection_score: 0.0,
                categories: vec![case.category.clone()],
                findings: vec![finding(&case.finding_kind, &case.raw_path)],
                redacted_payload: Some(json!({"content": "[REDACTED]"})),
                normalization_steps: Vec::new(),
                confidence: 0.95,
            };

            let event = GuardEvent::from_outcome(
                "ge_test",
                "2026-06-27T00:00:00Z",
                Some("tenant-a".to_string()),
                Some("agent-a".to_string()),
                "response",
                &outcome,
                true,
            );
            let rendered = serde_json::to_string(&event)?;

            assert!(case.id.starts_with("rt-pr8-"));
            assert!(event
                .findings_summary
                .iter()
                .any(|finding| finding.kind == case.finding_kind && finding.count == 1));
            assert!(!rendered.contains(&case.raw_path));
            assert!(!rendered.contains(&case.raw_value));
        }
        Ok(())
    }
}
