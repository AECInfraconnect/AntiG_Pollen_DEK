import { type ReactNode, useEffect, useState } from "react";
import { useSearchParams } from "react-router-dom";
import {
  Bot,
  BookOpen,
  CheckCircle2,
  Clock3,
  Database,
  FileKey,
  FolderTree,
  Gauge,
  Shield,
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
import {
  assessExpectedCapabilities,
  findReferenceIntel,
  type ReferenceIntel,
} from "../lib/entityReferenceIntel";
import {
  ReferenceIntelInline,
  ReferenceIntelMark,
} from "../components/reference/ReferenceIntelMark";

interface ToolItem {
  tool_id: string;
  name: string;
  description?: string;
  type: string;
  status: string;
  agent_id?: string;
  last_used?: string;
  call_count?: number;
  vendor?: string;
  source?: string;
}

interface ResourceItem {
  resource_id: string;
  name: string;
  description?: string;
  type: string;
  status: string;
  path?: string;
  host?: string;
  uri?: string;
  last_accessed?: string;
  source?: string;
  sensitivity?: string;
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
      .then(([toolPayload, resourcePayload]) => {
        setTools(
          Array.isArray(toolPayload)
            ? toolPayload
            : toolPayload?.items ?? toolPayload?.tools ?? [],
        );
        setResources(
          Array.isArray(resourcePayload)
            ? resourcePayload
            : resourcePayload?.items ?? resourcePayload?.resources ?? [],
        );
      })
      .finally(() => setLoading(false));
  }, []);

  return { tools, resources, loading };
}

function buildRelatedSections(nodes: GraphNode[], centerId: string): RelatedSection[] {
  const related = nodes.filter((node) => node.id !== centerId);
  const agents = related.filter((node) => node.type === "agent");
  const policies = related.filter((node) => node.type === "policy");
  const others = related.filter(
    (node) => !["agent", "policy"].includes(node.type),
  );

  const sections: RelatedSection[] = [
    {
      title: "Agents Using This",
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
      title: "Governing Policies",
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
                tone: policy.status === "enforcing" ? "success" : "info",
              }
            : undefined,
        }),
      ),
      viewAllHref: "/policies",
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

function formatDateTime(value?: string) {
  if (!value) return "Not recorded";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleString();
}

function emptyDash(value?: string | number | null) {
  if (value === undefined || value === null || value === "") return "-";
  return value;
}

function referencesForTool(tool: ToolItem) {
  return findReferenceIntel({
    entityKind: "tool",
    name: tool.name,
    vendor: tool.vendor,
    type: tool.type,
  });
}

function referencesForResource(resource: ResourceItem) {
  return findReferenceIntel({
    entityKind: "resource",
    name: resource.name,
    type: resource.type,
    uri: resource.uri,
    host: resource.host,
    path: resource.path,
  });
}

