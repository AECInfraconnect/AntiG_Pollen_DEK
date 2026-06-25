import { useConfirm } from "../components/ui/ConfirmDialog";
import { toast } from "sonner";
import { useState, useEffect } from "react";
import {
  Search,
  ShieldAlert,
  Info,
  Activity,
  Play,
  CheckCircle,
} from "lucide-react";
import { useSearchParams } from "react-router-dom";
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
import { SimplePolicyWizard } from "../components/simple/SimplePolicyWizard";

export function AutoDiscovery() {
  const { confirm } = useConfirm();

  const [candidates, setCandidates] = useState<DiscoveredAgentCandidateV2[]>(
    [],
  );
  const [loading, setLoading] = useState(true);
  const [scanJob, setScanJob] = useState<DiscoveryScanJob | null>(null);
  const [params, setParams] = useSearchParams();
  const selectedId = params.get("selected") ?? undefined;
  const [protectTarget, setProtectTarget] = useState<string | null>(null);
  const [confirmingId, setConfirmingId] = useState<string | null>(null);
  const [confirmTarget, setConfirmTarget] = useState<DiscoveredAgentCandidateV2 | null>(null);
  const [editName, setEditName] = useState("");
  const [filter, setFilter] = useState<"all" | "pending" | "registered">("all");
  const [isScanning, setIsScanning] = useState(false);

  const clearHistory = async () => {
    if (
      !(await confirm({
        title: "Confirm",
        description: "Are you sure you want to drop all discovery history?",
        danger: true,
      }))
    )
      return;
    try {
      await RegistryApi.clearDiscoveryCandidates();
      fetchCandidates();
      toast.success("Discovery history cleared");
    } catch (e) {
      console.error(e);
      toast.error("Failed to clear history");
    }
  };

  const fetchCandidates = () => {
    setLoading(true);
    Promise.all([
      RegistryApi.listDiscoveryCandidates(),
      RegistryApi.listAgents()
    ])
      .then(([discovered, agents]) => {
        const agentIds = new Set(agents.map(a => a.agent_id));
        
        // Update discovered status if they exist in agents
        const mergedCandidates = discovered.map(c => {
           if (agentIds.has(c.candidate_id)) {
              return { ...c, status: "registered" };
           }
           return c;
        });

        setCandidates(mergedCandidates);
      })
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

  const capabilityTags = (candidate: DiscoveredAgentCandidateV2) => {
    const fromLabels = Object.keys(candidate.labels ?? {})
      .filter((key) => key.startsWith("capability:"))
      .map((key) => key.slice("capability:".length));
    return Array.from(
      new Set([...(candidate.capability_tags ?? []), ...fromLabels]),
    ).sort();
  };

  const openConfirmDialog = (candidate: DiscoveredAgentCandidateV2) => {
    setConfirmTarget(candidate);
    setEditName(candidate.display_name || "");
  };

  const submitConfirmAgent = async () => {
    if (!confirmTarget) return;
    setConfirmingId(confirmTarget.candidate_id);
    try {
      const response = await RegistryApi.registerDiscoveryCandidate(
        confirmTarget.candidate_id,
        {
          agent_name: editName,
        },
      );
      toast.success(`Confirmed ${response.agent_name ?? editName}`);
      setConfirmTarget(null);
      fetchCandidates();
    } catch (e) {
      console.error("Failed to confirm agent:", e);
      toast.error("Failed to confirm agent");
    } finally {
      setConfirmingId(null);
    }
  };

  const triggerScan = async () => {
    if (isScanning) return;
    setIsScanning(true);
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
    } finally {
      setTimeout(() => setIsScanning(false), 500);
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
        <div className="flex items-center gap-2">
          <button
            onClick={clearHistory}
            className="rounded-lg bg-red-500/10 border border-red-500/20 px-4 py-2 text-sm font-medium text-red-400 hover:bg-red-500/20 shadow-sm transition-colors"
          >
            Clear History
          </button>
          <button
            onClick={triggerScan}
            disabled={
              isScanning || scanJob?.status === "queued" || scanJob?.status === "running"
            }
            className="flex items-center gap-2 rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 shadow-sm disabled:opacity-50"
          >
            {isScanning || scanJob?.status === "queued" || scanJob?.status === "running" ? (
              <Activity className="h-4 w-4 animate-spin" />
            ) : (
              <Search className="h-4 w-4" />
            )}
            Deep Scan
          </button>
        </div>
      </div>

      <MasterDetailLayout
        items={candidates.filter((c) => {
          if (filter === "registered") return c.status === "registered";
          if (filter === "pending") return c.status !== "registered";
          return true;
        })}
        loading={loading}
        selectedId={selectedId}
        onSelect={select}
        idSelector={(c) => c.candidate_id}
        toolbar={
          <div className="flex flex-col gap-3 mb-4">
            <div className="flex items-center gap-2">
              {(["all", "pending", "registered"] as const).map((f) => (
                <button
                  key={f}
                  onClick={() => setFilter(f)}
                  className={`px-3 py-1 text-xs font-medium rounded-full transition-colors ${
                    filter === f
                      ? "bg-primary text-primary-foreground"
                      : "bg-muted text-muted-foreground hover:bg-muted/80"
                  }`}
                >
                  {f.charAt(0).toUpperCase() + f.slice(1)}
                </button>
              ))}
            </div>
            <div className="flex items-center gap-2">
              <input
                type="text"
                placeholder="Search candidates..."
                className="px-3 py-1.5 text-sm rounded-md border bg-background w-full"
              />
            </div>
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
          const caps = capabilityTags(c);
          const isRegistered = c.status === "registered";

          return (
            <EntityCard
              title={c.display_name}
              subtitle={c.inferred_agent_type}
              icon={ShieldAlert}
              status={status}
              statusLabel={isRegistered ? "Registered" : "Pending"}
              meta={[
                {
                  label: "Confidence",
                  value: `${(c.confidence * 100).toFixed(0)}%`,
                },
                {
                  label: "Capabilities",
                  value:
                    caps.length > 0 ? caps.slice(0, 3).join(", ") : "Unknown",
                },
              ]}
              actions={
                isRegistered
                  ? []
                  : [
                      {
                        label:
                          confirmingId === c.candidate_id
                            ? "Confirming..."
                            : "Confirm Agent",
                        icon: CheckCircle,
                        primary: true,
                        disabled: confirmingId === c.candidate_id,
                        onClick: () => openConfirmDialog(c),
                      },
                    ]
              }
              selected={selected}
            />
          );
        }}
        renderDetail={(c) => {
          let status: UiStatus = "idle";
          if (c.status === "registered") status = "ok";
          else if (c.status === "pending_approval") status = "degraded";
          const caps = capabilityTags(c);
          const isRegistered = c.status === "registered";

          return (
            <DetailPane
              title={c.display_name}
              subtitle={c.inferred_agent_type}
              status={status}
              statusLabel={isRegistered ? "Registered" : "Pending"}
              actions={[
                ...(isRegistered
                  ? []
                  : [
                      {
                        label:
                          confirmingId === c.candidate_id
                            ? "Confirming..."
                            : "Confirm Agent",
                        primary: true,
                        icon: CheckCircle,
                        disabled: confirmingId === c.candidate_id,
                        onClick: () => openConfirmDialog(c),
                      },
                    ]),
                {
                  label: "Protect",
                  primary: isRegistered,
                  onClick: () => setProtectTarget(c.candidate_id),
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

                      <div className="p-4 bg-muted/30 rounded-xl border">
                        <h4 className="text-sm font-semibold mb-3">
                          Detected Capabilities
                        </h4>
                        {caps.length > 0 ? (
                          <div className="flex flex-wrap gap-2">
                            {caps.map((cap) => (
                              <span
                                key={cap}
                                className="rounded-md border bg-background px-2 py-1 text-xs font-medium text-foreground/80"
                              >
                                {cap}
                              </span>
                            ))}
                          </div>
                        ) : (
                          <p className="text-sm text-muted-foreground">
                            No capabilities inferred yet.
                          </p>
                        )}
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

      {protectTarget && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
          <div
            className="fixed inset-0 bg-background/80 backdrop-blur-sm transition-opacity"
            onClick={() => setProtectTarget(null)}
          />
          <div className="relative z-50 w-full max-w-3xl rounded-xl border bg-card p-6 shadow-lg max-h-[90vh] overflow-y-auto">
            <button
              onClick={() => setProtectTarget(null)}
              className="absolute right-4 top-4 rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100"
            >
              <span className="sr-only">Close</span>
              <svg
                width="15"
                height="15"
                viewBox="0 0 15 15"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
              >
                <path
                  d="M12.8536 2.85355C13.0488 2.65829 13.0488 2.34171 12.8536 2.14645C12.6583 1.95118 12.3417 1.95118 12.1464 2.14645L7.5 6.79289L2.85355 2.14645C2.65829 1.95118 2.34171 1.95118 2.14645 2.14645C1.95118 2.34171 1.95118 2.65829 2.14645 2.85355L6.79289 7.5L2.14645 12.1464C1.95118 12.3417 1.95118 12.6583 2.14645 12.8536C2.34171 13.0488 2.65829 13.0488 2.85355 12.8536L7.5 8.20711L12.1464 12.8536C12.3417 13.0488 12.6583 13.0488 12.8536 12.8536C13.0488 12.6583 13.0488 12.3417 12.8536 12.1464L8.20711 7.5L12.8536 2.85355Z"
                  fill="currentColor"
                  fillRule="evenodd"
                  clipRule="evenodd"
                ></path>
              </svg>
            </button>
            <SimplePolicyWizard
              agents={candidates.map((c) => ({
                id: c.candidate_id,
                label: c.display_name,
              }))}
              initialTarget={protectTarget}
              onComplete={() => {
                setProtectTarget(null);
                toast.success("Protection applied successfully");
                fetchCandidates();
              }}
            />
          </div>
        </div>
      )}

      {confirmTarget && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
          <div
            className="fixed inset-0 bg-background/80 backdrop-blur-sm transition-opacity"
            onClick={() => setConfirmTarget(null)}
          />
          <div className="relative z-50 w-full max-w-md rounded-xl border bg-card p-6 shadow-lg">
            <h3 className="text-lg font-semibold mb-4">Confirm Agent Registration</h3>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium mb-1 text-muted-foreground">
                  Friendly Name
                </label>
                <input
                  type="text"
                  value={editName}
                  onChange={(e) => setEditName(e.target.value)}
                  className="w-full px-3 py-2 text-sm rounded-md border bg-background"
                  autoFocus
                />
              </div>
              <p className="text-sm text-muted-foreground">
                This agent will be registered and appear in the Agents & Models inventory.
              </p>
              <div className="flex justify-end gap-2 mt-6">
                <button
                  onClick={() => setConfirmTarget(null)}
                  className="px-4 py-2 text-sm font-medium rounded-md border bg-background hover:bg-muted"
                >
                  Cancel
                </button>
                <button
                  onClick={submitConfirmAgent}
                  disabled={confirmingId === confirmTarget.candidate_id}
                  className="px-4 py-2 text-sm font-medium rounded-md bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
                >
                  {confirmingId === confirmTarget.candidate_id ? "Confirming..." : "Confirm"}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
