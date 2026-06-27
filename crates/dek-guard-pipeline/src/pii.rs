// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use crate::{ensure_rustls_crypto_provider, NerProvider};
use dek_plugin_sdk::{PluginResult, RedactionFinding};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::cmp::Reverse;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PiiSpan {
    pub entity_type: String,
    pub start: usize,
    pub end: usize,
    pub value: String,
    pub confidence: f32,
    pub source: &'static str,
}

#[derive(Clone, Copy)]
struct Recognizer {
    entity: &'static str,
    re: &'static Lazy<Result<Regex, regex::Error>>,
    base_confidence: f32,
    validate: fn(&str) -> bool,
    context: &'static [&'static str],
    require_context: bool,
}

static RE_EMAIL: Lazy<Result<Regex, regex::Error>> =
    Lazy::new(|| Regex::new(r"(?i)\b[A-Z0-9._%+\-]+@[A-Z0-9.\-]+\.[A-Z]{2,}\b"));
static RE_PHONE_TH: Lazy<Result<Regex, regex::Error>> =
    Lazy::new(|| Regex::new(r"(\+66|0)\s?[689]\s?\d{3}\s?\d{4}"));
static RE_CREDIT_CARD: Lazy<Result<Regex, regex::Error>> =
    Lazy::new(|| Regex::new(r"\b(?:\d[ \-]?){13,19}\b"));
static RE_THAI_ID: Lazy<Result<Regex, regex::Error>> =
    Lazy::new(|| Regex::new(r"\b\d-?\d{4}-?\d{5}-?\d{2}-?\d\b"));
static RE_PASSPORT: Lazy<Result<Regex, regex::Error>> =
    Lazy::new(|| Regex::new(r"\b[A-Z]{1,2}\d{6,7}\b"));

const RECOGNIZERS: &[Recognizer] = &[
    Recognizer {
        entity: "EMAIL",
        re: &RE_EMAIL,
        base_confidence: 0.95,
        validate: always_valid,
        context: &["email", "mail", "อีเมล"],
        require_context: false,
    },
    Recognizer {
        entity: "PHONE",
        re: &RE_PHONE_TH,
        base_confidence: 0.85,
        validate: always_valid,
        context: &["phone", "tel", "โทร", "เบอร์"],
        require_context: false,
    },
    Recognizer {
        entity: "CREDIT_CARD",
        re: &RE_CREDIT_CARD,
        base_confidence: 0.70,
        validate: luhn_valid,
        context: &["card", "credit", "visa", "master", "บัตร"],
        require_context: false,
    },
    Recognizer {
        entity: "THAI_NATIONAL_ID",
        re: &RE_THAI_ID,
        base_confidence: 0.50,
        validate: thai_id_valid,
        context: &["national id", "บัตรประชาชน", "เลขประจำตัว"],
        require_context: false,
    },
    Recognizer {
        entity: "PASSPORT",
        re: &RE_PASSPORT,
        base_confidence: 0.30,
        validate: always_valid,
        context: &["passport", "หนังสือเดินทาง"],
        require_context: true,
    },
];

#[derive(Debug, Default, Clone)]
pub struct PiiDetector;

#[derive(Debug, Clone)]
pub struct GlinerProvider {
    pub endpoint: String,
    pub labels: Vec<String>,
    pub min_confidence: f32,
    pub timeout: Duration,
}

impl GlinerProvider {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            labels: vec![
                "person".to_string(),
                "address".to_string(),
                "organization".to_string(),
            ],
            min_confidence: 0.80,
            timeout: Duration::from_millis(80),
        }
    }

    pub fn with_labels(mut self, labels: Vec<String>) -> Self {
        self.labels = labels;
        self
    }

    pub fn with_min_confidence(mut self, min_confidence: f32) -> Self {
        self.min_confidence = clamp_unit_score(min_confidence);
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

#[derive(Debug, Serialize)]
struct GlinerRequest<'a> {
    text: &'a str,
    labels: &'a [String],
}

