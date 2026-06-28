import { describe, expect, it } from "vitest";
import type { GuardEvent, GuardIncidentEnvelope } from "../../types/guard";
import {
  guardActionLabel,
  guardCategoryLabel,
  guardFindingLabel,
  mergeGuardEvents,
  normalizeGuardEvent,
} from "./GuardIncidentCard";

function event(overrides: Partial<GuardEvent>): GuardEvent {
  return {
    event_id: "guard_1",
    ts: "2026-06-28T10:00:00.000Z",
    tenant_id: "local",
    agent_id: "agent_codex",
    direction: "request",
    action: "allow",
    categories: ["prompt_injection"],
    injection_score: 0,
    findings_summary: [],
    severity: "info",
    remediation: {
      user_message: "Pollek observed this prompt safety event.",
      recommended_actions: [],
      can_override: false,
    },
    redaction_applied: false,
    ...overrides,
  };
}

describe("Guard incident helpers", () => {
  it("normalizes stored guard incident envelopes for the UI", () => {
    const raw: GuardIncidentEnvelope = {
      event_type: "guard_incident",
      event_id: "env_guard_1",
      timestamp: "2026-06-28T11:00:00.000Z",
      tenant_id: "local",
      agent_id: "agent_chatgpt",
      payload: {
        guard_event: event({
          event_id: "",
          ts: "",
          agent_id: null,
          action: "deny",
          severity: "critical",
        }),
        findings: [{ kind: "api_key", count: 2 }],
        redaction: { applied: true },
      },
    };

    const normalized = normalizeGuardEvent(raw);

    expect(normalized?.event_id).toBe("env_guard_1");
    expect(normalized?.ts).toBe("2026-06-28T11:00:00.000Z");
    expect(normalized?.agent_id).toBe("agent_chatgpt");
    expect(normalized?.findings_summary).toEqual([
      { kind: "api_key", count: 2 },
    ]);
    expect(normalized?.redaction_applied).toBe(true);
  });

  it("uses friendly labels instead of raw policy terms", () => {
    expect(guardActionLabel("deny")).toBe("Blocked");
    expect(guardActionLabel("redact")).toBe("Sensitive data protected");
    expect(guardCategoryLabel("prompt_injection")).toBe(
      "Prompt injection attempt",
    );
    expect(guardCategoryLabel("llm01_prompt_injection")).toBe(
      "Prompt injection attempt",
    );
    expect(guardCategoryLabel("llm02_sensitive_information_disclosure")).toBe(
      "Sensitive information disclosure",
    );
    expect(guardFindingLabel("api_key")).toBe("API key or secret");
  });

  it("deduplicates events and keeps the newest history first", () => {
    const older = event({
      event_id: "same",
      ts: "2026-06-28T09:00:00.000Z",
      action: "allow",
    });
    const newer = event({
      event_id: "same",
      ts: "2026-06-28T11:00:00.000Z",
      action: "deny",
    });
    const other = event({
      event_id: "other",
      ts: "2026-06-28T10:00:00.000Z",
      action: "redact",
    });

    const merged = mergeGuardEvents([older, other], [newer]);

    expect(merged.map((item) => item.event_id)).toEqual(["same", "other"]);
    expect(merged[0].action).toBe("deny");
  });
});
