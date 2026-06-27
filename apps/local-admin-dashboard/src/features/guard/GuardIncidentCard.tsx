import { useEffect, useState } from "react";
import { ShieldAlert, ShieldCheck } from "lucide-react";

import { TelemetryApi } from "../../services/api";
import type { GuardEvent, GuardIncidentEnvelope } from "../../types/guard";

const SEVERITY_STYLE: Record<string, string> = {
  critical: "border-red-500/80 bg-red-500/10 text-red-100",
  warn: "border-amber-500/80 bg-amber-500/10 text-amber-100",
  info: "border-slate-500/80 bg-card/70 text-foreground",
};

function normalizeGuardEvent(raw: GuardEvent | GuardIncidentEnvelope): GuardEvent | null {
  const envelope = raw as GuardIncidentEnvelope;
  const payload = envelope.payload;
  const nested =
    payload && "guard_event" in payload ? payload.guard_event : undefined;
  const direct =
    payload && "remediation" in payload ? (payload as GuardEvent) : undefined;
  const guardEvent = nested ?? direct;

  if (envelope.event_type === "guard_incident" && guardEvent) {
    const payloadFindings =
      payload && "findings" in payload ? payload.findings : undefined;
    const payloadRedaction =
      payload && "redaction" in payload ? payload.redaction : undefined;

    return {
      ...guardEvent,
      event_id: guardEvent.event_id || envelope.event_id || "",
      ts: guardEvent.ts || envelope.timestamp || "",
      tenant_id: guardEvent.tenant_id ?? envelope.tenant_id ?? null,
      agent_id: guardEvent.agent_id ?? envelope.agent_id ?? null,
      findings_summary:
        guardEvent.findings_summary ?? payloadFindings ?? envelope.findings ?? [],
      redaction_applied:
        guardEvent.redaction_applied ??
        payloadRedaction?.applied ??
        envelope.redaction_applied ??
        envelope.redaction?.applied ??
        false,
    };
  }

  const event = raw as GuardEvent;
  if (event.event_id && event.remediation) {
    return event;
  }

  return null;
}

export function GuardIncidentCard({ ev }: { ev: GuardEvent }) {
  const severityStyle =
    SEVERITY_STYLE[ev.severity] ?? "border-border bg-card/70 text-foreground";
  const categoryLabel = ev.categories.length ? ev.categories.join(", ") : "-";
  const timestamp = ev.ts ? new Date(ev.ts).toLocaleString("th-TH") : "-";

  return (
    <div className={`rounded-lg border border-l-4 p-4 ${severityStyle}`}>
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="min-w-0">
          <div className="flex items-center gap-2 text-sm font-semibold">
            <ShieldAlert className="h-4 w-4 shrink-0" />
            <span>{ev.action.toUpperCase()} · {categoryLabel}</span>
          </div>
          <div className="mt-1 text-xs text-muted-foreground">
            {ev.direction} · {ev.agent_id || "local-agent"} · {timestamp}
          </div>
        </div>
        <span className="rounded-full border border-current/20 px-2 py-1 text-xs">
          {ev.severity}
        </span>
      </div>

      <p className="mt-3 text-sm">{ev.remediation.user_message}</p>

      {ev.findings_summary.length > 0 && (
        <div className="mt-3 flex flex-wrap gap-2 text-xs">
          {ev.findings_summary.map((finding) => (
            <span
              key={finding.kind}
              className="rounded-full border border-current/20 px-2 py-1"
            >
              {finding.kind}: {finding.count} รายการ
            </span>
          ))}
        </div>
      )}

      {ev.remediation.recommended_actions.length > 0 && (
        <ul className="mt-3 list-disc space-y-1 pl-5 text-sm">
          {ev.remediation.recommended_actions.map((action) => (
            <li key={action}>{action}</li>
          ))}
        </ul>
      )}

      {ev.remediation.can_override && (
        <button className="mt-3 inline-flex items-center gap-2 rounded-md bg-primary px-3 py-2 text-xs font-medium text-primary-foreground">
          <ShieldCheck className="h-4 w-4" />
          ขออนุมัติ
        </button>
      )}
    </div>
  );
}

export function GuardIncidentFeed() {
  const [events, setEvents] = useState<GuardEvent[]>([]);

  useEffect(() => {
    const source = new EventSource(TelemetryApi.streamUrl("guard-events"));
    source.onmessage = (event) => {
      try {
        const parsed = normalizeGuardEvent(JSON.parse(event.data));
        if (!parsed) return;
        setEvents((current) => {
          if (current.some((item) => item.event_id === parsed.event_id)) {
            return current;
          }
          return [parsed, ...current].slice(0, 20);
        });
      } catch {}
    };

    return () => source.close();
  }, []);

  if (events.length === 0) {
    return (
      <div className="rounded-lg border border-dashed border-border/70 bg-card/30 p-8 text-center text-sm text-muted-foreground">
        ยังไม่มี Guard Incident จาก telemetry stream
      </div>
    );
  }

  return (
    <div className="space-y-3">
      {events.map((event) => (
        <GuardIncidentCard key={event.event_id} ev={event} />
      ))}
    </div>
  );
}
