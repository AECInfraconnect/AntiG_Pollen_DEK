/**
 * AgentsV2 — Entity-Centric 360° Agent Detail Page
 *
 * Redesigned to show full entity relationships in a Salesforce Lightning-style layout:
 * - Left: About section with agent properties
 * - Center: Activity feed, relationship graph, cost breakdown
 * - Right: Related Lists (Policies, Tools, Resources, Cost)
 *
 * This replaces the previous master-detail list view with a richer, context-aware page.
 */
import { useState, useEffect } from "react";
import { useSearchParams, useNavigate } from "react-router-dom";
import {
  Bot,
  FileKey,
  FolderTree,
  Shield,
  Wrench,
  Database,
  Trash2,
} from "lucide-react";
import { Entity360Page, type RelatedSection } from "../components/entity-360";
import type { RelatedListItem } from "../components/entity-360/RelatedList";
// MasterDetailLayout and EntityCard available but not used in grid view
// import { MasterDetailLayout } from "../components/master-detail/MasterDetailLayout";
// import { EntityCard } from "../components/master-detail/EntityCard";
import { useEntity360 } from "../features/entity-graph/useEntity360";
import { entityIcon } from "../features/entity-graph/graphUtils";
import type { GraphNode } from "../features/entity-graph/types";
import { RegistryApi, type AiAgent } from "../services/api";
import { useConfirm } from "../components/ui/ConfirmDialog";
import { toast } from "sonner";
import type { UiStatus } from "../lib/status";

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
    )
      return;
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
  if (agent.enforcement_mode === "Enforce")
    return { status: "ok", label: "Protected", tone: "success" };
  if (agent.enforcement_mode === "Observe")
    return { status: "info", label: "Observing", tone: "info" };
  if (agent.enforcement_mode === "Shadow")
    return { status: "degraded", label: "Shadow AI", tone: "warning" };
  return { status: "info", label: agent.enforcement_mode || "Registered", tone: "info" };
}

