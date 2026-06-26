import { useConfirm } from "../components/ui/ConfirmDialog";
import { toast } from "sonner";
import { useState, useEffect } from "react";
import { Database, Plus, FileKey, Activity, Info } from "lucide-react";
import { useSearchParams } from "react-router-dom";
import { RegistryApi, TelemetryApi } from "../services/api";
import type { Resource, ObservedResource } from "../services/api";

export interface UnifiedResource {
  id: string;
  name: string;
  resource_type: string;
  uri: string;
  classification?: string;
  is_registered: boolean;
  is_observed: boolean;
  observed_details?: ObservedResource;
  registered_details?: Resource;
}
import { MasterDetailLayout } from "../components/master-detail/MasterDetailLayout";
import { EntityCard } from "../components/master-detail/EntityCard";
import { DetailPane } from "../components/master-detail/DetailPane";
import { EmptyState } from "../components/master-detail/EmptyState";
import type { UiStatus } from "../lib/status";

export function Resources() {
  const { confirm } = useConfirm();

  const [resources, setResources] = useState<UnifiedResource[]>([]);
  const [loading, setLoading] = useState(true);
  const [params, setParams] = useSearchParams();
  const selectedId = params.get("selected") ?? undefined;

  const fetchResources = async () => {
    setLoading(true);
    try {
      const [regRes, obsRes] = await Promise.all([
        RegistryApi.listResources(),
        TelemetryApi.listResourceInventory().catch(() => ({ items: [] as ObservedResource[] })),
      ]);

      const unifiedMap = new Map<string, UnifiedResource>();

      for (const r of regRes) {
        unifiedMap.set(r.uri, {
          id: (r as any).resource_id || (r as any).id || r.uri,
          name: r.name,
          resource_type: r.resource_type,
          uri: r.uri,
          classification: r.classification,
          is_registered: true,
          is_observed: false,
          registered_details: r,
        });
      }

      for (const o of (obsRes.items || [])) {
        const uri = o.target_redacted;
        if (unifiedMap.has(uri)) {
          const existing = unifiedMap.get(uri)!;
          existing.is_observed = true;
          existing.observed_details = o;
        } else {
          unifiedMap.set(uri, {
            id: o.resource_id || uri,
            name: uri.split('/').pop() || uri,
            resource_type: o.kind,
            uri: uri,
            classification: o.classification,
            is_registered: false,
            is_observed: true,
            observed_details: o,
          });
        }
      }

      setResources(Array.from(unifiedMap.values()));
    } catch (err) {
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchResources();

    const source = new EventSource("/v1/telemetry/observations/stream");
    source.onmessage = (e) => {
      try {
        const data = JSON.parse(e.data);
        if (data.event_type === "resource_access") {
          fetchResources();
        }
      } catch (err) {}
    };

    return () => source.close();
  }, []);

  const select = (id: string) =>
    setParams((p) => {
      p.set("selected", id);
      return p;
    });

  const deleteResource = async (id: string) => {
    if (
      !(await confirm({
        title: "Confirm",
        description:
          "Are you sure you want to delete this resource? Make sure no active policies depend on it.",
        danger: true,
      }))
    )
      return;
    try {
      await RegistryApi.deleteResource(id);
      if (selectedId === id) {
        setParams((p) => {
          p.delete("selected");
          return p;
        });
      }
      fetchResources();
    } catch (err) {
      console.error("Failed to delete resource:", err);
      toast.error("Failed to delete resource");
    }
  };

  return (
    <div className="p-6 md:p-8 space-y-6">
      <div className="mb-2 rounded-md bg-blue-50/50 border border-blue-200 p-4 shadow-sm">
        <div className="flex">
          <div className="flex-shrink-0">
            <Info className="h-5 w-5 text-blue-600" aria-hidden="true" />
          </div>
          <div className="ml-3">
            <h3 className="text-sm font-medium text-blue-800">
              POLLEK is observing simulated cloud egress for testing.
            </h3>
            <div className="mt-1 text-sm text-blue-700">
              <p>Real network enforcement is not enabled yet. This device can currently Observe cloud egress. Blocking requires OS network integration.</p>
            </div>
          </div>
        </div>
      </div>

      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">
            Data Resources
          </h2>
          <p className="text-sm text-muted-foreground">
            Manage data boundaries and classifications for registered resources.
          </p>
        </div>
        <button className="flex items-center gap-2 rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 shadow-sm">
          <Plus className="h-4 w-4" />
          Add Resource
        </button>
      </div>

      <MasterDetailLayout
        idSelector={(x: UnifiedResource) => x.id}
        items={resources}
        loading={loading}
        selectedId={selectedId}
        onSelect={select}
        toolbar={
          <div className="flex items-center gap-2 mb-4">
            <input
              type="text"
              placeholder="Search resources..."
              className="px-3 py-1.5 text-sm rounded-md border bg-background"
            />
          </div>
        }
        emptyState={
          <EmptyState
            icon={Database}
            title="No resources registered"
            description="Add data resources like databases or object stores to begin protecting them."
            actionLabel="Add Resource"
          />
        }
        renderCard={(r: UnifiedResource, selected) => {
          let status: UiStatus = "ok";
          let label = "Protected";
          if (!r.is_registered) {
            status = "idle";
            label = "Observed";
          } else if (r.classification === "restricted") {
            status = "failed";
            label = "Restricted";
          } else if (r.classification === "confidential") {
            status = "degraded";
            label = "Confidential";
          }

          return (
            <EntityCard
              title={r.name}
              subtitle={r.resource_type}
              icon={Database}
              status={status}
              statusLabel={label}
              meta={[{ label: "URI", value: r.uri }]}
              selected={selected}
            />
          );
        }}
        renderDetail={(r: UnifiedResource) => {
          let status: UiStatus = "ok";
          let label = "Protected";
          if (!r.is_registered) {
            status = "idle";
            label = "Observed";
          } else if (r.classification === "restricted") {
            status = "failed";
            label = "Restricted";
          } else if (r.classification === "confidential") {
            status = "degraded";
            label = "Confidential";
          }

          return (
            <DetailPane
              title={r.name}
              subtitle={r.resource_type}
              status={status}
              statusLabel={label}
              actions={[
                {
                  label: r.is_registered ? "Apply Policy" : "Protect",
                  primary: true,
                  onClick: () => {
                    /* Open Wizard or apply policy */
                  },
                },
                r.is_registered
                  ? {
                      label: "Delete",
                      danger: true,
                      onClick: () => deleteResource(r.id),
                    }
                  : undefined,
              ].filter(Boolean) as any}
              tabs={[
                {
                  id: "overview",
                  label: "Overview",
                  content: (
                    <div className="space-y-6">
                      <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
                        <div className="p-4 bg-muted/30 rounded-xl border">
                          <span className="text-muted-foreground block mb-1">
                            URI
                          </span>
                          <span className="font-mono text-xs break-all">
                            {r.uri}
                          </span>
                        </div>
                        <div className="p-4 bg-muted/30 rounded-xl border">
                          <span className="text-muted-foreground block mb-1">
                            Classification
                          </span>
                          <span className="capitalize">{r.classification || "Unclassified"}</span>
                        </div>
                        {r.is_observed && r.observed_details && (
                          <>
                            <div className="p-4 bg-muted/30 rounded-xl border">
                              <span className="text-muted-foreground block mb-1">
                                Last Access
                              </span>
                              <span className="text-xs">
                                {new Date(r.observed_details.last_access).toLocaleString()}
                              </span>
                            </div>
                            <div className="p-4 bg-muted/30 rounded-xl border">
                              <span className="text-muted-foreground block mb-1">
                                Access Count
                              </span>
                              <span className="text-xs">{r.observed_details.access_count}</span>
                            </div>
                          </>
                        )}
                      </div>

                      <div>
                        <h4 className="font-medium mb-2 flex items-center gap-2 text-sm">
                          <Info className="h-4 w-4" /> Raw JSON
                        </h4>
                        <pre className="text-[10px] font-mono bg-muted/50 p-4 rounded-lg overflow-x-auto border">
                          {JSON.stringify(r, null, 2)}
                        </pre>
                      </div>
                    </div>
                  ),
                },
                {
                  id: "access",
                  label: "Access Policies",
                  content: (
                    <div className="flex flex-col items-center justify-center p-8 text-center border border-dashed rounded-lg text-muted-foreground">
                      <FileKey className="h-8 w-8 mb-4 opacity-50" />
                      <p className="text-sm mb-4">
                        Protect this resource by assigning an access policy to specific agents.
                      </p>
                      <button 
                        className="px-4 py-2 bg-primary text-primary-foreground rounded-md text-sm hover:bg-primary/90"
                        onClick={() => {
                          toast.success("Policy draft created. Redirecting to policy editor...");
                          // Here we would navigate to the policy editor with pre-filled target
                        }}
                      >
                        Create Policy
                      </button>
                    </div>
                  ),
                },
                {
                  id: "activity",
                  label: "Activity",
                  content: <ResourceActivityTimeline resource={r} />,
                },
              ]}
            />
          );
        }}
      />
    </div>
  );
}

