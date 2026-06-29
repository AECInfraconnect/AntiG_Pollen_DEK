//! Rule evaluation against normalized observed events.
//!
//! `ObservedEvent` is the engine's view of a `AiResourceActivityEventV1`
//! (doc #2 §6). In production the correlator maps the canonical event into this
//! shape; here it is self-contained so the engine and its tests stand alone.

use crate::spec::{parse_window_ms, DetectType, RuleSpec, Step};

/// Untrusted provenance origins (doc #3 §6). `provenance_taint: any` matches
/// when the event's origin is one of these.
const UNTRUSTED_ORIGINS: &[&str] = &[
    "web",
    "email",
    "untrusted_file",
    "rag_retrieval",
    "tool_output",
    "unknown",
];

/// Normalized event the evaluator consumes.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ObservedEvent {
    pub event_id: String,
    pub ts_ms: i64,
    pub agent_id: String,
    pub session_id: String,
    pub activity: String,
    pub action: String,
    pub resource_classification: Option<String>,
    /// Provenance origin string (e.g. "web", "user_direct").
    pub provenance_taint: Option<String>,
    pub path: Option<String>,
    pub host: Option<String>,
    /// Reputation band of host/provider (e.g. "neutral", "blocked").
    pub host_reputation: Option<String>,
    /// Whether this event's target is on the learned allowlist.
    pub in_allowlist: bool,
}

/// Result of a rule firing.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Detection {
    pub rule_id: String,
    pub matched_event_ids: Vec<String>,
}

/// Minimal glob matcher supporting `*` (within a path segment-ish span) and
/// `**` (across separators). Sufficient for secret-path signatures such as
/// `**/.env`, `**/.ssh/**`, `**/id_*`. Matching is done on the raw string with
/// `/` as the separator.
pub fn glob_match(pattern: &str, text: &str) -> bool {
    glob_inner(pattern.as_bytes(), text.as_bytes())
}

fn glob_inner(pat: &[u8], txt: &[u8]) -> bool {
    // Iterative backtracking glob (handles * and **).
    let (mut p, mut t) = (0usize, 0usize);
    let (mut star_p, mut star_t): (Option<usize>, usize) = (None, 0);
    let mut double_star = false;

    while t < txt.len() {
        if p < pat.len() && pat[p] == b'*' {
            // Collapse consecutive stars; remember if any was a `**`.
            double_star = false;
            while p < pat.len() && pat[p] == b'*' {
                if p + 1 < pat.len() && pat[p + 1] == b'*' {
                    double_star = true;
                }
                p += 1;
            }
            star_p = Some(p);
            star_t = t;
            if p == pat.len() {
                // Trailing star.
                return double_star || !txt[t..].contains(&b'/');
            }
            continue;
        }
        if p < pat.len() && (pat[p] == txt[t]) {
            p += 1;
            t += 1;
            continue;
        }
        // Mismatch: backtrack to last star if allowed to consume this char.
        if let Some(sp) = star_p {
            // `*` must not cross `/`; `**` may.
            if txt[star_t] == b'/' && !double_star {
                return false;
            }
            star_t += 1;
            t = star_t;
            p = sp;
            continue;
        }
        return false;
    }
    // Consume any trailing stars in the pattern.
    while p < pat.len() && pat[p] == b'*' {
        p += 1;
    }
    p == pat.len()
}

