import { useState, useEffect } from "react";
import { PlayCircle, CheckCircle2, XCircle } from "lucide-react";
import { PolicyApi } from "../services/api";
import type { PolicyDraft } from "../services/types";

export function Simulator() {
  const [policies, setPolicies] = useState<PolicyDraft[]>([]);
  const [selectedPolicyId, setSelectedPolicyId] = useState<string>("");
  const [action, setAction] = useState("read");
  const [resource, setResource] = useState("document:123");
  const [principal, setPrincipal] = useState("user:alice");
  const [contextStr, setContextStr] = useState("{\n  \"device\": \"trusted\"\n}");
  const [result, setResult] = useState<any>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    PolicyApi.list().then(data => {
      setPolicies(data);
      if (data.length > 0) {
        setSelectedPolicyId(data[0].policy_id);
      }
    }).catch(console.error);
  }, []);

  useEffect(() => {
    const policy = policies.find(p => p.policy_id === selectedPolicyId);
    if (policy && policy.source?.kind === "raw_text" && policy.source.language === "cedar" && policy.source.text) {
      const text = policy.source.text;
      const principalMatch = text.match(/principal\s*==\s*([A-Za-z0-9_]+::"[^"]+")/);
      if (principalMatch) setPrincipal(principalMatch[1]);
      
      const actionMatch = text.match(/action\s*==\s*([A-Za-z0-9_]+::"[^"]+")/);
      if (actionMatch) setAction(actionMatch[1]);
      
      const resourceMatch = text.match(/resource\s*==\s*([A-Za-z0-9_]+::"[^"]+")/);
      if (resourceMatch) setResource(resourceMatch[1]);

      const contextMatches = text.matchAll(/context\.([a-zA-Z0-9_]+)\s*==\s*"([^"]+)"/g);
      const ctxObj: any = {};
      let foundContext = false;
      for (const match of contextMatches) {
        ctxObj[match[1]] = match[2];
        foundContext = true;
      }
      if (foundContext) {
        setContextStr(JSON.stringify(ctxObj, null, 2));
      } else {
        setContextStr("{}");
      }
    }
  }, [selectedPolicyId, policies]);

  const handleSimulate = async () => {
    if (!selectedPolicyId) {
      setError("Please select a policy to simulate");
      return;
    }

    setLoading(true);
    setError(null);
    setResult(null);

    let ctx = {};
    try {
      ctx = JSON.parse(contextStr);
    } catch (e) {
      setError("Invalid JSON context");
      setLoading(false);
      return;
    }

    const payload = {
      action,
      resource,
      principal,
      context: ctx,
    };

    try {
      const res = await PolicyApi.simulate(selectedPolicyId, payload);
      setResult(res);
    } catch (e: any) {
      setError(e.message || String(e));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-6 max-w-4xl">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold tracking-tight flex items-center gap-2">
            <PlayCircle className="h-6 w-6 text-primary" /> Policy Simulator
          </h2>
          <p className="text-muted-foreground">
            Evaluate requests against a policy draft using dry-run mode.
          </p>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-6">
        <div className="glass p-6 rounded-xl space-y-4 border">
          <h3 className="font-medium">Request Context</h3>
          
          <div className="space-y-3">
            <div>
              <label className="text-xs font-medium text-muted-foreground">Target Policy</label>
              <select 
                value={selectedPolicyId} 
                onChange={e => setSelectedPolicyId(e.target.value)}
                className="mt-1 w-full rounded-md border bg-background px-3 py-2 text-sm"
              >
                {policies.length === 0 && <option value="">No policies available</option>}
                {policies.map(p => (
                  <option key={p.policy_id} value={p.policy_id}>{p.name} ({p.policy_id})</option>
                ))}
              </select>
            </div>
            <div>
              <label className="text-xs font-medium text-muted-foreground">Principal ID</label>
              <input value={principal} onChange={e => setPrincipal(e.target.value)}
                className="mt-1 w-full rounded-md border bg-background px-3 py-2 text-sm" />
            </div>
            <div>
              <label className="text-xs font-medium text-muted-foreground">Action</label>
              <input value={action} onChange={e => setAction(e.target.value)}
                className="mt-1 w-full rounded-md border bg-background px-3 py-2 text-sm" />
            </div>
            <div>
              <label className="text-xs font-medium text-muted-foreground">Resource ID</label>
              <input value={resource} onChange={e => setResource(e.target.value)}
                className="mt-1 w-full rounded-md border bg-background px-3 py-2 text-sm" />
            </div>
            <div>
              <label className="text-xs font-medium text-muted-foreground">Additional Context (JSON)</label>
              <textarea value={contextStr} onChange={e => setContextStr(e.target.value)} rows={5}
                className="mt-1 w-full rounded-md border bg-black/30 px-3 py-2 font-mono text-xs" spellCheck={false} />
            </div>
            
            {error && <div className="text-xs text-red-400 p-2 bg-red-400/10 rounded">{error}</div>}

            <button 
              onClick={handleSimulate}
              disabled={loading || policies.length === 0}
              className="w-full flex justify-center items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
            >
              <PlayCircle className="h-4 w-4" />
              {loading ? "Simulating..." : "Run Simulation"}
            </button>
          </div>
        </div>

        <div className="glass p-6 rounded-xl space-y-4 border h-full flex flex-col">
          <h3 className="font-medium">Decision Result</h3>
          
          <div className="flex-1 bg-black/40 rounded-lg p-4 font-mono text-xs overflow-auto border border-white/5 relative">
            {!result && !loading && (
              <div className="absolute inset-0 flex items-center justify-center text-muted-foreground">
                Run a simulation to see the result
              </div>
            )}
            {loading && (
              <div className="absolute inset-0 flex items-center justify-center text-muted-foreground animate-pulse">
                Evaluating policy...
              </div>
            )}
            {result && (
              <div className="space-y-4">
                <div className="flex items-center gap-2 text-lg">
                  {result.allowed ? (
                    <span className="flex items-center gap-2 text-emerald-400"><CheckCircle2 className="h-5 w-5"/> ALLOW</span>
                  ) : (
                    <span className="flex items-center gap-2 text-red-400"><XCircle className="h-5 w-5"/> DENY</span>
                  )}
                </div>
                <div className="text-muted-foreground whitespace-pre-wrap">
                  {JSON.stringify(result, null, 2)}
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
