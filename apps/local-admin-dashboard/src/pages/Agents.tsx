import { useState, useEffect } from "react";
import { Plus } from "lucide-react";
import { RegistryApi } from "../services/api";
import type { AiAgent } from "../services/api";
import { MasterDetailLayout } from "../components/layout/MasterDetailLayout";
import { EntityCard } from "../components/shared/EntityCard";
import type { EntityCardProps } from "../components/shared/EntityCard";

export function Agents({ hideHeader = false }: { hideHeader?: boolean }) {
  const [agents, setAgents] = useState<AiAgent[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedAgentId, setSelectedAgentId] = useState<string | null>(null);

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
    if (
      !confirm(
        "Are you sure you want to delete this agent? Note: Make sure no active policies depend on it."
      )
    )
      return;
    try {
      await RegistryApi.deleteAgent(id);
      setSelectedAgentId(null);
      fetchAgents();
    } catch (e) {
      console.error("Failed to delete agent:", e);
      alert("Failed to delete agent");
    }
  };

  const mappedCards: EntityCardProps[] = agents.map((agent) => ({
    id: agent.agent_id,
    kind: "agent",
    title: agent.name,
    subtitle: agent.agent_id,
    status:
      agent.enforcement_mode === "Enforce"
        ? "active"
        : agent.enforcement_mode === "Observe"
        ? "observe_only"
        : agent.enforcement_mode === "Shadow"
        ? "partial"
        : "unknown",
    statusLabel: agent.enforcement_mode || "Not Enforceable",
    summary: `Version: ${agent.runtime.version || "Unknown"}`,
    chips: [{ label: agent.runtime.runtime_name || "Unknown", tone: "neutral" }],
    lastUpdatedAt: agent.meta.updated_at,
  }));

  const selectedAgent = agents.find((a) => a.agent_id === selectedAgentId);

  const masterContent = (
    <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
      {loading ? (
        <div className="text-muted-foreground p-4">Loading agents...</div>
      ) : mappedCards.length === 0 ? (
        <div className="text-muted-foreground p-4">No agents registered.</div>
      ) : (
        mappedCards.map((card) => (
          <EntityCard
            key={card.id}
            {...card}
            selected={selectedAgentId === card.id}
            onClick={() => setSelectedAgentId(card.id)}
          />
        ))
      )}
    </div>
  );

  const detailContent = selectedAgent ? (
    <div className="space-y-6">
      <div>
        <h3 className="text-xl font-bold">{selectedAgent.name}</h3>
        <p className="text-sm text-muted-foreground font-mono mt-1">
          {selectedAgent.agent_id}
        </p>
      </div>

      <div className="space-y-4">
        <div className="p-4 bg-muted/50 rounded-lg border">
          <h4 className="text-sm font-semibold mb-2">Capabilities</h4>
          <ul className="text-sm space-y-1 text-muted-foreground">
            {selectedAgent.capabilities?.map((cap: string) => (
              <li key={cap} className="flex items-center gap-2">
                <div className="h-1.5 w-1.5 rounded-full bg-primary/50" />
                {cap}
              </li>
            )) || <li>No specific capabilities</li>}
          </ul>
        </div>

        <div className="p-4 bg-muted/50 rounded-lg border">
          <h4 className="text-sm font-semibold mb-2">Raw JSON</h4>
          <pre className="text-xs font-mono overflow-x-auto p-2 bg-background rounded border">
            {JSON.stringify(selectedAgent, null, 2)}
          </pre>
        </div>
      </div>

      <div className="flex gap-2 justify-end">
        <a
          href="/wizard"
          className="px-4 py-2 bg-primary/10 text-primary border border-primary/20 rounded-md text-sm font-medium hover:bg-primary/20"
        >
          Apply Policy
        </a>
        <button
          onClick={() => deleteAgent(selectedAgent.agent_id)}
          className="px-4 py-2 bg-red-500/10 text-red-500 border border-red-500/20 rounded-md text-sm font-medium hover:bg-red-500/20"
        >
          Delete
        </button>
      </div>
    </div>
  ) : null;

  return (
    <MasterDetailLayout
      title={hideHeader ? "" : "AI Agents"}
      description={
        hideHeader
          ? undefined
          : "Manage authorized AI agents and client identities in the local workspace."
      }
      actions={
        hideHeader ? null : (
          <a
            href="/discovery"
            className="flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors shadow-[0_0_15px_rgba(124,58,237,0.3)]"
          >
            <Plus className="h-4 w-4" />
            Discover & Register
          </a>
        )
      }
      masterContent={masterContent}
      detailContent={detailContent}
      onCloseDetail={() => setSelectedAgentId(null)}
    />
  );
}
