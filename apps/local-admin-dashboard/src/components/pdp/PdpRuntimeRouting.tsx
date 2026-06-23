import { useState, useEffect } from "react";
import { PdpRuntimeApi } from "../../services/api";
import type { PdpRuntime } from "../../services/api";
import { LocalEnginesTab } from "./LocalEnginesTab";
import { CloudPdpTab } from "./CloudPdpTab";
import { RoutingTab } from "./RoutingTab";
import { OpenFgaWizard } from "./OpenFgaWizard";

export function PdpRuntimeRouting() {
  const [activeTab, setActiveTab] = useState<
    "local" | "remote" | "cloud" | "routing"
  >("local");
  const [runtimes, setRuntimes] = useState<PdpRuntime[]>([]);
  const [testResults, setTestResults] = useState<Record<string, any>>({});
  const [newRemoteName, setNewRemoteName] = useState("");
  const [newRemoteKind, setNewRemoteKind] = useState<
    "opa_server" | "openfga_server" | "cedar_http"
  >("opa_server");
  const [newRemoteUrl, setNewRemoteUrl] = useState("http://localhost:8181");
  const [isWizardOpen, setIsWizardOpen] = useState(false);

  // Routing state moved to RoutingTab

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      const rtRes = await PdpRuntimeApi.list();
      setRuntimes(rtRes);
    } catch (e) {
      console.error(e);
    }
  };

  const handleTestRuntime = async (id: string) => {
    try {
      const res = await PdpRuntimeApi.probe(id);
      setTestResults((prev) => ({ ...prev, [id]: res }));
    } catch (e) {
      console.error(e);
      setTestResults((prev) => ({
        ...prev,
        [id]: { ok: false, latency_ms: 0, detail: "error" },
      }));
    }
  };

  const handleAddRemote = async () => {
    if (!newRemoteUrl || !newRemoteName) return;
    try {
      await PdpRuntimeApi.upsert({
        id: `${newRemoteKind}-${Date.now()}`,
        name: newRemoteName,
        category: "remote_connector",
        kind: newRemoteKind,
        enabled: true,
        status: "ready",
        endpoint: newRemoteUrl,
        mode: "strict_remote",
        system_managed: false,
        config_source: "manual",
        capabilities: [],
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      });
      setNewRemoteName("");
      setNewRemoteUrl("");
      loadData();
    } catch (e) {
      console.error(e);
    }
  };

  const handleDeleteRuntime = async (id: string) => {
    if (!confirm("Are you sure you want to delete this remote connector?"))
      return;
    try {
      await PdpRuntimeApi.delete(id);
      loadData();
    } catch (e) {
      console.error(e);
    }
  };

  const remoteRuntimes = runtimes.filter(
    (r) => r.category === "remote_connector",
  );

  return (
    <div className="glass p-6 rounded-xl space-y-6">
      <h3 className="text-lg font-medium">
        PDP Runtime & Routing Configuration
      </h3>

      <div className="flex border-b">
        <button
          className={`px-4 py-2 text-sm font-medium border-b-2 ${activeTab === "local" ? "border-primary text-primary" : "border-transparent text-muted-foreground hover:text-foreground"}`}
          onClick={() => setActiveTab("local")}
        >
          Local Engines
        </button>
        <button
          className={`px-4 py-2 text-sm font-medium border-b-2 ${activeTab === "remote" ? "border-primary text-primary" : "border-transparent text-muted-foreground hover:text-foreground"}`}
          onClick={() => setActiveTab("remote")}
        >
          Remote Connectors
        </button>
        <button
          className={`px-4 py-2 text-sm font-medium border-b-2 ${activeTab === "cloud" ? "border-primary text-primary" : "border-transparent text-muted-foreground hover:text-foreground"}`}
          onClick={() => setActiveTab("cloud")}
        >
          Pollen Cloud PDP
        </button>
        <button
          className={`px-4 py-2 text-sm font-medium border-b-2 ${activeTab === "routing" ? "border-primary text-primary" : "border-transparent text-muted-foreground hover:text-foreground"}`}
          onClick={() => setActiveTab("routing")}
        >
          Routing & Failover
        </button>
      </div>

      <div className="pt-4">
        {activeTab === "local" && <LocalEnginesTab />}
        {activeTab === "cloud" && <CloudPdpTab />}
        {activeTab === "routing" && <RoutingTab />}

        {activeTab === "remote" && (
          <div className="space-y-6">
            <div className="flex justify-between items-start">
              <p className="text-sm text-muted-foreground max-w-lg">
                Add third-party or custom external PDP servers to be used by the
                local DEK.
              </p>
              <button 
                onClick={() => setIsWizardOpen(true)}
                className="text-xs px-3 py-1.5 bg-emerald-500/10 text-emerald-600 border border-emerald-500/20 rounded hover:bg-emerald-500/20 transition-colors font-medium"
              >
                OpenFGA Quick Setup
              </button>
            </div>

            <div className="flex gap-2 max-w-2xl bg-muted/30 p-4 rounded-lg items-center">
              <input
                type="text"
                placeholder="Name"
                className="flex h-10 w-32 rounded-md border border-input bg-background px-3 py-2 text-sm"
                value={newRemoteName}
                onChange={(e) => setNewRemoteName(e.target.value)}
              />
              <select
                className="flex h-10 rounded-md border border-input bg-background px-3 py-2 text-sm w-40"
                value={newRemoteKind}
                onChange={(e) => {
                  setNewRemoteKind(e.target.value as any);
                  if (e.target.value === "opa_server")
                    setNewRemoteUrl("http://localhost:8181");
                  else if (e.target.value === "openfga_server")
                    setNewRemoteUrl("http://localhost:8080");
                  else if (e.target.value === "cedar_http")
                    setNewRemoteUrl("http://localhost:8081");
                }}
              >
                <option value="opa_server">OPA Server</option>
                <option value="openfga_server">OpenFGA</option>
                <option value="cedar_http">Cedar HTTP</option>
              </select>
              <input
                type="text"
                placeholder="http://localhost:8181"
                className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                value={newRemoteUrl}
                onChange={(e) => setNewRemoteUrl(e.target.value)}
              />
              <button
                onClick={handleAddRemote}
                className="h-10 px-4 py-2 bg-primary text-primary-foreground rounded-md text-sm font-medium hover:opacity-90 whitespace-nowrap"
              >
                Add
              </button>
            </div>

            <div className="rounded-md border">
              <table className="w-full text-sm text-left">
                <thead className="text-xs uppercase bg-muted/50">
                  <tr>
                    <th className="px-4 py-3">Name / ID</th>
                    <th className="px-4 py-3">Kind</th>
                    <th className="px-4 py-3">Endpoint</th>
                    <th className="px-4 py-3 text-right">Action</th>
                  </tr>
                </thead>
                <tbody>
                  {remoteRuntimes.map((c) => (
                    <tr
                      key={c.id}
                      className="border-b last:border-0 hover:bg-muted/30"
                    >
                      <td className="px-4 py-3 font-medium">
                        {c.name}
                        <div className="text-xs text-muted-foreground mt-0.5">
                          {c.id}
                        </div>
                      </td>
                      <td className="px-4 py-3">
                        <span className="bg-secondary text-secondary-foreground px-2 py-1 rounded text-xs">
                          {c.kind}
                        </span>
                      </td>
                      <td className="px-4 py-3 font-mono text-xs">
                        {c.endpoint}
                      </td>
                      <td className="px-4 py-3 text-right">
                        <div className="flex items-center justify-end gap-2">
                          {testResults[c.id] && (
                            <span
                              className={`text-xs px-2 py-1 rounded ${testResults[c.id].ok ? "bg-green-500/10 text-green-500" : "bg-red-500/10 text-red-500"}`}
                            >
                              {testResults[c.id].ok
                                ? `✓ (${testResults[c.id].latency_ms}ms)`
                                : `✗ unreachable`}
                            </span>
                          )}
                          <button
                            onClick={() => handleTestRuntime(c.id)}
                            className="px-3 py-1 bg-secondary text-secondary-foreground rounded text-xs hover:opacity-80"
                          >
                            Probe
                          </button>
                          <button
                            onClick={() => handleDeleteRuntime(c.id)}
                            className="px-3 py-1 bg-red-500/10 text-red-500 rounded text-xs hover:opacity-80"
                          >
                            Delete
                          </button>
                        </div>
                      </td>
                    </tr>
                  ))}
                  {remoteRuntimes.length === 0 && (
                    <tr>
                      <td
                        colSpan={4}
                        className="px-4 py-8 text-center text-muted-foreground"
                      >
                        No remote connectors configured. Add one above.
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>
          </div>
        )}
      </div>

      <OpenFgaWizard 
        isOpen={isWizardOpen} 
        onClose={() => setIsWizardOpen(false)} 
        onComplete={loadData} 
      />
    </div>
  );
}
