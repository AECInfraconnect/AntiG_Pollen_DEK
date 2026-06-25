import { useState, useEffect } from "react";
import { Plus, X, UploadCloud, Trash2, Pencil } from "lucide-react";
import { PolicyApi } from "../services/api";
import type { PolicyDraft, PolicyType } from "../services/api";
import { MasterDetailLayout } from "../components/layout/MasterDetailLayout";
import { EntityCard } from "../components/shared/EntityCard";
import type { EntityCardProps } from "../components/shared/EntityCard";

export function Policies() {
  const [policies, setPolicies] = useState<PolicyDraft[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedPolicyId, setSelectedPolicyId] = useState<string | null>(null);
  const [editorState, setEditorState] = useState<{
    mode: "create" | "edit" | "view";
    policy?: PolicyDraft;
  } | null>(null);
  const [publishing, setPublishing] = useState<string | null>(null);
  const [toast, setToast] = useState<string | null>(null);

  const reload = () => {
    setLoading(true);
    PolicyApi.list()
      .then(setPolicies)
      .catch(console.error)
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    reload();
  }, []);

  const onDelete = async (policyId: string) => {
    if (!confirm(`Are you sure you want to delete policy ${policyId}?`)) return;
    try {
      await PolicyApi.delete(policyId);
      setToast(`Deleted ${policyId}`);
      if (selectedPolicyId === policyId) setSelectedPolicyId(null);
      reload();
    } catch (e) {
      setToast(`Delete failed: ${String(e)}`);
    } finally {
      setTimeout(() => setToast(null), 5000);
    }
  };

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

  const mappedCards: EntityCardProps[] = policies.map((p) => {
    const targetCount =
      p.targets.agent_ids.length +
      p.targets.tool_ids.length +
      p.targets.resource_ids.length +
      p.targets.entity_ids.length;

    return {
      id: p.policy_id,
      kind: "policy",
      title: p.name,
      subtitle: p.policy_id,
      status: p.meta.status === "active" || p.meta.status === "published" ? "active" 
            : p.meta.status === "draft" ? "needs_approval" 
            : "unknown",
      statusLabel: p.meta.status,
      summary: `Targets: ${targetCount}`,
      chips: [
        { label: p.policy_type, tone: "neutral" }
      ],
      lastUpdatedAt: p.meta.updated_at,
    };
  });

  const selectedPolicy = policies.find((p) => p.policy_id === selectedPolicyId);

  const masterContent = (
    <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
      {loading ? (
        <div className="text-muted-foreground p-4">Loading policies...</div>
      ) : mappedCards.length === 0 ? (
        <div className="text-muted-foreground p-4">No policies yet. Create one to get started.</div>
      ) : (
        mappedCards.map((card) => (
          <EntityCard
            key={card.id}
            {...card}
            selected={selectedPolicyId === card.id}
            onClick={() => setSelectedPolicyId(card.id)}
          />
        ))
      )}
    </div>
  );

  const detailContent = selectedPolicy ? (
    <div className="space-y-6">
      <div>
        <h3 className="text-xl font-bold">{selectedPolicy.name}</h3>
        <p className="text-sm text-muted-foreground font-mono mt-1">
          {selectedPolicy.policy_id}
        </p>
      </div>

      <div className="flex gap-2">
         <span className="inline-flex items-center rounded-md bg-muted px-2 py-1 text-xs font-medium text-muted-foreground ring-1 ring-inset ring-border">
          Type: {selectedPolicy.policy_type}
        </span>
        <span className="inline-flex items-center rounded-md bg-muted px-2 py-1 text-xs font-medium text-muted-foreground ring-1 ring-inset ring-border">
          Status: {selectedPolicy.meta.status}
        </span>
      </div>

      <div className="space-y-4">
        <div className="p-4 bg-muted/50 rounded-lg border">
          <h4 className="text-sm font-semibold mb-2">Policy Source</h4>
          <pre className="text-xs font-mono overflow-x-auto p-4 bg-black/50 text-green-400 rounded border">
            {selectedPolicy.source?.kind === "raw_text" ? selectedPolicy.source.text : JSON.stringify(selectedPolicy.source, null, 2)}
          </pre>
        </div>

        <div className="p-4 bg-muted/50 rounded-lg border">
          <h4 className="text-sm font-semibold mb-2">Details</h4>
          <dl className="grid grid-cols-2 gap-x-4 gap-y-2 text-sm">
            <dt className="text-muted-foreground">Created By</dt>
            <dd>{selectedPolicy.meta.created_by}</dd>
            <dt className="text-muted-foreground">Source</dt>
            <dd>{selectedPolicy.meta.source}</dd>
            <dt className="text-muted-foreground">Targets</dt>
            <dd>
              {selectedPolicy.targets.agent_ids.length} agents, {selectedPolicy.targets.tool_ids.length} tools
            </dd>
          </dl>
        </div>
      </div>
      
      <div className="flex flex-wrap gap-2 justify-end">
        <button
          onClick={() => setEditorState({ mode: "edit", policy: selectedPolicy })}
          disabled={selectedPolicy.meta.source === "cloud_sync" || selectedPolicy.meta.created_by !== "local-admin"}
          className="px-4 py-2 bg-muted text-foreground border border-border rounded-md text-sm font-medium hover:bg-muted/80 disabled:opacity-50 inline-flex items-center gap-2"
        >
          <Pencil className="h-4 w-4" /> Edit
        </button>
        <button
          onClick={() => onPublish(selectedPolicy.policy_id)}
          disabled={publishing === selectedPolicy.policy_id}
          className="px-4 py-2 bg-blue-500/10 text-blue-500 border border-blue-500/20 rounded-md text-sm font-medium hover:bg-blue-500/20 disabled:opacity-50 inline-flex items-center gap-2"
        >
          <UploadCloud className="h-4 w-4" /> 
          {publishing === selectedPolicy.policy_id ? "Publishing..." : "Publish"}
        </button>
        <button 
          onClick={() => onDelete(selectedPolicy.policy_id)}
          disabled={selectedPolicy.meta.source === "cloud_sync" || selectedPolicy.meta.created_by !== "local-admin"}
          className="px-4 py-2 bg-red-500/10 text-red-500 border border-red-500/20 rounded-md text-sm font-medium hover:bg-red-500/20 disabled:opacity-50 inline-flex items-center gap-2"
        >
          <Trash2 className="h-4 w-4" /> Delete
        </button>
      </div>
    </div>
  ) : null;

  return (
    <>
      <MasterDetailLayout
        title="Policy Enforcer"
        description="Author, compile, and publish signed policy bundles to the local workspace."
        actions={
          <button
            onClick={() => setEditorState({ mode: "create" })}
            className="flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors shadow-[0_0_15px_rgba(124,58,237,0.3)]"
          >
            <Plus className="h-4 w-4" /> New Policy
          </button>
        }
        masterContent={
          <>
            {toast && (
              <div className="glass rounded-lg border px-4 py-3 text-sm mb-4">{toast}</div>
            )}
            {masterContent}
          </>
        }
        detailContent={detailContent}
        onCloseDetail={() => setSelectedPolicyId(null)}
      />

      {editorState && (
        <PolicyEditor
          mode={editorState.mode}
          policy={editorState.policy}
          onClose={() => setEditorState(null)}
          onCreated={() => {
            setEditorState(null);
            reload();
          }}
        />
      )}
    </>
  );
}

