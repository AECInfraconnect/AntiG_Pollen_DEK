import { useState, useEffect } from "react";
import { Users, Plus } from "lucide-react";
import { RegistryApi } from "../services/api";
import type { AiAgent } from "../services/api";
import { RegisterControlBar } from "../components/RegisterControlBar";

export function Agents({ hideHeader = false }: { hideHeader?: boolean }) {
  const [agents, setAgents] = useState<AiAgent[]>([]);
  const [loading, setLoading] = useState(true);

  const fetchAgents = () => {
    setLoading(true);
    RegistryApi.listAgents()
      .then(setAgents)
      .catch(console.error)
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    fetchAgents();
  }, []);

  const deleteAgent = async (id: string) => {
    if (!confirm("Are you sure you want to delete this agent? Note: Make sure no active policies depend on it.")) return;
    try {
      await RegistryApi.deleteAgent(id);
      fetchAgents();
    } catch (e) {
      console.error("Failed to delete agent:", e);
      alert("Failed to delete agent");
    }
  };

  return (
    <div className="space-y-6">
      {!hideHeader && (
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-bold tracking-tight">AI Agents</h2>
            <p className="text-muted-foreground">
              Manage authorized AI agents and client identities in the local
              workspace.
            </p>
          </div>
          <a
            href="/discovery"
            className="flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors shadow-lg shadow-primary/20"
          >
            <Plus className="h-4 w-4" />
            Discover & Register
          </a>
        </div>
      )}

      <div className="glass rounded-xl overflow-hidden border">
        <table className="w-full text-sm text-left">
          <thead className="bg-muted/50 text-muted-foreground">
            <tr>
              <th className="px-6 py-4 font-medium">Agent Name</th>
              <th className="px-6 py-4 font-medium">Agent ID</th>
              <th className="px-6 py-4 font-medium">Status</th>
              <th className="px-6 py-4 font-medium">Version</th>
              <th className="px-6 py-4 font-medium">Last Seen</th>
              <th className="px-6 py-4 font-medium text-right">Actions</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-border">
            {loading ? (
              <tr>
                <td
                  colSpan={6}
                  className="px-6 py-8 text-center text-muted-foreground"
                >
                  Loading agents...
                </td>
              </tr>
            ) : agents.length === 0 ? (
              <tr>
                <td
                  colSpan={6}
                  className="px-6 py-8 text-center text-muted-foreground"
                >
                  No agents registered.
                </td>
              </tr>
            ) : (
              agents.map((agent) => (
                <tr
                  key={agent.agent_id}
                  className="hover:bg-muted/30 transition-colors"
                >
                  <td className="px-6 py-4">
                    <div className="flex items-center gap-3">
                      <div className="h-8 w-8 rounded-full bg-primary/10 flex items-center justify-center">
                        <Users className="h-4 w-4 text-primary" />
                      </div>
                      <span className="font-medium">{agent.name}</span>
                    </div>
                  </td>
                  <td className="px-6 py-4 text-muted-foreground font-mono text-xs">
                    {agent.agent_id}
                  </td>
                  <td className="px-6 py-4">
                    <span
                      className={`inline-flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs font-semibold ${
                        agent.enforcement_mode === "Enforce"
                          ? "bg-emerald-500/15 text-emerald-600 dark:text-emerald-400 border border-emerald-500/20 shadow-sm shadow-emerald-500/10"
                          : agent.enforcement_mode === "Observe"
                          ? "bg-amber-500/15 text-amber-600 dark:text-amber-400 border border-amber-500/20 shadow-sm shadow-amber-500/10"
                          : agent.enforcement_mode === "Shadow"
                          ? "bg-blue-500/15 text-blue-600 dark:text-blue-400 border border-blue-500/20 shadow-sm shadow-blue-500/10"
                          : "bg-muted text-muted-foreground border border-border"
                      }`}
                    >
                      <span
                        className={`h-1.5 w-1.5 rounded-full shadow-sm ${
                          agent.enforcement_mode === "Enforce" ? "bg-emerald-500 shadow-emerald-500/50" 
                          : agent.enforcement_mode === "Observe" ? "bg-amber-500 shadow-amber-500/50"
                          : agent.enforcement_mode === "Shadow" ? "bg-blue-500 shadow-blue-500/50"
                          : "bg-muted-foreground"
                        }`}
                      />
                      {agent.enforcement_mode || "Not Enforceable"}
                    </span>
                  </td>
                  <td className="px-6 py-4 text-muted-foreground">
                    {agent.runtime.version || "Unknown"}
                  </td>
                  <td className="px-6 py-4 text-muted-foreground">
                    {new Date(agent.meta.updated_at).toLocaleString()}
                  </td>
                  <td className="px-6 py-4 text-right">
                    <div className="flex justify-end gap-2">
                      <RegisterControlBar agentId={agent.agent_id} tenantId="local" onSuccess={() => window.location.reload()} />
                      <button
                        onClick={() => deleteAgent(agent.agent_id)}
                        className="px-3 py-1 text-xs text-red-500 bg-red-500/10 hover:bg-red-500/20 rounded border border-red-500/20 transition-colors"
                      >
                        Delete
                      </button>
                    </div>
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
