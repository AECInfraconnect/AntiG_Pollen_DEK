import { useState, useEffect } from "react";
import { PdpRoutingApi } from "../../services/api";
import type { PdpRouteRule } from "../../services/api";

export function RoutingTab() {
  const [routes, setRoutes] = useState<PdpRouteRule[]>([]);
  const [loading, setLoading] = useState(true);

  // New Route form state
  const [newRouteName, setNewRouteName] = useState("");
  const [newRouteMode, setNewRouteMode] = useState<PdpRouteRule["mode"]>("local_primary_remote_fallback");
  const [newRoutePrimary, setNewRoutePrimary] = useState("");
  const [newRouteFallback, setNewRouteFallback] = useState("");
  const [newRouteFailure, setNewRouteFailure] = useState<PdpRouteRule["failure_behavior"]>("fallback");
  const [newRouteMatchCond, setNewRouteMatchCond] = useState<string>("{}");
  const [simulateResult, setSimulateResult] = useState<any>(null);

  const reload = async () => {
    setLoading(true);
    try {
      const data = await PdpRoutingApi.list();
      setRoutes(data);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    reload();
  }, []);

  const handleAddRoute = async () => {
    if (!newRouteName || !newRoutePrimary) return;
    try {
      let matchCond = {};
      try {
        matchCond = JSON.parse(newRouteMatchCond);
      } catch (e) {
        alert("Invalid JSON for Match Conditions");
        return;
      }
      await PdpRoutingApi.upsert({
        id: `route-${Date.now()}`,
        name: newRouteName,
        enabled: true,
        priority: 100,
        match_cond: matchCond,
        mode: newRouteMode,
        primary_pdp_id: newRoutePrimary,
        fallback_pdp_ids: newRouteFallback ? newRouteFallback.split(",").map((s) => s.trim()) : [],
        shadow_pdp_ids: [],
        merge_strategy: "first_decisive",
        failure_behavior: newRouteFailure,
        timeout_ms: 200,
        max_retries: 0,
      });
      setNewRouteName("");
      setNewRoutePrimary("");
      setNewRouteFallback("");
      reload();
    } catch (e) {
      console.error(e);
    }
  };

  const handleDeleteRoute = async (id: string) => {
    if (!confirm("Are you sure you want to delete this route?")) return;
    try {
      await PdpRoutingApi.delete(id);
      reload();
    } catch (e) {
      console.error(e);
    }
  };

  const handleSimulate = async () => {
    try {
      const res = await PdpRoutingApi.simulate({
        action: "simulate",
        resource: "test",
        principal: "admin",
        context: {}
      });
      setSimulateResult(res);
    } catch (e: any) {
      setSimulateResult({ error: e.message });
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <p className="text-sm text-muted-foreground">
          Manage how authorization requests are routed to the available PDP runtimes.
        </p>
        <button onClick={reload} className="px-3 py-1 bg-secondary text-secondary-foreground rounded text-xs hover:opacity-80">
          Refresh
        </button>
      </div>

      <div className="rounded-md border">
        <table className="w-full text-sm text-left">
          <thead className="text-xs uppercase bg-muted/50">
            <tr>
              <th className="px-4 py-3">Priority</th>
              <th className="px-4 py-3">Name</th>
              <th className="px-4 py-3">Mode</th>
              <th className="px-4 py-3">Primary PDP</th>
              <th className="px-4 py-3">Fallback</th>
              <th className="px-4 py-3 text-right">Action</th>
            </tr>
          </thead>
          <tbody>
            {routes.map((r) => (
              <tr key={r.id} className="border-b last:border-0 hover:bg-muted/30">
                <td className="px-4 py-3 font-mono">{r.priority}</td>
                <td className="px-4 py-3 font-medium">{r.name}</td>
                <td className="px-4 py-3">
                  <span className="bg-primary/10 text-primary px-2 py-1 rounded text-xs">{r.mode}</span>
                </td>
                <td className="px-4 py-3 font-mono text-xs">{r.primary_pdp_id}</td>
                <td className="px-4 py-3 font-mono text-xs">{r.fallback_pdp_ids?.join(", ")}</td>
                <td className="px-4 py-3 text-right">
                  <button onClick={() => handleDeleteRoute(r.id)} className="px-3 py-1 bg-red-500/10 text-red-500 rounded text-xs hover:opacity-80">
                    Delete
                  </button>
                </td>
              </tr>
            ))}
            {!loading && routes.length === 0 && (
              <tr>
                <td colSpan={6} className="px-4 py-8 text-center text-muted-foreground">
                  No routes configured. Add one below.
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mt-8 pt-6 border-t">
        <div>
          <h4 className="text-sm font-medium mb-4">Add New Route</h4>
          <div className="space-y-4 bg-muted/20 p-4 rounded-lg">
            <div className="space-y-2">
              <label className="text-xs font-medium">Route Name</label>
              <input
                type="text"
                className="flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm"
                placeholder="e.g. Default Routing"
                value={newRouteName}
                onChange={(e) => setNewRouteName(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <label className="text-xs font-medium">Routing Mode</label>
              <select
                className="flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm"
                value={newRouteMode}
                onChange={(e) => setNewRouteMode(e.target.value as any)}
              >
                <option value="local_only">Local Only</option>
                <option value="local_primary_remote_fallback">Local Primary, Remote Fallback</option>
                <option value="remote_primary_local_fallback">Remote Primary, Local Fallback</option>
                <option value="strict_remote">Strict Remote</option>
              </select>
            </div>
            <div className="space-y-2">
              <label className="text-xs font-medium">Primary PDP ID</label>
              <input
                type="text"
                className="flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm"
                placeholder="e.g. local.cedar"
                value={newRoutePrimary}
                onChange={(e) => setNewRoutePrimary(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <label className="text-xs font-medium">Fallback PDP IDs (comma separated)</label>
              <input
                type="text"
                className="flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm"
                placeholder="e.g. remote-opa"
                value={newRouteFallback}
                onChange={(e) => setNewRouteFallback(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <label className="text-xs font-medium">Match Conditions (JSON)</label>
              <textarea
                className="flex min-h-[60px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                placeholder='e.g. {"agent_ids": ["my-agent"]}'
                value={newRouteMatchCond}
                onChange={(e) => setNewRouteMatchCond(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <label className="text-xs font-medium">Failure Behavior</label>
              <select
                className="flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm"
                value={newRouteFailure}
                onChange={(e) => setNewRouteFailure(e.target.value as any)}
              >
                <option value="deny">Deny</option>
                <option value="allow">Allow</option>
                <option value="fallback">Fallback</option>
              </select>
            </div>
            <button
              onClick={handleAddRoute}
              className="h-9 px-4 py-2 bg-primary text-primary-foreground rounded-md text-sm font-medium hover:opacity-90 w-full mt-2"
            >
              Create Route
            </button>
          </div>
        </div>

        <div>
          <h4 className="text-sm font-medium mb-4">Simulate Routing</h4>
          <div className="space-y-4 bg-muted/20 p-4 rounded-lg">
            <p className="text-xs text-muted-foreground">Test how the router processes a sample request.</p>
            <button
              onClick={handleSimulate}
              className="h-9 px-4 py-2 bg-secondary text-secondary-foreground rounded-md text-sm font-medium hover:opacity-90 w-full"
            >
              Run Simulation
            </button>
            {simulateResult && (
              <pre className="mt-4 p-3 bg-background border rounded text-xs overflow-auto max-h-[300px]">
                {JSON.stringify(simulateResult, null, 2)}
              </pre>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
