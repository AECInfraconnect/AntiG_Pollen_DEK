import { type ReactNode, useEffect, useState } from "react";
import { useSearchParams } from "react-router-dom";
import {
  Bot,
  Clock3,
  Database,
  FileKey,
  FolderTree,
  Gauge,
  ShieldCheck,
  Wrench,
} from "lucide-react";
import {
  Entity360Page,
  type DetailSection,
  type RelatedSection,
} from "../components/entity-360";
import type { RelatedListItem } from "../components/entity-360/RelatedList";
import { EntityCard } from "../components/master-detail/EntityCard";
import { MasterDetailLayout } from "../components/master-detail/MasterDetailLayout";
import { entityIcon } from "../features/entity-graph/graphUtils";
import type {
  Entity360Response,
  GraphNode,
} from "../features/entity-graph/types";
import { useEntity360 } from "../features/entity-graph/useEntity360";
import { defaultClient } from "../services/api";

interface PolicyItem {
  policy_id: string;
  name: string;
  description?: string;
  engine: string;
  status: string;
  mode: string;
  scope?: string;
  created_at?: string;
  updated_at?: string;
  rules_count?: number;
  source?: string;
  last_deployed_at?: string;
  bundle_id?: string;
}

function usePolicies() {
  const [policies, setPolicies] = useState<PolicyItem[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    defaultClient
      .fetchApi("/policies")
      .then((data) => {
        setPolicies(Array.isArray(data) ? data : data?.items ?? []);
      })
      .catch(() => setPolicies([]))
      .finally(() => setLoading(false));
  }, []);

  return { policies, loading };
}

