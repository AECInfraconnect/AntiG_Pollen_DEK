import { useState, useEffect } from "react";
import { DollarSign, RefreshCw } from "lucide-react";
import { ObservationApi, RegistryApi } from "../services/api";
import { RegisterControlBar } from "../components/RegisterControlBar";

type AgentUsage = {
  cost: number;
  tokens: number;
  name: string;
  kind?: string;
};

export function CostLedger() {
  const [loading, setLoading] = useState(false);
  const [totalCost, setTotalCost] = useState<number>(0);
  const [totalTokens, setTotalTokens] = useState<number>(0);
  const [breakdown, setBreakdown] = useState<Record<string, AgentUsage>>({});

  const fetchCost = async () => {
    setLoading(true);
    try {
      const data: any = await ObservationApi.getCostSummary();
      const [agents, candidates] = await Promise.all([
        RegistryApi.listAgents().catch(() => []),
        RegistryApi.listDiscoveryCandidates().catch(() => []),
      ]);
      const agentNames = new Map<string, { name: string; kind?: string }>();
      for (const agent of agents) {
        agentNames.set(agent.agent_id, {
          name: agent.name || agent.agent_id,
          kind: agent.agent_type,
        });
      }
      for (const candidate of candidates) {
        const displayName = candidate.display_name || candidate.candidate_id;
        agentNames.set(candidate.candidate_id, {
          name: displayName,
          kind: candidate.inferred_agent_type,
        });
        const suggestedAgentId = candidate.suggested_registration?.agent_id;
        if (suggestedAgentId) {
          agentNames.set(suggestedAgentId, {
            name: displayName,
            kind: candidate.inferred_agent_type,
          });
        }
      }

      setTotalCost(data.total_estimated_cost_usd || 0);
      setTotalTokens(data.total_tokens || 0);
      const usageBreakdown: Record<string, AgentUsage> = {};
      const costBreakdown = data.agent_breakdown || {};
      const tokenBreakdown = data.agent_token_breakdown || {};

      for (const [agentId, value] of Object.entries<any>(
        data.agent_usage_breakdown || {},
      )) {
        const agent = agentNames.get(agentId);
        usageBreakdown[agentId] = {
          cost: Number(value?.cost || 0),
          tokens: Number(value?.tokens || 0),
          name: agent?.name || agentId,
          kind: agent?.kind,
        };
      }

      for (const [agentId, cost] of Object.entries<any>(costBreakdown)) {
        const agent = agentNames.get(agentId);
        usageBreakdown[agentId] = usageBreakdown[agentId] || {
          cost: Number(cost || 0),
          tokens: Number(tokenBreakdown[agentId] || 0),
          name: agent?.name || agentId,
          kind: agent?.kind,
        };
      }

      setBreakdown(usageBreakdown);
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchCost();
  }, []);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">
            Token & Cost Ledger
          </h2>
          <p className="text-muted-foreground">
            Monitor estimated costs and token usage across all observed AI
            agents.
          </p>
        </div>
        <button
          onClick={fetchCost}
          disabled={loading}
          className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none ring-offset-background bg-primary text-primary-foreground hover:bg-primary/90 h-10 py-2 px-4"
        >
          {loading ? (
            <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
          ) : (
            <RefreshCw className="mr-2 h-4 w-4" />
          )}
          Refresh
        </button>
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <div className="glass rounded-xl p-6 relative overflow-hidden group">
          <div className="relative flex items-center justify-between">
            <span className="text-sm font-medium text-muted-foreground">
              Total Estimated Cost
            </span>
            <DollarSign className="h-4 w-4 text-muted-foreground" />
          </div>
          <div className="mt-4 flex items-baseline gap-2">
            <span className="text-3xl font-bold">${totalCost.toFixed(2)}</span>
            <span className="text-xs font-medium text-muted-foreground">
              USD
            </span>
          </div>
        </div>
        <div className="glass rounded-xl p-6 relative overflow-hidden group">
          <div className="relative flex items-center justify-between">
            <span className="text-sm font-medium text-muted-foreground">
              Total Tokens
            </span>
          </div>
          <div className="mt-4 flex items-baseline gap-2">
            <span className="text-3xl font-bold">
              {totalTokens.toLocaleString()}
            </span>
            <span className="text-xs font-medium text-muted-foreground">
              tokens
            </span>
          </div>
        </div>
      </div>

      <div className="glass rounded-xl p-6">
        <h3 className="font-semibold mb-4">Cost & Token Breakdown by Agent</h3>
        {Object.keys(breakdown).length === 0 ? (
          <div className="flex h-[200px] items-center justify-center rounded-md border border-dashed border-muted">
            <p className="text-sm text-muted-foreground">
              No observed provider usage yet. Use an API proxy, SDK wrapper, or
              approved browser extension to capture token and cost data.
            </p>
          </div>
        ) : (
          <div className="space-y-4">
            {Object.entries(breakdown)
              .sort(
                ([, a], [, b]) =>
                  b.cost - a.cost ||
                  b.tokens - a.tokens ||
                  a.name.localeCompare(b.name),
              )
              .map(([agentId, usage]) => (
                <div
                  key={agentId}
                  className="flex flex-col gap-3 p-4 border rounded-lg md:flex-row md:items-center md:justify-between"
                >
                  <div className="min-w-0">
                    <div className="truncate font-medium">{usage.name}</div>
                    <div className="mt-1 flex flex-wrap gap-2 text-xs text-muted-foreground">
                      {usage.kind && <span>{usage.kind}</span>}
                      <span className="font-mono">{agentId}</span>
                    </div>
                  </div>
                  <div className="flex items-center gap-4">
                    <span className="text-muted-foreground tabular-nums">
                      {usage.tokens.toLocaleString()} tokens
                    </span>
                    <span className="text-muted-foreground tabular-nums">
                      ${usage.cost.toFixed(2)}
                    </span>
                    <RegisterControlBar agentId={agentId} tenantId="local" />
                  </div>
                </div>
              ))}
          </div>
        )}
      </div>
    </div>
  );
}