#[derive(Debug, Deserialize)]
struct GlinerEntity {
    label: String,
    start: usize,
    end: usize,
    score: f32,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum GlinerResponse {
    Items(Vec<GlinerEntity>),
    Wrapped { entities: Vec<GlinerEntity> },
}

impl GlinerResponse {
    fn into_entities(self) -> Vec<GlinerEntity> {
        match self {
            GlinerResponse::Items(items) => items,
            GlinerResponse::Wrapped { entities } => entities,
        }
    }
}

impl NerProvider for GlinerProvider {
    fn detect_entities(&self, text: &str) -> PluginResult<Vec<PiiSpan>> {
        ensure_rustls_crypto_provider();
        let client = match reqwest::blocking::Client::builder()
            .timeout(self.timeout)
            .build()
        {
            Ok(client) => client,
            Err(_) => return Ok(Vec::new()),
        };

        let response = match client
            .post(&self.endpoint)
            .json(&GlinerRequest {
                text,
                labels: &self.labels,
            })
            .send()
        {
            Ok(response) => response,
            Err(_) => return Ok(Vec::new()),
        };

        if !response.status().is_success() {
            return Ok(Vec::new());
        }

        let parsed = match response.json::<GlinerResponse>() {
            Ok(parsed) => parsed,
            Err(_) => return Ok(Vec::new()),
        };

        let mut spans = Vec::new();
        for entity in parsed.into_entities() {
            let confidence = clamp_unit_score(entity.score);
            if confidence < self.min_confidence {
                continue;
            }
            let Some(value) = span_value(text, entity.start, entity.end) else {
                continue;
            };
            spans.push(PiiSpan {
                entity_type: normalize_ner_label(&entity.label),
                start: entity.start,
                end: entity.end,
                value: value.to_string(),
                confidence,
                source: "ner",
            });
        }

        Ok(dedupe_overlaps(spans))
    }
}

impl PiiDetector {
    pub fn detect(&self, text: &str) -> Vec<PiiSpan> {
        let mut spans = Vec::new();
        for recognizer in RECOGNIZERS {
            let Ok(regex) = recognizer.re.as_ref() else {
                continue;
            };
            for matched in regex.find_iter(text) {
                if !(recognizer.validate)(matched.as_str()) {
                    continue;
                }
                let has_context =
                    context_contains(text, matched.start(), matched.end(), recognizer.context);
                if recognizer.require_context && !has_context {
                    continue;
                }
                let mut confidence = recognizer.base_confidence;
                if has_context {
                    confidence = (confidence + 0.30).min(0.99);
                }
                spans.push(PiiSpan {
                    entity_type: recognizer.entity.to_string(),
                    start: matched.start(),
                    end: matched.end(),
                    value: matched.as_str().to_string(),
                    confidence,
                    source: "regex",
                });
            }
        }
        dedupe_overlaps(spans)
    }

    pub fn anonymize(
        &self,
        text: &str,
        spans: Vec<PiiSpan>,
        min_confidence: f32,
    ) -> (String, Vec<RedactionFinding>) {
        self.anonymize_at_path(text, spans, min_confidence, "$")
    }

    pub fn redact_value(
        &self,
        value: &Value,
        min_confidence: f32,
    ) -> (Value, Vec<RedactionFinding>) {
        self.redact_value_with_ner(value, min_confidence, None)
    }

    pub fn redact_value_with_ner(
        &self,
        value: &Value,
        min_confidence: f32,
        ner: Option<&dyn NerProvider>,
    ) -> (Value, Vec<RedactionFinding>) {
        let mut findings = Vec::new();
        let redacted = self.redact_value_at_path(value, min_confidence, "$", ner, &mut findings);
        (redacted, findings)
    }

