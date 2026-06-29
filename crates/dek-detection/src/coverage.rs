//! Coverage compiler: turn the loaded rules into the framework-coverage
//! registry that powers the dashboard Coverage page (§4.4) and the audit
//! artifact (`coverage.yaml`).

use crate::spec::RuleSpec;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Coverage {
    /// framework -> control id -> rule ids covering it.
    pub frameworks: BTreeMap<String, BTreeMap<String, Vec<String>>>,
    pub rule_count: usize,
}

pub fn build_coverage(rules: &[RuleSpec]) -> Coverage {
    let mut frameworks: BTreeMap<String, BTreeMap<String, Vec<String>>> = BTreeMap::new();

    let mut add = |fw: &str, controls: &[String], rule_id: &str| {
        let entry = frameworks.entry(fw.to_string()).or_default();
        for c in controls {
            entry
                .entry(c.clone())
                .or_default()
                .push(rule_id.to_string());
        }
    };

    for r in rules {
        add("owasp_llm", &r.maps.owasp_llm, &r.id);
        add("owasp_agentic", &r.maps.owasp_agentic, &r.id);
        add("atlas", &r.maps.atlas, &r.id);
        add("attack", &r.maps.attack, &r.id);
        add("nist_rmf", &r.maps.nist_rmf, &r.id);
    }

    Coverage {
        frameworks,
        rule_count: rules.len(),
    }
}

/// Render coverage as YAML (for committing `coverage.yaml` in CI).
pub fn coverage_to_yaml(cov: &Coverage) -> Result<String, String> {
    serde_yaml::to_string(cov).map_err(|e| e.to_string())
}
