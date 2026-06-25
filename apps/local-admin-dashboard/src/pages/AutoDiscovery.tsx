import { useConfirm } from "../components/ui/ConfirmDialog";
import { toast } from "sonner";
import { useState, useEffect } from "react";
import { Search, ShieldAlert, Info, Activity, Play } from "lucide-react";
import { useSearchParams, useNavigate } from "react-router-dom";
import { RegistryApi } from "../services/api";
import type {
  DiscoveredAgentCandidateV2,
  DiscoveryScanJob,
} from "../services/types";
import { MasterDetailLayout } from "../components/master-detail/MasterDetailLayout";
import { EntityCard } from "../components/master-detail/EntityCard";
import { DetailPane } from "../components/master-detail/DetailPane";
import { EmptyState } from "../components/master-detail/EmptyState";
import type { UiStatus } from "../lib/status";

export function AutoDiscovery() {
  const { confirm } = useConfirm();

  const [candidates, setCandidates] = useState<DiscoveredAgentCandidateV2[]>(
    [],
  );
  const [loading, setLoading] = useState(true);
  const [scanJob, setScanJob] = useState<DiscoveryScanJob | null>(null);
  const [params, setParams] = useSearchParams();
  const navigate = useNavigate();
  const selectedId = params.get("selected") ?? undefined;

  const fetchCandidates = () => {
    setLoading(true);
    RegistryApi.listDiscoveryCandidates()
      .then(setCandidates)
      .catch(console.error)
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    fetchCandidates();
  }, []);

  useEffect(() => {
    let interval: ReturnType<typeof setInterval>;
    if (
      scanJob &&
      (scanJob.status === "queued" || scanJob.status === "running")
    ) {
      interval = setInterval(async () => {
        try {
          const status = await RegistryApi.getDiscoveryScanStatus(
            scanJob.scan_id,
          );
          setScanJob(status);
          if (
            status.status === "completed" ||
            status.status === "partial" ||
            status.status === "failed"
          ) {
            fetchCandidates();
            clearInterval(interval);
          }
        } catch (e) {
          console.error(e);
        }
      }, 2000);
    }
    return () => clearInterval(interval);
  }, [scanJob]);

  const select = (id: string) =>
    setParams((p) => {
      p.set("selected", id);
      return p;
    });

  const deleteCandidate = async (id: string) => {
    if (
      !(await confirm({
        title: "Confirm Action",
        description: "Are you sure you want to delete this candidate?",
        danger: true,
      }))
    )
      return;
    try {
      await RegistryApi.deleteDiscoveryCandidate(id);
      if (selectedId === id) {
        setParams((p) => {
          p.delete("selected");
          return p;
        });
      }
      fetchCandidates();
    } catch (e) {
      console.error("Failed to delete candidate:", e);
      toast.error("Failed to delete candidate");
    }
  };

  const triggerScan = async () => {
    try {
      const result = await RegistryApi.triggerDiscoveryScan({
        sources: [
          "process",
          "mcp_config",
          "local_model",
          "ide_extension",
          "cli_agent",
          "container",
          "browser_extension",
          "web_ai",
        ],
        privacy_mode: true,
      });
      setScanJob({
        scan_id: result.scan_id,
        tenant_id: "local",
        status: result.status as any,
        sources: [
          "process",
          "mcp_config",
          "local_model",
          "ide_extension",
          "cli_agent",
          "container",
          "browser_extension",
          "web_ai",
        ],
        candidates_found: 0,
      });
    } catch (e) {
      console.error(e);
    }
  };

  return (
    <div className="p-6 md:p-8 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">
            Auto Discovery
          </h2>
          <p className="text-sm text-muted-foreground">
            Find and manage local AI agents, MCP servers, and model endpoints.
          </p>
        </div>
        <button
          onClick={triggerScan}
          disabled={
            scanJob?.status === "queued" || scanJob?.status === "running"
          }
          className="flex items-center gap-2 rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 shadow-sm disabled:opacity-50"
        >
          {scanJob?.status === "queued" || scanJob?.status === "running" ? (
            <Activity className="h-4 w-4 animate-spin" />
          ) : (
            <Search className="h-4 w-4" />
          )}
          Deep Scan
        </button>
      </div>

      <MasterDetailLayout
        items={candidates}
        loading={loading}
        selectedId={selectedId}
        onSelect={select}
        idSelector={(c) => c.candidate_id}
        toolbar={
          <div className="flex items-center gap-2 mb-4">
            <input
              type="text"
              placeholder="Search candidates..."
              className="px-3 py-1.5 text-sm rounded-md border bg-background"
            />
          </div>
        }
        emptyState={
          <EmptyState
            icon={Search}
            title="No candidates discovered"
            description="Run a Deep Scan to automatically find AI components running locally."
            actionLabel="Deep Scan"
            onAction={triggerScan}
          />
        }
        renderCard={(c, selected) => {
          let status: UiStatus = "idle";
          if (c.status === "registered") status = "ok";
          else if (c.status === "pending_approval") status = "degraded";

          return (
            <EntityCard
              title={c.display_name}
              subtitle={c.inferred_agent_type}
              icon={ShieldAlert}
              status={status}
              statusLabel={c.status === "registered" ? "Registered" : "Pending"}
              meta={[
                {
                  label: "Confidence",
                  value: `${(c.confidence * 100).toFixed(0)}%`,
                },
              ]}
              selected={selected}
            />
          );
        }}
        renderDetail={(c) => {
          let status: UiStatus = "idle";
          if (c.status === "registered") status = "ok";
          else if (c.status === "pending_approval") status = "degraded";

          return (
            <DetailPane
              title={c.display_name}
              subtitle={c.inferred_agent_type}
              status={status}
              statusLabel={c.status === "registered" ? "Registered" : "Pending"}
              actions={[
                {
                  label: "Protect",
                  primary: true,
                  onClick: () => navigate(`/wizard?target=${c.candidate_id}`),
                },
                {
                  label: "Delete",
                  danger: true,
                  onClick: () => deleteCandidate(c.candidate_id),
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
                            Confidence
                          </span>
                          <span className="font-semibold">
                            {(c.confidence * 100).toFixed(0)}%
                          </span>
                        </div>
                        <div className="p-4 bg-muted/30 rounded-xl border">
                          <span className="text-muted-foreground block mb-1">
                            Risk Score
                          </span>
                          <span className="font-semibold text-amber-500">
                            {c.risk_score}
                          </span>
                        </div>
                        <div className="p-4 bg-muted/30 rounded-xl border">
                          <span className="text-muted-foreground block mb-1">
                            First Seen
                          </span>
                          <span>{new Date(c.first_seen).toLocaleString()}</span>
                        </div>
                        <div className="p-4 bg-muted/30 rounded-xl border">
                          <span className="text-muted-foreground block mb-1">
                            Last Seen
                          </span>
                          <span>{new Date(c.last_seen).toLocaleString()}</span>
                        </div>
                      </div>

                      {c.discovered_mcp_servers &&
                        c.discovered_mcp_servers.length > 0 && (
                          <div className="p-4 bg-muted/30 rounded-xl border">
                            <h4 className="text-sm font-semibold mb-2">
                              Discovered MCP Servers
                            </h4>
                            <ul className="text-sm space-y-1.5 text-muted-foreground">
                              {c.discovered_mcp_servers.map(
                                (mcp: any, i: number) => (
                                  <li
                                    key={i}
                                    className="flex items-center gap-2"
                                  >
                                    <Play className="h-3 w-3 text-primary" />
                                    <span className="text-foreground/80">
                                      {mcp.server_name} ({mcp.transport})
                                    </span>
                                  </li>
                                ),
                              )}
                            </ul>
                          </div>
                        )}
                    </div>
                  ),
                },
                {
                  id: "advanced",
                  label: "Advanced Evidence",
                  content: (
                    <div className="space-y-4">
                      <p className="text-sm text-muted-foreground mb-2">
                        Raw evidence details and system telemetry used to
                        identify this candidate.
                      </p>
                      <pre className="text-[10px] font-mono bg-muted/50 p-4 rounded-lg overflow-x-auto border">
                        {JSON.stringify(c.evidence, null, 2)}
                      </pre>
                      <h4 className="font-medium mt-4 mb-2 flex items-center gap-2 text-sm">
                        <Info className="h-4 w-4" /> Full JSON payload
                      </h4>
                      <pre className="text-[10px] font-mono bg-muted/50 p-4 rounded-lg overflow-x-auto border">
                        {JSON.stringify(c, null, 2)}
                      </pre>
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
