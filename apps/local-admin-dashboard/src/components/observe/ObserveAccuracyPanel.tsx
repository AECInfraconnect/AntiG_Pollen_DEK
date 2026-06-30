import { useEffect, useMemo, useState } from "react";
import {
  CheckCircle2,
  FileText,
  KeyRound,
  LockKeyhole,
  RefreshCw,
  ShieldCheck,
  Trash2,
} from "lucide-react";
import { toast } from "sonner";
import { cn } from "@/lib/utils";
import { Button } from "../ui/Button";
import { Dialog } from "../ui/Dialog";
import { useConfirm } from "../ui/ConfirmDialog";
import {
  ObserveAccuracyApi,
  type ObserveAccuracyResponse,
  type ObserveCredentialRequest,
  type ObserveInputKind,
} from "../../services/api";

type Props = {
  className?: string;
  compact?: boolean;
  forceDialogKind?: ObserveInputKind | null;
  onForceDialogClosed?: () => void;
  onChanged?: () => void;
};

const primaryKinds: ObserveInputKind[] = [
  "local_usage_log_path",
  "provider_usage_key",
];

const kindLabels: Record<ObserveInputKind, string> = {
  provider_usage_key: "Provider usage key",
  local_usage_log_path: "Local usage log",
  cloud_read_role: "Cloud read role",
  oauth_read_token: "OAuth read token",
  proxy_ca_trust: "Proxy trust",
  provider_admin_write: "Provider admin write",
};

const riskClass: Record<string, string> = {
  low: "border-emerald-500/25 bg-emerald-500/10 text-emerald-700",
  medium: "border-amber-500/25 bg-amber-500/10 text-amber-700",
  high: "border-red-500/25 bg-red-500/10 text-red-700",
};

