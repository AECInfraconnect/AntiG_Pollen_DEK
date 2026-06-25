import { useConfirm } from "../components/ui/ConfirmDialog";
import { toast } from "sonner";
import { useState, useEffect } from "react";
import { Database, Plus, FileKey, Activity, Info } from "lucide-react";
import { useSearchParams } from "react-router-dom";
import { RegistryApi } from "../services/api";
import type { Resource } from "../services/api";
import { MasterDetailLayout } from "../components/master-detail/MasterDetailLayout";
import { EntityCard } from "../components/master-detail/EntityCard";
import { DetailPane } from "../components/master-detail/DetailPane";
import { EmptyState } from "../components/master-detail/EmptyState";
import type { UiStatus } from "../lib/status";

export function Resources() {
  const { confirm } = useConfirm();

  const [resources, setResources] = useState<Resource[]>([]);
  const [loading, setLoading] = useState(true);
  const [params, setParams] = useSearchParams();
  const selectedId = params.get("selected") ?? undefined;

  const fetchResources = () => {
    setLoading(true);
    RegistryApi.listResources()
      .then(setResources)
      .catch(console.error)
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    fetchResources();
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
        idSelector={(x: any) => x.resource_id || x.id}
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
        renderCard={(r, selected) => {
          let status: UiStatus = "ok";
          let label = "Protected";
          if (r.classification === "restricted") {
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
        renderDetail={(r) => {
          let status: UiStatus = "ok";
          let label = "Protected";
          if (r.classification === "restricted") {
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
                  label: "Apply Policy",
                  primary: true,
                  onClick: () => {
                    /* Open Wizard */
                  },
                },
                {
                  label: "Delete",
                  danger: true,
                  onClick: () => deleteResource(r.resource_id),
                },
              ]}
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
                          <span className="capitalize">{r.classification}</span>
                        </div>
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
                      <FileKey className="h-8 w-8 mb-2 opacity-50" />
                      <p className="text-sm">
                        No specific access policies bound.
                      </p>
                    </div>
                  ),
                },
                {
                  id: "activity",
                  label: "Activity",
                  content: (
                    <div className="flex flex-col items-center justify-center p-8 text-center border border-dashed rounded-lg text-muted-foreground">
                      <Activity className="h-8 w-8 mb-2 opacity-50" />
                      <p className="text-sm">No activity recorded yet.</p>
                    </div>
                  ),
                },
              ]}
            />
          );
        }}
      />
    </div>
  );
}
