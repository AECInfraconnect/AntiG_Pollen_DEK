import type { ActivityTimelineItem } from "../entity-graph/types";

export type UserActivityCategory =
  | "files"
  | "web"
  | "email"
  | "apps"
  | "commands"
  | "ai_models"
  | "tools"
  | "safety"
  | "cost"
  | "unknown";

export type UserActivityAction =
  | "read"
  | "write"
  | "connect"
  | "run"
  | "send"
  | "use_model"
  | "call_tool"
  | "redact"
  | "spend"
  | "watch";

export type UserActivityResult =
  | "allowed"
  | "blocked"
  | "asked_first"
  | "asked_and_allowed"
  | "asked_and_denied"
  | "watched_only"
  | "warned"
  | "redacted"
  | "error";

export interface UserFriendlyActivityEvent {
  schema_version: "user-friendly-activity.v1";
  event_id: string;
  timestamp: string;
  agent_id?: string;
  agent_name: string;
  category: UserActivityCategory;
  action: UserActivityAction;
  target_label: string;
  target_kind: string;
  access_mode: "read" | "write" | "connect" | "run" | "send" | "unknown";
  result: UserActivityResult;
  result_label: string;
  plain_summary: string;
  rule_label?: string;
  capability_note: string;
  next_step: string;
  privacy_note: string;
  cost_usd?: number;
  tokens?: number;
  trace_id?: string;
  advanced?: {
    raw_item?: ActivityTimelineItem;
    decision?: string;
    mode?: string;
    pep_plane?: string | null;
    pdp_engine?: string | null;
  };
}

export interface UserFriendlyActivityResponse {
  schema_version: "user-friendly-activity-list.v1";
  tenant_id: string;
  generated_at: string;
  source?: string;
  items: UserFriendlyActivityEvent[];
  next_cursor?: string | null;
}

export interface SimpleRulePreset {
  id: string;
  label: string;
  description: string;
  intent:
    | "watch_all_activity"
    | "ask_before_writing_files"
    | "block_folder_access"
    | "ask_before_terminal_command"
    | "allow_website_domain"
    | "enable_prompt_guard"
    | "limit_ai_model_cost";
  category: UserActivityCategory;
  behavior: "watch" | "allow" | "ask_first" | "block";
}

export interface UserCapabilityItem {
  id: string;
  simple_label: string;
  plain_description: string;
  category: UserActivityCategory;
  can_watch: boolean;
  can_warn: boolean;
  can_ask_first: boolean;
  can_block: boolean;
  status: "ready" | "partial" | "needs_setup" | "not_supported";
  why: string;
  setup_action_ids: string[];
}
