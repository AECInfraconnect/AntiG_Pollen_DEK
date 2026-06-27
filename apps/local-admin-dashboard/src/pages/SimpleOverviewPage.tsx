import { useCallback, useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import {
  Activity,
  Bot,
  CheckCircle2,
  Eye,
  FolderSearch,
  History,
  ListChecks,
  ShieldCheck,
  Wrench,
} from "lucide-react";
import { CapabilityApi, RegistryApi, type AiAgent } from "../services/api";
import { EntityGraphApi } from "../services/entityGraphApi";
import type { ActivityTimelineItem } from "../features/entity-graph/types";
import type { LocalCapabilitySnapshotV2 } from "../services/types";
import {
  buildUserCapabilityMatrix,
  capabilityTone,
  formatDateTime,
  summarizeActivities,
  toUserFriendlyActivity,
} from "../features/user-activity/userActivityModel";
import type { UserCapabilityItem } from "../features/user-activity/types";
import { cn } from "@/lib/utils";

const toneClass: Record<string, string> = {
  success: "border-emerald-500/25 bg-emerald-500/10 text-emerald-700",
  info: "border-blue-500/25 bg-blue-500/10 text-blue-700",
  warning: "border-amber-500/25 bg-amber-500/10 text-amber-700",
  neutral: "border-border bg-background text-muted-foreground",
};

function QuickAction({
  to,
  icon: Icon,
  title,
  description,
}: {
  to: string;
  icon: any;
  title: string;
  description: string;
}) {
  return (
    <Link
      to={to}
      className="rounded-lg border bg-card/60 p-4 transition hover:border-primary/40 hover:bg-muted/40"
    >
      <div className="flex items-start gap-3">
        <div className="rounded-lg bg-primary/10 p-2 text-primary">
          <Icon className="h-4 w-4" />
        </div>
        <div>
          <h3 className="text-sm font-semibold">{title}</h3>
          <p className="mt-1 text-xs leading-5 text-muted-foreground">
            {description}
          </p>
        </div>
      </div>
    </Link>
  );
}

function CapabilityMini({ item }: { item: UserCapabilityItem }) {
  const tone = capabilityTone(item.status);
  return (
    <div className="rounded-md border bg-background/60 p-3">
      <div className="flex items-center justify-between gap-3">
        <div className="min-w-0">
          <p className="truncate text-sm font-medium">{item.simple_label}</p>
          <p className="mt-1 text-xs text-muted-foreground">
            {item.can_block
              ? "Can block"
              : item.can_ask_first
                ? "Can ask first"
                : item.can_watch
                  ? "Can watch"
                  : "Needs setup"}
          </p>
        </div>
        <span
          className={cn(
            "shrink-0 rounded-full border px-2 py-0.5 text-[11px]",
            toneClass[tone],
          )}
        >
          {item.can_block ? "Block" : item.can_watch ? "Watch" : "Setup"}
        </span>
      </div>
    </div>
  );
}

export function SimpleOverviewPage() {
  const [agents, setAgents] = useState<AiAgent[]>([]);
  const [rawActivity, setRawActivity] = useState<ActivityTimelineItem[]>([]);
  const [snapshot, setSnapshot] = useState<LocalCapabilitySnapshotV2 | null>(
    null,
  );
  const [loading, setLoading] = useState(true);

  const load = useCallback(() => {
    setLoading(true);
    Promise.all([
      RegistryApi.listAgents().catch(() => [] as AiAgent[]),
      EntityGraphApi.getActivity({ limit: 100 }).catch(() => ({ items: [] })),
      CapabilityApi.getSnapshotV2("desktop_simple").catch(() => null),
    ])
      .then(([agentRows, activityPage, capabilitySnapshot]) => {
        setAgents(agentRows);
        setRawActivity(activityPage.items ?? []);
        setSnapshot(capabilitySnapshot);
      })
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const activity = useMemo(
    () => rawActivity.map(toUserFriendlyActivity),
    [rawActivity],
  );
  const summary = useMemo(() => summarizeActivities(activity), [activity]);
  const matrix = useMemo(() => buildUserCapabilityMatrix(snapshot), [snapshot]);
  const needsSetup = matrix.filter(
    (item) => item.status === "needs_setup",
  ).length;
  const recent = activity.slice(0, 4);

  return (
    <div className="space-y-5">
      <section className="rounded-lg border bg-card/60 p-5">
        <div className="flex flex-col gap-4 xl:flex-row xl:items-center xl:justify-between">
          <div>
            <h1 className="flex items-center gap-2 text-2xl font-bold tracking-tight">
              <Eye className="h-6 w-6 text-primary" />
              Watch what your AI apps do
            </h1>
            <p className="mt-2 max-w-3xl text-sm leading-6 text-muted-foreground">
              Pollek shows which AI apps are on this computer, what files,
              websites, apps, commands, and tools they touch, and when rules
              allowed, blocked, or only watched an action.
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            <Link
              to="/scan"
              className="inline-flex h-9 items-center gap-2 rounded-md bg-primary px-3 text-sm text-primary-foreground hover:bg-primary/90"
            >
              <FolderSearch className="h-4 w-4" />
              Find AI apps
            </Link>
            <Link
              to="/activity"
              className="inline-flex h-9 items-center gap-2 rounded-md border px-3 text-sm hover:bg-muted"
            >
              <Activity className="h-4 w-4" />
              View activity
            </Link>
          </div>
        </div>
      </section>

      <section className="grid gap-3 sm:grid-cols-2 xl:grid-cols-5">
        <div className="rounded-lg border bg-card/60 p-4">
          <div className="flex items-center gap-2 text-2xl font-semibold">
            <Bot className="h-5 w-5 text-primary" />
            {agents.length}
          </div>
          <p className="mt-1 text-xs text-muted-foreground">AI apps found</p>
        </div>
        <div className="rounded-lg border bg-card/60 p-4">
          <div className="text-2xl font-semibold">{summary.total}</div>
          <p className="mt-1 text-xs text-muted-foreground">Activities seen</p>
        </div>
        <div className="rounded-lg border bg-card/60 p-4">
          <div className="text-2xl font-semibold">{summary.watched}</div>
          <p className="mt-1 text-xs text-muted-foreground">Watched only</p>
        </div>
        <div className="rounded-lg border bg-card/60 p-4">
          <div className="text-2xl font-semibold">{summary.blocked}</div>
          <p className="mt-1 text-xs text-muted-foreground">Blocked</p>
        </div>
        <div className="rounded-lg border bg-card/60 p-4">
          <div className="text-2xl font-semibold">{needsSetup}</div>
          <p className="mt-1 text-xs text-muted-foreground">Need setup</p>
        </div>
      </section>

      <section className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
        <QuickAction
          to="/my-ai-apps"
          icon={Bot}
          title="My AI Apps"
          description="See the AI assistants found on this computer."
        />
        <QuickAction
          to="/activity"
          icon={Activity}
          title="AI Activity"
          description="Review files, websites, commands, tools, and model usage."
        />
        <QuickAction
          to="/allowed-blocked"
          icon={ListChecks}
          title="Allowed & Blocked"
          description="Choose watch, ask first, allow, or block behavior."
        />
        <QuickAction
          to="/setup"
          icon={Wrench}
          title="Setup"
          description="Check what this OS can watch or block today."
        />
      </section>

      <section className="grid gap-3 xl:grid-cols-[1.1fr_0.9fr]">
        <div className="rounded-lg border bg-card/60 p-4">
          <div className="flex items-center justify-between gap-3">
            <h2 className="flex items-center gap-2 text-sm font-semibold">
              <Activity className="h-4 w-4 text-primary" />
              Recent activity
            </h2>
            <Link
              to="/history"
              className="inline-flex items-center gap-1 text-xs text-primary hover:underline"
            >
              <History className="h-3.5 w-3.5" />
              History
            </Link>
          </div>
          <div className="mt-3 space-y-2">
            {loading && recent.length === 0 ? (
              <p className="rounded-md border border-dashed p-4 text-sm text-muted-foreground">
                Loading activity...
              </p>
            ) : recent.length > 0 ? (
              recent.map((item) => (
                <div
                  key={item.event_id}
                  className="rounded-md border bg-background/60 p-3"
                >
                  <div className="flex flex-wrap items-center justify-between gap-2">
                    <p className="min-w-0 truncate text-sm font-medium">
                      {item.plain_summary}
                    </p>
                    <span className="rounded-full border px-2 py-0.5 text-[11px] text-muted-foreground">
                      {item.result_label}
                    </span>
                  </div>
                  <p className="mt-1 text-xs text-muted-foreground">
                    {formatDateTime(item.timestamp)}
                  </p>
                </div>
              ))
            ) : (
              <p className="rounded-md border border-dashed p-4 text-sm text-muted-foreground">
                No AI activity has been observed yet.
              </p>
            )}
          </div>
        </div>

        <div className="rounded-lg border bg-card/60 p-4">
          <div className="flex items-center justify-between gap-3">
            <h2 className="flex items-center gap-2 text-sm font-semibold">
              <ShieldCheck className="h-4 w-4 text-primary" />
              What Pollek can do here
            </h2>
            <Link
              to="/setup"
              className="inline-flex items-center gap-1 text-xs text-primary hover:underline"
            >
              <CheckCircle2 className="h-3.5 w-3.5" />
              Setup
            </Link>
          </div>
          <div className="mt-3 space-y-2">
            {matrix.slice(0, 5).map((item) => (
              <CapabilityMini key={item.id} item={item} />
            ))}
          </div>
        </div>
      </section>
    </div>
  );
}
