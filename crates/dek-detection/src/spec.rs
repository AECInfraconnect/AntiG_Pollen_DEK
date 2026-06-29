//! Detection rule specification types.
//!
//! These deserialize the `contracts/detections/**/*.yaml` rule files into a
//! typed, validatable representation. The same `RuleSpec` is the input to the
//! evaluator (`crate::eval`) and the coverage compiler (`crate::coverage`).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Maturity {
    Experimental,
    Beta,
    Stable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectType {
    /// Single-event exact/pattern match (like an AV signature).
    Signature,
    /// Single-event rule with weak signals (scored, not exact).
    Heuristic,
    /// Ordered multi-event match within a time window (kill-chain).
    Sequence,
    /// Count/rate threshold over a window vs a baseline.
    Anomaly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseAction {
    Observe,
    Warn,
    Ask,
    Block,
    Redact,
    Isolate,
}

/// Framework control IDs a rule covers. Required for coverage proof (§2/§4).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Maps {
    #[serde(default)]
    pub owasp_llm: Vec<String>,
    #[serde(default)]
    pub owasp_agentic: Vec<String>,
    #[serde(default)]
    pub atlas: Vec<String>,
    #[serde(default)]
    pub attack: Vec<String>,
    #[serde(default)]
    pub nist_rmf: Vec<String>,
}

/// A single predicate against one observed event. All present fields must
/// match (logical AND). Absent fields are ignored.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Step {
    /// `ActivityType` from the canonical event model (e.g. "FileRead").
    pub activity: Option<String>,
    /// `ResourceAction` (e.g. "read", "upload").
    pub action: Option<String>,
    /// Resource classification tag (e.g. "sensitive").
    pub resource_classification: Option<String>,
    /// Provenance taint requirement: "any" (any untrusted origin) or a
    /// specific origin like "web" / "email" / "tool_output".
    pub provenance_taint: Option<String>,
    /// Glob patterns matched against the (redacted) resource path.
    #[serde(default)]
    pub path_matches: Vec<String>,
    /// Acceptable reputation bands for the destination host/provider.
    #[serde(default)]
    pub host_reputation: Vec<String>,
    /// If true, the event must NOT be on the learned allowlist.
    #[serde(default)]
    pub negate_allowlist: bool,
    /// For anomaly rules: minimum matching events within the window to fire.
    pub min_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Detect {
    #[serde(rename = "type")]
    pub detect_type: DetectType,
    /// Window like "120s" / "5m". Required for sequence/anomaly.
    pub window: Option<String>,
    /// Ordered steps. Signature/heuristic/anomaly use exactly one step.
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub default: ResponseAction,
    pub enforce_if_capable: Option<ResponseAction>,
    /// MANDATORY safety valve — must be true (validated in loader).
    pub observe_only_fallback: bool,
    pub user_message: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SuppressRule {
    pub agent_id_in: Option<String>,
    pub host_in: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Tuning {
    #[serde(default)]
    pub suppress_if: Vec<SuppressRule>,
    pub baseline_learning_days: Option<u32>,
}

/// A complete detection rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSpec {
    pub id: String,
    pub name: String,
    pub severity: Severity,
    pub confidence: Confidence,
    pub maturity: Maturity,
    pub maps: Maps,
    pub detect: Detect,
    pub response: Response,
    #[serde(default)]
    pub tuning: Option<Tuning>,
    /// Allows shipping a rule in observe-only without removing it.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

/// Parse window strings like "120s", "5m", "2h" into milliseconds.
pub fn parse_window_ms(s: &str) -> Result<i64, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("empty window".into());
    }
    let (num, unit) = s.split_at(s.len() - 1);
    let value: i64 = num
        .parse()
        .map_err(|_| format!("invalid window number in '{s}'"))?;
    let mult = match unit {
        "s" => 1_000,
        "m" => 60_000,
        "h" => 3_600_000,
        _ => return Err(format!("invalid window unit in '{s}' (use s/m/h)")),
    };
    Ok(value * mult)
}