function buildRelatedSections(nodes: GraphNode[], centerId: string): RelatedSection[] {
  const related = nodes.filter((node) => node.id !== centerId);
  const agents = related.filter((node) => node.type === "agent");
  const tools = related.filter((node) => node.type === "tool");
  const resources = related.filter((node) => node.type === "resource");
  const others = related.filter(
    (node) => !["agent", "tool", "resource"].includes(node.type),
  );

  const sections: RelatedSection[] = [
    {
      title: "Affected Agents",
      icon: Bot,
      iconColor: "text-emerald-600",
      items: agents.map(
        (agent): RelatedListItem => ({
          id: agent.id,
          icon: Bot,
          iconColor: "text-emerald-600",
          title: agent.label,
          subtitle: agent.subtitle ?? undefined,
          href: `/agents?id=${agent.entity_id}`,
          badge: agent.status
            ? {
                label: agent.status,
                tone: agent.status === "active" ? "success" : "neutral",
              }
            : undefined,
          meta: agent.metrics.map((metric) => ({
            label: metric.label,
            value: metric.value,
          })),
        }),
      ),
      viewAllHref: "/agents",
    },
    {
      title: "Governed Tools",
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
          meta: tool.metrics.map((metric) => ({
            label: metric.label,
            value: metric.value,
          })),
        }),
      ),
      viewAllHref: "/tools",
    },
    {
      title: "Protected Resources",
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
        }),
      ),
      viewAllHref: "/resources",
    },
  ];

  if (others.length > 0) {
    sections.push({
      title: "Other Related",
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

function formatDate(value?: string) {
  if (!value) return "Not recorded";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleString();
}

function modeLabel(policy: PolicyItem) {
  if (policy.mode === "enforce") return "Enforcing";
  if (policy.mode === "observe") return "Observe Only";
  if (policy.mode === "ask") return "Ask Before Action";
  return policy.mode || "Unknown";
}

function modeTone(policy: PolicyItem) {
  if (policy.mode === "enforce") return "success" as const;
  if (policy.mode === "observe") return "info" as const;
  return "neutral" as const;
}

function policyDetailSections(
  policy: PolicyItem,
  data: Entity360Response | null | undefined,
): DetailSection[] {
  const graphNodes = data?.graph.nodes ?? [];
  const agents = graphNodes.filter((node) => node.type === "agent").length;
  const tools = graphNodes.filter((node) => node.type === "tool").length;
  const resources = graphNodes.filter((node) => node.type === "resource").length;
  const lifecycle = `${formatDate(policy.created_at)} -> ${formatDate(
    policy.updated_at,
  )}`;

  return [
    {
      title: "Current Status",
      description:
        "Policy state, engine, mode, and whether it is ready to drive governance decisions.",
      icon: Gauge,
      fields: [
        {
          label: "Mode",
          value: modeLabel(policy),
          status: policy.mode === "enforce" ? "ok" : "info",
          source: "policy registry",
          history: lifecycle,
        },
        {
          label: "Status",
          value: policy.status,
          status: policy.status === "published" ? "ok" : "warning",
          source: "policy registry",
        },
        {
          label: "Engine",
          value: policy.engine,
          source: "policy metadata",
          note:
            "Decision evidence should reflect the actual decision path used by the PDP router.",
        },
        {
          label: "Rules",
          value: policy.rules_count ?? 0,
          source: "compiled policy metadata",
        },
      ],
    },
    {
      title: "Impact Surface",
      description:
        "Entities this policy can affect according to registry and observation links.",
      icon: ShieldCheck,
      fields: [
        {
          label: "Affected Agents",
          value: agents,
          status: agents ? "ok" : "warning",
          source: "entity graph",
        },
        {
          label: "Governed Tools",
          value: tools,
          source: "entity graph",
        },
        {
          label: "Protected Resources",
          value: resources,
          source: "entity graph",
        },
        {
          label: "Scope",
          value: policy.scope ?? "All agents",
          source: "policy scope",
        },
      ],
    },
    {
      title: "Deployment & History",
      description:
        "Publication and deployment metadata used for audit and rollback review.",
      icon: Clock3,
      fields: [
        {
          label: "Bundle",
          value: policy.bundle_id ?? "Not bundled",
          status: policy.bundle_id ? "ok" : "unknown",
          source: "bundle registry",
        },
        {
          label: "Last Deployed",
          value: formatDate(policy.last_deployed_at),
          source: "deployment telemetry",
        },
        {
          label: "Created",
          value: formatDate(policy.created_at),
          source: "policy registry",
        },
        {
          label: "Updated",
          value: formatDate(policy.updated_at),
          source: "policy registry",
        },
      ],
    },
  ];
}

function PolicyDetailView({ policy }: { policy: PolicyItem }) {
  const { data } = useEntity360("policy", policy.policy_id);
  const relatedSections = data
    ? buildRelatedSections(data.graph.nodes, data.entity.id)
    : [];

  return (
    <Entity360Page
      header={{
        entityType: "Policy",
        entityName: policy.name,
        icon: FileKey,
        helpTopicId: "entity.policy",
        iconColor: "text-amber-600",
        status: { label: modeLabel(policy), tone: modeTone(policy) },
        badges: [
          { label: policy.engine },
          ...(policy.scope ? [{ label: policy.scope }] : []),
        ],
        subtitle: policy.description ?? "No description provided",
        meta: [
          { label: "Engine", value: policy.engine },
          { label: "Status", value: policy.status },
          ...(policy.rules_count != null
            ? [{ label: "Rules", value: String(policy.rules_count) }]
            : []),
          ...(policy.updated_at
            ? [
                {
                  label: "Updated",
                  value: new Date(policy.updated_at).toLocaleDateString(),
                },
              ]
            : []),
        ],
      }}
      aboutSection={
        <div className="space-y-3">
          <PropertyRow label="Policy ID" value={policy.policy_id} />
          <PropertyRow label="Engine" value={policy.engine} />
          <PropertyRow label="Mode" value={policy.mode} />
          <PropertyRow label="Status" value={policy.status} />
          <PropertyRow label="Scope" value={policy.scope ?? "All agents"} />
          <PropertyRow label="Description" value={policy.description ?? "-"} />
          <PropertyRow label="Created" value={formatDate(policy.created_at)} />
        </div>
      }
      relatedSections={relatedSections}
      data={data}
      detailSections={policyDetailSections(policy, data)}
    />
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

export default function PoliciesV2() {
  const [searchParams, setSearchParams] = useSearchParams();
  const selectedId = searchParams.get("id") ?? undefined;
  const { policies, loading } = usePolicies();

  const handleSelect = (id: string) => {
    setSearchParams({ id });
  };

  const selectedPolicy = policies.find((policy) => policy.policy_id === selectedId);
  if (selectedPolicy) {
    return (
      <div className="space-y-4">
        <button
          type="button"
          onClick={() => setSearchParams({})}
          className="inline-flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground"
        >
          Back to all policies
        </button>
        <PolicyDetailView policy={selectedPolicy} />
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div>
        <h1 className="text-2xl font-bold">Policies</h1>
        <p className="text-sm text-muted-foreground">
          Governance policies with impact, deployment, decision, and evidence
          history.
        </p>
      </div>

      <MasterDetailLayout
        items={policies}
        selectedId={selectedId}
        onSelect={handleSelect}
        idSelector={(policy) => policy.policy_id}
        loading={loading}
        renderCard={(policy, selected) => {
          const status =
            policy.mode === "enforce"
              ? ("ok" as const)
              : policy.mode === "observe"
                ? ("info" as const)
                : ("idle" as const);
          return (
            <EntityCard
              title={policy.name}
              subtitle={policy.description ?? policy.engine}
              summary={`Mode: ${modeLabel(policy)} - Updated: ${formatDate(
                policy.updated_at,
              )}`}
              icon={FileKey}
              status={status}
              statusLabel={policy.mode || "Unknown"}
              meta={[
                { label: "Engine", value: policy.engine },
                ...(policy.rules_count != null
                  ? [{ label: "Rules", value: String(policy.rules_count) }]
                  : []),
              ]}
              selected={selected}
            />
          );
        }}
        renderDetail={(policy) => (
          <div className="p-4">
            <button
              type="button"
              onClick={() => handleSelect(policy.policy_id)}
              className="inline-flex items-center gap-2 rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90"
            >
              Open Record View
            </button>
            <p className="mt-2 text-sm text-muted-foreground">
              Review affected agents, governed tools, protected resources, and
              decision history.
            </p>
          </div>
        )}
      />
    </div>
  );
}
