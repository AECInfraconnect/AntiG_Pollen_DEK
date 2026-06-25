
const fs = require("fs");
const file = "apps/local-admin-dashboard/src/services/types.ts";
let content = fs.readFileSync(file, "utf8");

const newTypes = `
export type EventCategory =
  | "discovery"
  | "capability"
  | "policy_feasibility"
  | "deployment"
  | "approval"
  | "enforcement"
  | "observation"
  | "telemetry"
  | "health"
  | "rollback";

export type EventStatus = "pending" | "success" | "warning" | "error" | "info";

export interface UserVisibleEvent {
  event_id: string;
  correlation_id: string;
  scan_id?: string;
  deployment_id?: string;
  agent_id?: string;
  entity_id?: string;
  policy_id?: string;
  category: EventCategory;
  status: EventStatus;
  title: LocalizedText;
  detail: LocalizedText;
  next_action?: RequiredUserAction;
  advanced?: any;
  created_at: string;
}

export type FallbackBehavior =
  | "downgrade_to_observe"
  | "warn_then_observe"
  | "require_user_setup"
  | "none";

export interface RoutePreview {
  user_control_method: ControlMethod;
  advanced_pep?: InternalPep;
  advanced_pdp?: InternalPdp;
  fallback: FallbackBehavior;
  warm_check_required: boolean;
  explanation: LocalizedText;
}

export type EntityStatus = "active" | "inactive" | "pending" | "error" | "observing";

export interface EntityCardModel {
  entity_id: string;
  kind: string;
  display_name: string;
  icon_url?: string;
  status: EntityStatus;
  primary_status_text: LocalizedText;
  secondary_status_text?: LocalizedText;
  tags: string[];
  last_updated_at: string;
}
`;

if (!content.includes("export interface RoutePreview")) {
    fs.appendFileSync(file, "\n" + newTypes);
    console.log("Types added.");
} else {
    console.log("Types already exist.");
}

