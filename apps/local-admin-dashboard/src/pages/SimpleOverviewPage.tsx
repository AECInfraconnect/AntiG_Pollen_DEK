import { useCallback, useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import { toast } from "sonner";
import {
  Activity,
  Bot,
  CheckCircle2,
  DollarSign,
  Eye,
  FileText,
  FolderSearch,
  Globe2,
  History,
  ListChecks,
  Mail,
  Plug,
  ShieldAlert,
  ShieldCheck,
  TerminalSquare,
  Wrench,
} from "lucide-react";
import {
  CapabilityApi,
  LocalObserveApi,
  RegistryApi,
  type AiAgent,
  type LocalObserveRefreshResponse,
} from "../services/api";
import { Collapsible } from "../components/ui";
import type { LocalCapabilitySnapshotV2 } from "../services/types";
import { UserActivityApi } from "../features/user-activity/api";
import {
  buildUserCapabilityMatrix,
  capabilityTone,
  formatDateTime,
  summarizeActivities,
} from "../features/user-activity/userActivityModel";
import type {
  UserActivityCategory,
  UserCapabilityItem,
  UserFriendlyActivityEvent,
} from "../features/user-activity/types";
import { cn } from "@/lib/utils";

const toneClass: Record<string, string> = {
  success: "border-emerald-500/25 bg-emerald-500/10 text-emerald-700",
  info: "border-blue-500/25 bg-blue-500/10 text-blue-700",
  warning: "border-amber-500/25 bg-amber-500/10 text-amber-700",
  neutral: "border-border bg-background text-muted-foreground",
};

const observeSurfaceCopy: Array<{
  category: UserActivityCategory;
  icon: any;
  title: string;
  description: string;
  setup: string;
}> = [
  {
    category: "files",
    icon: FileText,
    title: "Files and folders",
    description:
      "Shows when an AI app reads, changes, or tries to reach local files and folders.",
    setup:
      "Folder-level blocking may need OS permission, a file guard, or the AI app's own folder settings.",
  },
  {
    category: "web",
    icon: Globe2,
    title: "Websites and network",
    description:
      "Shows websites, domains, and network destinations Pollek can identify from local signals.",
    setup:
      "Exact browser actions may need a browser connector, network permission, proxy, or plugin.",
  },
  {
    category: "commands",
    icon: TerminalSquare,
    title: "Apps and commands",
    description:
      "Shows when an AI app launches tools, scripts, terminals, or other local programs.",
    setup:
      "Blocking command execution depends on host capability and how the AI app launches commands.",
  },
  {
    category: "email",
    icon: Mail,
    title: "Email and calendar",
    description:
      "Shows connector-level access when email or calendar integrations are installed and allowed.",
    setup:
      "Pollek will show this as setup-required until an email/calendar connector is installed.",
  },
  {
    category: "tools",
    icon: Plug,
    title: "AI tools and MCP",
    description:
      "Shows tool calls, MCP resources, and connector activity when they emit local telemetry.",
    setup:
      "Use a wrapper, connector, or plugin for exact tool/resource call visibility.",
  },
  {
    category: "safety",
    icon: ShieldAlert,
    title: "Prompts and private data",
    description:
      "Shows Prompt Guard incidents, redactions, secrets, PII, or prompt-injection signals.",
    setup:
      "Enable Prompt Guard in the AI app path for warning, ask-first, or redaction behavior.",
  },
  {
    category: "ai_models",
    icon: Bot,
    title: "Model usage",
    description:
      "Shows model/provider usage when exact provider data, wrapper logs, or estimates are available.",
    setup:
      "Exact tokens need provider telemetry or a wrapper; browser-only activity may stay estimated.",
  },
  {
    category: "cost",
    icon: DollarSign,
    title: "AI usage and cost",
    description:
      "Shows exact or estimated spend as observe evidence, not only as a billing report.",
    setup:
      "Exact cost needs provider usage data; otherwise Pollek labels estimates clearly.",
  },
];

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

function ObserveSurfaceCard({
  surface,
  item,
}: {
  surface: (typeof observeSurfaceCopy)[number];
  item?: UserCapabilityItem;
}) {
  const Icon = surface.icon;
  const canWatch = Boolean(item?.can_watch);
  const tone = item ? capabilityTone(item.status) : "neutral";
  const statusLabel = item?.can_block
    ? "Watch and block"
    : item?.can_ask_first
      ? "Can ask first"
      : canWatch
        ? "Watch now"
        : item?.status === "needs_setup"
          ? "Needs setup"
          : "Not available yet";

  return (
    <div className="rounded-lg border bg-card/60 p-4">
      <div className="flex items-start gap-3">
        <div className={cn("rounded-lg p-2", toneClass[tone])}>
          <Icon className="h-4 w-4" />
        </div>
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <h3 className="text-sm font-semibold">{surface.title}</h3>
            <span
              className={cn(
                "rounded-full border px-2 py-0.5 text-[11px]",
                toneClass[tone],
              )}
            >
              {statusLabel}
            </span>
          </div>
          <p className="mt-2 text-xs leading-5 text-muted-foreground">
            {surface.description}
          </p>
          <p className="mt-2 text-xs leading-5 text-muted-foreground">
            {item?.plain_description ?? surface.setup}
          </p>
        </div>
      </div>
    </div>
  );
}

export function SimpleOverviewPage() {
  const [agents, setAgents] = useState<AiAgent[]>([]);
  const [activity, setActivity] = useState<UserFriendlyActivityEvent[]>([]);
  const [snapshot, setSnapshot] = useState<LocalCapabilitySnapshotV2 | null>(
    null,
  );
  const [loading, setLoading] = useState(true);
  const [observing, setObserving] = useState(false);
  const [observeResult, setObserveResult] =
    useState<LocalObserveRefreshResponse | null>(null);

  const load = useCallback(() => {
    setLoading(true);
    Promise.all([
      RegistryApi.listAgents().catch(() => [] as AiAgent[]),
      UserActivityApi.list({ limit: 100 }).catch(() => ({ items: [] })),
      CapabilityApi.getSnapshotV2("desktop_simple").catch(() => null),
    ])
      .then(([agentRows, activityPage, capabilitySnapshot]) => {
        setAgents(agentRows);
        setActivity(activityPage.items ?? []);
        setSnapshot(capabilitySnapshot);
      })
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const observeNow = useCallback(async () => {
    setObserving(true);
    try {
      const result = await LocalObserveApi.refresh({ include_estimates: true });
      setObserveResult(result);
      toast.success(
        `Observed ${result.candidates_found} AI app(s) and ${result.resource_events} resource event(s).`,
      );
      load();
    } catch (error) {
      toast.error(
        error instanceof Error ? error.message : "Local observe refresh failed",
      );
    } finally {
      setObserving(false);
    }
  }, [load]);

  const summary = useMemo(() => summarizeActivities(activity), [activity]);
  const matrix = useMemo(() => buildUserCapabilityMatrix(snapshot), [snapshot]);
  const observeSurfaces = useMemo(
    () =>
      observeSurfaceCopy.map((surface) => ({
        surface,
        item: matrix.find((item) => item.category === surface.category),
      })),
    [matrix],
  );
  const needsSetup = matrix.filter(
    (item) => item.status === "needs_setup",
  ).length;
  const watchReady = matrix.filter((item) => item.can_watch).length;
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
            <button
              type="button"
              onClick={observeNow}
              disabled={observing}
              className="inline-flex h-9 items-center gap-2 rounded-md bg-primary px-3 text-sm text-primary-foreground hover:bg-primary/90 disabled:opacity-60"
            >
              <Eye className={cn("h-4 w-4", observing && "animate-pulse")} />
              {observing ? "Observing" : "Observe now"}
            </button>
            <Link
              to="/scan"
              className="inline-flex h-9 items-center gap-2 rounded-md border px-3 text-sm hover:bg-muted"
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

      {observeResult && (
        <section className="rounded-lg border bg-card/60 p-4">
          <h2 className="text-sm font-semibold">Latest observe refresh</h2>
          <p className="mt-1 text-xs leading-5 text-muted-foreground">
            Pollek wrote {observeResult.resource_events} resource event(s),{" "}
            {observeResult.tool_events} tool event(s),{" "}
            {observeResult.exact_usage_events} exact usage event(s), and{" "}
            {observeResult.estimated_usage_events} estimated usage event(s) to
            the local timeline.
          </p>
        </section>
      )}

      <Collapsible
        className="rounded-lg bg-card/60"
        title={
          <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
            <div>
              <div className="text-sm font-semibold">
                What Pollek can see on this device
              </div>
              <div className="text-xs text-muted-foreground">
                Coverage areas and setup requirements.
              </div>
            </div>
            <div className="flex flex-wrap gap-2 text-xs text-muted-foreground">
              <span className="rounded-full border bg-background px-2.5 py-1">
                {watchReady} watchable
              </span>
              <span className="rounded-full border bg-background px-2.5 py-1">
                {needsSetup} need setup
              </span>
            </div>
          </div>
        }
      >
        <div className="flex flex-col gap-3 md:flex-row md:items-start md:justify-between">
          <p className="max-w-3xl text-sm leading-6 text-muted-foreground">
            Coverage depends on this OS, permissions, connectors, and how each
            AI app is launched. Pollek labels watch-only or setup-required areas
            so you know when to configure the AI app itself.
          </p>
          <Link
            to="/setup"
            className="inline-flex h-9 w-fit items-center gap-2 rounded-md border px-3 text-sm hover:bg-muted"
          >
            <Wrench className="h-4 w-4" />
            Setup details
          </Link>
        </div>
        <div className="mt-3 grid gap-3 md:grid-cols-2 xl:grid-cols-4">
          {observeSurfaces.map(({ surface, item }) => (
            <ObserveSurfaceCard
              key={surface.category}
              surface={surface}
              item={item}
            />
          ))}
        </div>
      </Collapsible>

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
