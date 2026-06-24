import { useState, useEffect } from "react";
import { RefreshCw, Search, ShieldAlert, CheckCircle } from "lucide-react";
import { RegistryApi, PolicyFirstApi } from "../services/api";
import type {
  DiscoveredAgentCandidateV2,
  DiscoveryScanJob,
} from "../services/types";

export function AutoDiscovery() {
  const [activeTab, setActiveTab] = useState("candidates");
  const [candidates, setCandidates] = useState<DiscoveredAgentCandidateV2[]>(
    [],
  );
  const [loading, setLoading] = useState(false);
  const [loadingCandidates, setLoadingCandidates] = useState(true);
  const [scanJob, setScanJob] = useState<DiscoveryScanJob | null>(null);
  const [showModal, setShowModal] = useState(false);
  const [scanType, setScanType] = useState("deep");
  const [privacyMode, setPrivacyMode] = useState(true);
  const [scanHistory, setScanHistory] = useState<DiscoveryScanJob[]>([]);
  const [snapshot, setSnapshot] = useState<any>(null);
  const [suggestions, setSuggestions] = useState<any[]>([]);
  const [deployModal, setDeployModal] = useState<{
    show: boolean;
    policy: any | null;
  }>({ show: false, policy: null });
  const [deploying, setDeploying] = useState(false);

  const fetchCapabilities = async () => {
    try {
      const snap = await PolicyFirstApi.getLatestSnapshot();
      setSnapshot(snap);
    } catch (e) {
      console.error(e);
    }
  };

  const fetchSuggestions = async () => {
    try {
      const suggs = await PolicyFirstApi.getPolicySuggestions();
      setSuggestions(suggs);
    } catch (e) {
      console.error(e);
    }
  };

  const fetchCandidates = () => {
    setLoadingCandidates(true);
    RegistryApi.listDiscoveryCandidates()
      .then(setCandidates)
      .catch(console.error)
      .finally(() => setLoadingCandidates(false));
  };

  const clearCandidates = async () => {
    if (
      !confirm(
        "Are you sure you want to clear all discovery candidates? This cannot be undone.",
      )
    )
      return;
    try {
      await RegistryApi.clearDiscoveryCandidates();
      fetchCandidates();
    } catch (e) {
      console.error("Failed to clear candidates:", e);
      alert("Failed to clear candidates");
    }
  };

  const deleteCandidate = async (candidateId: string) => {
    if (!confirm("Are you sure you want to delete this candidate?")) return;
    try {
      await RegistryApi.deleteDiscoveryCandidate(candidateId);
      fetchCandidates();
    } catch (e) {
      console.error("Failed to delete candidate:", e);
      alert("Failed to delete candidate");
    }
  };

  useEffect(() => {
    fetchCandidates();
    fetchCapabilities();
    fetchSuggestions();
  }, []);

  const handleDeploy = async () => {
    if (!deployModal.policy) return;
    setDeploying(true);
    try {
      await PolicyFirstApi.createDeploymentSession({
        policy_template_id: deployModal.policy.policy_template_id,
        target_agent_ids: deployModal.policy.target_agent_ids,
      });
      alert("Deployment session created successfully!");
      setDeployModal({ show: false, policy: null });
    } catch (e) {
      alert("Failed to deploy");
      console.error(e);
    } finally {
      setDeploying(false);
    }
  };

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
            setLoading(false);
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

  useEffect(() => {
    if (activeTab === "history") {
      RegistryApi.listDiscoveryScans()
        .then(setScanHistory)
        .catch(console.error);
    }
  }, [activeTab]);

  const triggerScan = () => {
    setShowModal(true);
  };

  const confirmScan = async () => {
    setShowModal(false);
    setLoading(true);
    try {
      const sources =
        scanType === "quick"
          ? ["process", "mcp_config"]
          : [
              "process",
              "mcp_config",
              "local_model",
              "ide_extension",
              "cli_agent",
              "container",
              "browser_extension",
              "web_ai",
            ];
      const result = await RegistryApi.triggerDiscoveryScan({
        sources,
        privacy_mode: privacyMode,
      });
      setScanJob({
        scan_id: result.scan_id,
        tenant_id: "local",
        status: result.status as any,
        sources: sources,
        candidates_found: 0,
      });
    } catch (e) {
      console.error(e);
      setLoading(false);
    }
  };

  const cancelScan = async () => {
    if (scanJob?.scan_id) {
      try {
        await RegistryApi.cancelDiscoveryScan(scanJob.scan_id);
      } catch (e) {
        console.error(e);
      }
    }
    setLoading(false);
    setScanJob((prev) =>
      prev ? { ...prev, status: "cancelled" as any } : null,
    );
  };

  const [identifyModal, setIdentifyModal] = useState<{
    show: boolean;
    candidate: DiscoveredAgentCandidateV2 | null;
    customName: string;
    learnSignature: boolean;
  }>({ show: false, candidate: null, customName: "", learnSignature: true });

  const [isIdentifying, setIsIdentifying] = useState(false);

  const handleConfirmIdentity = async () => {
    const candidate = identifyModal.candidate;
    if (!candidate || isIdentifying) return;
    setIsIdentifying(true);
    try {
      await RegistryApi.confirmCandidate(candidate.candidate_id, {
        candidate_id: candidate.candidate_id,
        custom_display_name: identifyModal.customName || undefined,
        confirmed_agent_type: candidate.inferred_agent_type,
        confirmed_capability_tags: candidate.capability_tags || [],
        make_local_signature: identifyModal.learnSignature,
        confirmed_by: "admin",
      });
      alert(`Successfully confirmed identity for ${candidate.candidate_id}`);
      setIdentifyModal({
        show: false,
        candidate: null,
        customName: "",
        learnSignature: true,
      });
      fetchCandidates();
    } catch (e: any) {
      console.error("Failed to confirm identity:", e);
      alert(`Failed to confirm identity: ${e.message}`);
    } finally {
      setIsIdentifying(false);
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">Auto Discovery</h2>
          <p className="text-muted-foreground">
            Find and manage local AI agents, MCP servers, and model endpoints.
          </p>
        </div>
        {loading ? (
          <button
            onClick={cancelScan}
            className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 ring-offset-background bg-red-500 text-white hover:bg-red-600 h-10 py-2 px-4 shadow-lg"
          >
            <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
            Stop Scan
          </button>
        ) : (
          <button
            onClick={triggerScan}
            className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 ring-offset-background bg-primary text-primary-foreground hover:bg-primary/90 h-10 py-2 px-4 shadow-lg shadow-primary/20"
          >
            <Search className="mr-2 h-4 w-4" />
            Deep Scan
          </button>
        )}
        <button
          onClick={clearCandidates}
          className="bg-red-500/10 text-red-400 hover:bg-red-500/20 h-10 px-4 rounded-md text-sm font-medium transition-colors border border-red-500/20"
        >
          Clear History
        </button>
      </div>

      {scanJob && (
        <div className="p-4 border rounded-md bg-muted/20">
          <p className="text-sm font-medium">
            Scan Status: <span className="uppercase">{scanJob.status}</span>
          </p>
          <p className="text-xs text-muted-foreground">
            Scan ID: {scanJob.scan_id}
          </p>
          {scanJob.error && (
            <p className="text-xs text-red-500">Error: {scanJob.error}</p>
          )}
        </div>
      )}

      <div className="border-b border-border">
        <nav className="-mb-px flex space-x-6">
          {[
            "candidates",
            "capabilities",
            "policies",
            "timeline",
            "evidence",
          ].map((tab) => (
            <button
              key={tab}
              onClick={() => setActiveTab(tab)}
              className={
                "whitespace-nowrap pb-4 px-1 border-b-2 font-medium text-sm " +
                (activeTab === tab
                  ? "border-primary text-foreground"
                  : "border-transparent text-muted-foreground hover:text-foreground hover:border-border")
              }
            >
              {tab === "candidates"
                ? "Agents"
                : tab === "capabilities"
                  ? "What POLLEK can do"
                  : tab === "policies"
                    ? "Recommended Policies"
                    : tab === "timeline"
                      ? "Timeline"
                      : tab === "evidence"
                        ? "Advanced Details"
                        : tab}
            </button>
          ))}
        </nav>
      </div>

      <div className="glass rounded-xl p-6">
        {activeTab === "candidates" && (
          <div>
            <h3 className="font-semibold mb-4">Discovered Agents</h3>
            {loadingCandidates ? (
              <div className="flex h-[200px] items-center justify-center rounded-md border border-dashed border-muted">
                <p className="text-sm text-muted-foreground">
                  Loading candidates...
                </p>
              </div>
            ) : candidates.length === 0 ? (
              <div className="flex h-[200px] items-center justify-center rounded-md border border-dashed border-muted">
                <p className="text-sm text-muted-foreground">
                  No discovered agents yet. Click "Deep Scan" to begin.
                </p>
              </div>
            ) : (
              <div className="space-y-8">
                {(() => {
                  const sortedCandidates = [...candidates].sort(
                    (a, b) =>
                      new Date(b.last_seen).getTime() -
                      new Date(a.last_seen).getTime(),
                  );
                  const latestScanTime =
                    sortedCandidates.length > 0
                      ? new Date(sortedCandidates[0].last_seen).getTime()
                      : 0;
                  const latestCandidates = sortedCandidates.filter(
                    (c) =>
                      new Date(c.last_seen).getTime() > latestScanTime - 60000,
                  );
                  const previousCandidates = sortedCandidates.filter(
                    (c) =>
                      new Date(c.last_seen).getTime() <= latestScanTime - 60000,
                  );

                  const renderCandidateList = (
                    list: DiscoveredAgentCandidateV2[],
                  ) => (
                    <div className="space-y-4">
                      {list.map((c, idx) => (
                        <div
                          key={`${c.candidate_id}-${idx}`}
                          className="border rounded-lg p-4 hover:bg-muted/30 transition-colors"
                        >
                          <div className="flex justify-between items-start mb-2">
                            <div className="flex items-center gap-2">
                              <ShieldAlert className="h-5 w-5 text-primary" />
                              <h4 className="font-medium text-lg">
                                {c.display_name}{" "}
                                <span className="text-xs text-muted-foreground">
                                  ({c.inferred_agent_type})
                                </span>
                              </h4>
                            </div>
                            <div className="flex gap-2">
                              {c.status === "registered" ? (
                                <span className="inline-flex items-center gap-1 text-xs text-green-500 bg-green-500/10 border border-green-500/20 font-medium px-2.5 py-1 rounded-md">
                                  <CheckCircle className="w-3.5 h-3.5" />{" "}
                                  Registered
                                </span>
                              ) : c.status === "pending_approval" ? (
                                <div className="flex flex-col items-end">
                                  <button
                                    onClick={() =>
                                      setIdentifyModal({
                                        show: true,
                                        candidate: c,
                                        customName: c.display_name,
                                        learnSignature: true,
                                      })
                                    }
                                    className="text-xs border border-orange-500/50 px-3 py-1.5 rounded bg-orange-500/10 text-orange-500 hover:bg-orange-500/20 font-medium transition-colors"
                                  >
                                    Identify this Agent
                                  </button>
                                  <span className="text-[10px] text-muted-foreground mt-1">
                                    Low confidence or unverified
                                  </span>
                                </div>
                              ) : (
                                <div className="flex flex-col items-end">
                                  <a
                                    href={`/wizard?target=${c.candidate_id}`}
                                    className="text-xs border px-3 py-1.5 rounded bg-primary text-primary-foreground font-medium transition-colors"
                                  >
                                    Apply Policy
                                  </a>
                                  <span className="text-[10px] text-muted-foreground mt-1">
                                    System will select best control method
                                  </span>
                                </div>
                              )}
                              <button
                                onClick={() => deleteCandidate(c.candidate_id)}
                                className="text-xs border border-red-500/20 bg-red-500/10 text-red-400 px-3 py-1.5 rounded hover:bg-red-500/20 font-medium transition-colors ml-2"
                              >
                                Delete
                              </button>
                            </div>
                          </div>
                          <div className="space-y-1">
                            <p className="text-muted-foreground text-sm font-mono bg-muted/20 inline-block px-2 py-0.5 rounded border mb-1">
                              ID: {c.candidate_id}
                            </p>
                            <p className="text-muted-foreground text-sm">
                              <span className="font-medium">Risk Score:</span>{" "}
                              {c.risk_score} |{" "}
                              <span className="font-medium">Confidence:</span>{" "}
                              {(c.confidence * 100).toFixed(0)}% <br />
                              <span className="font-medium">
                                First seen:
                              </span>{" "}
                              {new Date(c.first_seen).toLocaleString()} <br />
                              <span className="font-medium">
                                Last seen:
                              </span>{" "}
                              {new Date(c.last_seen).toLocaleString()}
                            </p>
                          </div>

                          {c.discovered_mcp_servers &&
                            c.discovered_mcp_servers.length > 0 && (
                              <div className="mt-3 p-3 bg-muted/40 rounded-md">
                                <h5 className="text-xs font-semibold mb-1">
                                  Discovered MCP Servers:
                                </h5>
                                <ul className="text-xs space-y-1">
                                  {c.discovered_mcp_servers.map(
                                    (mcp: any, i: number) => (
                                      <li key={i}>
                                        - {mcp.server_name} ({mcp.transport})
                                      </li>
                                    ),
                                  )}
                                </ul>
                              </div>
                            )}
                        </div>
                      ))}
                    </div>
                  );

                  return (
                    <>
                      {latestCandidates.length > 0 && (
                        <div>
                          <h4 className="font-medium text-sm text-primary mb-3 flex items-center gap-2">
                            <div className="w-2 h-2 rounded-full bg-primary animate-pulse" />
                            Latest Scan Results
                          </h4>
                          {renderCandidateList(latestCandidates)}
                        </div>
                      )}

                      {previousCandidates.length > 0 && (
                        <div className="pt-4 border-t">
                          <h4 className="font-medium text-sm text-muted-foreground mb-3">
                            Previously Discovered
                          </h4>
                          {renderCandidateList(previousCandidates)}
                        </div>
                      )}
                    </>
                  );
                })()}
              </div>
            )}
          </div>
        )}

        {activeTab === "capabilities" && (
          <div>
            <h3 className="font-semibold mb-4">
              What POLLEK can do on this device
            </h3>
            <p className="text-sm text-muted-foreground mb-4">
              Overview of local control capabilities automatically detected by
              the Policy Enforcement Point (PEP).
            </p>

            {!snapshot ? (
              <div className="flex h-[150px] items-center justify-center rounded-md border border-dashed border-muted">
                <p className="text-sm text-muted-foreground">
                  No snapshot available.
                </p>
              </div>
            ) : (
              <div className="space-y-4">
                <div className="border rounded-lg p-4 bg-muted/10">
                  <h4 className="font-medium text-primary mb-2">Device Info</h4>
                  <p className="text-sm">
                    OS: {snapshot.os?.type} {snapshot.os?.version} (
                    {snapshot.os?.arch})
                  </p>
                  <p className="text-sm">Device ID: {snapshot.device_id}</p>
                </div>

                <h4 className="font-medium mt-4">Detected Control Methods</h4>
                {snapshot.methods && snapshot.methods.length > 0 ? (
                  snapshot.methods.map((m: any, idx: number) => (
                    <div
                      key={idx}
                      className="border rounded-lg p-4 bg-muted/10 flex justify-between items-center"
                    >
                      <div>
                        <p className="font-medium">{m.method}</p>
                        <p className="text-xs text-muted-foreground">
                          Status: {m.status}
                        </p>
                      </div>
                      <div className="flex gap-2">
                        {m.can_observe && (
                          <span className="text-xs bg-blue-500/10 text-blue-500 px-2 py-1 rounded">
                            Can Observe
                          </span>
                        )}
                        {m.can_enforce && (
                          <span className="text-xs bg-green-500/10 text-green-500 px-2 py-1 rounded">
                            Can Enforce
                          </span>
                        )}
                      </div>
                    </div>
                  ))
                ) : (
                  <div className="text-sm text-muted-foreground p-4 border rounded-lg bg-muted/10 text-center">
                    No capabilities found yet.
                  </div>
                )}
              </div>
            )}
          </div>
        )}

        {activeTab === "policies" && (
          <div>
            <h3 className="font-semibold mb-4">Recommended Policies</h3>
            <p className="text-sm text-muted-foreground mb-4">
              These policies are suggested based on the discovered agents and
              local capabilities.
            </p>

            {suggestions.length === 0 ? (
              <div className="flex h-[150px] items-center justify-center rounded-md border border-dashed border-muted">
                <p className="text-sm text-muted-foreground">
                  No suggestions available.
                </p>
              </div>
            ) : (
              <div className="space-y-4">
                {suggestions.map((sugg, idx) => (
                  <div key={idx} className="border rounded-lg overflow-hidden">
                    <div className="bg-muted/30 p-3 border-b flex justify-between items-center">
                      <h4 className="font-medium">
                        {sugg.display_name?.en || sugg.suggestion_id}
                      </h4>
                      <span className="text-xs text-muted-foreground font-mono bg-muted/50 px-2 py-0.5 rounded border">
                        Feasibility: {sugg.feasibility}
                      </span>
                    </div>
                    <div className="p-4 space-y-3">
                      <p className="text-sm">{sugg.description?.en || ""}</p>
                      <p className="text-xs text-muted-foreground">
                        Target Agents: {sugg.target_agent_ids.join(", ")}
                      </p>

                      <div className="flex justify-end pt-2">
                        <button
                          onClick={() =>
                            setDeployModal({ show: true, policy: sugg })
                          }
                          className="text-sm bg-primary text-primary-foreground px-4 py-2 rounded font-medium shadow-sm hover:opacity-90"
                        >
                          Deploy Preview
                        </button>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        {activeTab === "evidence" && (
          <div>
            <h3 className="font-semibold mb-4">
              Advanced / Discovery Evidence
            </h3>
            <p className="text-sm text-muted-foreground mb-4">
              Raw telemetry collected by scanners.
            </p>
            {candidates.length === 0 ? (
              <div className="flex h-[150px] items-center justify-center rounded-md border border-dashed border-muted">
                <p className="text-sm text-muted-foreground">
                  No evidence collected.
                </p>
              </div>
            ) : (
              <div className="space-y-4">
                {candidates.map((c) =>
                  c.evidence.map((ev, i) => (
                    <div
                      key={`${c.candidate_id}-${i}`}
                      className="border rounded p-3 text-sm flex flex-col gap-1 font-mono bg-muted/10"
                    >
                      <div className="flex justify-between items-center text-xs">
                        <span className="font-bold text-primary">
                          {ev.source}
                        </span>
                        <span className="text-muted-foreground">
                          {ev.observed_at}
                        </span>
                      </div>
                      <div className="text-xs text-muted-foreground">
                        Privacy Class: {ev.privacy_class}
                      </div>
                      <pre className="mt-2 text-xs overflow-x-auto p-2 bg-muted rounded border">
                        {JSON.stringify(ev.data, null, 2)}
                      </pre>
                    </div>
                  )),
                )}
              </div>
            )}
          </div>
        )}

        {activeTab === "history" && (
          <div>
            <h3 className="font-semibold mb-4">Scan History</h3>
            {scanHistory.length === 0 ? (
              <div className="flex h-[200px] items-center justify-center rounded-md border border-dashed border-muted">
                <p className="text-sm text-muted-foreground">
                  No scans have been performed yet.
                </p>
              </div>
            ) : (
              <div className="space-y-3">
                {scanHistory
                  .sort(
                    (a, b) =>
                      new Date(b.started_at || 0).getTime() -
                      new Date(a.started_at || 0).getTime(),
                  )
                  .map((job) => (
                    <div
                      key={job.scan_id}
                      className="border rounded p-3 text-sm flex justify-between items-center bg-muted/10"
                    >
                      <div>
                        <div className="font-medium">{job.scan_id}</div>
                        <div className="text-xs text-muted-foreground">
                          Sources: {job.sources.join(", ")}
                        </div>
                      </div>
                      <div className="text-right">
                        <div
                          className={
                            "font-medium uppercase " +
                            (job.status === "failed"
                              ? "text-red-500"
                              : job.status === "cancelled"
                                ? "text-yellow-500"
                                : "text-primary")
                          }
                        >
                          {job.status}
                        </div>
                        <div className="text-xs text-muted-foreground">
                          {job.started_at
                            ? new Date(job.started_at).toLocaleString()
                            : ""}
                        </div>
                      </div>
                    </div>
                  ))}
              </div>
            )}
          </div>
        )}
      </div>

      {showModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="bg-background border rounded-xl shadow-xl w-full max-w-md p-6">
            <h3 className="text-xl font-bold mb-4">Start Discovery Scan</h3>
            <div className="space-y-4">
              <label className="flex items-start gap-3 p-3 border rounded cursor-pointer hover:bg-muted/20">
                <input
                  type="radio"
                  name="scan_type"
                  value="quick"
                  checked={scanType === "quick"}
                  onChange={() => setScanType("quick")}
                  className="mt-1"
                />
                <div>
                  <div className="font-medium">Quick Scan</div>
                  <div className="text-xs text-muted-foreground">
                    Process scan and MCP config only. High confidence.
                  </div>
                </div>
              </label>
              <label className="flex items-start gap-3 p-3 border rounded cursor-pointer hover:bg-muted/20">
                <input
                  type="radio"
                  name="scan_type"
                  value="deep"
                  checked={scanType === "deep"}
                  onChange={() => setScanType("deep")}
                  className="mt-1"
                />
                <div>
                  <div className="font-medium">Deep Scan</div>
                  <div className="text-xs text-muted-foreground">
                    Includes IDE extensions, CLI tools, and Local Model servers.
                  </div>
                </div>
              </label>
              <div className="flex items-center gap-2 pt-2">
                <input
                  type="checkbox"
                  id="privacy"
                  checked={privacyMode}
                  onChange={(e) => setPrivacyMode(e.target.checked)}
                />
                <label htmlFor="privacy" className="text-sm">
                  Redact sensitive paths locally (Privacy Mode)
                </label>
              </div>
            </div>
            <div className="mt-6 flex justify-end gap-3">
              <button
                onClick={() => setShowModal(false)}
                className="px-4 py-2 text-sm border rounded hover:bg-muted"
              >
                Cancel
              </button>
              <button
                onClick={confirmScan}
                className="px-4 py-2 text-sm bg-primary text-primary-foreground rounded hover:bg-primary/90"
              >
                Start Scan
              </button>
            </div>
          </div>
        </div>
      )}

      {identifyModal.show && identifyModal.candidate && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="bg-background border rounded-xl shadow-xl w-full max-w-lg p-6">
            <h3 className="text-xl font-bold mb-2">Identify Agent</h3>
            <p className="text-sm text-muted-foreground mb-4">
              The system found an agent but needs your confirmation due to low
              confidence or ambiguity.
            </p>

            <div className="p-4 border rounded-lg bg-muted/20 space-y-3 mb-4">
              <div className="flex justify-between items-start">
                <div>
                  <p className="text-xs text-muted-foreground uppercase font-semibold">
                    Top Guess
                  </p>
                  <p className="text-lg font-medium">
                    {identifyModal.candidate.display_name}
                  </p>
                  <p className="text-xs text-muted-foreground">
                    Type: {identifyModal.candidate.inferred_agent_type}
                  </p>
                </div>
                <div className="text-right">
                  <p className="text-xs text-muted-foreground uppercase font-semibold">
                    Confidence
                  </p>
                  <p className="text-lg font-mono text-orange-500">
                    {(identifyModal.candidate.confidence * 100).toFixed(0)}%
                  </p>
                </div>
              </div>

              {identifyModal.candidate.matched_signals &&
                identifyModal.candidate.matched_signals.length > 0 && (
                  <div className="pt-2 border-t border-dashed border-muted-foreground/30">
                    <p className="text-xs text-muted-foreground mb-2">
                      Match Evidence (Redacted):
                    </p>
                    <ul className="text-xs space-y-1 font-mono text-muted-foreground">
                      {identifyModal.candidate.matched_signals.map((sig, i) => (
                        <li key={i} className="flex gap-2">
                          <span className="text-foreground/70 w-24 shrink-0 truncate">
                            {sig.kind}:
                          </span>
                          <span className="truncate" title={sig.detail}>
                            {sig.detail}
                          </span>
                        </li>
                      ))}
                    </ul>
                  </div>
                )}
            </div>

            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium mb-1">
                  Confirm Name (or provide custom name)
                </label>
                <input
                  type="text"
                  value={identifyModal.customName}
                  onChange={(e) =>
                    setIdentifyModal({
                      ...identifyModal,
                      customName: e.target.value,
                    })
                  }
                  className="w-full border rounded px-3 py-2 text-sm bg-background focus:ring-2 focus:ring-primary focus:outline-none"
                />
              </div>
              <label className="flex items-center gap-2 text-sm cursor-pointer">
                <input
                  type="checkbox"
                  checked={identifyModal.learnSignature}
                  onChange={(e) =>
                    setIdentifyModal({
                      ...identifyModal,
                      learnSignature: e.target.checked,
                    })
                  }
                />
                <span>Learn this signature (auto-identify next time)</span>
              </label>
            </div>

            <div className="mt-6 flex justify-end gap-3">
              <button
                onClick={() =>
                  setIdentifyModal({
                    show: false,
                    candidate: null,
                    customName: "",
                    learnSignature: true,
                  })
                }
                className="px-4 py-2 text-sm border rounded hover:bg-muted"
              >
                Cancel
              </button>
              <button
                onClick={handleConfirmIdentity}
                disabled={!identifyModal.customName.trim() || isIdentifying}
                className="px-4 py-2 text-sm bg-orange-500 text-white rounded hover:bg-orange-600 disabled:opacity-50 flex items-center gap-2 font-medium"
              >
                {isIdentifying ? "Confirming..." : "Identify & Confirm"}
              </button>
            </div>
          </div>
        </div>
      )}

      {deployModal.show && deployModal.policy && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="bg-background border rounded-xl shadow-xl w-full max-w-md p-6">
            <h3 className="text-xl font-bold mb-4">Deploy Policy Preview</h3>
            <div className="space-y-4">
              <div className="p-3 bg-muted/10 rounded-md border">
                <p className="font-medium text-sm">
                  {deployModal.policy.display_name?.en}
                </p>
                <p className="text-xs text-muted-foreground mt-1">
                  {deployModal.policy.description?.en}
                </p>
              </div>
              <div className="p-3 bg-primary/10 rounded-md border border-primary/20">
                <p className="font-medium text-sm text-primary mb-1">
                  Control Level Options
                </p>
                <p className="text-xs text-primary/80">
                  The system will automatically configure the required PEP
                  mechanisms.
                </p>
                <div className="mt-3 space-y-2">
                  <label className="flex items-center gap-2 text-sm cursor-pointer">
                    <input
                      type="radio"
                      name="controlLevel"
                      value="warn"
                      defaultChecked={
                        deployModal.policy.recommended_control_level === "Warn"
                      }
                    />
                    <span>Warn Only</span>
                  </label>
                  <label className="flex items-center gap-2 text-sm cursor-pointer">
                    <input
                      type="radio"
                      name="controlLevel"
                      value="approval"
                      defaultChecked={
                        deployModal.policy.recommended_control_level ===
                        "Approval"
                      }
                    />
                    <span>Require Approval</span>
                  </label>
                  <label className="flex items-center gap-2 text-sm cursor-pointer">
                    <input
                      type="radio"
                      name="controlLevel"
                      value="enforce"
                      defaultChecked={
                        deployModal.policy.recommended_control_level ===
                        "Enforce"
                      }
                    />
                    <span>Strict Enforce</span>
                  </label>
                </div>
              </div>
            </div>
            <div className="mt-6 flex justify-end gap-3">
              <button
                onClick={() => setDeployModal({ show: false, policy: null })}
                className="px-4 py-2 text-sm border rounded hover:bg-muted"
                disabled={deploying}
              >
                Cancel
              </button>
              <button
                onClick={handleDeploy}
                disabled={deploying}
                className="px-4 py-2 text-sm bg-primary text-primary-foreground rounded hover:bg-primary/90 disabled:opacity-50"
              >
                {deploying ? "Deploying..." : "Confirm Deployment"}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