    fn redact_value_at_path(
        &self,
        value: &Value,
        min_confidence: f32,
        path: &str,
        ner: Option<&dyn NerProvider>,
        findings: &mut Vec<RedactionFinding>,
    ) -> Value {
        match value {
            Value::String(text) => {
                let mut spans = self.detect(text);
                if let Some(provider) = ner {
                    if let Ok(mut ner_spans) = provider.detect_entities(text) {
                        spans.append(&mut ner_spans);
                    }
                }
                let (redacted, mut local_findings) =
                    self.anonymize_at_path(text, spans, min_confidence, path);
                findings.append(&mut local_findings);
                Value::String(redacted)
            }
            Value::Array(items) => Value::Array(
                items
                    .iter()
                    .enumerate()
                    .map(|(index, item)| {
                        self.redact_value_at_path(
                            item,
                            min_confidence,
                            &format!("{path}[{index}]"),
                            ner,
                            findings,
                        )
                    })
                    .collect(),
            ),
            Value::Object(map) => {
                let mut redacted = Map::new();
                for (key, child) in map {
                    redacted.insert(
                        key.clone(),
                        self.redact_value_at_path(
                            child,
                            min_confidence,
                            &format!("{path}.{key}"),
                            ner,
                            findings,
                        ),
                    );
                }
                Value::Object(redacted)
            }
            _ => value.clone(),
        }
    }

