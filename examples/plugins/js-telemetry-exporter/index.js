export function redactForExport(event) {
  const payload = event?.payload ?? {};
  return {
    schema_version: "pollek.plugin.telemetry_export.v1",
    event_id: event?.event_id,
    event_type: event?.event_type,
    timestamp: event?.timestamp,
    agent_id: payload.agent_id ?? payload.agentId ?? null,
    action: payload.action ?? null,
    resource_kind: payload.resource_kind ?? null,
    target_redacted: payload.target_redacted ?? payload.url_host ?? null,
    raw_content_stored: false,
  };
}

export async function exportBatch(events, send) {
  const body = {
    events: events.map(redactForExport),
  };
  await send("https://splunk.example.com:443/services/collector", body);
  return {
    exported: body.events.length,
    raw_content_stored: false,
  };
}
