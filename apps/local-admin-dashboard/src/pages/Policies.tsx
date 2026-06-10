import { useState, useEffect } from "react";
import { FileKey, Plus, UploadCloud, X } from "lucide-react";
import { PolicyApi } from "../services/api";
import type { PolicyDraft, PolicyType } from "../services/api";

const TYPE_BADGE: Record<PolicyType, string> = {
  cedar: "bg-blue-500/15 text-blue-400",
  rego: "bg-purple-500/15 text-purple-400",
  open_fga: "bg-emerald-500/15 text-emerald-400",
  pii_redaction: "bg-amber-500/15 text-amber-400",
  route: "bg-slate-500/15 text-slate-400",
  composite: "bg-pink-500/15 text-pink-400",
};

const STATUS_BADGE: Record<string, string> = {
  draft: "bg-slate-500/15 text-slate-400",
  published: "bg-emerald-500/15 text-emerald-400",
  active: "bg-emerald-500/15 text-emerald-400",
  compiled: "bg-blue-500/15 text-blue-400",
};

export function Policies() {
  const [policies, setPolicies] = useState<PolicyDraft[]>([]);
  const [loading, setLoading] = useState(true);
  const [showEditor, setShowEditor] = useState(false);
  const [publishing, setPublishing] = useState<string | null>(null);
  const [toast, setToast] = useState<string | null>(null);

  const reload = () =>
    PolicyApi.list().then(setPolicies).catch(console.error).finally(() => setLoading(false));

  useEffect(() => { reload(); }, []);

  const onPublish = async (policyId: string) => {
    setPublishing(policyId);
    try {
      const r = await PolicyApi.publish(policyId);
      setToast(`Published ${policyId} → bundle ${r.bundle_id} (build #${r.build_number})`);
      reload();
    } catch (e) {
      setToast(`Publish failed: ${String(e)}`);
    } finally {
      setPublishing(null);
      setTimeout(() => setToast(null), 5000);
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold tracking-tight flex items-center gap-2">
            <FileKey className="h-6 w-6 text-primary" /> Policy Enforcer
          </h2>
          <p className="text-muted-foreground">
            Author, compile, and publish signed policy bundles to the local workspace.
          </p>
        </div>
        <button
          onClick={() => setShowEditor(true)}
          className="flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors shadow-lg shadow-primary/20"
        >
          <Plus className="h-4 w-4" /> New Policy
        </button>
      </div>

      {toast && (
        <div className="glass rounded-lg border px-4 py-3 text-sm">{toast}</div>
      )}

      <div className="glass rounded-xl overflow-hidden border">
        <table className="w-full text-sm text-left">
          <thead className="bg-muted/50 text-muted-foreground">
            <tr>
              <th className="px-6 py-4 font-medium">Name</th>
              <th className="px-6 py-4 font-medium">Type</th>
              <th className="px-6 py-4 font-medium">Status</th>
              <th className="px-6 py-4 font-medium">Targets</th>
              <th className="px-6 py-4 font-medium text-right">Actions</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-border">
            {loading ? (
              <tr><td colSpan={5} className="px-6 py-8 text-center text-muted-foreground">Loading policies...</td></tr>
            ) : policies.length === 0 ? (
              <tr><td colSpan={5} className="px-6 py-8 text-center text-muted-foreground">No policies yet. Create one to get started.</td></tr>
            ) : policies.map((p) => {
              const targetCount =
                p.targets.agent_ids.length + p.targets.tool_ids.length +
                p.targets.resource_ids.length + p.targets.entity_ids.length;
              return (
                <tr key={p.policy_id} className="hover:bg-muted/30 transition-colors">
                  <td className="px-6 py-4">
                    <div className="font-medium">{p.name}</div>
                    <div className="text-xs text-muted-foreground">{p.policy_id}</div>
                  </td>
                  <td className="px-6 py-4">
                    <span className={`rounded-full px-2 py-1 text-xs font-medium ${TYPE_BADGE[p.policy_type] ?? ""}`}>
                      {p.policy_type}
                    </span>
                  </td>
                  <td className="px-6 py-4">
                    <span className={`rounded-full px-2 py-1 text-xs font-medium ${STATUS_BADGE[p.meta.status] ?? "bg-slate-500/15 text-slate-400"}`}>
                      {p.meta.status}
                    </span>
                  </td>
                  <td className="px-6 py-4 text-muted-foreground">{targetCount} target(s)</td>
                  <td className="px-6 py-4 text-right">
                    <button
                      onClick={() => onPublish(p.policy_id)}
                      disabled={publishing === p.policy_id}
                      className="inline-flex items-center gap-1.5 rounded-md border px-3 py-1.5 text-xs font-medium hover:bg-muted/50 disabled:opacity-50"
                    >
                      <UploadCloud className="h-3.5 w-3.5" />
                      {publishing === p.policy_id ? "Publishing..." : "Publish"}
                    </button>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>

      {showEditor && <PolicyEditor onClose={() => setShowEditor(false)} onCreated={() => { setShowEditor(false); reload(); }} />}
    </div>
  );
}

function PolicyEditor({ onClose, onCreated }: { onClose: () => void; onCreated: () => void }) {
  const [name, setName] = useState("");
  const [type, setType] = useState<PolicyType>("cedar");
  const [text, setText] = useState('permit(principal, action, resource);');
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const langFor: Record<string, string> = { cedar: "cedar", rego: "rego", open_fga: "fga" };

  const save = async () => {
    setSaving(true); setError(null);
    const now = new Date().toISOString();
    const policy_id = `pol-${Date.now()}`;
    const draft: PolicyDraft = {
      meta: {
        schema_version: "1.0", tenant_id: "local", workspace_id: "default", environment_id: "local",
        created_at: now, updated_at: now, created_by: "local-admin", updated_by: "local-admin",
        source: "manual", status: "draft", tags: [],
      },
      policy_id, name, description: undefined, policy_type: type,
      targets: { agent_ids: [], tool_ids: [], resource_ids: [], entity_ids: [], route_ids: [] },
      source: { kind: "raw_text", language: langFor[type] ?? "text", text },
      compile_options: { fail_on_warnings: true },
    };
    try {
      await PolicyApi.create(draft);
      onCreated();
    } catch (e) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm" onClick={onClose}>
      <div className="glass w-full max-w-2xl rounded-xl border p-6 space-y-4" onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center justify-between">
          <h3 className="text-lg font-semibold">New Policy</h3>
          <button onClick={onClose} className="text-muted-foreground hover:text-foreground"><X className="h-5 w-5" /></button>
        </div>

        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="text-xs font-medium text-muted-foreground">Name</label>
            <input value={name} onChange={(e) => setName(e.target.value)}
              className="mt-1 w-full rounded-md border bg-transparent px-3 py-2 text-sm" placeholder="Allow safe tools" />
          </div>
          <div>
            <label className="text-xs font-medium text-muted-foreground">Engine</label>
            <select value={type} onChange={(e) => setType(e.target.value as PolicyType)}
              className="mt-1 w-full rounded-md border bg-transparent px-3 py-2 text-sm">
              <option value="cedar">Cedar</option>
              <option value="rego">OPA / Rego</option>
              <option value="open_fga">OpenFGA</option>
            </select>
          </div>
        </div>

        <div>
          <label className="text-xs font-medium text-muted-foreground">Policy source (compiled on the control plane, not the DEK)</label>
          <textarea value={text} onChange={(e) => setText(e.target.value)} rows={10}
            className="mt-1 w-full rounded-md border bg-black/30 px-3 py-2 font-mono text-xs" spellCheck={false} />
        </div>

        {error && <div className="rounded-md bg-red-500/10 px-3 py-2 text-xs text-red-400">{error}</div>}

        <div className="flex justify-end gap-2">
          <button onClick={onClose} className="rounded-md border px-4 py-2 text-sm hover:bg-muted/50">Cancel</button>
          <button onClick={save} disabled={saving || !name}
            className="rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50">
            {saving ? "Saving..." : "Save draft"}
          </button>
        </div>
      </div>
    </div>
  );
}
