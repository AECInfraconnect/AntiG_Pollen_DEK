export type GuardAction = "allow" | "redact" | "deny";

export type GuardFindingSummary = {
  kind: string;
  count: number;
};

export type GuardRemediation = {
  user_message: string;
  recommended_actions: string[];
  doc_url?: string | null;
  can_override: boolean;
};

export type GuardEvent = {
  event_id: string;
  ts: string;
  tenant_id?: string | null;
  agent_id?: string | null;
  direction: "request" | "response" | string;
  action: GuardAction;
  categories: string[];
  injection_score: number;
  findings_summary: GuardFindingSummary[];
  severity: "critical" | "warn" | "info" | string;
  remediation: GuardRemediation;
  redaction_applied: boolean;
};

export type GuardIncidentEnvelope = {
  schema_version?: string;
  event_id?: string;
  event_type?: string;
  timestamp?: string;
  tenant_id?: string | null;
  agent_id?: string | null;
  findings?: GuardFindingSummary[];
  redaction?: {
    applied?: boolean;
  };
  redaction_applied?: boolean;
  payload?:
    | GuardEvent
    | {
        guard_event?: GuardEvent;
        findings?: GuardFindingSummary[];
        redaction?: {
          applied?: boolean;
        };
      };
};