function PolicyEditor({
  mode,
  policy,
  onClose,
  onCreated,
}: {
  mode: "create" | "edit" | "view";
  policy?: PolicyDraft;
  onClose: () => void;
  onCreated: () => void;
}) {
  const DEFAULT_TEMPLATES: Record<PolicyType, string> = {
    cedar: "permit(principal, action, resource);",
    rego: 'package authz\n\ndefault allow = false\n\nallow {\n  input.action == "read"\n}',
    open_fga: "model\n  schema 1.1\ntype user\ntype document\n  relations\n    define viewer: [user]",
    pii_redaction: "",
    route: "",
    composite: "",
  };

  const [name, setName] = useState(policy?.name ?? "");
  const [type, setType] = useState<PolicyType>(policy?.policy_type ?? "cedar");
  const initialText =
    policy?.source?.kind === "raw_text"
      ? policy.source.text
      : DEFAULT_TEMPLATES["cedar"];
  const [text, setText] = useState(initialText);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [isTyping, setIsTyping] = useState(mode !== "create");

  useEffect(() => {
    if (mode === "create" && !isTyping) {
      setText(DEFAULT_TEMPLATES[type] || "");
    }
  }, [type, mode, isTyping]);

  const handleTextChange = (newText: string) => {
    setText(newText);
    setIsTyping(true);
    if (mode === "create") {
      if (newText.includes("permit(") || newText.includes("forbid(")) {
        setType("cedar");
      } else if (newText.includes("package ")) {
        setType("rego");
      } else if (newText.includes("model") && newText.includes("type ")) {
        setType("open_fga");
      }
    }
  };

  const readOnly = mode === "view";
  const langFor: Record<string, string> = {
    cedar: "cedar",
    rego: "rego",
    open_fga: "fga",
  };

  const save = async () => {
    setSaving(true);
    setError(null);

    if (type === "rego" && !text.includes("package")) {
      setError("Invalid OPA/Rego policy: Must contain a package declaration.");
      setSaving(false);
      return;
    }
    if (type === "cedar" && !text.includes("permit") && !text.includes("forbid")) {
      setError("Invalid Cedar policy: Must contain at least one permit or forbid statement.");
      setSaving(false);
      return;
    }
    if (type === "open_fga" && !text.includes("model")) {
      setError("Invalid OpenFGA model: Must contain a model declaration.");
      setSaving(false);
      return;
    }

    const now = new Date().toISOString();
    const policy_id = policy?.policy_id ?? `pol-${Date.now()}`;
    const draft: PolicyDraft = {
      meta: policy?.meta ?? {
        schema_version: "1.0",
        tenant_id: "local",
        workspace_id: "default",
        environment_id: "local",
        created_at: now,
        updated_at: now,
        created_by: "local-admin",
        updated_by: "local-admin",
        source: "manual",
        status: "draft",
        tags: [],
      },
      policy_id,
      name,
      description: policy?.description,
      policy_type: type,
      targets: policy?.targets ?? {
        agent_ids: [],
        tool_ids: [],
        resource_ids: [],
        entity_ids: [],
        route_ids: [],
      },
      source: { kind: "raw_text", language: langFor[type] ?? "text", text },
      compile_options: policy?.compile_options ?? { fail_on_warnings: true },
    };
    try {
      if (mode === "edit") {
        draft.meta.updated_at = now;
        await PolicyApi.update(policy_id, draft);
      } else {
        await PolicyApi.create(draft);
      }
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
          <h3 className="text-lg font-semibold">{mode === "create" ? "New Policy" : mode === "edit" ? "Edit Policy" : "View Policy"}</h3>
          <button onClick={onClose} className="text-muted-foreground hover:text-foreground">
            <X className="h-5 w-5" />
          </button>
        </div>

        <div className="grid grid-cols-2 gap-4">
          <div>
            <label htmlFor="policy-name" className="text-xs font-medium text-muted-foreground">Name</label>
            <input id="policy-name" value={name} onChange={(e) => setName(e.target.value)} disabled={readOnly} className="mt-1 w-full rounded-md border bg-transparent px-3 py-2 text-sm disabled:opacity-50" placeholder="e.g. pol-net-deny" />
          </div>
          <div>
            <label htmlFor="policy-engine" className="text-xs font-medium text-muted-foreground">Engine</label>
            <select id="policy-engine" value={type} onChange={(e) => setType(e.target.value as PolicyType)} disabled={readOnly || mode === "edit"} className="mt-1 w-full rounded-md border bg-background px-3 py-2 text-sm disabled:opacity-50">
              <option value="cedar">Cedar</option>
              <option value="rego">OPA / Rego</option>
              <option value="open_fga">OpenFGA</option>
            </select>
          </div>
        </div>

        <div>
          <label htmlFor="policy-source" className="text-xs font-medium text-muted-foreground">Policy source</label>
          <textarea id="policy-source" value={text} onChange={(e) => handleTextChange(e.target.value)} rows={10} disabled={readOnly} className="mt-1 w-full rounded-md border bg-black/30 px-3 py-2 font-mono text-xs disabled:opacity-50" spellCheck={false} />
        </div>

        {error && (
          <div className="rounded-md bg-red-500/10 px-3 py-2 text-xs text-red-400">{error}</div>
        )}

        <div className="flex justify-end gap-2">
          <button onClick={onClose} className="rounded-md border px-4 py-2 text-sm hover:bg-muted/50">{readOnly ? "Close" : "Cancel"}</button>
          {!readOnly && (
            <button onClick={save} disabled={saving || !name} className="rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50">
              {saving ? "Saving..." : "Save"}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
