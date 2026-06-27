import type {
  ControlMethodCapabilityV2,
  LocalCapabilitySnapshotV2,
  ObservationSourceCapabilityV2,
} from "../../services/types";
import type { ActivityTimelineItem } from "../entity-graph/types";
import type {
  SimpleRulePreset,
  UserActivityAction,
  UserActivityCategory,
  UserActivityResult,
  UserCapabilityItem,
  UserFriendlyActivityEvent,
} from "./types";

const categoryLabels: Record<UserActivityCategory, string> = {
  files: "Files & folders",
  web: "Websites & network",
  email: "Email & calendar",
  apps: "Apps",
  commands: "Commands",
  ai_models: "AI models",
  tools: "AI tools",
  cost: "Cost",
  unknown: "Other activity",
};

const actionText: Record<UserActivityAction, string> = {
  read: "read",
  write: "changed",
  connect: "connected to",
  run: "ran",
  send: "sent",
  use_model: "used",
  call_tool: "called",
  spend: "spent tokens on",
  watch: "was seen using",
};

const resultLabels: Record<UserActivityResult, string> = {
  allowed: "Allowed",
  blocked: "Blocked",
  asked_first: "Ask first",
  asked_and_allowed: "Asked and allowed",
  asked_and_denied: "Asked and blocked",
  watched_only: "Watched only",
  warned: "Warned",
  error: "Error",
};

export const SIMPLE_RULE_PRESETS: SimpleRulePreset[] = [
  {
    id: "watch_everything",
    label: "Watch what this AI does",
    description: "Record activity without blocking anything.",
    intent: "watch_all_activity",
    category: "unknown",
    behavior: "watch",
  },
  {
    id: "ask_before_file_write",
    label: "Ask before this AI changes files",
    description: "AI can read files, but must ask before writing or deleting.",
    intent: "ask_before_writing_files",
    category: "files",
    behavior: "ask_first",
  },
  {
    id: "block_sensitive_folder",
    label: "Block a private folder",
    description: "AI cannot read or write files in selected folders.",
    intent: "block_folder_access",
    category: "files",
    behavior: "block",
  },
  {
    id: "ask_before_terminal",
    label: "Ask before running commands",
    description: "AI must ask before running terminal commands or programs.",
    intent: "ask_before_terminal_command",
    category: "commands",
    behavior: "ask_first",
  },
  {
    id: "allow_only_known_websites",
    label: "Only allow approved websites",
    description: "AI can connect only to websites you approve.",
    intent: "allow_website_domain",
    category: "web",
    behavior: "allow",
  },
  {
    id: "limit_ai_spend",
    label: "Limit AI usage cost",
    description: "Warn or block when token or cost usage exceeds your limit.",
    intent: "limit_ai_model_cost",
    category: "cost",
    behavior: "ask_first",
  },
];

