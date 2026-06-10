import { useState } from "react";
import { PlayCircle, CheckCircle2, XCircle } from "lucide-react";
import { PolicyApi } from "../services/api";

export function Simulator() {
  const [action, setAction] = useState("read");
  const [resource, setResource] = useState("document:123");
  const [principal, setPrincipal] = useState("user:alice");
  const [contextStr, setContextStr] = useState("{\n  \"device\": \"trusted\"\n}");
  const [result, setResult] = useState<any>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSimulate = async () => {
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
      resource: { resource_type: "item", resource_id: resource },
      principal: { id: principal, roles: [] },
      context: ctx,
    };

    try {
      const res = await PolicyApi.simulate(payload);
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
            Evaluate requests against the current compiled policy bundle.
          </p>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-6">
        <div className="glass p-6 rounded-xl space-y-4 border">
          <h3 className="font-medium">Request Context</h3>
          
          <div className="space-y-3">
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
              disabled={loading}
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
                Evaluating policies...
              </div>
            )}
            {result && (
              <div className="space-y-4">
                <div className="flex items-center gap-2 text-lg">
                  {result.allow ? (
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