function referenceSectionFor(input: {
  entityKind: "tool" | "resource";
  name?: string;
  vendor?: string;
  type?: string;
  uri?: string;
  host?: string;
  path?: string;
}): DetailSection | null {
  const references = findReferenceIntel(input);
  if (!references.length) return null;

  return {
    title: "Reference Intel",
    description:
      "Well-known external context matched from observed names, domains, vendors, or paths. This enriches the record but is not enforcement evidence.",
    icon: BookOpen,
    fields: references.map((reference) => ({
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
  };
}

function capabilityChecklistSection(
  references: ReferenceIntel[],
  observedCapabilities: string[],
): DetailSection | null {
  const capabilities = assessExpectedCapabilities(references, observedCapabilities);
  if (!capabilities.length) return null;

  return {
    title: "Known Capability Checklist",
    description:
      "Standard capabilities expected for matched well-known entities. Green means local evidence detected a matching capability.",
    icon: CheckCircle2,
    fields: capabilities.map((capability) => ({
      label: capability.label,
      value: capability.detected ? "Detected" : "Not observed yet",
      status: capability.detected ? "ok" : "unknown",
      source: `definition:${capability.referenceTitle}`,
      note: capability.detected
        ? "Matched against observed local resource or tool evidence."
        : "Expected by reference intel, but not yet confirmed by local evidence.",
    })),
  };
}

function toolDetailSections(
  tool: ToolItem,
  data: Entity360Response | null | undefined,
): DetailSection[] {
  const graphNodes = data?.graph.nodes ?? [];
  const agents = graphNodes.filter((node) => node.type === "agent").length;
  const policies = graphNodes.filter((node) => node.type === "policy").length;
  const referenceMatches = referencesForTool(tool);
  const references = referenceSectionFor({
    entityKind: "tool",
    name: tool.name,
    vendor: tool.vendor,
    type: tool.type,
  });
  const capabilityChecklist = capabilityChecklistSection(referenceMatches, [
    tool.name,
    tool.type,
    tool.description ?? "",
    tool.source ?? "",
  ]);

  const sections: DetailSection[] = [
    {
      title: "Current Status",
      description: "Tool registration, runtime activity, and decision readiness.",
      icon: Gauge,
      fields: [
        {
          label: "Status",
          value: tool.status || "Registered",
          status: tool.status === "active" ? "ok" : "info",
          source: "tool registry",
        },
        {
          label: "Type",
          value: tool.type,
          source: "tool registry",
        },
        {
          label: "Calls",
          value: tool.call_count ?? 0,
          source: "tool invocation telemetry",
        },
        {
          label: "Last Used",
          value: formatDateTime(tool.last_used),
          source: "tool invocation telemetry",
        },
      ],
    },
    {
      title: "Ownership & Control",
      description: "Who owns the tool and what policy relationships exist.",
      icon: Shield,
      fields: [
        {
          label: "Owner Agent",
          value: tool.agent_id ?? "Not linked",
          status: tool.agent_id ? "ok" : "warning",
          source: "registry.owner_agent_id",
        },
        {
          label: "Observed Agents",
          value: agents,
          source: "entity graph",
        },
        {
          label: "Policies",
          value: policies,
          status: policies ? "ok" : "warning",
          source: "entity graph",
          note: policies
            ? "At least one policy relationship is present."
            : "No governing policy is linked yet.",
        },
      ],
    },
    {
      title: "Data Sources & History",
      description: "Where values came from and whether they are observed or declared.",
      icon: Clock3,
      fields: [
        {
          label: "Primary Source",
          value: tool.source ?? "registry/tools endpoint",
          source: "dashboard API",
        },
        {
          label: "Graph Snapshot",
          value: data ? formatDateTime(data.generated_at) : "Not loaded",
          source: "entity-360 endpoint",
        },
        {
          label: "Description",
          value: tool.description ?? "No description",
          source: "tool registry",
        },
      ],
    },
  ];

  if (references) sections.push(references);
  if (capabilityChecklist) sections.push(capabilityChecklist);
  return sections;
}

function resourceDetailSections(
  resource: ResourceItem,
  data: Entity360Response | null | undefined,
): DetailSection[] {
  const graphNodes = data?.graph.nodes ?? [];
  const agents = graphNodes.filter((node) => node.type === "agent").length;
  const policies = graphNodes.filter((node) => node.type === "policy").length;
  const referenceMatches = referencesForResource(resource);
  const references = referenceSectionFor({
    entityKind: "resource",
    name: resource.name,
    type: resource.type,
    uri: resource.uri,
    host: resource.host,
    path: resource.path,
  });
  const capabilityChecklist = capabilityChecklistSection(referenceMatches, [
    resource.name,
    resource.type,
    resource.description ?? "",
    resource.uri ?? "",
    resource.host ?? "",
    resource.path ?? "",
    resource.source ?? "",
  ]);

  const sections: DetailSection[] = [
    {
      title: "Current Status",
      description:
        "Resource classification, location, and last observed access metadata.",
      icon: Database,
      fields: [
        {
          label: "Status",
          value: resource.status || "Registered",
          status: resource.status === "active" ? "ok" : "info",
          source: "resource registry",
        },
        {
          label: "Type",
          value: resource.type,
          source: "resource registry",
        },
        {
          label: "Sensitivity",
          value: resource.sensitivity ?? "Unknown",
          status: resource.sensitivity ? "warning" : "unknown",
          source: "classification metadata",
        },
        {
          label: "Last Accessed",
          value: formatDateTime(resource.last_accessed),
          source: "resource access telemetry",
        },
      ],
    },
    {
      title: "Location & Access",
      description:
        "The most specific local or cloud location Pollek currently knows.",
      icon: FolderTree,
      fields: [
        {
          label: "URI",
          value: emptyDash(resource.uri),
          source: "resource inventory",
        },
        {
          label: "Host",
          value: emptyDash(resource.host),
          source: "network/browser/resource observer",
        },
        {
          label: "Path",
          value: emptyDash(resource.path),
          source: "filesystem/database/cloud observer",
          note:
            "For folders, files, and databases this should become folder, file, table, or query-level evidence when the OS/source supports it.",
        },
      ],
    },
    {
      title: "Relationships & Policy",
      description:
        "Agents and policies currently connected to this resource through observation or registry links.",
      icon: Shield,
      fields: [
        {
          label: "Observed Agents",
          value: agents,
          status: agents ? "ok" : "warning",
          source: "entity graph",
        },
        {
          label: "Policies",
          value: policies,
          status: policies ? "ok" : "warning",
          source: "entity graph",
        },
        {
          label: "Graph Snapshot",
          value: data ? formatDateTime(data.generated_at) : "Not loaded",
          source: "entity-360 endpoint",
        },
      ],
    },
  ];

  if (references) sections.push(references);
  if (capabilityChecklist) sections.push(capabilityChecklist);
  return sections;
}

function ToolDetailView({ tool }: { tool: ToolItem }) {
  const { data } = useEntity360("tool", tool.tool_id);
  const primaryReference = referencesForTool(tool)[0];
  const relatedSections = data
    ? buildRelatedSections(data.graph.nodes, data.entity.id)
    : [];

  return (
    <Entity360Page
      header={{
        entityType: "Tool",
        entityName: tool.name,
        icon: Wrench,
        helpTopicId: "entity.tool",
        visual: primaryReference ? (
          <ReferenceIntelMark reference={primaryReference} />
        ) : undefined,
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
          {primaryReference && (
            <PropertyRow
              label="Known Entity"
              value={<ReferenceIntelInline reference={primaryReference} />}
            />
          )}
          <PropertyRow label="Tool ID" value={tool.tool_id} />
          <PropertyRow label="Type" value={tool.type} />
          <PropertyRow label="Status" value={tool.status} />
          <PropertyRow label="Description" value={tool.description ?? "-"} />
          <PropertyRow label="Owner Agent" value={tool.agent_id ?? "-"} />
        </div>
      }
      relatedSections={relatedSections}
      data={data}
      detailSections={toolDetailSections(tool, data)}
    />
  );
}

function ResourceDetailView({ resource }: { resource: ResourceItem }) {
  const { data } = useEntity360("resource", resource.resource_id);
  const primaryReference = referencesForResource(resource)[0];
  const relatedSections = data
    ? buildRelatedSections(data.graph.nodes, data.entity.id)
    : [];

  return (
    <Entity360Page
      header={{
        entityType: "Data Resource",
        entityName: resource.name,
        icon: Database,
        helpTopicId: "entity.resource",
        visual: primaryReference ? (
          <ReferenceIntelMark reference={primaryReference} />
        ) : undefined,
        iconColor: "text-purple-600",
        status: {
          label: resource.status || "Registered",
          tone: resource.status === "active" ? "success" : "neutral",
        },
        subtitle: resource.description ?? resource.path ?? resource.host ?? "No location available",
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
          {primaryReference && (
            <PropertyRow
              label="Known Entity"
              value={<ReferenceIntelInline reference={primaryReference} />}
            />
          )}
          <PropertyRow label="Resource ID" value={resource.resource_id} />
          <PropertyRow label="Type" value={resource.type} />
          <PropertyRow label="Status" value={resource.status} />
          <PropertyRow label="URI" value={resource.uri ?? "-"} />
          <PropertyRow label="Path" value={resource.path ?? "-"} />
          <PropertyRow label="Host" value={resource.host ?? "-"} />
        </div>
      }
      relatedSections={relatedSections}
      data={data}
      detailSections={resourceDetailSections(resource, data)}
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

export default function ToolsResourcesV2() {
  const [searchParams, setSearchParams] = useSearchParams();
  const [activeTab, setActiveTab] = useState<"tools" | "resources">(
    (searchParams.get("tab") as "tools" | "resources") ?? "tools",
  );
  const selectedId = searchParams.get("id") ?? undefined;
  const { tools, resources, loading } = useToolsAndResources();

  const updateTab = (tab: "tools" | "resources") => {
    setActiveTab(tab);
    setSearchParams({ tab });
  };

  const handleSelect = (id: string) => {
    setSearchParams({ tab: activeTab, id });
  };

  if (activeTab === "tools" && selectedId) {
    const tool = tools.find((item) => item.tool_id === selectedId);
    if (tool) {
      return (
        <div className="space-y-4">
          <button
            type="button"
            onClick={() => setSearchParams({ tab: "tools" })}
            className="inline-flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground"
          >
            Back to Tools & Resources
          </button>
          <ToolDetailView tool={tool} />
        </div>
      );
    }
  }

  if (activeTab === "resources" && selectedId) {
    const resource = resources.find((item) => item.resource_id === selectedId);
    if (resource) {
      return (
        <div className="space-y-4">
          <button
            type="button"
            onClick={() => setSearchParams({ tab: "resources" })}
            className="inline-flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground"
          >
            Back to Tools & Resources
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
          Local tools, cloud APIs, files, folders, databases, and data sources
          visible to AI agents. Open a record to inspect evidence, policies, and
          reference intel.
        </p>
      </div>

      <div className="flex w-fit gap-1 rounded-lg border bg-muted/30 p-1">
        <button
          type="button"
          onClick={() => updateTab("tools")}
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
          onClick={() => updateTab("resources")}
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

      {activeTab === "tools" && (
        <MasterDetailLayout
          items={tools}
          selectedId={selectedId}
          onSelect={handleSelect}
          idSelector={(tool) => tool.tool_id}
          loading={loading}
          renderCard={(tool, selected) => {
            const primaryReference = referencesForTool(tool)[0];
            return (
              <EntityCard
                title={tool.name}
                subtitle={tool.description ?? tool.type}
                summary={`Source: ${tool.source ?? "registry"} - Last used: ${formatDateTime(
                  tool.last_used,
                )}`}
                icon={Wrench}
                visual={
                  primaryReference ? (
                    <ReferenceIntelMark reference={primaryReference} />
                  ) : undefined
                }
                status={tool.status === "active" ? "ok" : "info"}
                statusLabel={tool.status || "Registered"}
                meta={[
                  { label: "Type", value: tool.type },
                  ...(primaryReference
                    ? [{ label: "Known", value: primaryReference.title }]
                    : []),
                  ...(tool.call_count != null
                    ? [{ label: "Calls", value: String(tool.call_count) }]
                    : []),
                ]}
                selected={selected}
              />
            );
          }}
          renderDetail={(tool) => (
            <div className="p-4">
              <button
                type="button"
                onClick={() => handleSelect(tool.tool_id)}
                className="inline-flex items-center gap-2 rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90"
              >
                Open Record View
              </button>
            </div>
          )}
        />
      )}

      {activeTab === "resources" && (
        <MasterDetailLayout
          items={resources}
          selectedId={selectedId}
          onSelect={handleSelect}
          idSelector={(resource) => resource.resource_id}
          loading={loading}
          renderCard={(resource, selected) => {
            const primaryReference = referencesForResource(resource)[0];
            return (
              <EntityCard
                title={resource.name}
                subtitle={resource.description ?? resource.type}
                summary={`Location: ${
                  resource.path ?? resource.uri ?? resource.host ?? "not captured"
                }`}
                icon={Database}
                visual={
                  primaryReference ? (
                    <ReferenceIntelMark reference={primaryReference} />
                  ) : undefined
                }
                status={resource.status === "active" ? "ok" : "info"}
                statusLabel={resource.status || "Registered"}
                meta={[
                  { label: "Type", value: resource.type },
                  ...(primaryReference
                    ? [{ label: "Known", value: primaryReference.title }]
                    : []),
                  ...(resource.host
                    ? [{ label: "Host", value: resource.host }]
                    : []),
                  ...(resource.last_accessed
                    ? [
                        {
                          label: "Last",
                          value: formatDateTime(resource.last_accessed),
                        },
                      ]
                    : []),
                ]}
                selected={selected}
              />
            );
          }}
          renderDetail={(resource) => (
            <div className="p-4">
              <button
                type="button"
                onClick={() => handleSelect(resource.resource_id)}
                className="inline-flex items-center gap-2 rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90"
              >
                Open Record View
              </button>
            </div>
          )}
        />
      )}
    </div>
  );
}