function buildRelatedSections(
  nodes: GraphNode[],
  centerId: string,
): RelatedSection[] {
  const related = nodes.filter((n) => n.id !== centerId);

  const policies = related.filter((n) => n.type === "policy");
  const tools = related.filter((n) => n.type === "tool");
  const resources = related.filter((n) => n.type === "resource");
  const others = related.filter(
    (n) => !["policy", "tool", "resource"].includes(n.type),
  );

  const sections: RelatedSection[] = [];

  if (policies.length > 0 || true) {
    sections.push({
      title: "Policies",
      icon: FileKey,
      iconColor: "text-amber-600",
      items: policies.map(
        (p): RelatedListItem => ({
          id: p.id,
          icon: Shield,
          iconColor: "text-amber-600",
          title: p.label,
          subtitle: p.subtitle ?? undefined,
          href: `/policies?id=${p.entity_id}`,
          badge: p.status
            ? {
                label: p.status,
                tone:
                  p.status === "enforcing"
                    ? "success"
                    : p.status === "observe"
                      ? "info"
                      : "neutral",
              }
            : undefined,
          meta: p.metrics.map((m) => ({ label: m.label, value: m.value })),
        }),
      ),
      viewAllHref: "/policies",
    });
  }

  if (tools.length > 0 || true) {
    sections.push({
      title: "Tools",
      icon: Wrench,
      iconColor: "text-blue-600",
      items: tools.map(
        (t): RelatedListItem => ({
          id: t.id,
          icon: Wrench,
          iconColor: "text-blue-600",
          title: t.label,
          subtitle: t.subtitle ?? undefined,
          href: `/tools?id=${t.entity_id}`,
          badge: t.status
            ? {
                label: t.status,
                tone: t.status === "active" ? "success" : "neutral",
              }
            : undefined,
          meta: t.metrics.map((m) => ({ label: m.label, value: m.value })),
        }),
      ),
      viewAllHref: "/tools",
    });
  }

  if (resources.length > 0 || true) {
    sections.push({
      title: "Resources",
      icon: Database,
      iconColor: "text-purple-600",
      items: resources.map(
        (r): RelatedListItem => ({
          id: r.id,
          icon: Database,
          iconColor: "text-purple-600",
          title: r.label,
          subtitle: r.subtitle ?? undefined,
          href: `/resources?id=${r.entity_id}`,
          meta: r.metrics.map((m) => ({ label: m.label, value: m.value })),
        }),
      ),
      viewAllHref: "/resources",
    });
  }

  if (others.length > 0) {
    sections.push({
      title: "Other Entities",
      icon: FolderTree,
      items: others.map(
        (o): RelatedListItem => ({
          id: o.id,
          icon: entityIcon(o.type),
          title: o.label,
          subtitle: `${o.type} - ${o.status}`,
        }),
      ),
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

  const relatedSections = data
    ? buildRelatedSections(data.graph.nodes, data.entity.id)
    : [];

  return (
    <Entity360Page
      header={{
        entityType: "Agent",
        entityName: agent.name,
        icon: Bot,
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
              className="inline-flex h-9 items-center gap-1.5 rounded-lg bg-primary px-4 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors"
            >
              <Shield className="h-3.5 w-3.5" />
              Apply Policy
            </button>
            <button
              type="button"
              onClick={onDelete}
              className="inline-flex h-9 items-center gap-1.5 rounded-lg border border-red-500/30 bg-red-500/10 px-3 text-sm font-medium text-red-600 hover:bg-red-500/15 transition-colors"
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
      extraTabs={[
        {
          id: "capabilities",
          label: "Capabilities",
          content: <AgentCapabilities agent={agent} />,
        },
      ]}
    />
  );
}

function AgentAboutSection({ agent }: { agent: AiAgent }) {
  return (
    <div className="space-y-3">
      <PropertyRow label="Agent ID" value={agent.agent_id} />
      <PropertyRow label="Type" value={agent.agent_type} />
      <PropertyRow
        label="Runtime"
        value={agent.runtime?.runtime_name ?? "Unknown"}
      />
      <PropertyRow label="Version" value={agent.runtime?.version ?? "—"} />
      <PropertyRow label="Trust Level" value={agent.trust_level} />
      <PropertyRow
        label="Enforcement"
        value={agent.enforcement_mode ?? "—"}
      />
      <PropertyRow
        label="Process Path"
        value={agent.identity?.process_path ?? "—"}
      />
      <PropertyRow
        label="SPIFFE ID"
        value={agent.identity?.spiffe_id ?? "Not bound"}
      />
      <PropertyRow
        label="User Subject"
        value={agent.identity?.user_subject ?? "Local"}
      />
      <PropertyRow
        label="Vendor"
        value={agent.vendor ?? "—"}
      />
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
    ...(agent.declared_tools ?? []).map((t) => `tool: ${t}`),
    ...(agent.declared_resources ?? []).map((r) => `resource: ${r}`),
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
      {allCaps.map((cap: string) => (
        <div
          key={cap}
          className="flex items-center gap-2 rounded-lg border bg-muted/30 px-3 py-2 text-sm"
        >
          <div className="h-2 w-2 rounded-full bg-primary/60" />
          <span className="font-medium">{cap}</span>
        </div>
      ))}
    </div>
  );
}

function PropertyRow({
  label,
  value,
}: {
  label: string;
  value: string | React.ReactNode;
}) {
  return (
    <div className="flex items-start justify-between gap-2 border-b border-border/30 pb-2 last:border-0 last:pb-0">
      <span className="text-xs text-muted-foreground whitespace-nowrap">
        {label}
      </span>
      <span className="text-right text-xs font-medium text-foreground/80 break-all">
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

  // If an agent is selected, show the 360° detail view
  const selectedAgent = agents.find((a) => a.agent_id === selectedId);

  if (selectedAgent) {
    return (
      <div className="space-y-4">
        {/* Back button */}
        <button
          type="button"
          onClick={() => setSearchParams({})}
          className="inline-flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground transition-colors"
        >
          ← Back to all agents
        </button>
        <AgentDetailView
          agent={selectedAgent}
          onDelete={() => {
            deleteAgent(selectedAgent.agent_id);
            setSearchParams({});
          }}
        />
      </div>
    );
  }

  // Otherwise show the master list with cards
  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Agents & Models</h1>
          <p className="text-sm text-muted-foreground">
            All discovered and registered AI agents on this device. Click any
            agent to see its full 360° view with policies, tools, activity, and
            cost.
          </p>
        </div>
      </div>

      {loading ? (
        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
          {[1, 2, 3].map((i) => (
            <div key={i} className="h-32 animate-pulse rounded-xl border bg-muted/30" />
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
          {agents.map((a) => {
            const { tone, label } = agentStatus(a);
            return (
              <button
                key={a.agent_id}
                type="button"
                onClick={() => handleSelect(a.agent_id)}
                className="group relative flex flex-col rounded-xl border bg-card/60 p-4 text-left transition-all hover:border-primary/30 hover:bg-primary/5 hover:shadow-md"
              >
                <div className="flex items-start gap-3">
                  <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-primary/10 text-primary">
                    <Bot className="h-5 w-5" />
                  </div>
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                      <span className="truncate text-sm font-semibold">{a.name}</span>
                      <span className={`inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[10px] font-medium ${
                        tone === "success" ? "bg-emerald-500/10 text-emerald-700" :
                        tone === "info" ? "bg-blue-500/10 text-blue-700" :
                        tone === "warning" ? "bg-amber-500/10 text-amber-700" :
                        "bg-red-500/10 text-red-700"
                      }`}>
                        <span className={`h-1.5 w-1.5 rounded-full ${
                          tone === "success" ? "bg-emerald-500" :
                          tone === "info" ? "bg-blue-500" :
                          tone === "warning" ? "bg-amber-500" :
                          "bg-red-500"
                        }`} />
                        {label}
                      </span>
                    </div>
                    <p className="mt-0.5 truncate text-xs text-muted-foreground">
                      {a.runtime?.runtime_name} {a.runtime?.version ? `v${a.runtime.version}` : ""}
                    </p>
                  </div>
                </div>
                <div className="mt-3 flex flex-wrap gap-1.5">
                  <span className="rounded border border-border bg-muted/50 px-1.5 py-0.5 text-[10px] font-medium uppercase">
                    {a.agent_type}
                  </span>
                  <span className="rounded border border-border bg-muted/50 px-1.5 py-0.5 text-[10px] font-medium uppercase">
                    Trust: {a.trust_level}
                  </span>
                </div>
                <div className="mt-2 text-[11px] text-muted-foreground">
                  {a.declared_tools?.length ?? 0} tools • {a.capabilities?.length ?? 0} capabilities
                </div>
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
}
