import { useCallback, useEffect, useMemo, useState } from "react";
import { useSearchParams } from "react-router-dom";
import {
  Activity,
  AlertTriangle,
  Download,
  Eye,
  FileText,
  RefreshCw,
  Search,
  ShieldCheck,
  ShieldX,
} from "lucide-react";
import { EntityGraphApi } from "../services/entityGraphApi";
import type { ActivityTimelineItem } from "../features/entity-graph/types";
import {
  categoryLabel,
  formatDateTime,
  summarizeActivities,
  toUserFriendlyActivity,
} from "../features/user-activity/userActivityModel";
import type {
  UserActivityCategory,
  UserActivityResult,
  UserFriendlyActivityEvent,
} from "../features/user-activity/types";
import { cn } from "@/lib/utils";

type Filters = {
  search: string;
  category: "" | UserActivityCategory;
  result: "" | UserActivityResult;
  agent: string;
};

const resultTone: Record<UserActivityResult, string> = {
  allowed: "border-emerald-500/25 bg-emerald-500/10 text-emerald-600",
  blocked: "border-red-500/25 bg-red-500/10 text-red-600",
  asked_first: "border-amber-500/25 bg-amber-500/10 text-amber-600",
  asked_and_allowed: "border-emerald-500/25 bg-emerald-500/10 text-emerald-600",
  asked_and_denied: "border-red-500/25 bg-red-500/10 text-red-600",
  watched_only: "border-blue-500/25 bg-blue-500/10 text-blue-600",
  warned: "border-amber-500/25 bg-amber-500/10 text-amber-600",
  error: "border-red-500/25 bg-red-500/10 text-red-600",
};

const categories: UserActivityCategory[] = [
  "files",
  "web",
  "email",
  "apps",
  "commands",
  "ai_models",
  "tools",
  "cost",
  "unknown",
];

function exportJson(items: UserFriendlyActivityEvent[]) {
  const blob = new Blob([JSON.stringify(items, null, 2)], {
    type: "application/json",
  });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = "pollek-ai-activity.json";
  link.click();
  URL.revokeObjectURL(url);
}