/// Does a single step predicate match a single event?
pub fn step_matches(step: &Step, ev: &ObservedEvent) -> bool {
    if let Some(a) = &step.activity {
        if !a.eq_ignore_ascii_case(&ev.activity) {
            return false;
        }
    }
    if let Some(a) = &step.action {
        if !a.eq_ignore_ascii_case(&ev.action) {
            return false;
        }
    }
    if let Some(rc) = &step.resource_classification {
        match &ev.resource_classification {
            Some(evrc) if evrc.eq_ignore_ascii_case(rc) => {}
            _ => return false,
        }
    }
    if let Some(taint) = &step.provenance_taint {
        let origin = ev.provenance_taint.as_deref().unwrap_or("");
        let ok = if taint.eq_ignore_ascii_case("any") {
            UNTRUSTED_ORIGINS
                .iter()
                .any(|o| o.eq_ignore_ascii_case(origin))
        } else {
            taint.eq_ignore_ascii_case(origin)
        };
        if !ok {
            return false;
        }
    }
    if !step.path_matches.is_empty() {
        let path = ev.path.as_deref().unwrap_or("");
        if !step.path_matches.iter().any(|p| glob_match(p, path)) {
            return false;
        }
    }
    if !step.host_reputation.is_empty() {
        let rep = ev.host_reputation.as_deref().unwrap_or("");
        if !step
            .host_reputation
            .iter()
            .any(|r| r.eq_ignore_ascii_case(rep))
        {
            return false;
        }
    }
    if step.negate_allowlist && ev.in_allowlist {
        return false;
    }
    true
}

/// Evaluate a rule against a single agent session's events.
/// `events` should be sorted ascending by `ts_ms`.
pub fn evaluate(rule: &RuleSpec, events: &[ObservedEvent]) -> Option<Detection> {
    if !rule.enabled || rule.detect.steps.is_empty() {
        return None;
    }
    match rule.detect.detect_type {
        DetectType::Signature | DetectType::Heuristic => {
            let step = &rule.detect.steps[0];
            events
                .iter()
                .find(|ev| step_matches(step, ev))
                .map(|ev| Detection {
                    rule_id: rule.id.clone(),
                    matched_event_ids: vec![ev.event_id.clone()],
                })
        }
        DetectType::Sequence => evaluate_sequence(rule, events),
        DetectType::Anomaly => evaluate_anomaly(rule, events),
    }
}

fn evaluate_sequence(rule: &RuleSpec, events: &[ObservedEvent]) -> Option<Detection> {
    let window_ms = rule
        .detect
        .window
        .as_deref()
        .and_then(|w| parse_window_ms(w).ok())
        .unwrap_or(i64::MAX);
    let steps = &rule.detect.steps;

    // Greedy in-order match: advance through steps, each next match must be a
    // later event, and the whole chain must fit inside the window.
    for start in 0..events.len() {
        if !step_matches(&steps[0], &events[start]) {
            continue;
        }
        let first_ts = events[start].ts_ms;
        let mut matched = vec![events[start].event_id.clone()];
        let mut step_idx = 1usize;
        let mut ev_idx = start + 1;
        while step_idx < steps.len() && ev_idx < events.len() {
            let ev = &events[ev_idx];
            if ev.ts_ms - first_ts > window_ms {
                break;
            }
            if step_matches(&steps[step_idx], ev) {
                matched.push(ev.event_id.clone());
                step_idx += 1;
            }
            ev_idx += 1;
        }
        if step_idx == steps.len() {
            return Some(Detection {
                rule_id: rule.id.clone(),
                matched_event_ids: matched,
            });
        }
    }
    None
}

fn evaluate_anomaly(rule: &RuleSpec, events: &[ObservedEvent]) -> Option<Detection> {
    let window_ms = rule
        .detect
        .window
        .as_deref()
        .and_then(|w| parse_window_ms(w).ok())
        .unwrap_or(i64::MAX);
    let step = &rule.detect.steps[0];
    let min_count = step.min_count.unwrap_or(1).max(1) as usize;

    let hits: Vec<&ObservedEvent> = events.iter().filter(|ev| step_matches(step, ev)).collect();
    if hits.len() < min_count {
        return None;
    }
    // Sliding window: any `min_count` hits within `window_ms`.
    for i in 0..hits.len() {
        let j = i + min_count - 1;
        if j < hits.len() && hits[j].ts_ms - hits[i].ts_ms <= window_ms {
            return Some(Detection {
                rule_id: rule.id.clone(),
                matched_event_ids: hits[i..=j].iter().map(|e| e.event_id.clone()).collect(),
            });
        }
    }
    None
}