export function ObserveAccuracyPanel({
  className,
  compact = false,
  forceDialogKind,
  onForceDialogClosed,
  onChanged,
}: Props) {
  const { confirm } = useConfirm();
  const [loading, setLoading] = useState(false);
  const [accuracy, setAccuracy] = useState<ObserveAccuracyResponse | null>(
    null,
  );
  const [dialogKind, setDialogKind] = useState<ObserveInputKind | null>(null);

  const load = async () => {
    setLoading(true);
    try {
      setAccuracy(await ObserveAccuracyApi.get());
    } catch (error) {
      console.error(error);
      toast.error("Failed to load observe accuracy inputs");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void load();
  }, []);

  useEffect(() => {
    if (forceDialogKind) setDialogKind(forceDialogKind);
  }, [forceDialogKind]);

  const requestsByKind = useMemo(() => {
    const map = new Map<ObserveInputKind, ObserveCredentialRequest>();
    for (const request of accuracy?.available_requests ?? []) {
      map.set(request.kind, request);
    }
    return map;
  }, [accuracy]);
  const visibleSuggestions = compact
    ? (accuracy?.suggested_local_log_paths ?? []).slice(0, 3)
    : (accuracy?.suggested_local_log_paths ?? []);

  const revoke = async (inputId: string, label: string) => {
    const ok = await confirm({
      title: "Remove observe accuracy input?",
      description: `Pollek will forget ${label}. Any exact usage based on that local input will stop on the next observe run, but already recorded ledger events remain as history.`,
      confirmText: "Remove input",
      cancelText: "Keep it",
      danger: true,
    });
    if (!ok) return;
    try {
      await ObserveAccuracyApi.revokeInput(inputId);
      toast.success("Observe accuracy input removed");
      await load();
      onChanged?.();
    } catch (error) {
      console.error(error);
      toast.error(error instanceof Error ? error.message : "Remove failed");
    }
  };

  const closeDialog = () => {
    setDialogKind(null);
    onForceDialogClosed?.();
  };

  return (
    <section className={cn("glass rounded-lg p-5", className)}>
      <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
        <div>
          <div className="flex items-center gap-2">
            <ShieldCheck className="h-4 w-4 text-primary" />
            <h3 className="font-semibold">Observe accuracy inputs</h3>
          </div>
          <p className="mt-1 max-w-3xl text-sm leading-6 text-muted-foreground">
            Pollek observes first. When you want better accuracy, add a narrow
            local log path or a read-only provider usage source. The UI will
            still label data honestly as estimated, response-usage exact, or
            provider-billed ready.
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          {primaryKinds.map((kind) => {
            const Icon = kind === "local_usage_log_path" ? FileText : KeyRound;
            return (
              <Button
                key={kind}
                type="button"
                variant={kind === "local_usage_log_path" ? "primary" : "outline"}
                size="sm"
                leftIcon={<Icon className="h-4 w-4" />}
                onClick={() => setDialogKind(kind)}
              >
                {kind === "local_usage_log_path"
                  ? "Add local log"
                  : "Connect usage key"}
              </Button>
            );
          })}
          <Button
            type="button"
            variant="ghost"
            size="icon"
            loading={loading}
            onClick={() => void load()}
            aria-label="Refresh observe accuracy inputs"
          >
            <RefreshCw className="h-4 w-4" />
          </Button>
        </div>
      </div>

      <div className="mt-4 grid gap-3 lg:grid-cols-3">
        {(accuracy?.ladder ?? []).map((level) => (
          <article key={level.level} className="rounded-lg border bg-card/60 p-3">
            <div className="flex items-center justify-between gap-2">
              <h4 className="text-sm font-semibold">{level.label}</h4>
              <span
                className={cn(
                  "rounded-full border px-2 py-0.5 text-[11px]",
                  level.status.includes("connected") ||
                    level.status.includes("strengthened")
                    ? "border-emerald-500/25 bg-emerald-500/10 text-emerald-700"
                    : level.status.includes("needs")
                      ? "border-amber-500/25 bg-amber-500/10 text-amber-700"
                      : "border-border bg-background text-muted-foreground",
                )}
              >
                {level.status.replace(/_/g, " ")}
              </span>
            </div>
            <p className="mt-2 text-xs leading-5 text-muted-foreground">
              {level.description}
            </p>
          </article>
        ))}
      </div>

      {!compact && accuracy?.active_level_label && (
        <div className="mt-3 rounded-lg border border-primary/20 bg-primary/5 p-3 text-sm">
          <span className="font-medium">Current level: </span>
          {accuracy.active_level_label}
        </div>
      )}

      <div className="mt-4 grid gap-3 lg:grid-cols-[1fr_0.9fr]">
        <div className="rounded-lg border bg-card/60 p-4">
          <div className="flex items-center justify-between gap-2">
            <h4 className="text-sm font-semibold">Connected local inputs</h4>
            <span className="rounded-full border bg-background px-2 py-0.5 text-[11px] text-muted-foreground">
              {accuracy?.inputs.length ?? 0}
            </span>
          </div>
          <div className="mt-3 space-y-2">
            {accuracy?.inputs.length ? (
              accuracy.inputs.map((input) => (
                <article
                  key={input.input_id}
                  className="rounded-lg border bg-background/60 p-3"
                >
                  <div className="flex flex-wrap items-start justify-between gap-2">
                    <div className="min-w-0">
                      <div className="flex flex-wrap items-center gap-2">
                        <span className="font-medium">{input.label}</span>
                        <span className="rounded-full border px-2 py-0.5 text-[11px] text-muted-foreground">
                          {kindLabels[input.kind]}
                        </span>
                      </div>
                      <p className="mt-1 break-words text-xs text-muted-foreground">
                        {input.redacted_preview}
                      </p>
                    </div>
                    <Button
                      type="button"
                      variant="ghost"
                      size="icon"
                      aria-label={`Remove ${input.label}`}
                      onClick={() => void revoke(input.input_id, input.label)}
                    >
                      <Trash2 className="h-4 w-4 text-red-500" />
                    </Button>
                  </div>
                  <div className="mt-2 flex flex-wrap gap-2 text-[11px] text-muted-foreground">
                    <span>{input.status.replace(/_/g, " ")}</span>
                    <span>{new Date(input.connected_at).toLocaleString()}</span>
                    <span className="font-mono">{input.fingerprint}</span>
                  </div>
                </article>
              ))
            ) : (
              <div className="rounded-lg border border-dashed p-4 text-sm text-muted-foreground">
                No user-supplied observe inputs yet. Add a narrow local usage
                log path first if your AI agent writes JSON usage metadata.
              </div>
            )}
          </div>
        </div>

        <div className="space-y-3">
          <div className="rounded-lg border bg-card/60 p-4">
            <h4 className="text-sm font-semibold">Suggested local paths</h4>
            <p className="mt-1 text-xs leading-5 text-muted-foreground">
              If you do not know where logs are, start here. Pollek checks
              common folders for local AI apps and marks whether the path
              exists on this computer.
            </p>
            <div className="mt-3 space-y-2">
              {visibleSuggestions.map((item) => (
                  <button
                    key={item.path}
                    type="button"
                    onClick={() => setDialogKind("local_usage_log_path")}
                    className="w-full cursor-pointer rounded-lg border bg-background/60 p-3 text-left hover:border-primary/40 hover:bg-primary/5"
                  >
                    <div className="flex flex-wrap items-center justify-between gap-2">
                      <span className="text-sm font-medium">{item.label}</span>
                      <span
                        className={cn(
                          "rounded-full border px-2 py-0.5 text-[11px]",
                          item.exists
                            ? "border-emerald-500/25 bg-emerald-500/10 text-emerald-700"
                            : "border-border bg-background text-muted-foreground",
                        )}
                      >
                        {item.exists ? "Found" : "Not found"}
                      </span>
                    </div>
                    <div className="mt-1 break-words text-xs text-muted-foreground">
                      {item.redacted_path}
                    </div>
                  </button>
                ))}
            </div>
          </div>
          <div className="rounded-lg border bg-card/60 p-4">
            <h4 className="flex items-center gap-2 text-sm font-semibold">
              <LockKeyhole className="h-4 w-4 text-primary" />
              Data handling
            </h4>
            <ul className="mt-3 space-y-2 text-xs leading-5 text-muted-foreground">
              {(accuracy?.data_handling ?? []).slice(0, 4).map((item) => (
                <li key={item} className="flex gap-2">
                  <CheckCircle2 className="mt-0.5 h-3.5 w-3.5 shrink-0 text-emerald-600" />
                  <span>{item}</span>
                </li>
              ))}
            </ul>
          </div>
          <div className="rounded-lg border bg-card/60 p-4">
            <h4 className="text-sm font-semibold">Next best steps</h4>
            <ul className="mt-3 space-y-2 text-xs leading-5 text-muted-foreground">
              {(accuracy?.next_steps ?? []).map((item) => (
                <li key={item}>{item}</li>
              ))}
            </ul>
          </div>
        </div>
      </div>

      {!compact && (
        <div className="mt-4 rounded-lg border bg-card/60 p-4">
          <h4 className="text-sm font-semibold">Future connector requests</h4>
          <div className="mt-3 grid gap-2 md:grid-cols-2 xl:grid-cols-4">
            {(accuracy?.available_requests ?? [])
              .filter((request) => !primaryKinds.includes(request.kind))
              .map((request) => (
                <button
                  key={request.kind}
                  type="button"
                  disabled={!request.supported_now}
                  onClick={() => setDialogKind(request.kind)}
                  className="cursor-not-allowed rounded-lg border bg-background/60 p-3 text-left opacity-70"
                >
                  <div className="flex items-center justify-between gap-2">
                    <span className="text-sm font-medium">{request.title}</span>
                    <span
                      className={cn(
                        "rounded-full border px-2 py-0.5 text-[11px]",
                        riskClass[request.risk_level] ??
                          "border-border bg-background text-muted-foreground",
                      )}
                    >
                      {request.risk_level}
                    </span>
                  </div>
                  <p className="mt-2 text-xs leading-5 text-muted-foreground">
                    {request.supported_now
                      ? request.required_scope
                      : "Design extension point; requires a connector/runtime before it can collect data."}
                  </p>
                </button>
              ))}
          </div>
        </div>
      )}

      <ConnectObserveInputDialog
        kind={dialogKind}
        request={dialogKind ? requestsByKind.get(dialogKind) : undefined}
        suggestions={accuracy?.suggested_local_log_paths ?? []}
        open={Boolean(dialogKind)}
        onClose={closeDialog}
        onSaved={async () => {
          await load();
          onChanged?.();
        }}
      />
    </section>
  );
}

