import { toast } from "sonner";
import { useState, useEffect } from "react";
import { Wrench, Info, FileKey } from "lucide-react";
import { useSearchParams } from "react-router-dom";
import { RegistryApi } from "../services/api";
import type { Tool } from "../services/api";
import { MasterDetailLayout } from "../components/master-detail/MasterDetailLayout";
import { EntityCard } from "../components/master-detail/EntityCard";
import { DetailPane } from "../components/master-detail/DetailPane";
import { EmptyState } from "../components/master-detail/EmptyState";
import { RegisterControlBar } from "../components/RegisterControlBar";
import type { UiStatus } from "../lib/status";
import { useConfirm } from "../components/ui/ConfirmDialog";

export function Tools({ hideHeader = false }: { hideHeader?: boolean }) {
  const [tools, setTools] = useState<Tool[]>([]);
  const [loading, setLoading] = useState(true);
  const [params, setParams] = useSearchParams();
  const selectedId = params.get("selected") ?? undefined;
  const { confirm } = useConfirm();

  const fetchTools = () => {
    setLoading(true);
    RegistryApi.listTools()
      .then(setTools)
      .catch(console.error)
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    fetchTools();
  }, []);

  const select = (id: string) =>
    setParams((p) => {
      p.set("selected", id);
      return p;
    });

  const deleteTool = async (id: string) => {
    if (
      !(await confirm({
        title: "Delete Tool",
        description: "Are you sure you want to delete this tool?",
        danger: true,
      }))
    )
      return;
    try {
      await RegistryApi.deleteTool(id);
      if (selectedId === id) {
        setParams((p) => {
          p.delete("selected");
          return p;
        });
      }
      toast.success("Tool deleted successfully");
      fetchTools();
    } catch (e) {
      console.error("Failed to delete tool:", e);
      toast.error("Failed to delete tool");
    }
  };

  return (
    <div className={hideHeader ? "space-y-6" : "p-6 md:p-8 space-y-6"}>
      {!hideHeader && (
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-semibold tracking-tight">Tools</h2>
            <p className="text-sm text-muted-foreground">
              Manage function-calling definitions available to AI Agents.
            </p>
          </div>
        </div>
      )}

      <MasterDetailLayout
        items={tools}
        loading={loading}
        selectedId={selectedId}
        onSelect={select}
        idSelector={(t: any) => t.tool_id}
        toolbar={
          <div className="flex items-center gap-2 mb-4">
            <input
              type="text"
              placeholder="Search tools..."
              className="px-3 py-1.5 text-sm rounded-md border bg-background"
            />
          </div>
        }
        emptyState={
          <EmptyState
            icon={Wrench}
            title="No tools found"
            description="Register JSON schemas for tools that your agents can invoke."
          />
        }
        renderCard={(t, selected) => {
          let status: UiStatus = "ok";
          if (t.risk_level === "high" || t.risk_level === "critical")
            status = "failed";
          else if (t.risk_level === "medium") status = "degraded";

          return (
            <EntityCard
              title={t.name}
              subtitle={t.description || "No description"}
              icon={Wrench}
              status={status}
              statusLabel={
                t.risk_level ? t.risk_level.toUpperCase() : "UNKNOWN"
              }
              meta={[{ label: "Data Access", value: t.data_access_level }]}
              selected={selected}
            />
          );
        }}
        renderDetail={(t) => {
          let status: UiStatus = "ok";
          if (t.risk_level === "high" || t.risk_level === "critical")
            status = "failed";
          else if (t.risk_level === "medium") status = "degraded";

          return (
            <DetailPane
              title={t.name}
              subtitle={t.description}
              status={status}
              statusLabel={
                t.risk_level ? t.risk_level.toUpperCase() : "UNKNOWN"
              }
              actions={[
                {
                  label: "Delete",
                  danger: true,
                  onClick: () => deleteTool(t.tool_id),
                },
              ]}
              tabs={[
                {
                  id: "overview",
                  label: "Overview",
                  content: (
                    <div className="space-y-6">
                      <div className="grid grid-cols-2 gap-4 text-sm">
                        <div className="p-4 bg-muted/30 rounded-xl border">
                          <span className="text-muted-foreground block mb-1">
                            Data Access
                          </span>
                          <span className="capitalize">
                            {t.data_access_level}
                          </span>
                        </div>
                        <div className="p-4 bg-muted/30 rounded-xl border">
                          <span className="text-muted-foreground block mb-1">
                            Side Effects
                          </span>
                          <span className="capitalize">
                            {t.side_effect_level}
                          </span>
                        </div>
                      </div>

                      <div className="p-4 bg-muted/30 rounded-xl border">
                        <h4 className="text-sm font-semibold mb-2">
                          Registration Status
                        </h4>
                        <RegisterControlBar
                          agentId={t.tool_id}
                          tenantId="local"
                          onSuccess={() => fetchTools()}
                        />
                      </div>
                    </div>
                  ),
                },
                {
                  id: "schema",
                  label: "Schema",
                  content: (
                    <div>
                      <h4 className="font-medium mb-2 flex items-center gap-2 text-sm">
                        <Info className="h-4 w-4" /> JSON Schema
                      </h4>
                      <pre className="text-[10px] font-mono bg-muted/50 p-4 rounded-lg overflow-x-auto border">
                        {JSON.stringify((t as any).schema, null, 2)}
                      </pre>
                    </div>
                  )
                },
                {
                  id: "policies",
                  label: "Policies",
                  content: (
                    <div className="flex flex-col items-center justify-center p-8 text-center border border-dashed rounded-lg text-muted-foreground">
                      <FileKey className="h-8 w-8 mb-2 opacity-50" />
                      <p className="text-sm">
                        No specific policies bound to this tool.
                      </p>
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
