import type { UserFriendlyActivityEvent } from "../features/user-activity/types";
import type { AiAgent, DiscoveredAgentCandidateV2 } from "../services/types";

type SuggestedRegistration = {
  agent_id?: unknown;
  name?: unknown;
};

type UsageAgentAliasSource = {
  agent_id?: unknown;
  agent_type?: unknown;
  shadow_candidate_id?: unknown;
  provider?: unknown;
};

function textValue(value: unknown) {
  return typeof value === "string" ? value.trim() : "";
}

function mapIfPresent(map: Map<string, string>, key: unknown, label: unknown) {
  const normalizedKey = textValue(key);
  const normalizedLabel = textValue(label);
  if (normalizedKey && normalizedLabel) {
    map.set(normalizedKey, normalizedLabel);
  }
}

function suggestedRegistration(candidate: DiscoveredAgentCandidateV2) {
  return (candidate.suggested_registration ?? {}) as SuggestedRegistration;
}

export function buildAgentNameMap(
  agents: AiAgent[],
  candidates: DiscoveredAgentCandidateV2[],
) {
  const names = new Map<string, string>();

  for (const agent of agents) {
    mapIfPresent(names, agent.agent_id, agent.name);
    mapIfPresent(names, agent.name, agent.name);
  }

  for (const candidate of candidates) {
    const displayName = candidate.display_name || candidate.candidate_id;
    const registration = suggestedRegistration(candidate);
    const extended = candidate as DiscoveredAgentCandidateV2 & {
      matched_signature_id?: unknown;
    };
    mapIfPresent(names, candidate.candidate_id, displayName);
    mapIfPresent(names, candidate.canonical_service_id, displayName);
    mapIfPresent(names, candidate.surface_group_id, displayName);
    mapIfPresent(names, extended.matched_signature_id, displayName);
    mapIfPresent(names, registration.agent_id, displayName);
    mapIfPresent(names, registration.name, displayName);
    mapIfPresent(names, candidate.labels?.registered_agent_id, displayName);
    mapIfPresent(names, candidate.labels?.agent_id, displayName);
    mapIfPresent(names, candidate.labels?.suggested_agent_id, displayName);
    mapIfPresent(names, candidate.labels?.canonical_service_id, displayName);
    mapIfPresent(names, candidate.labels?.surface_group_id, displayName);
  }

  return names;
}

export function addUsageEventAgentAliases(
  names: Map<string, string>,
  events: UsageAgentAliasSource[],
) {
  for (const event of events) {
    const agentId = textValue(event.agent_id);
    if (!agentId) continue;
    const resolved =
      names.get(textValue(event.shadow_candidate_id)) ??
      names.get(textValue(event.agent_type));
    if (resolved) {
      names.set(agentId, resolved);
    }
  }
  return names;
}

function agentKeys(item: UserFriendlyActivityEvent) {
  return [
    item.agent_id,
    item.agent_name,
    item.advanced?.raw_agent_label,
    item.advanced?.raw_item?.actor?.entity_id,
    item.advanced?.raw_item?.actor?.label,
  ];
}

function isUsefulResolvedName(name: string, current: string) {
  if (!name) return false;
  if (name === current) return false;
  return !/^agent_[0-9a-f-]+$/i.test(name);
}

export function resolveActivityAgentName(
  item: UserFriendlyActivityEvent,
  names: Map<string, string>,
) {
  const resolved = agentKeys(item)
    .map((key) => names.get(textValue(key)))
    .find((name) => isUsefulResolvedName(name ?? "", item.agent_name));

  if (!resolved) return item;

  const previousName = item.agent_name;
  const plainSummary = item.plain_summary?.includes(previousName)
    ? item.plain_summary.replace(previousName, resolved)
    : item.plain_summary;

  return {
    ...item,
    agent_name: resolved,
    plain_summary: plainSummary,
    advanced: {
      ...item.advanced,
      raw_agent_label: item.advanced?.raw_agent_label ?? previousName,
    },
  };
}

export function resolveActivityAgentNames(
  items: UserFriendlyActivityEvent[],
  names: Map<string, string>,
) {
  return items.map((item) => resolveActivityAgentName(item, names));
}
