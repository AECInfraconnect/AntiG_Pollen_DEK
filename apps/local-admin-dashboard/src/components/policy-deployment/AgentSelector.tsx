import { useState, useEffect } from "react";
import { RegistryApi } from "../../services/api";
import type { AiAgent, BlackboxAiProvider } from "../../services/api";

export function AgentSelector({
  selectedAgents,
  onSelectionChange,
  selectedProviders,
  onProviderSelectionChange,
  onDataLoaded,
}: {
  selectedAgents: string[];
  onSelectionChange: (agents: string[]) => void;
  selectedProviders?: string[];
  onProviderSelectionChange?: (providers: string[]) => void;
  onDataLoaded?: (hasAgents: boolean) => void;
}) {
  const [agents, setAgents] = useState<AiAgent[]>([]);
  const [providers, setProviders] = useState<BlackboxAiProvider[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    Promise.all([
      RegistryApi.listAgents().catch(() => []),
      RegistryApi.listBlackboxAiProviders().catch(() => []),
    ]).then(([agentsData, providersData]) => {
      setAgents(agentsData);
      setProviders(providersData);
      setLoading(false);
      if (onDataLoaded) {
        onDataLoaded(agentsData.length > 0 || providersData.length > 0);
      }
    });
  }, [onDataLoaded]);

  const handleToggleAgent = (agentId: string) => {
    if (selectedAgents.includes(agentId)) {
      onSelectionChange(selectedAgents.filter((a) => a !== agentId));
    } else {
      onSelectionChange([...selectedAgents, agentId]);
    }
  };

  const handleToggleProvider = (providerId: string) => {
    if (!selectedProviders || !onProviderSelectionChange) return;
    if (selectedProviders.includes(providerId)) {
      onProviderSelectionChange(selectedProviders.filter((p) => p !== providerId));
    } else {
      onProviderSelectionChange([...selectedProviders, providerId]);
    }
  };

  const handleToggleAll = () => {
    const allSelected = 
      selectedAgents.length === agents.length && 
      (selectedProviders?.length || 0) === providers.length;
      
    if (allSelected) {
      onSelectionChange([]);
      if (onProviderSelectionChange) onProviderSelectionChange([]);
    } else {
      onSelectionChange(agents.map((a) => a.agent_id));
      if (onProviderSelectionChange) onProviderSelectionChange(providers.map((p) => p.provider_id));
    }
  };

  return (
    <div className="space-y-4">
      <h4 className="font-medium">Select Target Agents & Providers</h4>
      <div className="p-4 border rounded bg-muted/20 space-y-4">
        {loading ? (
          <div className="text-sm text-muted-foreground animate-pulse">
            Loading...
          </div>
        ) : agents.length === 0 && providers.length === 0 ? (
          <div className="text-sm text-muted-foreground">
            No agents or providers found. Please register one first.
          </div>
        ) : (
          <>
            <label className="flex items-center gap-2 border-b pb-3 mb-3 cursor-pointer">
              <input
                type="checkbox"
                checked={selectedAgents.length === agents.length && (selectedProviders?.length || 0) === providers.length && (agents.length > 0 || providers.length > 0)}
                onChange={handleToggleAll}
                className="w-4 h-4 rounded border-gray-300"
              />
              <span className="font-medium">All Compatible Agents & Providers</span>
            </label>
            <div className="space-y-2 max-h-48 overflow-y-auto">
              {agents.map((agent) => (
                <label
                  key={`agent-${agent.agent_id}`}
                  className="flex items-center gap-3 p-2 hover:bg-muted/50 rounded-md cursor-pointer transition-colors"
                >
                  <input
                    type="checkbox"
                    checked={selectedAgents.includes(agent.agent_id)}
                    onChange={() => handleToggleAgent(agent.agent_id)}
                    className="w-4 h-4 rounded border-gray-300"
                  />
                  <div className="flex flex-col">
                    <span className="text-sm font-medium">{agent.name} <span className="ml-2 text-[10px] uppercase tracking-wider bg-primary/10 text-primary px-1.5 py-0.5 rounded">Local Agent</span></span>
                    <span className="text-xs text-muted-foreground">
                      {agent.agent_type}
                    </span>
                  </div>
                </label>
              ))}
              
              {providers.map((provider) => (
                <label
                  key={`provider-${provider.provider_id}`}
                  className="flex items-center gap-3 p-2 hover:bg-muted/50 rounded-md cursor-pointer transition-colors"
                >
                  <input
                    type="checkbox"
                    checked={selectedProviders?.includes(provider.provider_id) || false}
                    onChange={() => handleToggleProvider(provider.provider_id)}
                    className="w-4 h-4 rounded border-gray-300"
                  />
                  <div className="flex flex-col">
                    <span className="text-sm font-medium">{provider.name} <span className="ml-2 text-[10px] uppercase tracking-wider bg-purple-500/10 text-purple-600 px-1.5 py-0.5 rounded">Blackbox AI</span></span>
                    <span className="text-xs text-muted-foreground">
                      {provider.provider_type}
                    </span>
                  </div>
                </label>
              ))}
            </div>
          </>
        )}
      </div>
    </div>
  );
}
