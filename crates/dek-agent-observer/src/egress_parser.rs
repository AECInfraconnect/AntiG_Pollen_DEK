// SPDX-License-Identifier: Apache-2.0

use crate::model::{AgentObservationEvent, EventKind, TokenUsage};

fn provider_from_host(host: &str) -> String {
    if host.contains("api.openai.com") {
        "openai".into()
    } else if host.contains("api.anthropic.com") {
        "anthropic".into()
    } else if host.contains("11434") {
        "ollama".into()
    } else {
        "local".into()
    }
}

/// detect provider + parse usage จาก response body (รองรับ OpenAI/Anthropic/Ollama schema)
pub fn parse_llm_usage(host: &str, body: &serde_json::Value) -> Option<(String, TokenUsage)> {
    let provider = provider_from_host(host);
    let model = body.get("model").and_then(|m| m.as_str()).map(String::from);

    // If streaming SSE chunk, usage might be inside chunk directly (Ollama)
    // or inside Anthropic's message_delta
    let usage = body
        .get("usage")
        .or_else(|| body.get("message_delta").and_then(|m| m.get("usage")))?;

    // OpenAI/Ollama/vLLM/NIM: prompt_tokens/completion_tokens
    let (input, output) = if let Some(p) = usage.get("prompt_tokens") {
        (
            p.as_i64(),
            usage.get("completion_tokens").and_then(|v| v.as_i64()),
        )
    // Anthropic: input_tokens/output_tokens
    } else if let Some(i) = usage.get("input_tokens") {
        (
            i.as_i64(),
            usage.get("output_tokens").and_then(|v| v.as_i64()),
        )
    } else {
        (None, None)
    };

    Some((
        provider,
        TokenUsage {
            input_tokens: input,
            output_tokens: output,
            total_tokens: usage
                .get("total_tokens")
                .and_then(|v| v.as_i64())
                .or_else(|| Some(input.unwrap_or(0) + output.unwrap_or(0))),
            model,
        },
    ))
}

/// สร้าง observation event จากการเรียก LLM หนึ่งครั้ง (เส้นทาง egress)
pub fn llm_call_event(
    tenant: &str,
    trace_id: &str,
    agent_id: Option<String>,
    host: &str,
    body: &serde_json::Value,
    latency_ms: i64,
) -> Option<AgentObservationEvent> {
    let (provider, usage) = parse_llm_usage(host, body)?;
    Some(AgentObservationEvent {
        event_id: uuid::Uuid::new_v4().to_string(),
        tenant_id: tenant.into(),
        trace_id: trace_id.into(),
        agent_id,
        shadow_candidate_id: None,
        tool_id: None,
        resource_id: None,
        surface: "llm_egress".into(),
        action: "chat.completion".into(),
        pep_type: Some("network_egress".into()),
        risk_level: None,
        timestamp: chrono::Utc::now().to_rfc3339(),
        payload_json: "{}".into(),
        token_usage: Some(usage),
        event_kind: EventKind::LlmCall,
        decision: None,
        tool_call: None,
        resource_access: None,
        latency_ms: Some(latency_ms),
        provider: Some(provider),
    })
}
