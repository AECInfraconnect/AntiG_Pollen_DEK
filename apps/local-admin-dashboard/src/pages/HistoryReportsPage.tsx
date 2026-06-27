import { useCallback, useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import {
  Activity,
  BarChart3,
  CalendarDays,
  Download,
  FileText,
  History,
  RefreshCw,
} from "lucide-react";
import { EntityGraphApi } from "../services/entityGraphApi";
import type { ActivityTimelineItem } from "../features/entity-graph/types";
import {
  categoryLabel,
  summarizeActivities,
  toUserFriendlyActivity,
} from "../features/user-activity/userActivityModel";
import type { UserFriendlyActivityEvent } from "../features/user-activity/types";
import { cn } from "@/lib/utils";

type Range = "7d" | "30d" | "all";

function exportReport(
  items: UserFriendlyActivityEvent[],
  format: "json" | "csv",
) {
  if (format === "json") {
    const blob = new Blob([JSON.stringify(items, null, 2)], {
      type: "application/json",
    });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = "pollek-ai-history.json";
    link.click();
    URL.revokeObjectURL(url);
    return;
  }

  const rows = items.map((item) => [
    item.timestamp,
    item.agent_name,
    item.category,
    item.action,
    item.target_label,
    item.result_label,
    item.rule_label ?? "",
    item.capability_note,
  ]);
  const csv = [
    [
      "timestamp",
      "ai_app",
      "category",
      "action",
      "target",
      "result",
      "rule",
      "capability_note",
    ],
    ...rows,
  ]
    .map((row) =>
      row.map((cell) => `"${String(cell).replaceAll('"', '""')}"`).join(","),
    )
    .join("\n");
  const blob = new Blob([csv], { type: "text/csv;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = "pollek-ai-history.csv";
  link.click();
  URL.revokeObjectURL(url);
}

function inRange(item: UserFriendlyActivityEvent, range: Range) {
  if (range === "all") return true;
  const date = new Date(item.timestamp);
  if (Number.isNaN(date.getTime())) return true;
  const days = range === "7d" ? 7 : 30;
  return Date.now() - date.getTime() <= days * 24 * 60 * 60 * 1000;
}

function countBy<T extends string>(
  items: UserFriendlyActivityEvent[],
  select: (item: UserFriendlyActivityEvent) => T,
) {
  const counts = new Map<T, number>();
  for (const item of items) {
    const key = select(item);
    counts.set(key, (counts.get(key) ?? 0) + 1);
  }
  return Array.from(counts, ([key, count]) => ({ key, count })).sort(
    (left, right) => right.count - left.count,
  );
}

function ReportRow({
  label,
  count,
  total,
}: {
  label: string;
  count: number;
  total: number;
}) {
  const percent = total > 0 ? Math.round((count / total) * 100) : 0;
  return (
    <div className="rounded-md border bg-background/60 p-3">
      <div className="flex items-center justify-between gap-3 text-sm">
        <span className="min-w-0 truncate font-medium">{label}</span>
        <span className="shrink-0 text-muted-foreground">{count}</span>
      </div>
      <div className="mt-2 h-1.5 overflow-hidden rounded-full bg-muted">
        <div
          className="h-full rounded-full bg-primary"
          style={{ width: `${percent}%` }}
        />
      </div>
    </div>
  );
}

export function HistoryReportsPage() {
  const [rawItems, setRawItems] = useState<ActivityTimelineItem[]>([]);
  const [range, setRange] = useState<Range>("7d");
  const [loading, setLoading] = useState(true);

  const load = useCallback(() => {
    setLoading(true);
    EntityGraphApi.getActivity({ limit: 1000 })
      .then((response) => setRawItems(response.items ?? []))
      .catch(() => setRawItems([]))
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const items = useMemo(
    () =>
      rawItems
        .map(toUserFriendlyActivity)
        .filter((item) => inRange(item, range)),
    [rawItems, range],
  );
  const summary = useMemo(() => summarizeActivities(items), [items]);
  const byAgent = useMemo(
    () => countBy(items, (item) => item.agent_name).slice(0, 8),
    [items],
  );
  const byCategory = useMemo(
    () => countBy(items, (item) => categoryLabel(item.category)).slice(0, 8),
    [items],
  );
  const byResult = useMemo(
    () => countBy(items, (item) => item.result_label).slice(0, 8),
    [items],
  );

  return (
    <div className="space-y-5">
      <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
        <div>
          <h2 className="flex items-center gap-2 text-2xl font-bold tracking-tight">
            <History className="h-6 w-6 text-primary" />
            History
          </h2>
          <p className="text-sm text-muted-foreground">
            Review what each AI app did, what was allowed, and what was blocked.
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          <div className="inline-flex h-9 overflow-hidden rounded-md border bg-background">
            {(["7d", "30d", "all"] as Range[]).map((option) => (
              <button
                key={option}
                type="button"
                onClick={() => setRange(option)}
                className={cn(
                  "px-3 text-sm hover:bg-muted",
                  range === option && "bg-muted text-foreground",
                )}
              >
                {option === "all" ? "All" : option.toUpperCase()}
              </button>
            ))}
          </div>
          <button
            type="button"
            onClick={() => exportReport(items, "csv")}
            className="inline-flex h-9 items-center gap-2 rounded-md border px-3 text-sm hover:bg-muted"
          >
            <FileText className="h-4 w-4" />
            CSV
          </button>
          <button
            type="button"
            onClick={() => exportReport(items, "json")}
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

      <section className="grid gap-3 sm:grid-cols-2 xl:grid-cols-5">
        <div className="rounded-lg border bg-card/60 p-4">
          <div className="text-2xl font-semibold">{summary.total}</div>
          <p className="mt-1 text-xs text-muted-foreground">Events</p>
        </div>
        <div className="rounded-lg border bg-card/60 p-4">
          <div className="text-2xl font-semibold">{summary.files}</div>
          <p className="mt-1 text-xs text-muted-foreground">Files</p>
        </div>
        <div className="rounded-lg border bg-card/60 p-4">
          <div className="text-2xl font-semibold">{summary.web}</div>
          <p className="mt-1 text-xs text-muted-foreground">Websites</p>
        </div>
        <div className="rounded-lg border bg-card/60 p-4">
          <div className="text-2xl font-semibold">{summary.blocked}</div>
          <p className="mt-1 text-xs text-muted-foreground">Blocked</p>
        </div>
        <div className="rounded-lg border bg-card/60 p-4">
          <div className="text-2xl font-semibold">
            ${summary.costUsd.toFixed(2)}
          </div>
          <p className="mt-1 text-xs text-muted-foreground">Estimated cost</p>
        </div>
      </section>

      <section className="grid gap-3 xl:grid-cols-3">
        <div className="rounded-lg border bg-card/60 p-4">
          <h3 className="flex items-center gap-2 text-sm font-semibold">
            <Activity className="h-4 w-4 text-primary" />
            By AI app
          </h3>
          <div className="mt-3 space-y-2">
            {byAgent.length > 0 ? (
              byAgent.map((row) => (
                <ReportRow
                  key={row.key}
                  label={row.key}
                  count={row.count}
                  total={summary.total}
                />
              ))
            ) : (
              <p className="text-sm text-muted-foreground">No activity yet.</p>
            )}
          </div>
        </div>
        <div className="rounded-lg border bg-card/60 p-4">
          <h3 className="flex items-center gap-2 text-sm font-semibold">
            <BarChart3 className="h-4 w-4 text-primary" />
            By activity type
          </h3>
          <div className="mt-3 space-y-2">
            {byCategory.length > 0 ? (
              byCategory.map((row) => (
                <ReportRow
                  key={row.key}
                  label={row.key}
                  count={row.count}
                  total={summary.total}
                />
              ))
            ) : (
              <p className="text-sm text-muted-foreground">No activity yet.</p>
            )}
          </div>
        </div>
        <div className="rounded-lg border bg-card/60 p-4">
          <h3 className="flex items-center gap-2 text-sm font-semibold">
            <CalendarDays className="h-4 w-4 text-primary" />
            By result
          </h3>
          <div className="mt-3 space-y-2">
            {byResult.length > 0 ? (
              byResult.map((row) => (
                <ReportRow
                  key={row.key}
                  label={row.key}
                  count={row.count}
                  total={summary.total}
                />
              ))
            ) : (
              <p className="text-sm text-muted-foreground">No activity yet.</p>
            )}
          </div>
        </div>
      </section>

      <section className="rounded-lg border bg-card/60 p-4">
        <div className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
          <div>
            <h3 className="text-sm font-semibold">Need the full timeline?</h3>
            <p className="mt-1 text-sm text-muted-foreground">
              Open AI Activity to inspect individual file, website, command, and
              tool events.
            </p>
          </div>
          <Link
            to="/activity"
            className="inline-flex h-9 items-center gap-2 rounded-md border px-3 text-sm hover:bg-muted"
          >
            <Activity className="h-4 w-4" />
            Open activity
          </Link>
        </div>
      </section>
    </div>
  );
}
