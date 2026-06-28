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

export type GuardCapture = {
  source?: string | null;
  engine?: string | null;
  surface?: string | null;
  session_id?: string | null;
  url_host?: string | null;
  text_length?: number | null;
  raw_text_persisted?: boolean | null;
};

export type GuardAnalysisPipeline = {
  mode?: string | null;
  steps?: string[];
  enterprise_cloud_ner_supported?: boolean;
  enterprise_cloud_ner_enabled?: boolean;
  third_party_provider?: string | null;
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
  source?: string | null;
  matched_rules?: string[];
  normalization_steps?: string[];
  capture?: GuardCapture | null;
  analysis_pipeline?: GuardAnalysisPipeline | null;
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