function exportCsv(items: UserFriendlyActivityEvent[]) {
  const header = [
    "timestamp",
    "ai_app",
    "category",
    "action",
    "target",
    "result",
    "rule",
    "capability_note",
    "next_step",
  ];
  const rows = items.map((item) => [
    item.timestamp,
    item.agent_name,
    item.category,
    item.action,
    item.target_label,
    item.result,
    item.rule_label ?? "",
    item.capability_note,
    item.next_step,
  ]);
  const csv = [header, ...rows]
    .map((row) =>
      row.map((cell) => `"${String(cell).replaceAll('"', '""')}"`).join(","),
    )
    .join("\n");
  const blob = new Blob([csv], { type: "text/csv;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = "pollek-ai-activity.csv";
  link.click();
  URL.revokeObjectURL(url);
}

function ActivityResultIcon({ result }: { result: UserActivityResult }) {
  if (result === "blocked" || result === "asked_and_denied") {
    return <ShieldX className="h-4 w-4 text-red-500" />;
  }
  if (result === "allowed" || result === "asked_and_allowed") {
    return <ShieldCheck className="h-4 w-4 text-emerald-500" />;
  }
  if (result === "error")
    return <AlertTriangle className="h-4 w-4 text-red-500" />;
  return <Eye className="h-4 w-4 text-blue-500" />;
}

function SummaryTile({
  label,
  value,
}: {
  label: string;
  value: string | number;
}) {
  return (
    <div className="rounded-lg border bg-card/60 p-4">
      <div className="text-2xl font-semibold">{value}</div>
      <p className="mt-1 text-xs font-medium text-muted-foreground">{label}</p>
    </div>
  );
}

function ActivityCard({ item }: { item: UserFriendlyActivityEvent }) {
  return (
    <article className="rounded-lg border bg-card/60 p-4">
      <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-2">
            <ActivityResultIcon result={item.result} />
            <h3 className="min-w-0 text-sm font-semibold">
              {item.plain_summary}
            </h3>
            <span
              className={cn(
                "rounded-full border px-2 py-0.5 text-[11px] font-medium",
                resultTone[item.result],
              )}
            >
              {item.result_label}
            </span>
            <span className="rounded-full border bg-background px-2 py-0.5 text-[11px] text-muted-foreground">
              {categoryLabel(item.category)}
            </span>
          </div>

          <div className="mt-3 grid gap-2 text-sm md:grid-cols-3">
            <div className="rounded-md border bg-background/60 p-3">
              <div className="text-xs text-muted-foreground">AI app</div>
              <div className="mt-1 truncate font-medium">{item.agent_name}</div>
            </div>
            <div className="rounded-md border bg-background/60 p-3">
              <div className="text-xs text-muted-foreground">Touched</div>
              <div className="mt-1 truncate font-medium">
                {item.target_label}
              </div>
            </div>
            <div className="rounded-md border bg-background/60 p-3">
              <div className="text-xs text-muted-foreground">Access</div>
              <div className="mt-1 truncate font-medium capitalize">
                {item.access_mode}
              </div>
            </div>
          </div>

          <div className="mt-3 grid gap-2 text-xs md:grid-cols-2">
            <p className="rounded-md border border-blue-500/20 bg-blue-500/10 p-3 text-blue-700">
              {item.capability_note}
            </p>
            <p className="rounded-md border bg-background/60 p-3 text-muted-foreground">
              {item.next_step}
            </p>
          </div>

          <div className="mt-3 flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
            <span>{formatDateTime(item.timestamp)}</span>
            {item.rule_label && <span>Rule: {item.rule_label}</span>}
            {item.tokens && <span>{item.tokens.toLocaleString()} tokens</span>}
            {item.cost_usd && <span>${item.cost_usd.toFixed(4)}</span>}
          </div>
        </div>

        <details className="shrink-0 rounded-md border bg-background px-3 py-2 text-xs">
          <summary className="cursor-pointer text-muted-foreground">
            Advanced
          </summary>
          <div className="mt-2 max-w-xs space-y-1 text-muted-foreground">
            <p>Trace: {item.trace_id ?? "none"}</p>
            <p>Decision: {item.advanced?.decision ?? "unknown"}</p>
            <p>Mode: {item.advanced?.mode ?? "unknown"}</p>
          </div>
        </details>
      </div>
    </article>
  );
}

export function AiActivityPage() {
  const [searchParams] = useSearchParams();
  const [rawItems, setRawItems] = useState<ActivityTimelineItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);
  const [filters, setFilters] = useState<Filters>({
    search: searchParams.get("q") ?? "",
    category: "",
    result: "",
    agent: "",
  });

  const load = useCallback(() => {
    setLoading(true);
    EntityGraphApi.getActivity({ limit: 300 })
      .then((response) => {
        setRawItems(response.items ?? []);
        setError(null);
      })
      .catch((err) =>
        setError(err instanceof Error ? err : new Error(String(err))),
      )
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => {
    load();
    const timer = window.setInterval(load, 15000);
    return () => window.clearInterval(timer);
  }, [load]);

  const items = useMemo(() => rawItems.map(toUserFriendlyActivity), [rawItems]);
  const agentOptions = useMemo(
    () => Array.from(new Set(items.map((item) => item.agent_name))).sort(),
    [items],
  );
  const filtered = useMemo(() => {
    const query = filters.search.trim().toLowerCase();
    return items.filter((item) => {
      if (filters.category && item.category !== filters.category) return false;
      if (filters.result && item.result !== filters.result) return false;
      if (filters.agent && item.agent_name !== filters.agent) return false;
      if (!query) return true;
      return [
        item.agent_name,
        item.target_label,
        item.plain_summary,
        item.rule_label,
        item.capability_note,
        item.next_step,
      ]
        .filter(Boolean)
        .join(" ")
        .toLowerCase()
        .includes(query);
    });
  }, [filters, items]);
  const summary = useMemo(() => summarizeActivities(filtered), [filtered]);

  return (
    <div className="space-y-5">
      <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
        <div>
          <h2 className="flex items-center gap-2 text-2xl font-bold tracking-tight">
            <Activity className="h-6 w-6 text-primary" />
            AI Activity
          </h2>
          <p className="text-sm text-muted-foreground">
            Files, websites, tools, commands, model usage, and decisions in
            plain language.
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          <button
            type="button"
            onClick={() => exportCsv(filtered)}
            className="inline-flex h-9 items-center gap-2 rounded-md border px-3 text-sm hover:bg-muted"
          >
            <FileText className="h-4 w-4" />
            CSV
          </button>
          <button
            type="button"
            onClick={() => exportJson(filtered)}
            className="inline-flex h-9 items-center gap-2 rounded-md border px-3 text-sm hover:bg-muted"
          >
            <Download className="h-4 w-4" />
            JSON
          </button>
          <button
            type="button"
            onClick={load}
            className="inline-flex h-9 items-center gap-2 rounded-md border px-3 text-sm hover:bg-muted"
          >
            <RefreshCw className={cn("h-4 w-4", loading && "animate-spin")} />
            Refresh
          </button>
        </div>
      </div>

      <section className="grid gap-3 sm:grid-cols-2 xl:grid-cols-6">
        <SummaryTile label="Events" value={summary.total} />
        <SummaryTile label="File activity" value={summary.files} />
        <SummaryTile label="Web activity" value={summary.web} />
        <SummaryTile label="Commands" value={summary.commands} />
        <SummaryTile label="Blocked" value={summary.blocked} />
        <SummaryTile
          label="Estimated cost"
          value={`$${summary.costUsd.toFixed(2)}`}
        />
      </section>

      <section className="rounded-lg border bg-card/60 p-4">
        <div className="grid gap-3 lg:grid-cols-[1.5fr_0.9fr_0.9fr_0.9fr]">
          <label className="relative block">
            <span className="sr-only">Search activity</span>
            <Search className="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground" />
            <input
              value={filters.search}
              onChange={(event) =>
                setFilters((current) => ({
                  ...current,
                  search: event.target.value,
                }))
              }
              placeholder="Search AI app, file, folder, website, command..."
              className="h-9 w-full rounded-md border bg-background pl-9 pr-3 text-sm"
            />
          </label>
          <select
            value={filters.agent}
            onChange={(event) =>
              setFilters((current) => ({
                ...current,
                agent: event.target.value,
              }))
            }
            className="h-9 rounded-md border bg-background px-3 text-sm"
          >
            <option value="">All AI apps</option>
            {agentOptions.map((agent) => (
              <option key={agent} value={agent}>
                {agent}
              </option>
            ))}
          </select>
          <select
            value={filters.category}
            onChange={(event) =>
              setFilters((current) => ({
                ...current,
                category: event.target.value as Filters["category"],
              }))
            }
            className="h-9 rounded-md border bg-background px-3 text-sm"
          >
            <option value="">All activity</option>
            {categories.map((category) => (
              <option key={category} value={category}>
                {categoryLabel(category)}
              </option>
            ))}
          </select>
          <select
            value={filters.result}
            onChange={(event) =>
              setFilters((current) => ({
                ...current,
                result: event.target.value as Filters["result"],
              }))
            }
            className="h-9 rounded-md border bg-background px-3 text-sm"
          >
            <option value="">All results</option>
            <option value="watched_only">Watched only</option>
            <option value="allowed">Allowed</option>
            <option value="blocked">Blocked</option>
            <option value="asked_first">Ask first</option>
            <option value="warned">Warned</option>
            <option value="error">Error</option>
          </select>
        </div>
      </section>

      {error && (
        <div className="rounded-lg border border-amber-500/20 bg-amber-500/10 p-4 text-sm text-amber-700">
          {error.message}
        </div>
      )}

      <div className="space-y-3">
        {loading && rawItems.length === 0 ? (
          <div className="rounded-lg border border-dashed p-8 text-center text-sm text-muted-foreground">
            Loading AI activity...
          </div>
        ) : filtered.length === 0 ? (
          <div className="rounded-lg border border-dashed p-8 text-center text-sm text-muted-foreground">
            No AI activity matches this view yet.
          </div>
        ) : (
          filtered.map((item) => (
            <ActivityCard key={item.event_id} item={item} />
          ))
        )}
      </div>
    </div>
  );
}
