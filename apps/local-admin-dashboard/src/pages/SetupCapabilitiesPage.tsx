import { useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import {
  CheckCircle2,
  Eye,
  HelpCircle,
  Plug,
  RefreshCw,
  Shield,
  ShieldCheck,
  Wrench,
} from "lucide-react";
import { toast } from "sonner";
import { CapabilityApi } from "../services/api";
import type {
  LocalCapabilitySnapshotV2,
  RuntimeModeV2,
  SetupActionV2,
} from "../services/types";
import { useMode } from "../context/ModeContext";
import { appModeToRuntimeMode, isAdvanceMode } from "../lib/modes";
import {
  buildUserCapabilityMatrix,
  capabilityTone,
  formatDateTime,
  labelize,
} from "../features/user-activity/userActivityModel";
import type { UserCapabilityItem } from "../features/user-activity/types";
import { cn } from "@/lib/utils";

type DemoTarget = "host" | "windows" | "linux" | "macos";
type DemoProfile = "ready" | "observe_only" | "needs_setup";

const toneClass: Record<string, string> = {
  success: "border-emerald-500/25 bg-emerald-500/10 text-emerald-700",
  info: "border-blue-500/25 bg-blue-500/10 text-blue-700",
  warning: "border-amber-500/25 bg-amber-500/10 text-amber-700",
  neutral: "border-border bg-background text-muted-foreground",
};

function CapabilityPill({ label, active }: { label: string; active: boolean }) {
  return (
    <span
      className={cn(
        "rounded-full border px-2 py-0.5 text-[11px]",
        active
          ? "border-emerald-500/25 bg-emerald-500/10 text-emerald-700"
          : "border-border bg-background text-muted-foreground",
      )}
    >
      {label}
    </span>
  );
}

function UserCapabilityCard({
  item,
  setupActions,
}: {
  item: UserCapabilityItem;
  setupActions: SetupActionV2[];
}) {
  const tone = capabilityTone(item.status);
  const matchingActions = item.setup_action_ids
    .map((id) => setupActions.find((action) => action.action_id === id))
    .filter(Boolean) as SetupActionV2[];
  const Icon =
    item.status === "ready"
      ? ShieldCheck
      : item.status === "partial"
        ? Eye
        : item.status === "needs_setup"
          ? Wrench
          : HelpCircle;

  return (
    <article className="rounded-lg border bg-card/60 p-4">
      <div className="flex items-start gap-3">
        <div className={cn("rounded-lg p-2", toneClass[tone])}>
          <Icon className="h-4 w-4" />
        </div>
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <h3 className="text-sm font-semibold">{item.simple_label}</h3>
            <span
              className={cn(
                "rounded-full border px-2 py-0.5 text-[11px]",
                toneClass[tone],
              )}
            >
              {labelize(item.status)}
            </span>
          </div>
          <p className="mt-1 text-xs text-muted-foreground">
            {item.plain_description}
          </p>
          <div className="mt-3 flex flex-wrap gap-1.5">
            <CapabilityPill label="Watch" active={item.can_watch} />
            <CapabilityPill label="Warn" active={item.can_warn} />
            <CapabilityPill label="Ask first" active={item.can_ask_first} />
            <CapabilityPill label="Block" active={item.can_block} />
          </div>
          <p className="mt-3 text-xs leading-5 text-muted-foreground">
            {item.why}
          </p>
          {matchingActions.length > 0 && (
            <div className="mt-3 space-y-2">
              {matchingActions.slice(0, 2).map((action) => (
                <div
                  key={action.action_id}
                  className="rounded-md border border-amber-500/20 bg-amber-500/10 p-3 text-xs text-amber-800"
                >
                  <div className="font-medium">{action.title_en}</div>
                  <div className="mt-1 text-amber-800/80">
                    {action.detail_en}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </article>
  );
}

function SetupActionCard({ action }: { action: SetupActionV2 }) {
  return (
    <article className="rounded-lg border bg-card/60 p-4">
      <div className="flex items-start gap-3">
        <div className="rounded-lg bg-amber-500/10 p-2 text-amber-600">
          <Wrench className="h-4 w-4" />
        </div>
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-start justify-between gap-2">
            <h3 className="text-sm font-semibold">{action.title_en}</h3>
            <span className="rounded-full border bg-background px-2 py-0.5 text-[11px] text-muted-foreground">
              {action.estimated_minutes} min
            </span>
          </div>
          <p className="mt-1 text-xs leading-5 text-muted-foreground">
            {action.detail_en}
          </p>
          <div className="mt-3 flex flex-wrap gap-1.5">
            {action.requires_admin && (
              <span className="rounded-full border px-2 py-0.5 text-[11px]">
                Admin needed
              </span>
            )}
            {action.requires_restart && (
              <span className="rounded-full border px-2 py-0.5 text-[11px]">
                Restart needed
              </span>
            )}
            {action.safe_to_skip && (
              <span className="rounded-full border px-2 py-0.5 text-[11px]">
                Safe to skip
              </span>
            )}
          </div>
        </div>
      </div>
    </article>
  );
}

export function SetupCapabilitiesPage() {
  const { mode } = useMode();
  const runtimeMode: RuntimeModeV2 = appModeToRuntimeMode(mode);
  const showDemoControls = isAdvanceMode(mode);
  const [snapshot, setSnapshot] = useState<LocalCapabilitySnapshotV2 | null>(
    null,
  );
  const [loading, setLoading] = useState(true);
  const [demoTarget, setDemoTarget] = useState<DemoTarget>("host");
  const [demoProfile, setDemoProfile] = useState<DemoProfile>("ready");

  const load = async (refresh = false) => {
    setLoading(true);
    try {
      const demo =
        demoTarget === "host"
          ? undefined
          : { os: demoTarget, profile: demoProfile };
      const data = refresh
        ? await CapabilityApi.refreshSnapshotV2(runtimeMode, demo)
        : await CapabilityApi.getSnapshotV2(runtimeMode, demo);
      setSnapshot(data);
    } catch (error) {
      console.error(error);
      toast.error("Failed to load setup status");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void load(false);
  }, [runtimeMode, demoTarget, demoProfile]);

  const matrix = useMemo(() => buildUserCapabilityMatrix(snapshot), [snapshot]);
  const ready = matrix.filter((item) => item.status === "ready").length;
  const partial = matrix.filter((item) => item.status === "partial").length;
  const needsSetup = matrix.filter(
    (item) => item.status === "needs_setup",
  ).length;
  const safetyCapability = matrix.find((item) => item.category === "safety");
  const pluginCapability = matrix.find((item) => item.category === "plugins");

  return (
    <div className="space-y-5">
      <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
        <div>
          <h2 className="flex items-center gap-2 text-2xl font-bold tracking-tight">
            <Shield className="h-6 w-6 text-primary" />
            Setup
          </h2>
          <p className="text-sm text-muted-foreground">
            What Pollek can watch, warn about, ask before, or block on this
            computer.
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          {showDemoControls ? (
            <>
              <div className="inline-flex h-9 overflow-hidden rounded-md border bg-background">
                {(["host", "windows", "linux", "macos"] as DemoTarget[]).map(
                  (target) => (
                    <button
                      key={target}
                      type="button"
                      onClick={() => setDemoTarget(target)}
                      className={cn(
                        "px-3 text-sm capitalize hover:bg-muted",
                        demoTarget === target && "bg-muted text-foreground",
                      )}
                    >
                      {target}
                    </button>
                  ),
                )}
              </div>
              {demoTarget !== "host" && (
                <select
                  value={demoProfile}
                  onChange={(event) =>
                    setDemoProfile(event.target.value as DemoProfile)
                  }
                  className="h-9 rounded-md border bg-background px-3 text-sm"
                >
                  <option value="ready">Ready</option>
                  <option value="observe_only">Observe only</option>
                  <option value="needs_setup">Needs setup</option>
                </select>
              )}
            </>
          ) : (
            <span className="inline-flex h-9 items-center rounded-md border bg-background px-3 text-sm font-medium">
              This computer
            </span>
          )}
          <button
            type="button"
            onClick={() => void load(true)}
            disabled={loading}
            className="inline-flex h-9 items-center gap-2 rounded-md border px-3 text-sm hover:bg-muted disabled:opacity-50"
          >
            <RefreshCw className={cn("h-4 w-4", loading && "animate-spin")} />
            Check
          </button>
        </div>
      </div>

      <section className="grid gap-3 sm:grid-cols-2 xl:grid-cols-5">
        <div className="rounded-lg border bg-card/60 p-4 xl:col-span-2">
          <p className="text-xs font-medium text-muted-foreground">Computer</p>
          <div className="mt-1 text-lg font-semibold">
            {snapshot
              ? `${labelize(snapshot.os.family)} ${snapshot.os.version}`
              : "Checking local computer"}
          </div>
          <p className="mt-1 text-xs text-muted-foreground">
            {snapshot?.os.elevated ? "Elevated session" : "User-level session"}{" "}
            / {snapshot?.os.arch ?? "unknown arch"} /{" "}
            {formatDateTime(snapshot?.generated_at)}
          </p>
        </div>
        <div className="rounded-lg border bg-card/60 p-4">
          <div className="text-2xl font-semibold">{ready}</div>
          <p className="mt-1 text-xs text-muted-foreground">Can block</p>
        </div>
        <div className="rounded-lg border bg-card/60 p-4">
          <div className="text-2xl font-semibold">{partial}</div>
          <p className="mt-1 text-xs text-muted-foreground">Can watch now</p>
        </div>
        <div className="rounded-lg border bg-card/60 p-4">
          <div className="text-2xl font-semibold">{needsSetup}</div>
          <p className="mt-1 text-xs text-muted-foreground">Need setup</p>
        </div>
      </section>

      <section className="rounded-lg border border-emerald-500/20 bg-emerald-500/10 p-4">
        <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
          <div className="flex items-start gap-3">
            <div className="rounded-lg bg-emerald-500/15 p-2 text-emerald-700">
              <ShieldCheck className="h-4 w-4" />
            </div>
            <div>
              <h3 className="text-sm font-semibold text-emerald-900 dark:text-emerald-100">
                Prompt Guard setup
              </h3>
              <p className="mt-1 max-w-3xl text-sm leading-6 text-emerald-900/80 dark:text-emerald-100/80">
                Prompt Guard watches for prompt injection, secrets, PII, and
                unsafe prompt/output paths. If Pollek can only watch this
                category, also tighten the safety settings inside each AI app.
              </p>
              {safetyCapability && (
                <p className="mt-2 text-xs leading-5 text-emerald-900/75 dark:text-emerald-100/75">
                  Current computer: {safetyCapability.plain_description}
                </p>
              )}
            </div>
          </div>
          <div className="flex flex-wrap gap-2">
            <Link
              to="/protect?intent=enable_prompt_guard"
              className="inline-flex h-9 items-center gap-2 rounded-md bg-emerald-600 px-3 text-sm font-medium text-white hover:bg-emerald-700"
            >
              <ShieldCheck className="h-4 w-4" />
              Enable Prompt Guard
            </Link>
            <Link
              to="/activity?q=prompt"
              className="inline-flex h-9 items-center gap-2 rounded-md border border-emerald-500/25 bg-background/70 px-3 text-sm hover:bg-background"
            >
              <Eye className="h-4 w-4" />
              View safety activity
            </Link>
          </div>
        </div>
      </section>

      <section className="rounded-lg border border-blue-500/20 bg-blue-500/10 p-4">
        <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
          <div className="flex items-start gap-3">
            <div className="rounded-lg bg-blue-500/15 p-2 text-blue-700">
              <Plug className="h-4 w-4" />
            </div>
            <div>
              <h3 className="text-sm font-semibold text-blue-950 dark:text-blue-100">
                Plugins and connectors
              </h3>
              <p className="mt-1 max-w-3xl text-sm leading-6 text-blue-950/80 dark:text-blue-100/80">
                Some observe coverage needs a connector or plugin, such as a
                browser connector, Prompt Guard path, email connector,
                definition feed, or telemetry exporter. Pollek records plugin
                install, enable, disable, and uninstall events in Activity and
                History.
              </p>
              {pluginCapability && (
                <p className="mt-2 text-xs leading-5 text-blue-950/75 dark:text-blue-100/75">
                  Current computer: {pluginCapability.plain_description}
                </p>
              )}
            </div>
          </div>
          <div className="flex flex-wrap gap-2">
            <Link
              to="/plugin-marketplace"
              className="inline-flex h-9 items-center gap-2 rounded-md bg-blue-600 px-3 text-sm font-medium text-white hover:bg-blue-700"
            >
              <Plug className="h-4 w-4" />
              Open Marketplace
            </Link>
            <Link
              to="/activity?category=plugins"
              className="inline-flex h-9 items-center gap-2 rounded-md border border-blue-500/25 bg-background/70 px-3 text-sm hover:bg-background"
            >
              <Eye className="h-4 w-4" />
              View plugin activity
            </Link>
          </div>
        </div>
      </section>

      <section className="grid gap-3 xl:grid-cols-2">
        {matrix.map((item) => (
          <UserCapabilityCard
            key={item.id}
            item={item}
            setupActions={snapshot?.setup_actions ?? []}
          />
        ))}
      </section>

      <section className="space-y-3">
        <div className="flex items-center gap-2">
          <Wrench className="h-4 w-4 text-amber-600" />
          <h3 className="text-sm font-semibold">Setup actions</h3>
        </div>
        {(snapshot?.setup_actions.length ?? 0) > 0 ? (
          <div className="grid gap-3 xl:grid-cols-2">
            {snapshot!.setup_actions.map((action) => (
              <SetupActionCard key={action.action_id} action={action} />
            ))}
          </div>
        ) : (
          <div className="flex items-center gap-2 rounded-lg border bg-card/60 p-4 text-sm text-emerald-700">
            <CheckCircle2 className="h-4 w-4" />
            No setup action is required by the latest local snapshot.
          </div>
        )}
      </section>

      <section className="rounded-lg border bg-card/60 p-4">
        <h3 className="text-sm font-semibold">
          Agent settings are another path
        </h3>
        <p className="mt-1 text-sm text-muted-foreground">
          When Pollek can only watch an activity, the next step can still be
          useful: review the AI app settings, remove risky connectors, disable
          command execution, or narrow folder access inside that AI app.
        </p>
      </section>
    </div>
  );
}
