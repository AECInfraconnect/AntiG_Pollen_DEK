import { useState, useEffect } from "react";
import { Package, RefreshCw, CheckCircle, Clock } from "lucide-react";
import { BundleApi } from "../services/api";

export function Bundles() {
  const [bundles, setBundles] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [syncing, setSyncing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const load = async () => {
    setLoading(true);
    try {
      // In a real implementation this would fetch from GET /bundles
      // Since local control plane might not have a /bundles endpoint returning an array 
      // of historical bundles unless it's Mock-Cloud, we mock or fetch it.
      const data = await BundleApi.list();
      setBundles(Array.isArray(data) ? data : []);
    } catch (e: any) {
      // If endpoint doesn't exist on local DEK yet, just show empty
      console.warn("Bundles endpoint issue:", e);
      setBundles([]);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    load();
  }, []);

  const handleSync = async () => {
    setSyncing(true);
    setError(null);
    try {
      await BundleApi.sync();
      await load();
    } catch (e: any) {
      setError(`Sync failed: ${e.message || String(e)}`);
    } finally {
      setSyncing(false);
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold tracking-tight flex items-center gap-2">
            <Package className="h-6 w-6 text-primary" /> Bundles &amp; Deployments
          </h2>
          <p className="text-muted-foreground">
            Manage deployed policy bundles and synchronize with the control plane.
          </p>
        </div>
        <button 
          onClick={handleSync}
          disabled={syncing}
          className="flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50 transition-colors shadow-lg shadow-primary/20"
        >
          <RefreshCw className={`h-4 w-4 ${syncing ? "animate-spin" : ""}`} />
          {syncing ? "Syncing..." : "Sync Now"}
        </button>
      </div>

      {error && (
        <div className="rounded-md bg-red-500/10 px-4 py-3 text-sm text-red-400 border border-red-500/20">
          {error}
        </div>
      )}

      <div className="glass rounded-xl overflow-hidden border">
        <table className="w-full text-sm text-left">
          <thead className="bg-muted/50 text-muted-foreground">
            <tr>
              <th className="px-6 py-4 font-medium">Bundle ID</th>
              <th className="px-6 py-4 font-medium">Version</th>
              <th className="px-6 py-4 font-medium">Status</th>
              <th className="px-6 py-4 font-medium">Deployed At</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-border">
            {loading ? (
              <tr>
                <td colSpan={4} className="px-6 py-8 text-center text-muted-foreground">
                  Loading deployments...
                </td>
              </tr>
            ) : bundles.length === 0 ? (
              <tr>
                <td colSpan={4} className="px-6 py-8 text-center text-muted-foreground">
                  No bundles found or endpoint unavailable in this profile.
                </td>
              </tr>
            ) : bundles.map((b, i) => (
              <tr key={b.bundle_id || i} className="hover:bg-muted/30 transition-colors">
                <td className="px-6 py-4 font-medium font-mono text-xs">
                  {b.bundle_id || 'unknown'}
                </td>
                <td className="px-6 py-4 text-muted-foreground">
                  {b.version || 'v1.0'}
                </td>
                <td className="px-6 py-4">
                  <span className={`inline-flex items-center gap-1.5 rounded-full px-2 py-1 text-xs font-medium ${
                    i === 0 ? 'bg-emerald-500/10 text-emerald-500' : 'bg-muted text-muted-foreground'
                  }`}>
                    {i === 0 ? <CheckCircle className="h-3 w-3" /> : <Clock className="h-3 w-3" />}
                    {i === 0 ? 'Active' : 'Archived'}
                  </span>
                </td>
                <td className="px-6 py-4 text-muted-foreground">
                  {b.deployed_at ? new Date(b.deployed_at).toLocaleString() : 'Just now'}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
