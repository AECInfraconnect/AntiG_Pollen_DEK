use crate::model::AgentObservationEvent;
use opentelemetry::{
    global,
    trace::{SpanKind, Tracer},
    KeyValue,
};

pub fn emit_span(event: &AgentObservationEvent) {
    let tracer = global::tracer("dek-agent-observer");
    let mut attrs = vec![
        KeyValue::new("gen_ai.operation.name", event.action.clone()),
        KeyValue::new(
            "pollen.agent_id",
            event.agent_id.clone().unwrap_or_default(),
        ),
        KeyValue::new("pollen.tenant_id", event.tenant_id.clone()),
    ];
    if let Some(p) = &event.provider {
        attrs.push(KeyValue::new("gen_ai.system", p.clone()));
    }
    if let Some(u) = &event.token_usage {
        if let Some(m) = &u.model {
            attrs.push(KeyValue::new("gen_ai.request.model", m.clone()));
        }
        attrs.push(KeyValue::new(
            "gen_ai.usage.input_tokens",
            u.input_tokens.unwrap_or(0),
        ));
        attrs.push(KeyValue::new(
            "gen_ai.usage.output_tokens",
            u.output_tokens.unwrap_or(0),
        ));
    }
    if let Some(t) = &event.tool_call {
        attrs.push(KeyValue::new("gen_ai.tool.name", t.tool_name.clone()));
    }
    tracer
        .span_builder(event.action.clone())
        .with_kind(SpanKind::Client)
        .with_attributes(attrs)
        .start(&tracer);
}
