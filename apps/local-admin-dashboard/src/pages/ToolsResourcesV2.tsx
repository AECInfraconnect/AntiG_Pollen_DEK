/**
 * ToolsResourcesV2 — Combined Tools & Resources page with Entity 360° view
 *
 * Combines Tools and Data Resources into a single tabbed page.
 * Each item opens a full 360° detail view showing related agents, policies, and activity.
 */
import { useState } from "react";
import { useSearchParams } from "react-router-dom";
import {
  Bot,
  Database,
  FileKey,
  FolderTree,
  Shield,
  Wrench,
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
import { useEffect } from "react";

interface ToolItem {
  tool_id: string;
  name: string;
  description?: string;
  type: string;
  status: string;
  agent_id?: string;
  last_used?: string;
  call_count?: number;
}

interface ResourceItem {
  resource_id: string;
  name: string;
  description?: string;
  type: string;
  status: string;
  path?: string;
  host?: string;
  last_accessed?: string;
}

function useToolsAndResources() {
  const [tools, setTools] = useState<ToolItem[]>([]);
  const [resources, setResources] = useState<ResourceItem[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    Promise.all([
      defaultClient.fetchApi("/tools").catch(() => []),
      defaultClient.fetchApi("/resources").catch(() => []),
    ])
      .then(([t, r]) => {
        setTools(Array.isArray(t) ? t : t?.items ?? []);
        setResources(Array.isArray(r) ? r : r?.items ?? []);
      })
      .finally(() => setLoading(false));
  }, []);

  return { tools, resources, loading };
}

function buildRelatedSections(nodes: GraphNode[], centerId: string): RelatedSection[] {
  const related = nodes.filter((n) => n.id !== centerId);
  const agents = related.filter((n) => n.type === "agent");
  const policies = related.filter((n) => n.type === "policy");
  const others = related.filter(
    (n) => !["agent", "policy"].includes(n.type),
  );

  const sections: RelatedSection[] = [];

  sections.push({
    title: "Agents Using This",
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
    title: "Governing Policies",
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
              tone: p.status === "enforcing" ? "success" : "info",
            }
          : undefined,
      }),
    ),
    viewAllHref: "/policies",
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

function ToolDetailView({ tool }: { tool: ToolItem }) {
  const { data } = useEntity360("tool", tool.tool_id);
  const relatedSections = data
    ? buildRelatedSections(data.graph.nodes, data.entity.id)
    : [];

  return (
    <Entity360Page
      header={{
        entityType: "Tool",
        entityName: tool.name,
        icon: Wrench,
        iconColor: "text-blue-600",
        status: {
          label: tool.status || "Registered",
          tone: tool.status === "active" ? "success" : "neutral",
        },
        subtitle: tool.description ?? "No description available",
        meta: [
          { label: "Type", value: tool.type },
          ...(tool.call_count != null
            ? [{ label: "Calls", value: String(tool.call_count) }]
            : []),
          ...(tool.last_used
            ? [
                {
                  label: "Last Used",
                  value: new Date(tool.last_used).toLocaleString(),
                },
              ]
            : []),
        ],
      }}
      aboutSection={
        <div className="space-y-3">
          <PropertyRow label="Tool ID" value={tool.tool_id} />
          <PropertyRow label="Type" value={tool.type} />
          <PropertyRow label="Status" value={tool.status} />
          <PropertyRow
            label="Description"
            value={tool.description ?? "—"}
          />
          {tool.agent_id && (
            <PropertyRow label="Owner Agent" value={tool.agent_id} />
          )}
        </div>
      }
      relatedSections={relatedSections}
      data={data}
    />
  );
}