function ConnectObserveInputDialog({
  kind,
  request,
  suggestions,
  open,
  onClose,
  onSaved,
}: {
  kind: ObserveInputKind | null;
  request?: ObserveCredentialRequest;
  suggestions: NonNullable<ObserveAccuracyResponse["suggested_local_log_paths"]>;
  open: boolean;
  onClose: () => void;
  onSaved: () => Promise<void> | void;
}) {
  const [value, setValue] = useState("");
  const [label, setLabel] = useState("");
  const [provider, setProvider] = useState("");
  const [scopeNote, setScopeNote] = useState("");
  const [ack, setAck] = useState(false);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (!open) return;
    setValue("");
    setLabel("");
    setProvider("");
    setScopeNote("");
    setAck(false);
  }, [open, kind]);

  if (!kind) return null;

  const isPath = kind === "local_usage_log_path";
  const disabled = !request?.supported_now;

  const submit = async () => {
    if (!request) return;
    setSaving(true);
    try {
      await ObserveAccuracyApi.storeInput({
        kind,
        input_value: value,
        label: label || request.title,
        provider: provider || undefined,
        scope_note: scopeNote || request.required_scope,
        consent_ack: ack,
        consent_statement: `I approved ${request.title} for ${request.required_scope}.`,
      });
      toast.success(
        isPath
          ? "Local usage log path connected. Run Observe Now to use it."
          : "Read-only usage source saved locally.",
      );
      await onSaved();
      onClose();
    } catch (error) {
      console.error(error);
      toast.error(error instanceof Error ? error.message : "Save failed");
    } finally {
      setSaving(false);
    }
  };

  return (
    <Dialog
      open={open}
      onClose={onClose}
      size="lg"
      title={request?.title ?? kindLabels[kind]}
      description={request?.why}
      footer={
        <>
          <Button type="button" variant="outline" onClick={onClose}>
            Cancel
          </Button>
          <Button
            type="button"
            loading={saving}
            disabled={disabled || !value.trim() || !ack}
            onClick={() => void submit()}
          >
            Save local input
          </Button>
        </>
      }
    >
      <div className="space-y-4">
        {request && (
          <div className="grid gap-3 md:grid-cols-2">
            <InfoBlock label="What Pollek asks for" value={request.what_we_ask} />
            <InfoBlock label="Required scope" value={request.required_scope} />
            <InfoBlock
              label="Least privilege"
              value={request.least_privilege_tip}
            />
            <InfoBlock
              label="Risk"
              value={`${request.risk_level} risk`}
              tone={request.risk_level}
            />
          </div>
        )}

        {disabled ? (
          <div className="rounded-lg border border-amber-500/30 bg-amber-500/10 p-3 text-sm text-amber-800">
            This request is a design extension point. Install or enable the
            matching connector/runtime before Pollek can use it.
          </div>
        ) : (
          <>
            <label className="block text-sm">
              <span className="font-medium">
                {isPath ? "Local log file or folder path" : "Read-only usage key"}
              </span>
              <input
                value={value}
                onChange={(event) => setValue(event.target.value)}
                type={isPath ? "text" : "password"}
                placeholder={
                  isPath
                    ? "C:\\Users\\you\\.codex\\sessions or /Users/you/.claude"
                    : "Read-only usage or billing key"
                }
                className="mt-1 h-10 w-full rounded-md border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
              />
            </label>
            {isPath && suggestions.length > 0 && (
              <div className="rounded-lg border bg-card/60 p-3">
                <div className="text-sm font-medium">
                  Suggested paths on this computer
                </div>
                <p className="mt-1 text-xs leading-5 text-muted-foreground">
                  Pick a found path if you are not sure. If none are found, run
                  Observe Now first or check the AI app settings for log/export
                  options.
                </p>
                <div className="mt-3 grid gap-2 md:grid-cols-2">
                  {suggestions.map((item) => (
                    <button
                      key={item.path}
                      type="button"
                      onClick={() => {
                        setValue(item.path);
                        setLabel(`${item.label} usage logs`);
                        setProvider(item.label.split(" ")[0] ?? "");
                      }}
                      className={cn(
                        "cursor-pointer rounded-lg border p-3 text-left text-xs hover:border-primary/40 hover:bg-primary/5",
                        item.exists
                          ? "bg-emerald-500/5"
                          : "bg-background/60 opacity-75",
                      )}
                    >
                      <div className="flex items-center justify-between gap-2">
                        <span className="font-medium">{item.label}</span>
                        <span
                          className={cn(
                            "rounded-full border px-2 py-0.5 text-[11px]",
                            item.exists
                              ? "border-emerald-500/25 bg-emerald-500/10 text-emerald-700"
                              : "border-border bg-background text-muted-foreground",
                          )}
                        >
                          {item.exists ? "Found" : "Not found"}
                        </span>
                      </div>
                      <div className="mt-1 break-words text-muted-foreground">
                        {item.redacted_path}
                      </div>
                      <div className="mt-1 text-muted-foreground">
                        {item.reason}
                      </div>
                    </button>
                  ))}
                </div>
              </div>
            )}
            <div className="grid gap-3 md:grid-cols-2">
              <label className="block text-sm">
                <span className="font-medium">Display label</span>
                <input
                  value={label}
                  onChange={(event) => setLabel(event.target.value)}
                  placeholder={request?.title ?? kindLabels[kind]}
                  className="mt-1 h-10 w-full rounded-md border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
                />
              </label>
              <label className="block text-sm">
                <span className="font-medium">Provider or app</span>
                <input
                  value={provider}
                  onChange={(event) => setProvider(event.target.value)}
                  placeholder={isPath ? "Codex, Claude, Gemini..." : "OpenAI, Anthropic..."}
                  className="mt-1 h-10 w-full rounded-md border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
                />
              </label>
            </div>
            <label className="block text-sm">
              <span className="font-medium">Scope note</span>
              <input
                value={scopeNote}
                onChange={(event) => setScopeNote(event.target.value)}
                placeholder={request?.required_scope}
                className="mt-1 h-10 w-full rounded-md border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
              />
            </label>
          </>
        )}

        <div className="rounded-lg border bg-card/60 p-3">
          <h4 className="text-sm font-semibold">Data handling</h4>
          <ul className="mt-2 space-y-1 text-xs leading-5 text-muted-foreground">
            {(request?.data_handling ?? []).map((item) => (
              <li key={item}>{item}</li>
            ))}
          </ul>
        </div>

        <label className="flex items-start gap-2 rounded-lg border bg-background/60 p-3 text-sm">
          <input
            type="checkbox"
            checked={ack}
            disabled={disabled}
            onChange={(event) => setAck(event.target.checked)}
            className="mt-1"
          />
          <span>
            I understand this input is local, revocable, and should use the
            narrowest read-only scope available. I am not entering a primary
            password or broad admin key.
          </span>
        </label>
      </div>
    </Dialog>
  );
}

function InfoBlock({
  label,
  value,
  tone,
}: {
  label: string;
  value: string;
  tone?: string;
}) {
  return (
    <div
      className={cn(
        "rounded-lg border bg-card/60 p-3 text-sm",
        tone ? riskClass[tone] : "",
      )}
    >
      <div className="text-xs font-medium uppercase text-muted-foreground">
        {label}
      </div>
      <div className="mt-1 leading-5">{value}</div>
    </div>
  );
}
