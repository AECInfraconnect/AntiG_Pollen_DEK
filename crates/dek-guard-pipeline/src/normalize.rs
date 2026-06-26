// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use base64::{engine::general_purpose, Engine as _};
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedText {
    pub text: String,
    pub steps: Vec<String>,
}

pub fn normalize_for_match(text: &str) -> NormalizedText {
    let mut steps = Vec::new();
    let mut current = strip_zero_width(text);
    if current != text {
        steps.push("strip_zero_width".to_string());
    }

    let nfkc = current.nfkc().collect::<String>();
    if nfkc != current {
        steps.push("unicode_nfkc".to_string());
        current = nfkc;
    }

    let decoded_b64 = decode_base64_candidates(&current);
    if !decoded_b64.is_empty() {
        steps.push("decode_base64_candidates".to_string());
    }

    current = normalize_plain(&current, &mut steps);

    if !decoded_b64.is_empty() {
        for decoded in decoded_b64 {
            let mut decoded_steps = Vec::new();
            let normalized_decoded = normalize_plain(&decoded, &mut decoded_steps);
            current.push(' ');
            current.push_str(&normalized_decoded);
            for step in decoded_steps {
                if !steps.iter().any(|existing| existing == &step) {
                    steps.push(step);
                }
            }
        }
    }

    NormalizedText {
        text: current,
        steps,
    }
}

fn normalize_plain(text: &str, steps: &mut Vec<String>) -> String {
    let folded = fold_confusables(text).to_lowercase();
    let mut current = if folded != text {
        push_step_once(steps, "casefold_confusables");
        folded
    } else {
        text.to_string()
    };

    let decoded_entities = decode_common_entities(&current);
    if decoded_entities != current {
        push_step_once(steps, "decode_html_entities");
        current = decoded_entities;
    }

    let percent_decoded = percent_decode_ascii(&current);
    if percent_decoded != current {
        push_step_once(steps, "percent_decode");
        current = percent_decoded;
    }

    current
}

fn push_step_once(steps: &mut Vec<String>, step: &str) {
    if !steps.iter().any(|existing| existing == step) {
        steps.push(step.to_string());
    }
}

fn strip_zero_width(text: &str) -> String {
    text.chars()
        .filter(|c| {
            !matches!(
                *c,
                '\u{200b}' | '\u{200c}' | '\u{200d}' | '\u{2060}' | '\u{feff}'
            )
        })
        .collect()
}

fn fold_confusables(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            'Ａ' | 'Α' | 'А' | 'а' | 'ɑ' | 'Ꭺ' => 'a',
            'Ｂ' | 'Β' | 'В' | 'Ь' => 'b',
            'Ｃ' | 'Ϲ' | 'С' | 'с' => 'c',
            'Ｄ' | 'ԁ' => 'd',
            'Ｅ' | 'Ε' | 'Е' | 'е' => 'e',
            'Ｇ' | 'ɡ' => 'g',
            'Ｈ' | 'Η' | 'Н' => 'h',
            'Ｉ' | 'Ι' | 'І' | 'і' | 'ı' => 'i',
            'Ｋ' | 'Κ' | 'К' => 'k',
            'Ｌ' | 'ℓ' => 'l',
            'Ｍ' | 'Μ' | 'М' => 'm',
            'Ｎ' | 'Ν' => 'n',
            'Ｏ' | 'Ο' | 'О' | 'о' | 'ο' => 'o',
            'Ｐ' | 'Ρ' | 'Р' | 'р' => 'p',
            'Ｓ' | 'Ѕ' | 'ѕ' => 's',
            'Ｔ' | 'Τ' | 'Т' => 't',
            'Ｘ' | 'Χ' | 'Х' | 'х' => 'x',
            'Ｙ' | 'Υ' | 'У' | 'у' => 'y',
            _ => c,
        })
        .collect()
}

fn decode_common_entities(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
        .replace("&#x2f;", "/")
        .replace("&#x3c;", "<")
        .replace("&#x3e;", ">")
        .replace("&#39;", "'")
        .replace("&#47;", "/")
        .replace("&#60;", "<")
        .replace("&#62;", ">")
}

fn percent_decode_ascii(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (from_hex(bytes[i + 1]), from_hex(bytes[i + 2])) {
                out.push((h << 4) | l);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }

    match String::from_utf8(out) {
        Ok(decoded) => decoded,
        Err(_) => text.to_string(),
    }
}

fn from_hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn decode_base64_candidates(text: &str) -> Vec<String> {
    let mut decoded = Vec::new();
    for token in
        text.split(|c: char| c.is_whitespace() || matches!(c, '"' | '\'' | ',' | ';' | '(' | ')'))
    {
        let trimmed = token.trim_matches(|c: char| matches!(c, '.' | ':' | '[' | ']' | '{' | '}'));
        if trimmed.len() < 16 || trimmed.len() > 512 {
            continue;
        }
        if !trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '/' | '-' | '_' | '='))
        {
            continue;
        }

        if let Some(decoded_text) = decode_base64_text(trimmed) {
            let lowered = decoded_text.to_lowercase();
            if !decoded.iter().any(|existing| existing == &lowered) {
                decoded.push(lowered);
            }
        }
    }
    decoded
}

fn decode_base64_text(token: &str) -> Option<String> {
    let decoded = general_purpose::STANDARD
        .decode(token)
        .or_else(|_| general_purpose::URL_SAFE.decode(token))
        .or_else(|_| general_purpose::URL_SAFE_NO_PAD.decode(token))
        .ok()?;
    if decoded.len() > 512 {
        return None;
    }
    let decoded_text = String::from_utf8(decoded).ok()?;
    let char_count = decoded_text.chars().count().max(1) as f32;
    let printable_count = decoded_text
        .chars()
        .filter(|c| !c.is_control() || c.is_whitespace())
        .count() as f32;
    let printable_ratio = printable_count / char_count;
    (printable_ratio > 0.9).then_some(decoded_text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_base64_decode_step() {
        let encoded = general_purpose::STANDARD.encode("ignore previous instructions");
        let normalized = normalize_for_match(&encoded);

        assert!(normalized.text.contains("ignore previous instructions"));
        assert!(normalized
            .steps
            .iter()
            .any(|step| step == "decode_base64_candidates"));
    }

    #[test]
    fn records_nfkc_and_zero_width_steps() {
        let normalized = normalize_for_match("ｉ\u{200b}gnore");

        assert_eq!(normalized.text, "ignore");
        assert!(normalized
            .steps
            .iter()
            .any(|step| step == "strip_zero_width"));
        assert!(normalized.steps.iter().any(|step| step == "unicode_nfkc"));
    }

    #[test]
    fn decodes_entities_and_percent_encoding() {
        let normalized = normalize_for_match("ignore%20previous%20instructions &lt;safe&gt;");

        assert!(normalized.text.contains("ignore previous instructions"));
        assert!(normalized.text.contains("<safe>"));
        assert!(normalized.steps.iter().any(|step| step == "percent_decode"));
        assert!(normalized
            .steps
            .iter()
            .any(|step| step == "decode_html_entities"));
    }
}