function ResourceDetailView({ resource }: { resource: ResourceItem }) {
  const { data } = useEntity360("resource", resource.resource_id);
  const relatedSections = data
    ? buildRelatedSections(data.graph.nodes, data.entity.id)
    : [];

  return (
    <Entity360Page
      header={{
        entityType: "Data Resource",
        entityName: resource.name,
        icon: Database,
        iconColor: "text-purple-600",
        status: {
          label: resource.status || "Registered",
          tone: resource.status === "active" ? "success" : "neutral",
        },
        subtitle: resource.description ?? "No description available",
        meta: [
          { label: "Type", value: resource.type },
          ...(resource.host ? [{ label: "Host", value: resource.host }] : []),
          ...(resource.last_accessed
            ? [
                {
                  label: "Last Accessed",
                  value: new Date(resource.last_accessed).toLocaleString(),
                },
              ]
            : []),
        ],
      }}
      aboutSection={
        <div className="space-y-3">
          <PropertyRow label="Resource ID" value={resource.resource_id} />
          <PropertyRow label="Type" value={resource.type} />
          <PropertyRow label="Status" value={resource.status} />
          <PropertyRow label="Path" value={resource.path ?? "—"} />
          <PropertyRow label="Host" value={resource.host ?? "—"} />
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

export default function ToolsResourcesV2() {
  const [searchParams, setSearchParams] = useSearchParams();
  const [activeTab, setActiveTab] = useState<"tools" | "resources">(
    (searchParams.get("tab") as "tools" | "resources") ?? "tools",
  );
  const selectedId = searchParams.get("id") ?? undefined;
  const { tools, resources, loading } = useToolsAndResources();

  const handleSelect = (id: string) => {
    setSearchParams({ tab: activeTab, id });
  };

  // Detail view for selected tool
  if (activeTab === "tools" && selectedId) {
    const tool = tools.find((t) => t.tool_id === selectedId);
    if (tool) {
      return (
        <div className="space-y-4">
          <button
            type="button"
            onClick={() => setSearchParams({ tab: "tools" })}
            className="inline-flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground"
          >
            ← Back to Tools & Resources
          </button>
          <ToolDetailView tool={tool} />
        </div>
      );
    }
  }

  // Detail view for selected resource
  if (activeTab === "resources" && selectedId) {
    const resource = resources.find((r) => r.resource_id === selectedId);
    if (resource) {
      return (
        <div className="space-y-4">
          <button
            type="button"
            onClick={() => setSearchParams({ tab: "resources" })}
            className="inline-flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground"
          >
            ← Back to Tools & Resources
          </button>
          <ResourceDetailView resource={resource} />
        </div>
      );
    }
  }

  return (
    <div className="space-y-4">
      <div>
        <h1 className="text-2xl font-bold">Tools & Resources</h1>
        <p className="text-sm text-muted-foreground">
          All tools and data resources accessible by AI agents. Click any item
          to see its 360° view with related agents, policies, and activity.
        </p>
      </div>

      {/* Tab switcher */}
      <div className="flex gap-1 rounded-lg border bg-muted/30 p-1 w-fit">
        <button
          type="button"
          onClick={() => {
            setActiveTab("tools");
            setSearchParams({ tab: "tools" });
          }}
          className={`flex items-center gap-1.5 rounded-md px-3 py-1.5 text-sm font-medium transition-colors ${
            activeTab === "tools"
              ? "bg-background text-foreground shadow-sm"
              : "text-muted-foreground hover:text-foreground"
          }`}
        >
          <Wrench className="h-3.5 w-3.5" />
          Tools ({tools.length})
        </button>
        <button
          type="button"
          onClick={() => {
            setActiveTab("resources");
            setSearchParams({ tab: "resources" });
          }}
          className={`flex items-center gap-1.5 rounded-md px-3 py-1.5 text-sm font-medium transition-colors ${
            activeTab === "resources"
              ? "bg-background text-foreground shadow-sm"
              : "text-muted-foreground hover:text-foreground"
          }`}
        >
          <Database className="h-3.5 w-3.5" />
          Resources ({resources.length})
        </button>
      </div>

      {/* Tools list */}
      {activeTab === "tools" && (
        <MasterDetailLayout
          items={tools}
          selectedId={selectedId}
          onSelect={handleSelect}
          idSelector={(t) => t.tool_id}
          loading={loading}
          renderCard={(t, selected) => (
            <EntityCard
              title={t.name}
              subtitle={t.type}
              icon={Wrench}
              status={t.status === "active" ? "ok" : "info"}
              statusLabel={t.status || "Registered"}
              meta={[
                ...(t.call_count != null
                  ? [{ label: "Calls", value: String(t.call_count) }]
                  : []),
              ]}
              selected={selected}
            />
          )}
          renderDetail={(t) => (
            <div className="p-4">
              <button
                type="button"
                onClick={() => handleSelect(t.tool_id)}
                className="inline-flex items-center gap-2 rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90"
              >
                Open 360° View →
              </button>
            </div>
          )}
        />
      )}

      {/* Resources list */}
      {activeTab === "resources" && (
        <MasterDetailLayout
          items={resources}
          selectedId={selectedId}
          onSelect={handleSelect}
          idSelector={(r) => r.resource_id}
          loading={loading}
          renderCard={(r, selected) => (
            <EntityCard
              title={r.name}
              subtitle={r.type}
              icon={Database}
              status={r.status === "active" ? "ok" : "info"}
              statusLabel={r.status || "Registered"}
              meta={[
                ...(r.host ? [{ label: "Host", value: r.host }] : []),
              ]}
              selected={selected}
            />
          )}
          renderDetail={(r) => (
            <div className="p-4">
              <button
                type="button"
                onClick={() => handleSelect(r.resource_id)}
                className="inline-flex items-center gap-2 rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90"
              >
                Open 360° View →
              </button>
            </div>
          )}
        />
      )}
    </div>
  );
}
