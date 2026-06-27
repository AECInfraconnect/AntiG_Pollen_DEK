import { type ReactNode, useEffect, useState } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import {
  Bot,
  BookOpen,
  CheckCircle2,
  Clock3,
  Database,
  FileKey,
  Fingerprint,
  FolderTree,
  Gauge,
  Shield,
  ShieldCheck,
  Trash2,
  Wrench,
} from "lucide-react";
import {
  Entity360Page,
  type DetailSection,
  type RelatedSection,
} from "../components/entity-360";
import type { RelatedListItem } from "../components/entity-360/RelatedList";
import { entityIcon } from "../features/entity-graph/graphUtils";
import type {
  Entity360Response,
  GraphNode,
} from "../features/entity-graph/types";
import { useEntity360 } from "../features/entity-graph/useEntity360";
import { useConfirm } from "../components/ui/ConfirmDialog";
import { RegistryApi, type AiAgent } from "../services/api";
import type { UiStatus } from "../lib/status";
import {
  assessExpectedCapabilities,
  findAgentReferenceIntel,
} from "../lib/entityReferenceIntel";
import {
  ReferenceIntelInline,
  ReferenceIntelMark,
} from "../components/reference/ReferenceIntelMark";
import { toast } from "sonner";

function useAgents() {
  const [agents, setAgents] = useState<AiAgent[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    RegistryApi.listAgents()
      .then(setAgents)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  return { agents, loading };
}

function useDeleteAgent() {
  const { confirm } = useConfirm();
  return async (id: string) => {
    if (
      !(await confirm({
        title: "Delete Agent",
        description: "Are you sure you want to delete this agent?",
        danger: true,
      }))
    ) {
      return;
    }

    try {
      await RegistryApi.deleteAgent(id);
      toast.success("Agent deleted");
    } catch {
      toast.error("Failed to delete agent");
    }
  };
}

function agentStatus(agent: AiAgent): {
  status: UiStatus;
  label: string;
  tone: "success" | "warning" | "danger" | "info";
} {
  if (agent.enforcement_mode === "Enforce") {
    return { status: "ok", label: "Protected", tone: "success" };
  }
  if (agent.enforcement_mode === "Observe") {
    return { status: "info", label: "Observing", tone: "info" };
  }
  if (agent.enforcement_mode === "Shadow") {
    return { status: "degraded", label: "Shadow AI", tone: "warning" };
  }
  return {
    status: "info",
    label: agent.enforcement_mode || "Registered",
    tone: "info",
  };
}

function buildRelatedSections(
  nodes: GraphNode[],
  centerId: string,
): RelatedSection[] {
  const related = nodes.filter((node) => node.id !== centerId);
  const policies = related.filter((node) => node.type === "policy");
  const tools = related.filter((node) => node.type === "tool");
  const resources = related.filter((node) => node.type === "resource");
  const others = related.filter(
    (node) => !["policy", "tool", "resource"].includes(node.type),
  );

  const sections: RelatedSection[] = [
    {
      title: "Policies",
      icon: FileKey,
      iconColor: "text-amber-600",
      items: policies.map(
        (policy): RelatedListItem => ({
          id: policy.id,
          icon: Shield,
          iconColor: "text-amber-600",
          title: policy.label,
          subtitle: policy.subtitle ?? undefined,
          href: `/policies?id=${policy.entity_id}`,
          badge: policy.status
            ? {
                label: policy.status,
                tone:
                  policy.status === "enforcing"
                    ? "success"
                    : policy.status === "observe"
                      ? "info"
                      : "neutral",
              }
            : undefined,
          meta: policy.metrics.map((metric) => ({
            label: metric.label,
            value: metric.value,
          })),
        }),
      ),
      viewAllHref: "/policies",
    },
    {
      title: "Tools",
      icon: Wrench,
      iconColor: "text-blue-600",
      items: tools.map(
        (tool): RelatedListItem => ({
          id: tool.id,
          icon: Wrench,
          iconColor: "text-blue-600",
          title: tool.label,
          subtitle: tool.subtitle ?? undefined,
          href: `/tools?id=${tool.entity_id}`,
          badge: tool.status
            ? {
                label: tool.status,
                tone: tool.status === "active" ? "success" : "neutral",
              }
            : undefined,
          meta: tool.metrics.map((metric) => ({
            label: metric.label,
            value: metric.value,
          })),
        }),
      ),
      viewAllHref: "/tools",
    },
    {
      title: "Resources",
      icon: Database,
      iconColor: "text-purple-600",
      items: resources.map(
        (resource): RelatedListItem => ({
          id: resource.id,
          icon: Database,
          iconColor: "text-purple-600",
          title: resource.label,
          subtitle: resource.subtitle ?? undefined,
          href: `/resources?id=${resource.entity_id}`,
          meta: resource.metrics.map((metric) => ({
            label: metric.label,
            value: metric.value,
          })),
        }),
      ),
      viewAllHref: "/resources",
    },
  ];

  if (others.length > 0) {
    sections.push({
      title: "Other Entities",
      icon: FolderTree,
      items: others.map(
        (other): RelatedListItem => ({
          id: other.id,
          icon: entityIcon(other.type),
          title: other.label,
          subtitle: `${other.type} - ${other.status}`,
        }),
      ),
    });
  }

  return sections;
}

function formatDateTime(value?: string) {
  if (!value) return "Not recorded";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleString();
}

function summarizeSource(agent: AiAgent) {
  const source = agent.meta?.source ?? "registry";
  if (source === "discovery") return "Auto Discovery";
  if (source === "agent_self_registration") return "Agent self registration";
  if (source === "cloud_sync") return "Pollek Cloud sync";
  return source.replace(/_/g, " ");
}

function referencesForAgent(agent: AiAgent) {
  return findAgentReferenceIntel({
    name: agent.name,
    vendor: agent.vendor,
    agentType: agent.agent_type,
    runtimeName: agent.runtime?.runtime_name,
  });
}

function agentDetailSections(
  agent: AiAgent,
  data: Entity360Response | null | undefined,
  statusLabel: string,
): DetailSection[] {
  const graphNodes = data?.graph.nodes ?? [];
  const relatedTools = graphNodes.filter((node) => node.type === "tool").length;
  const relatedResources = graphNodes.filter(
    (node) => node.type === "resource",
  ).length;
  const relatedPolicies = graphNodes.filter(
    (node) => node.type === "policy",
  ).length;
  const tokenBindings = agent.identity?.token_bindings ?? [];
  const source = summarizeSource(agent);
  const referenceIntel = referencesForAgent(agent);
  const capabilityChecklist = assessExpectedCapabilities(referenceIntel, [
    ...(agent.capabilities ?? []),
    ...(agent.declared_tools ?? []),
    ...(agent.declared_resources ?? []),
    agent.agent_type,
    agent.runtime?.runtime_name ?? "",
  ]);
  const lifecycle = `${formatDateTime(agent.meta?.created_at)} -> ${formatDateTime(
    agent.meta?.updated_at,
  )}`;

  const sections: DetailSection[] = [
    {
      title: "Current Status",
      description:
        "Operational state resolved from registry values and latest entity telemetry.",
      icon: Gauge,
      fields: [
        {
          label: "Status",
          value: statusLabel,
          status: statusLabel === "Protected" ? "ok" : "info",
          source,
          history: lifecycle,
        },
        {
          label: "Enforcement",
          value: agent.enforcement_mode ?? "Not configured",
          status:
            agent.enforcement_mode === "Enforce"
              ? "ok"
              : agent.enforcement_mode === "Shadow"
                ? "warning"
                : "info",
          source: "registry.enforcement_mode",
          history: `Last updated ${formatDateTime(agent.meta?.updated_at)}`,
        },
        {
          label: "Trust Level",
          value: agent.trust_level,
          status:
            agent.trust_level === "high" || agent.trust_level === "system"
              ? "ok"
              : agent.trust_level === "untrusted"
                ? "danger"
                : "warning",
          source: "registry.trust_level",
        },
        {
          label: "Runtime",
          value: `${agent.runtime?.runtime_name ?? "Unknown"}${
            agent.runtime?.version ? ` ${agent.runtime.version}` : ""
          }`,
          source: "runtime fingerprint",
          confidence: data ? "entity graph confirmed" : "registry only",
        },
      ],
    },
    {
      title: "Identity Binding",
      description:
        "How this local agent is tied to process identity, SPIFFE, and cloud-capable tokens.",
      icon: Fingerprint,
      fields: [
        {
          label: "SPIFFE ID",
          value: agent.identity?.spiffe_id ?? "Not bound",
          status: agent.identity?.spiffe_id ? "ok" : "warning",
          source: "agent identity binding",
          note: agent.identity?.spiffe_id
            ? "This agent can be traced across local and cloud control planes."
            : "Local-only observation is available, but enterprise tracing needs a SPIFFE binding.",
        },
        {
          label: "Process Path",
          value: agent.identity?.process_path ?? "Not captured",
          status: agent.identity?.process_path ? "info" : "unknown",
          source: "process observer",
        },
        {
          label: "User Subject",
          value: agent.identity?.user_subject ?? "Local",
          source: "OS account resolver",
        },
        {
          label: "Token Bindings",
          value: tokenBindings.length,
          status: tokenBindings.length ? "ok" : "unknown",
          source: "identity binding registry",
          note: tokenBindings.length
            ? tokenBindings
                .map((token) => `${token.kind}:${token.provider}`)
                .join(", ")
            : "No OAuth/OIDC/JWT-SVID token binding has been registered.",
        },
      ],
    },
    {
      title: "Relationships",
      description:
        "Links that make this agent actionable in policies, tool calls, and data access.",
      icon: ShieldCheck,
      fields: [
        {
          label: "Policies",
          value: relatedPolicies,
          source: "entity graph",
          status: relatedPolicies ? "ok" : "warning",
          note: relatedPolicies
            ? "Policy relationships are available for impact review."
            : "No policy is currently connected to this agent.",
        },
        {
          label: "Tools",
          value: relatedTools || agent.declared_tools?.length || 0,
          source: relatedTools ? "observed graph links" : "declared tools",
        },
        {
          label: "Resources",
          value: relatedResources || agent.declared_resources?.length || 0,
          source: relatedResources
            ? "observed graph links"
            : "declared resources",
        },
        {
          label: "Capabilities",
          value: agent.capabilities?.length ?? 0,
          source: "capability inventory",
          note:
            agent.capabilities?.slice(0, 8).join(", ") ||
            "No explicit capability tags recorded.",
        },
      ],
    },
    {
      title: "Data Sources & History",
      description:
        "Where record values came from, when they changed, and whether they are local or synced.",
      icon: Clock3,
      fields: [
        {
          label: "Primary Source",
          value: source,
          source: "object meta",
        },
        {
          label: "Created",
          value: formatDateTime(agent.meta?.created_at),
          source: "registry object meta",
        },
        {
          label: "Updated",
          value: formatDateTime(agent.meta?.updated_at),
          source: "registry object meta",
        },
        {
          label: "Labels",
          value: Object.keys(agent.labels ?? {}).length,
          source: "registry.labels",
          note: Object.entries(agent.labels ?? {})
            .map(([key, value]) => `${key}=${value}`)
            .join(", "),
        },
      ],
    },
  ];

  if (referenceIntel.length > 0) {
    sections.push({
      title: "Reference Intel",
      description:
        "Well-known external context matched from observed names, vendors, hosts, or versions. This enriches the record but is not enforcement evidence.",
      icon: BookOpen,
      fields: referenceIntel.map((reference) => ({
        label: reference.title,
        value: (
          <a
            href={reference.sourceUrl}
            target="_blank"
            rel="noreferrer"
            className="text-primary underline-offset-4 hover:underline"
          >
            {reference.category}
          </a>
        ),
        status: "info",
        source: reference.sourceLabel,
        history: `Reviewed ${reference.reviewedAt}`,
        note: `${reference.description} Control note: ${reference.controlNotes}`,
      })),
    });
  }

  if (capabilityChecklist.length > 0) {
    sections.push({
      title: "Known Capability Checklist",
      description:
        "Standard capabilities expected for matched well-known entities. Green means local evidence detected a matching capability.",
      icon: CheckCircle2,
      fields: capabilityChecklist.map((capability) => ({
        label: capability.label,
        value: capability.detected ? "Detected" : "Not observed yet",
        status: capability.detected ? "ok" : "unknown",
        source: `definition:${capability.referenceTitle}`,
        note: capability.detected
          ? "Matched against observed or declared local capability evidence."
          : "Expected by reference intel, but not yet confirmed by local evidence.",
      })),
    });
  }

  return sections;
}

function AgentDetailView({
  agent,
  onDelete,
}: {
  agent: AiAgent;
  onDelete: () => void;
}) {
  const navigate = useNavigate();
  const { data } = useEntity360("agent", agent.agent_id);
  const { label, tone } = agentStatus(agent);
  const primaryReference = referencesForAgent(agent)[0];

  const relatedSections = data
    ? buildRelatedSections(data.graph.nodes, data.entity.id)
    : [];
  const detailSections = agentDetailSections(agent, data, label);

  return (
    <Entity360Page
      header={{
        entityType: "Agent",
        entityName: agent.name,
        icon: Bot,
        helpTopicId: "entity.agent",
        visual: primaryReference ? (
          <ReferenceIntelMark reference={primaryReference} />
        ) : undefined,
        status: { label, tone },
        badges: [
          ...(agent.runtime?.runtime_name
            ? [{ label: agent.runtime.runtime_name }]
            : []),
          ...(agent.trust_level
            ? [{ label: `Trust: ${agent.trust_level}` }]
            : []),
        ],
        subtitle: agent.identity?.spiffe_id ?? "Local process agent",
        actions: (
          <>
            <button
              type="button"
              onClick={() => navigate(`/policies?agent=${agent.agent_id}`)}
              className="inline-flex h-9 items-center gap-1.5 rounded-lg bg-primary px-4 text-sm font-medium text-primary-foreground transition-colors hover:bg-primary/90"
            >
              <Shield className="h-3.5 w-3.5" />
              Apply Policy
            </button>
            <button
              type="button"
              onClick={onDelete}
              className="inline-flex h-9 items-center gap-1.5 rounded-lg border border-red-500/30 bg-red-500/10 px-3 text-sm font-medium text-red-600 transition-colors hover:bg-red-500/15"
              aria-label="Delete agent"
            >
              <Trash2 className="h-3.5 w-3.5" />
            </button>
          </>
        ),
        meta: [
          { label: "Type", value: agent.agent_type },
          { label: "Version", value: agent.runtime?.version ?? "Unknown" },
          {
            label: "Identity",
            value: agent.identity?.spiffe_id ? "SPIFFE" : "Local",
          },
        ],
      }}
      aboutSection={<AgentAboutSection agent={agent} />}
      relatedSections={relatedSections}
      data={data}
      detailSections={detailSections}
      extraTabs={[
        {
          id: "capabilities",
          label: "Capabilities",
          icon: Gauge,
          content: <AgentCapabilities agent={agent} />,
        },
      ]}
    />
  );
}

function AgentAboutSection({ agent }: { agent: AiAgent }) {
  const primaryReference = referencesForAgent(agent)[0];

  return (
    <div className="space-y-3">
      {primaryReference && (
        <PropertyRow
          label="Known Entity"
          value={<ReferenceIntelInline reference={primaryReference} />}
        />
      )}
      <PropertyRow label="Agent ID" value={agent.agent_id} />
      <PropertyRow label="Type" value={agent.agent_type} />
      <PropertyRow
        label="Runtime"
        value={agent.runtime?.runtime_name ?? "Unknown"}
      />
      <PropertyRow label="Version" value={agent.runtime?.version ?? "-"} />
      <PropertyRow label="Trust Level" value={agent.trust_level} />
      <PropertyRow
        label="Enforcement"
        value={agent.enforcement_mode ?? "-"}
      />
      <PropertyRow
        label="Process Path"
        value={agent.identity?.process_path ?? "-"}
      />
      <PropertyRow
        label="SPIFFE ID"
        value={agent.identity?.spiffe_id ?? "Not bound"}
      />
      <PropertyRow
        label="User Subject"
        value={agent.identity?.user_subject ?? "Local"}
      />
      <PropertyRow label="Vendor" value={agent.vendor ?? "-"} />
      <PropertyRow
        label="Declared Tools"
        value={
          agent.declared_tools?.length
            ? agent.declared_tools.join(", ")
            : "None"
        }
      />
    </div>
  );
}

function AgentCapabilities({ agent }: { agent: AiAgent }) {
  const allCaps = [
    ...(agent.capabilities ?? []),
    ...(agent.declared_tools ?? []).map((tool) => `tool: ${tool}`),
    ...(agent.declared_resources ?? []).map((resource) => `resource: ${resource}`),
  ];

  if (!allCaps.length) {
    return (
      <div className="text-sm text-muted-foreground">
        No specific capabilities registered for this agent.
      </div>
    );
  }

  return (
    <div className="grid gap-2 sm:grid-cols-2 lg:grid-cols-3">
      {allCaps.map((capability) => (
        <div
          key={capability}
          className="flex items-center gap-2 rounded-lg border bg-muted/30 px-3 py-2 text-sm"
        >
          <div className="h-2 w-2 rounded-full bg-primary/60" />
          <span className="font-medium">{capability}</span>
        </div>
      ))}
    </div>
  );
}

function PropertyRow({ label, value }: { label: string; value: ReactNode }) {
  return (
    <div className="flex items-start justify-between gap-2 border-b border-border/30 pb-2 last:border-0 last:pb-0">
      <span className="whitespace-nowrap text-xs text-muted-foreground">
        {label}
      </span>
      <span className="break-all text-right text-xs font-medium text-foreground/80">
        {value}
      </span>
    </div>
  );
}

export default function AgentsV2() {
  const [searchParams, setSearchParams] = useSearchParams();
  const selectedId = searchParams.get("id") ?? undefined;
  const { agents, loading } = useAgents();
  const deleteAgent = useDeleteAgent();

  const handleSelect = (id: string) => {
    setSearchParams({ id });
  };

  const selectedAgent = agents.find((agent) => agent.agent_id === selectedId);

  if (selectedAgent) {
    return (
      <div className="space-y-4">
        <button
          type="button"
          onClick={() => setSearchParams({})}
          className="inline-flex items-center gap-1 text-sm text-muted-foreground transition-colors hover:text-foreground"
        >
          Back to all agents
        </button>
        <AgentDetailView
          agent={selectedAgent}
          onDelete={() => {
            void deleteAgent(selectedAgent.agent_id);
            setSearchParams({});
          }}
        />
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Agents & Models</h1>
          <p className="text-sm text-muted-foreground">
            All discovered and registered AI agents on this device. Click any
            agent to see its full record view with policies, tools, activity,
            and cost.
          </p>
        </div>
      </div>

      {loading ? (
        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
          {[1, 2, 3].map((item) => (
            <div
              key={item}
              className="h-32 animate-pulse rounded-xl border bg-muted/30"
            />
          ))}
        </div>
      ) : agents.length === 0 ? (
        <div className="flex flex-col items-center justify-center rounded-xl border border-dashed p-12 text-center">
          <Bot className="mb-3 h-10 w-10 text-muted-foreground/50" />
          <p className="text-sm font-medium">No agents discovered yet</p>
          <p className="mt-1 text-xs text-muted-foreground">
            Run a scan to discover AI agents on this device.
          </p>
        </div>
      ) : (
        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
          {agents.map((agent) => {
            const { tone, label } = agentStatus(agent);
            const primaryReference = referencesForAgent(agent)[0];
            return (
              <button
                key={agent.agent_id}
                type="button"
                onClick={() => handleSelect(agent.agent_id)}
                className="group relative flex flex-col rounded-xl border bg-card/60 p-4 text-left transition-all hover:border-primary/30 hover:bg-primary/5 hover:shadow-md"
              >
                <div className="flex items-start gap-3">
                  {primaryReference ? (
                    <ReferenceIntelMark reference={primaryReference} />
                  ) : (
                    <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-primary/10 text-primary">
                      <Bot className="h-5 w-5" />
                    </div>
                  )}
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                      <span className="truncate text-sm font-semibold">
                        {agent.name}
                      </span>
                      <span
                        className={`inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[10px] font-medium ${
                          tone === "success"
                            ? "bg-emerald-500/10 text-emerald-700"
                            : tone === "info"
                              ? "bg-blue-500/10 text-blue-700"
                              : tone === "warning"
                                ? "bg-amber-500/10 text-amber-700"
                                : "bg-red-500/10 text-red-700"
                        }`}
                      >
                        <span
                          className={`h-1.5 w-1.5 rounded-full ${
                            tone === "success"
                              ? "bg-emerald-500"
                              : tone === "info"
                                ? "bg-blue-500"
                                : tone === "warning"
                                  ? "bg-amber-500"
                                  : "bg-red-500"
                          }`}
                        />
                        {label}
                      </span>
                    </div>
                    <p className="mt-0.5 truncate text-xs text-muted-foreground">
                      {agent.runtime?.runtime_name}{" "}
                      {agent.runtime?.version ? `v${agent.runtime.version}` : ""}
                    </p>
                  </div>
                </div>
                <div className="mt-3 flex flex-wrap gap-1.5">
                  <span className="rounded border border-border bg-muted/50 px-1.5 py-0.5 text-[10px] font-medium uppercase">
                    {agent.agent_type}
                  </span>
                  <span className="rounded border border-border bg-muted/50 px-1.5 py-0.5 text-[10px] font-medium uppercase">
                    Trust: {agent.trust_level}
                  </span>
                </div>
                <div className="mt-2 text-[11px] text-muted-foreground">
                  {agent.declared_tools?.length ?? 0} tools -{" "}
                  {agent.capabilities?.length ?? 0} capabilities
                </div>
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
}