function ResourceActivityTimeline({ resource }: { resource: UnifiedResource }) {
  const [events, setEvents] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let mounted = true;
    setLoading(true);
    TelemetryApi.getObservations({ target: resource.uri }).then((res) => {
      if (mounted) {
        setEvents(res.items || []);
        setLoading(false);
      }
    });
    return () => { mounted = false; };
  }, [resource.uri]);

  if (loading) return <div className="p-8 text-center text-sm text-muted-foreground">Loading activity...</div>;
  if (events.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center p-8 text-center border border-dashed rounded-lg text-muted-foreground">
        <Activity className="h-8 w-8 mb-2 opacity-50" />
        <p className="text-sm">No activity recorded yet.</p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {events.map((ev, i) => (
        <div key={i} className="flex gap-4 p-4 border rounded-lg bg-card">
          <div className="mt-1"><Activity className="h-4 w-4 text-primary" /></div>
          <div>
            <p className="text-sm font-medium">Access by Agent: {ev.agent_id || "Unknown"}</p>
            <p className="text-xs text-muted-foreground mt-1">
              Mode: {ev.details?.mode || ev.mode || "read"} • {new Date(ev.observed_at || ev.timestamp).toLocaleString()}
            </p>
          </div>
        </div>
      ))}
    </div>
  );
}
