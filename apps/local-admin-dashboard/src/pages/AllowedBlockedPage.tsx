import { useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import {
  AlertTriangle,
  CheckCircle2,
  Eye,
  FileKey,
  ListChecks,
  Plus,
  ShieldCheck,
  ShieldX,
} from "lucide-react";
import { CapabilityApi, PolicyApi } from "../services/api";
import type { LocalCapabilitySnapshotV2, PolicyDraft } from "../services/types";
import {
  SIMPLE_RULE_PRESETS,
  buildUserCapabilityMatrix,
  categoryLabel,
  capabilityTone,
  labelize,
} from "../features/user-activity/userActivityModel";
import type { SimpleRulePreset } from "../features/user-activity/types";
import { cn } from "@/lib/utils";

const toneClass: Record<string, string> = {
  success: "border-emerald-500/25 bg-emerald-500/10 text-emerald-700",
  info: "border-blue-500/25 bg-blue-500/10 text-blue-700",
  warning: "border-amber-500/25 bg-amber-500/10 text-amber-700",
  neutral: "border-border bg-background text-muted-foreground",
};

function behaviorIcon(behavior: SimpleRulePreset["behavior"]) {
  if (behavior === "block") return ShieldX;
  if (behavior === "ask_first") return AlertTriangle;
  if (behavior === "allow") return ShieldCheck;
  return Eye;
}

function PolicyCard({ policy }: { policy: PolicyDraft }) {
  const status = policy.meta?.status ?? "draft";
  return (
    <article className="rounded-lg border bg-card/60 p-4">
      <div className="flex items-start gap-3">
        <div className="rounded-lg bg-primary/10 p-2 text-primary">
          <FileKey className="h-4 w-4" />
        </div>
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-start justify-between gap-2">
            <h3 className="text-sm font-semibold">{policy.name}</h3>
            <span className="rounded-full border bg-background px-2 py-0.5 text-[11px] capitalize text-muted-foreground">
              {labelize(status)}
            </span>
          </div>
          <p className="mt-1 text-xs leading-5 text-muted-foreground">
            {policy.description ||
              "Technical policy available in Advanced Mode."}
          </p>
          <div className="mt-3 flex flex-wrap gap-1.5">
            <span className="rounded-full border px-2 py-0.5 text-[11px]">
              {labelize(policy.policy_type)}
            </span>
            <span className="rounded-full border px-2 py-0.5 text-[11px]">
              {policy.targets?.agent_ids?.length ?? 0} AI apps
            </span>
            <span className="rounded-full border px-2 py-0.5 text-[11px]">
              {policy.targets?.resource_ids?.length ?? 0} data targets
            </span>
          </div>
        </div>
      </div>
    </article>
  );
}

function PresetCard({
  preset,
  snapshot,
}: {
  preset: SimpleRulePreset;
  snapshot: LocalCapabilitySnapshotV2 | null;
}) {
  const Icon = behaviorIcon(preset.behavior);
  const matrix = buildUserCapabilityMatrix(snapshot);
  const capability =
    matrix.find((item) => item.category === preset.category) ??
    matrix.find((item) => item.id === "unknown");
  const tone = capability ? capabilityTone(capability.status) : "neutral";
  const statusText = capability
    ? capability.can_block
      ? "Can block here"
      : capability.can_ask_first
        ? "Can ask first"
        : capability.can_watch
          ? "Can watch now"
          : "Needs setup"
    : "Needs setup";

  return (
    <article className="rounded-lg border bg-card/60 p-4">
      <div className="flex items-start gap-3">
        <div className={cn("rounded-lg p-2", toneClass[tone])}>
          <Icon className="h-4 w-4" />
        </div>
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-start justify-between gap-2">
            <h3 className="text-sm font-semibold">{preset.label}</h3>
            <span
              className={cn(
                "rounded-full border px-2 py-0.5 text-[11px]",
                toneClass[tone],
              )}
            >
              {statusText}
            </span>
          </div>
          <p className="mt-1 text-xs leading-5 text-muted-foreground">
            {preset.description}
          </p>
          <div className="mt-3 flex flex-wrap gap-1.5">
            <span className="rounded-full border bg-background px-2 py-0.5 text-[11px] text-muted-foreground">
              {preset.category === "unknown"
                ? "All activity"
                : categoryLabel(preset.category)}
            </span>
            <span className="rounded-full border bg-background px-2 py-0.5 text-[11px] text-muted-foreground">
              {labelize(preset.behavior)}
            </span>
          </div>
          {capability && !capability.can_block && (
            <p className="mt-3 rounded-md border border-blue-500/20 bg-blue-500/10 p-3 text-xs text-blue-700">
              {capability.can_watch
                ? "Pollek can observe this now and guide the AI app setting until blocking is available."
                : capability.why}
            </p>
          )}
          <div className="mt-3">
            <Link
              to={`/protect?intent=${encodeURIComponent(preset.intent)}`}
              className="inline-flex h-8 items-center gap-2 rounded-md border px-3 text-xs text-primary hover:bg-primary/10"
            >
              <Plus className="h-3.5 w-3.5" />
              Start with this rule
            </Link>
          </div>
        </div>
      </div>
    </article>
  );
}

export function AllowedBlockedPage() {
  const [policies, setPolicies] = useState<PolicyDraft[]>([]);
  const [snapshot, setSnapshot] = useState<LocalCapabilitySnapshotV2 | null>(
    null,
  );
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    Promise.all([
      PolicyApi.list().catch(() => [] as PolicyDraft[]),
      CapabilityApi.getSnapshotV2("desktop_simple").catch(() => null),
    ])
      .then(([policyList, capabilitySnapshot]) => {
        setPolicies(policyList);
        setSnapshot(capabilitySnapshot);
      })
      .finally(() => setLoading(false));
  }, []);

  const activePolicies = useMemo(
    () =>
      policies.filter((policy) =>
        ["published", "active", "approved", "validated"].includes(
          policy.meta?.status ?? "",
        ),
      ),
    [policies],
  );
  const draftPolicies = policies.filter(
    (policy) => !activePolicies.includes(policy),
  );

  return (
    <div className="space-y-5">
      <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
        <div>
          <h2 className="flex items-center gap-2 text-2xl font-bold tracking-tight">
            <ListChecks className="h-6 w-6 text-primary" />
            Allowed & Blocked
          </h2>
          <p className="text-sm text-muted-foreground">
            Choose what each AI app can do, and see when Pollek can only watch.
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          <Link
            to="/protect"
            className="inline-flex h-9 items-center gap-2 rounded-md bg-primary px-3 text-sm text-primary-foreground hover:bg-primary/90"
          >
            <Plus className="h-4 w-4" />
            Create rule
          </Link>
          <Link
            to="/policies"
            className="inline-flex h-9 items-center gap-2 rounded-md border px-3 text-sm hover:bg-muted"
          >
            <FileKey className="h-4 w-4" />
            Advanced policies
          </Link>
        </div>
      </div>

      <section className="grid gap-3 sm:grid-cols-3">
        <div className="rounded-lg border bg-card/60 p-4">
          <div className="text-2xl font-semibold">{activePolicies.length}</div>
          <p className="mt-1 text-xs text-muted-foreground">Active rules</p>
        </div>
        <div className="rounded-lg border bg-card/60 p-4">
          <div className="text-2xl font-semibold">{draftPolicies.length}</div>
          <p className="mt-1 text-xs text-muted-foreground">Draft rules</p>
        </div>
        <div className="rounded-lg border bg-card/60 p-4">
          <div className="text-2xl font-semibold">
            {SIMPLE_RULE_PRESETS.length}
          </div>
          <p className="mt-1 text-xs text-muted-foreground">Plain presets</p>
        </div>
      </section>

      <section className="space-y-3">
        <div className="flex items-center gap-2">
          <CheckCircle2 className="h-4 w-4 text-emerald-600" />
          <h3 className="text-sm font-semibold">Active rules</h3>
        </div>
        {loading ? (
          <div className="rounded-lg border border-dashed p-8 text-center text-sm text-muted-foreground">
            Loading rules...
          </div>
        ) : activePolicies.length > 0 ? (
          <div className="grid gap-3 xl:grid-cols-2">
            {activePolicies.map((policy) => (
              <PolicyCard key={policy.policy_id} policy={policy} />
            ))}
          </div>
        ) : (
          <div className="rounded-lg border bg-card/60 p-4 text-sm text-muted-foreground">
            No active rules yet.
          </div>
        )}
      </section>

      <section className="space-y-3">
        <div className="flex items-center gap-2">
          <Plus className="h-4 w-4 text-primary" />
          <h3 className="text-sm font-semibold">Suggested rules</h3>
        </div>
        <div className="grid gap-3 xl:grid-cols-2">
          {SIMPLE_RULE_PRESETS.map((preset) => (
            <PresetCard key={preset.id} preset={preset} snapshot={snapshot} />
          ))}
        </div>
      </section>

      {draftPolicies.length > 0 && (
        <section className="space-y-3">
          <div className="flex items-center gap-2">
            <FileKey className="h-4 w-4 text-muted-foreground" />
            <h3 className="text-sm font-semibold">Draft technical policies</h3>
          </div>
          <div className="grid gap-3 xl:grid-cols-2">
            {draftPolicies.map((policy) => (
              <PolicyCard key={policy.policy_id} policy={policy} />
            ))}
          </div>
        </section>
      )}
    </div>
  );
}