export function labelize(value?: string | null) {
  if (!value) return "Unknown";
  return value
    .replace(/[_:.-]+/g, " ")
    .split(" ")
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

export function formatDateTime(value?: string) {
  if (!value) return "Not recorded";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleString();
}

export function categoryLabel(category: UserActivityCategory) {
  return categoryLabels[category];
}

function rawText(item: ActivityTimelineItem) {
  return [
    item.action,
    item.actor?.label,
    item.tool?.label,
    item.resource?.label,
    item.resource?.type,
    item.explanation,
  ]
    .filter(Boolean)
    .join(" ")
    .toLowerCase();
}

function inferCategory(item: ActivityTimelineItem): UserActivityCategory {
  const text = rawText(item);
  const resourceType = item.resource?.type?.toLowerCase() ?? "";
  const toolType = item.tool?.type?.toLowerCase() ?? "";

  if (
    resourceType.includes("file") ||
    resourceType.includes("folder") ||
    text.includes("file") ||
    text.includes("folder") ||
    text.includes("read") ||
    text.includes("write")
  ) {
    return "files";
  }
  if (
    resourceType.includes("domain") ||
    resourceType.includes("url") ||
    text.includes("http") ||
    text.includes("network") ||
    text.includes("domain") ||
    text.includes("connect")
  ) {
    return "web";
  }
  if (text.includes("email") || text.includes("calendar")) return "email";
  if (
    toolType.includes("terminal") ||
    text.includes("terminal") ||
    text.includes("shell") ||
    text.includes("command")
  ) {
    return "commands";
  }
  if (
    text.includes("model") ||
    text.includes("token") ||
    text.includes("llm")
  ) {
    return "ai_models";
  }
  if (item.tool) return "tools";
  if (item.cost?.total_cost_usd || item.cost?.total_tokens) return "cost";
  if (text.includes("process") || text.includes("app")) return "apps";
  return "unknown";
}

function inferAction(
  item: ActivityTimelineItem,
  category: UserActivityCategory,
): UserActivityAction {
  const text = rawText(item);
  if (
    text.includes("write") ||
    text.includes("delete") ||
    text.includes("edit")
  ) {
    return "write";
  }
  if (text.includes("read") || text.includes("open")) return "read";
  if (category === "web") return "connect";
  if (category === "commands" || category === "apps") return "run";
  if (category === "email") return text.includes("send") ? "send" : "read";
  if (category === "ai_models") return "use_model";
  if (category === "tools") return "call_tool";
  if (category === "cost") return "spend";
  return "watch";
}

function inferResult(item: ActivityTimelineItem): UserActivityResult {
  const decision = item.decision.toLowerCase();
  const mode = item.enforcement_mode.toLowerCase();
  if (decision === "deny" || decision === "blocked") return "blocked";
  if (decision === "error") return "error";
  if (decision === "warn") return "warned";
  if (decision === "require_approval") return "asked_first";
  if (decision === "asked_and_allowed") return "asked_and_allowed";
  if (decision === "asked_and_denied") return "asked_and_denied";
  if (mode.includes("observe") || decision === "observe") return "watched_only";
  return "allowed";
}

function accessMode(
  action: UserActivityAction,
): UserFriendlyActivityEvent["access_mode"] {
  if (action === "read" || action === "use_model" || action === "call_tool") {
    return "read";
  }
  if (action === "write") return "write";
  if (action === "connect") return "connect";
  if (action === "run") return "run";
  if (action === "send") return "send";
  return "unknown";
}

function targetLabel(item: ActivityTimelineItem) {
  return (
    item.resource?.label ??
    item.tool?.label ??
    item.cost?.model ??
    item.cost?.provider ??
    "an unknown target"
  );
}

function capabilityNote(
  result: UserActivityResult,
  category: UserActivityCategory,
) {
  if (result === "blocked") return "Pollek blocked this action.";
  if (result === "allowed") return "Pollek saw this action and it was allowed.";
  if (result === "warned") return "Pollek warned about this action.";
  if (result === "asked_first")
    return "Pollek can ask before this kind of action.";
  if (category === "files" || category === "web" || category === "commands") {
    return "Pollek can watch this now. Blocking may require OS setup or an agent-specific setting.";
  }
  return "Pollek can watch this activity and explain what to review next.";
}

function nextStep(result: UserActivityResult, category: UserActivityCategory) {
  if (result === "blocked")
    return "Review the rule if this should be allowed next time.";
  if (category === "files") {
    return "Set a rule for this folder, or restrict file access inside the AI app settings.";
  }
  if (category === "web") {
    return "Set an approved website rule, or restrict network access in the AI app settings.";
  }
  if (category === "commands" || category === "apps") {
    return "Ask before commands, or disable command execution inside the AI app.";
  }
  if (category === "email") {
    return "Keep email access opt-in and review the connector permissions.";
  }
  return "Keep watching or create a rule from similar activity.";
}

export function toUserFriendlyActivity(
  item: ActivityTimelineItem,
): UserFriendlyActivityEvent {
  const category = inferCategory(item);
  const action = inferAction(item, category);
  const result = inferResult(item);
  const agentName = item.actor?.label ?? "Unknown AI app";
  const target = targetLabel(item);
  const summary = `${agentName} ${actionText[action]} ${target}`;

  return {
    schema_version: "user-friendly-activity.v1",
    event_id: item.event_id,
    timestamp: item.timestamp,
    agent_id: item.actor?.entity_id,
    agent_name: agentName,
    category,
    action,
    target_label: target,
    target_kind: categoryLabels[category],
    access_mode: accessMode(action),
    result,
    result_label: resultLabels[result],
    plain_summary: summary,
    rule_label: item.policies[0]?.label,
    capability_note: capabilityNote(result, category),
    next_step: nextStep(result, category),
    privacy_note:
      "Pollek shows activity metadata here, not file contents, email bodies, raw prompts, or raw responses.",
    cost_usd: item.cost?.total_cost_usd ?? undefined,
    tokens: item.cost?.total_tokens ?? undefined,
    trace_id: item.trace_id ?? undefined,
    advanced: {
      raw_item: item,
      decision: item.decision,
      mode: item.enforcement_mode,
      pep_plane: item.pep_plane,
      pdp_engine: item.pdp_engine,
    },
  };
}

export function summarizeActivities(items: UserFriendlyActivityEvent[]) {
  return {
    total: items.length,
    files: items.filter((item) => item.category === "files").length,
    web: items.filter((item) => item.category === "web").length,
    commands: items.filter((item) => item.category === "commands").length,
    blocked: items.filter((item) => item.result === "blocked").length,
    watched: items.filter((item) => item.result === "watched_only").length,
    costUsd: items.reduce((total, item) => total + (item.cost_usd ?? 0), 0),
  };
}

function canReachLevel(
  method: ControlMethodCapabilityV2,
  level: "warn" | "ask" | "enforce" | "strict_deny",
) {
  const order = ["observe", "warn", "ask", "enforce", "strict_deny"];
  return order.indexOf(method.max_level) >= order.indexOf(level);
}

function domainsForCategory(category: UserActivityCategory) {
  const lookup: Record<UserActivityCategory, string[]> = {
    files: ["file_access"],
    web: ["network_egress", "dns", "browser_ai_session"],
    email: ["identity_use", "browser_ai_session"],
    apps: ["process_launch"],
    commands: ["process_launch"],
    ai_models: ["token_cost", "prompt_content"],
    tools: ["mcp_tool_call"],
    cost: ["token_cost"],
    unknown: [],
  };
  return lookup[category];
}

function sourceCovers(
  source: ObservationSourceCapabilityV2,
  category: UserActivityCategory,
) {
  const domains = domainsForCategory(category);
  return (
    domains.length === 0 ||
    source.domains.some((domain) => domains.includes(domain))
  );
}

function methodCovers(
  method: ControlMethodCapabilityV2,
  category: UserActivityCategory,
) {
  const domains = domainsForCategory(category);
  return (
    domains.length === 0 ||
    method.domains.some((domain) => domains.includes(domain))
  );
}

export function buildUserCapabilityMatrix(
  snapshot: LocalCapabilitySnapshotV2 | null,
): UserCapabilityItem[] {
  const methods = snapshot?.control_methods ?? [];
  const sources = snapshot?.observation_sources ?? [];
  const categories: UserActivityCategory[] = [
    "files",
    "web",
    "email",
    "commands",
    "tools",
    "ai_models",
    "cost",
  ];

  return categories.map((category) => {
    const observing = sources.filter((source) =>
      sourceCovers(source, category),
    );
    const controls = methods.filter((method) => methodCovers(method, category));
    const canWatch = observing.some((source) =>
      ["available", "degraded"].includes(source.status),
    );
    const canWarn = controls.some((method) => canReachLevel(method, "warn"));
    const canAsk = controls.some((method) => canReachLevel(method, "ask"));
    const canBlock = controls.some((method) =>
      canReachLevel(method, "enforce"),
    );
    const setupIds = [...observing, ...controls].flatMap(
      (item) => item.setup_action_ids ?? [],
    );
    const needsSetup = [...observing, ...controls].some((item) =>
      ["needs_install", "needs_permission", "needs_configuration"].includes(
        item.status,
      ),
    );
    const status: UserCapabilityItem["status"] = canBlock
      ? "ready"
      : canWatch || canWarn || canAsk
        ? "partial"
        : needsSetup
          ? "needs_setup"
          : "not_supported";

    return {
      id: category,
      simple_label: categoryLabels[category],
      plain_description:
        status === "ready"
          ? "Pollek can watch and control this on this computer."
          : status === "partial"
            ? "Pollek can watch this now. Some controls may need setup."
            : status === "needs_setup"
              ? "Install or approve the setup steps before Pollek can watch this well."
              : "No local source reports support for this yet.",
      category,
      can_watch: canWatch,
      can_warn: canWarn,
      can_ask_first: canAsk,
      can_block: canBlock,
      status,
      why:
        status === "ready"
          ? "A local control method reports blocking support."
          : status === "partial"
            ? "Observation is available, but blocking is not ready for every path."
            : status === "needs_setup"
              ? "The local host reports missing permissions, configuration, or components."
              : "No matching local observation or control method was reported.",
      setup_action_ids: Array.from(new Set(setupIds)),
    };
  });
}

export function capabilityTone(status: UserCapabilityItem["status"]) {
  if (status === "ready") return "success";
  if (status === "partial") return "info";
  if (status === "needs_setup") return "warning";
  return "neutral";
}
