import { useCallback, useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import {
  Activity,
  Bot,
  CheckCircle2,
  Clock3,
  Eye,
  FolderSearch,
  Search,
  ShieldCheck,
  ShieldX,
} from "lucide-react";
import { EntityGraphApi } from "../services/entityGraphApi";
import { RegistryApi, type AiAgent } from "../services/api";
import type { ActivityTimelineItem } from "../features/entity-graph/types";
import {
  formatDateTime,
  labelize,
  toUserFriendlyActivity,
} from "../features/user-activity/userActivityModel";
import type { UserFriendlyActivityEvent } from "../features/user-activity/types";
import { cn } from "@/lib/utils";

function agentSource(agent: AiAgent) {
  const source = agent.meta?.source ?? "registry";
  if (source === "discovery") return "Found by scan";
  if (source === "agent_self_registration") return "Reported by AI app";
  return labelize(source);
}

function agentStatus(agent: AiAgent) {
  if (agent.enforcement_mode === "Enforce") {
    return {
      label: "Rules can block",
      icon: ShieldCheck,
      className: "border-emerald-500/25 bg-emerald-500/10 text-emerald-700",
    };
  }
  if (agent.enforcement_mode === "Observe") {
    return {
      label: "Watching",
      icon: Eye,
      className: "border-blue-500/25 bg-blue-500/10 text-blue-700",
    };
  }
  if (agent.enforcement_mode === "NotEnforceable") {
    return {
      label: "Watch only",
      icon: ShieldX,
      className: "border-amber-500/25 bg-amber-500/10 text-amber-700",
    };
  }
  return {
    label: agent.enforcement_mode ?? "Registered",
    icon: Bot,
    className: "border-border bg-background text-muted-foreground",
  };
}

function eventsForAgent(agent: AiAgent, activity: UserFriendlyActivityEvent[]) {
  const agentName = agent.name.toLowerCase();
  return activity.filter(
    (event) =>
      event.agent_id === agent.agent_id ||
      event.agent_name.toLowerCase() === agentName,
  );
}

function AgentCard({
  agent,
  activity,
}: {
  agent: AiAgent;
  activity: UserFriendlyActivityEvent[];
}) {
  const status = agentStatus(agent);
  const StatusIcon = status.icon;
  const events = eventsForAgent(agent, activity);
  const lastEvent = events[0];
  const blocked = events.filter((event) => event.result === "blocked").length;

  return (
    <article className="rounded-lg border bg-card/60 p-4">
      <div className="flex items-start gap-3">
        <div className="rounded-lg bg-primary/10 p-2 text-primary">
          <Bot className="h-4 w-4" />
        </div>
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-start justify-between gap-2">
            <div className="min-w-0">
              <h3 className="truncate text-sm font-semibold">{agent.name}</h3>
              <p className="mt-1 truncate text-xs text-muted-foreground">
                {labelize(agent.agent_type)} /{" "}
                {agent.vendor ?? "Unknown vendor"}
              </p>
            </div>
            <span
              className={cn(
                "inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-[11px]",
                status.className,
              )}
            >
              <StatusIcon className="h-3 w-3" />
              {status.label}
            </span>
          </div>

          <div className="mt-3 grid gap-2 text-xs sm:grid-cols-3">
            <div className="rounded-md border bg-background/60 p-3">
              <div className="text-muted-foreground">Activity</div>
              <div className="mt-1 text-sm font-semibold">{events.length}</div>
            </div>
            <div className="rounded-md border bg-background/60 p-3">
              <div className="text-muted-foreground">Blocked</div>
              <div className="mt-1 text-sm font-semibold">{blocked}</div>
            </div>
            <div className="rounded-md border bg-background/60 p-3">
              <div className="text-muted-foreground">Trust</div>
              <div className="mt-1 text-sm font-semibold capitalize">
                {agent.trust_level}
              </div>
            </div>
          </div>

          <p className="mt-3 text-xs leading-5 text-muted-foreground">
            {lastEvent
              ? `Last seen: ${lastEvent.plain_summary} (${formatDateTime(
                  lastEvent.timestamp,
                )})`
              : "No recent activity is linked to this AI app yet."}
          </p>

          <div className="mt-3 flex flex-wrap gap-2">
            <Link
              to={`/activity?q=${encodeURIComponent(agent.name)}`}
              className="inline-flex h-8 items-center gap-2 rounded-md border px-3 text-xs hover:bg-muted"
            >
              <Activity className="h-3.5 w-3.5" />
              Activity
            </Link>
            <Link
              to={`/allowed-blocked?q=${encodeURIComponent(agent.name)}`}
              className="inline-flex h-8 items-center gap-2 rounded-md border px-3 text-xs hover:bg-muted"
            >
              <ShieldCheck className="h-3.5 w-3.5" />
              Rules
            </Link>
            <Link
              to="/setup"
              className="inline-flex h-8 items-center gap-2 rounded-md border px-3 text-xs hover:bg-muted"
            >
              <CheckCircle2 className="h-3.5 w-3.5" />
              Setup
            </Link>
          </div>

          <div className="mt-3 flex flex-wrap gap-1.5">
            <span className="rounded-full border bg-background px-2 py-0.5 text-[11px] text-muted-foreground">
              {agentSource(agent)}
            </span>
            <span className="rounded-full border bg-background px-2 py-0.5 text-[11px] text-muted-foreground">
              {agent.runtime?.runtime_name ?? "Unknown runtime"}
            </span>
            <span className="rounded-full border bg-background px-2 py-0.5 text-[11px] text-muted-foreground">
              {agent.declared_tools?.length ?? 0} tools
            </span>
          </div>
        </div>
      </div>
    </article>
  );
}

export function MyAiAppsPage() {
  const [agents, setAgents] = useState<AiAgent[]>([]);
  const [rawActivity, setRawActivity] = useState<ActivityTimelineItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState("");

  const load = useCallback(() => {
    setLoading(true);
    Promise.all([
      RegistryApi.listAgents().catch(() => [] as AiAgent[]),
      EntityGraphApi.getActivity({ limit: 300 }).catch(() => ({ items: [] })),
    ])
      .then(([agentRows, activityPage]) => {
        setAgents(agentRows);
        setRawActivity(activityPage.items ?? []);
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
  const filtered = useMemo(() => {
    const query = search.trim().toLowerCase();
    if (!query) return agents;
    return agents.filter((agent) =>
      [
        agent.name,
        agent.vendor,
        agent.agent_type,
        agent.runtime?.runtime_name,
        agent.enforcement_mode,
      ]
        .filter(Boolean)
        .join(" ")
        .toLowerCase()
        .includes(query),
    );
  }, [agents, search]);

  return (
    <div className="space-y-5">
      <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
        <div>
          <h2 className="flex items-center gap-2 text-2xl font-bold tracking-tight">
            <Bot className="h-6 w-6 text-primary" />
            My AI Apps
          </h2>
          <p className="text-sm text-muted-foreground">
            AI assistants found on this computer, with what Pollek can currently
            see.
          </p>
        </div>
        <Link
          to="/scan"
          className="inline-flex h-9 items-center gap-2 rounded-md border px-3 text-sm hover:bg-muted"
        >
          <FolderSearch className="h-4 w-4" />
          Find AI apps
        </Link>
      </div>

      <section className="grid gap-3 sm:grid-cols-3">
        <div className="rounded-lg border bg-card/60 p-4">
          <div className="text-2xl font-semibold">{agents.length}</div>
          <p className="mt-1 text-xs text-muted-foreground">AI apps found</p>
        </div>
        <div className="rounded-lg border bg-card/60 p-4">
          <div className="text-2xl font-semibold">{activity.length}</div>
          <p className="mt-1 text-xs text-muted-foreground">
            Recent activities
          </p>
        </div>
        <div className="rounded-lg border bg-card/60 p-4">
          <div className="flex items-center gap-2 text-2xl font-semibold">
            <Clock3 className="h-5 w-5 text-primary" />
            Live
          </div>
          <p className="mt-1 text-xs text-muted-foreground">
            Local dashboard data
          </p>
        </div>
      </section>

      <section className="rounded-lg border bg-card/60 p-4">
        <label className="relative block">
          <span className="sr-only">Search AI apps</span>
          <Search className="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground" />
          <input
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            placeholder="Search AI app, vendor, runtime..."
            className="h-9 w-full rounded-md border bg-background pl-9 pr-3 text-sm"
          />
        </label>
      </section>

      <section className="grid gap-3 xl:grid-cols-2">
        {loading && agents.length === 0 ? (
          <div className="rounded-lg border border-dashed p-8 text-center text-sm text-muted-foreground xl:col-span-2">
            Loading AI apps...
          </div>
        ) : filtered.length > 0 ? (
          filtered.map((agent) => (
            <AgentCard key={agent.agent_id} agent={agent} activity={activity} />
          ))
        ) : (
          <div className="rounded-lg border border-dashed p-8 text-center text-sm text-muted-foreground xl:col-span-2">
            No AI apps match this search yet.
          </div>
        )}
      </section>
    </div>
  );
}