    fn anonymize_at_path(
        &self,
        text: &str,
        mut spans: Vec<PiiSpan>,
        min_confidence: f32,
        path: &str,
    ) -> (String, Vec<RedactionFinding>) {
        spans = dedupe_overlaps(spans);
        spans.retain(|span| span.confidence >= min_confidence);
        spans.sort_by_key(|span| Reverse(span.start));

        let mut out = text.to_string();
        let mut findings = Vec::new();
        for span in &spans {
            let replacement = format!("[REDACTED_{}]", span.entity_type);
            out.replace_range(span.start..span.end, &replacement);
            findings.push(RedactionFinding {
                kind: span.entity_type.clone(),
                confidence: span.confidence,
                path: format!("{path}:offset:{}", span.start),
                replacement,
            });
        }
        (out, findings)
    }
}

pub fn luhn_valid(value: &str) -> bool {
    let digits: Vec<u32> = value.chars().filter_map(|ch| ch.to_digit(10)).collect();
    if digits.len() < 13 || digits.len() > 19 {
        return false;
    }

    let mut sum = 0u32;
    let mut double_digit = false;
    for digit in digits.iter().rev() {
        let mut value = *digit;
        if double_digit {
            value *= 2;
            if value > 9 {
                value -= 9;
            }
        }
        sum += value;
        double_digit = !double_digit;
    }
    sum % 10 == 0
}

pub fn thai_id_valid(value: &str) -> bool {
    let digits: Vec<u32> = value.chars().filter_map(|ch| ch.to_digit(10)).collect();
    if digits.len() != 13 {
        return false;
    }

    let sum: u32 = digits
        .iter()
        .take(12)
        .enumerate()
        .map(|(index, digit)| digit * (13 - index as u32))
        .sum();
    let check = (11 - (sum % 11)) % 10;
    digits.get(12).is_some_and(|digit| *digit == check)
}

fn clamp_unit_score(score: f32) -> f32 {
    if !score.is_finite() || score <= 0.0 {
        0.0
    } else if score >= 1.0 {
        1.0
    } else {
        score
    }
}

fn normalize_ner_label(label: &str) -> String {
    match label.trim().to_ascii_lowercase().as_str() {
        "person" | "name" => "PERSON".to_string(),
        "address" | "location" => "ADDRESS".to_string(),
        "organization" | "org" => "ORGANIZATION".to_string(),
        other => other.to_ascii_uppercase(),
    }
}

fn span_value(text: &str, start: usize, end: usize) -> Option<&str> {
    if start >= end || end > text.len() {
        return None;
    }
    if !text.is_char_boundary(start) || !text.is_char_boundary(end) {
        return None;
    }
    text.get(start..end)
}

fn always_valid(_value: &str) -> bool {
    true
}

fn context_contains(text: &str, start: usize, end: usize, keywords: &[&str]) -> bool {
    if keywords.is_empty() {
        return false;
    }
    let window_start = previous_char_boundary(text, start.saturating_sub(40));
    let window_end = next_char_boundary(text, end.saturating_add(40).min(text.len()));
    let context = text[window_start..window_end].to_lowercase();
    keywords.iter().any(|keyword| context.contains(keyword))
}

fn previous_char_boundary(text: &str, index: usize) -> usize {
    let mut cursor = index.min(text.len());
    while cursor > 0 && !text.is_char_boundary(cursor) {
        cursor -= 1;
    }
    cursor
}

fn next_char_boundary(text: &str, index: usize) -> usize {
    let mut cursor = index.min(text.len());
    while cursor < text.len() && !text.is_char_boundary(cursor) {
        cursor += 1;
    }
    cursor
}

fn dedupe_overlaps(mut spans: Vec<PiiSpan>) -> Vec<PiiSpan> {
    spans.sort_by(|left, right| right.confidence.total_cmp(&left.confidence));
    let mut selected: Vec<PiiSpan> = Vec::new();
    for span in spans {
        if !selected
            .iter()
            .any(|existing| ranges_overlap(existing, &span))
        {
            selected.push(span);
        }
    }
    selected.sort_by_key(|span| span.start);
    selected
}

fn ranges_overlap(left: &PiiSpan, right: &PiiSpan) -> bool {
    left.start < right.end && right.start < left.end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thai_id_checksum_rejects_invalid_number() {
        let detector = PiiDetector;
        let spans = detector.detect("บัตรประชาชน 1101700207031");

        assert!(!thai_id_valid("1101700207031"));
        assert!(!spans
            .iter()
            .any(|span| span.entity_type == "THAI_NATIONAL_ID"));
    }

    #[test]
    fn thai_id_context_boost_reaches_redaction_threshold() {
        let detector = PiiDetector;
        let spans = detector.detect("บัตรประชาชน 1101700207030");

        assert!(thai_id_valid("1101700207030"));
        assert!(spans
            .iter()
            .any(|span| { span.entity_type == "THAI_NATIONAL_ID" && span.confidence >= 0.80 }));
    }

    #[test]
    fn credit_card_luhn_rejects_invalid_number() {
        let detector = PiiDetector;
        let spans = detector.detect("credit card 4111 1111 1111 1112");

        assert!(!luhn_valid("4111 1111 1111 1112"));
        assert!(!spans.iter().any(|span| span.entity_type == "CREDIT_CARD"));
    }

    #[test]
    fn credit_card_luhn_accepts_valid_number() {
        let detector = PiiDetector;
        let spans = detector.detect("credit card 4111 1111 1111 1111");

        assert!(luhn_valid("4111 1111 1111 1111"));
        assert!(spans
            .iter()
            .any(|span| { span.entity_type == "CREDIT_CARD" && span.confidence >= 0.80 }));
    }

    #[test]
    fn passport_requires_context_keyword() {
        let detector = PiiDetector;
        let without_context = detector.detect("reference AB1234567");
        let with_context = detector.detect("passport AB1234567");

        assert!(!without_context
            .iter()
            .any(|span| span.entity_type == "PASSPORT"));
        assert!(with_context
            .iter()
            .any(|span| span.entity_type == "PASSPORT"));
    }

    #[test]
    fn gliner_endpoint_failure_returns_empty_spans() -> PluginResult<()> {
        let provider =
            GlinerProvider::new("http://127.0.0.1:9/ner").with_timeout(Duration::from_millis(5));
        let spans = provider.detect_entities("สมชายอยู่กรุงเทพ")?;

        assert!(spans.is_empty());
        Ok(())
    }

    #[test]
    fn redact_value_replaces_nested_string_without_raw_value_in_finding() {
        let detector = PiiDetector;
        let (redacted, findings) = detector.redact_value(
            &serde_json::json!({
                "profile": {
                    "national_id": "บัตรประชาชน 1101700207030"
                }
            }),
            0.80,
        );
        let rendered = redacted.to_string();

        assert!(rendered.contains("[REDACTED_THAI_NATIONAL_ID]"));
        assert!(!rendered.contains("1101700207030"));
        assert!(findings.iter().any(|finding| {
            finding.kind == "THAI_NATIONAL_ID"
                && finding.path.starts_with("$.profile.national_id:offset:")
                && finding.replacement == "[REDACTED_THAI_NATIONAL_ID]"
        }));
    }
}
