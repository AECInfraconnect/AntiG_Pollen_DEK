import { useNavigate } from "react-router-dom";
import { Activity, ArrowRight, Clock, Shield, Zap } from "lucide-react";
import { cn } from "@/lib/utils";
import type { ActivityTimelineItem } from "../../features/entity-graph/types";
import { entityIcon, entityRoute } from "../../features/entity-graph/graphUtils";

interface ActivityFeedProps {
  items: ActivityTimelineItem[];
  maxVisible?: number;
  showFilters?: boolean;
  compact?: boolean;
}

const decisionStyles: Record<string, { bg: string; text: string; dot: string }> = {
  allow: {
    bg: "bg-emerald-500/10",
    text: "text-emerald-700 dark:text-emerald-400",
    dot: "bg-emerald-500",
  },
  deny: {
    bg: "bg-red-500/10",
    text: "text-red-700 dark:text-red-400",
    dot: "bg-red-500",
  },
  observe: {
    bg: "bg-blue-500/10",
    text: "text-blue-700 dark:text-blue-400",
    dot: "bg-blue-500",
  },
};

function formatTime(timestamp: string): string {
  const date = new Date(timestamp);
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  if (diff < 60000) return "Just now";
  if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
  if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
  return date.toLocaleDateString(undefined, { month: "short", day: "numeric" });
}

function EntityLink({
  entityRef,
  className,
}: {
  entityRef: { id: string; type: string; entity_id: string; label: string };
  className?: string;
}) {
  const navigate = useNavigate();
  const Icon = entityIcon(entityRef.type);
  const route = entityRoute({
    id: entityRef.id,
    type: entityRef.type,
    entity_id: entityRef.entity_id,
    label: entityRef.label,
    status: "",
    badges: [],
    metrics: [],
  });

  return (
    <button
      type="button"
      onClick={(e) => {
        e.stopPropagation();
        navigate(route);
      }}
      className={cn(
        "inline-flex items-center gap-1 rounded-md border border-border/50 bg-muted/30 px-1.5 py-0.5 text-[11px] font-medium text-foreground/80 hover:bg-primary/10 hover:text-primary hover:border-primary/30 transition-colors",
        className,
      )}
    >
      <Icon className="h-3 w-3" />
      <span className="max-w-[120px] truncate">{entityRef.label}</span>
    </button>
  );
}

export function ActivityFeed({
  items,
  maxVisible = 10,
  compact = false,
}: ActivityFeedProps) {
  const visibleItems = items.slice(0, maxVisible);

  if (!items.length) {
    return (
      <div className="flex flex-col items-center justify-center rounded-lg border border-dashed p-8 text-center">
        <Activity className="mb-2 h-8 w-8 text-muted-foreground/50" />
        <p className="text-sm text-muted-foreground">
          No activity recorded yet. Events will appear here as agents interact with tools and resources.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-1">
      {visibleItems.map((item) => {
        const style = decisionStyles[item.decision] ?? decisionStyles.observe;
        return (
          <div
            key={item.event_id}
            className="group relative flex gap-3 rounded-lg px-3 py-2.5 transition-colors hover:bg-muted/40"
          >
            {/* Timeline dot */}
            <div className="flex flex-col items-center pt-1.5">
              <div className={cn("h-2.5 w-2.5 rounded-full", style.dot)} />
              <div className="mt-1 flex-1 w-px bg-border/50 group-last:hidden" />
            </div>

            {/* Content */}
            <div className="min-w-0 flex-1">
              {/* Action line */}
              <div className="flex flex-wrap items-center gap-1.5 text-sm">
                {item.actor && <EntityLink entityRef={item.actor} />}
                <ArrowRight className="h-3 w-3 text-muted-foreground/50" />
                <span className="font-medium text-foreground/90">{item.action}</span>
                {item.tool && <EntityLink entityRef={item.tool} />}
                {item.resource && <EntityLink entityRef={item.resource} />}
              </div>

              {/* Context line */}
              <div className="mt-1.5 flex flex-wrap items-center gap-2 text-[11px] text-muted-foreground">
                {/* Decision badge */}
                <span
                  className={cn(
                    "inline-flex items-center gap-1 rounded-full px-2 py-0.5 font-medium",
                    style.bg,
                    style.text,
                  )}
                >
                  <Shield className="h-2.5 w-2.5" />
                  {item.decision}
                </span>

                {/* Enforcement mode */}
                {item.enforcement_mode && (
                  <span className="rounded border border-border bg-background px-1.5 py-0.5 font-medium">
                    {item.enforcement_mode}
                  </span>
                )}

                {/* Policies */}
                {item.policies.length > 0 && (
                  <span className="flex items-center gap-1">
                    <Shield className="h-3 w-3" />
                    {item.policies.map((p) => (
                      <EntityLink key={p.id} entityRef={p} />
                    ))}
                  </span>
                )}

                {/* Cost */}
                {item.cost?.total_cost_usd != null && (
                  <span className="flex items-center gap-1">
                    <Zap className="h-3 w-3" />$
                    {item.cost.total_cost_usd.toFixed(4)}
                  </span>
                )}

                {/* Time */}
                <span className="ml-auto flex items-center gap-1">
                  <Clock className="h-3 w-3" />
                  {formatTime(item.timestamp)}
                </span>
              </div>

              {/* Explanation */}
              {!compact && item.explanation && (
                <p className="mt-1.5 text-xs text-muted-foreground/80 italic">
                  {item.explanation}
                </p>
              )}
            </div>
          </div>
        );
      })}
    </div>
  );
}
