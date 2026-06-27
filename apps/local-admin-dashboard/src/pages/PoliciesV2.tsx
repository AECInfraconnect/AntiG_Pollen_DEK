/**
 * PoliciesV2 — Entity-Centric 360° Policy Detail Page
 *
 * Shows each policy with full relationship context:
 * - Which agents are affected
 * - Which tools/resources are governed
 * - Activity timeline of decisions made by this policy
 * - Impact analysis (observe vs enforce)
 */
import { useState, useEffect } from "react";
import { useSearchParams } from "react-router-dom";
import {
  Bot,
  Database,
  FileKey,
  Wrench,
  FolderTree,
} from "lucide-react";
import {
  Entity360Page,
  type RelatedSection,
} from "../components/entity-360";
import type { RelatedListItem } from "../components/entity-360/RelatedList";
import { MasterDetailLayout } from "../components/master-detail/MasterDetailLayout";
import { EntityCard } from "../components/master-detail/EntityCard";
import { useEntity360 } from "../features/entity-graph/useEntity360";
import { entityIcon } from "../features/entity-graph/graphUtils";
import type { GraphNode } from "../features/entity-graph/types";
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
  const related = nodes.filter((n) => n.id !== centerId);
  const agents = related.filter((n) => n.type === "agent");
  const tools = related.filter((n) => n.type === "tool");
  const resources = related.filter((n) => n.type === "resource");
  const others = related.filter(
    (n) => !["agent", "tool", "resource"].includes(n.type),
  );

  const sections: RelatedSection[] = [];

  sections.push({
    title: "Affected Agents",
    icon: Bot,
    iconColor: "text-emerald-600",
    items: agents.map(
      (a): RelatedListItem => ({
        id: a.id,
        icon: Bot,
        iconColor: "text-emerald-600",
        title: a.label,
        subtitle: a.subtitle ?? undefined,
        href: `/agents?id=${a.entity_id}`,
        badge: a.status
          ? {
              label: a.status,
              tone: a.status === "active" ? "success" : "neutral",
            }
          : undefined,
        meta: a.metrics.map((m) => ({ label: m.label, value: m.value })),
      }),
    ),
    viewAllHref: "/agents",
  });

  sections.push({
    title: "Governed Tools",
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
        meta: t.metrics.map((m) => ({ label: m.label, value: m.value })),
      }),
    ),
    viewAllHref: "/tools",
  });

  sections.push({
    title: "Protected Resources",
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
      }),
    ),
    viewAllHref: "/resources",
  });

  if (others.length > 0) {
    sections.push({
      title: "Other Related",
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

function PolicyDetailView({ policy }: { policy: PolicyItem }) {
  const { data } = useEntity360("policy", policy.policy_id);
  const relatedSections = data
    ? buildRelatedSections(data.graph.nodes, data.entity.id)
    : [];

  const modeLabel =
    policy.mode === "enforce"
      ? "Enforcing"
      : policy.mode === "observe"
        ? "Observe Only"
        : policy.mode || "Unknown";
  const modeTone =
    policy.mode === "enforce"
      ? "success"
      : policy.mode === "observe"
        ? "info"
        : ("neutral" as const);

  return (
    <Entity360Page
      header={{
        entityType: "Policy",
        entityName: policy.name,
        icon: FileKey,
        iconColor: "text-amber-600",
        status: { label: modeLabel, tone: modeTone },
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
          <PropertyRow
            label="Description"
            value={policy.description ?? "—"}
          />
          {policy.created_at && (
            <PropertyRow
              label="Created"
              value={new Date(policy.created_at).toLocaleDateString()}
            />
          )}
        </div>
      }
      relatedSections={relatedSections}
      data={data}
    />
  );
}

function PropertyRow({ label, value }: { label: string; value: string }) {
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

export default function PoliciesV2() {
  const [searchParams, setSearchParams] = useSearchParams();
  const selectedId = searchParams.get("id") ?? undefined;
  const { policies, loading } = usePolicies();

  const handleSelect = (id: string) => {
    setSearchParams({ id });
  };

  // Detail view
  const selectedPolicy = policies.find((p) => p.policy_id === selectedId);
  if (selectedPolicy) {
    return (
      <div className="space-y-4">
        <button
          type="button"
          onClick={() => setSearchParams({})}
          className="inline-flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground"
        >
          ← Back to all policies
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
          All governance policies. Click any policy to see which agents, tools,
          and resources it affects, along with decision history.
        </p>
      </div>

      <MasterDetailLayout
        items={policies}
        selectedId={selectedId}
        onSelect={handleSelect}
        idSelector={(p) => p.policy_id}
        loading={loading}
        renderCard={(p, selected) => {
          const modeTone =
            p.mode === "enforce"
              ? "ok" as const
              : p.mode === "observe"
                ? "info" as const
                : "idle" as const;
          return (
            <EntityCard
              title={p.name}
              subtitle={p.engine}
              icon={FileKey}
              status={modeTone}
              statusLabel={p.mode || "Unknown"}
              meta={[
                { label: "Engine", value: p.engine },
                ...(p.rules_count != null
                  ? [{ label: "Rules", value: String(p.rules_count) }]
                  : []),
              ]}
              selected={selected}
            />
          );
        }}
        renderDetail={(p) => (
          <div className="p-4">
            <button
              type="button"
              onClick={() => handleSelect(p.policy_id)}
              className="inline-flex items-center gap-2 rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90"
            >
              Open 360° View →
            </button>
            <p className="mt-2 text-sm text-muted-foreground">
              See full impact analysis: affected agents, governed tools, decision
              history, and cost.
            </p>
          </div>
        )}
      />
    </div>
  );
}
