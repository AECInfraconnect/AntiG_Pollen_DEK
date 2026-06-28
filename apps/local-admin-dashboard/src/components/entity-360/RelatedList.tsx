import { type ReactNode } from "react";
import { ChevronDown, ChevronUp, ExternalLink } from "lucide-react";
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { cn } from "@/lib/utils";
import { renderDisplayValue } from "@/lib/displayValue";

export interface RelatedListItem {
  id: string;
  icon: any;
  iconColor?: string;
  title: string;
  subtitle?: string;
  href?: string;
  meta?: { label: string; value: string | ReactNode }[];
  badge?: { label: string; tone: "neutral" | "info" | "success" | "warning" | "danger" };
}

interface RelatedListProps {
  title: string;
  icon: any;
  iconColor?: string;
  items: RelatedListItem[];
  maxVisible?: number;
  emptyMessage?: string;
  onViewAll?: () => void;
  viewAllHref?: string;
}

const toneBg: Record<string, string> = {
  neutral: "bg-muted text-muted-foreground",
  info: "bg-blue-500/10 text-blue-700 dark:text-blue-400",
  success: "bg-emerald-500/10 text-emerald-700 dark:text-emerald-400",
  warning: "bg-amber-500/10 text-amber-700 dark:text-amber-400",
  danger: "bg-red-500/10 text-red-700 dark:text-red-400",
};

export function RelatedList({
  title,
  icon: GroupIcon,
  iconColor = "text-primary",
  items,
  maxVisible = 3,
  emptyMessage = "No related records yet.",
  onViewAll,
  viewAllHref,
}: RelatedListProps) {
  const navigate = useNavigate();
  const [expanded, setExpanded] = useState(false);
  const visibleItems = expanded ? items : items.slice(0, maxVisible);
  const hasMore = items.length > maxVisible;

  return (
    <section className="rounded-xl border border-border bg-card/60 backdrop-blur-sm">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-border/50 px-4 py-3">
        <div className="flex items-center gap-2.5">
          <div className={cn("rounded-lg bg-primary/10 p-1.5", iconColor)}>
            <GroupIcon className="h-4 w-4" />
          </div>
          <h3 className="text-sm font-semibold">
            {title}{" "}
            <span className="ml-1 text-xs font-normal text-muted-foreground">
              ({items.length})
            </span>
          </h3>
        </div>
        {hasMore && (
          <button
            type="button"
            onClick={() => setExpanded(!expanded)}
            className="flex items-center gap-1 rounded-md px-2 py-1 text-xs text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
          >
            {expanded ? (
              <>
                Collapse <ChevronUp className="h-3 w-3" />
              </>
            ) : (
              <>
                Show all <ChevronDown className="h-3 w-3" />
              </>
            )}
          </button>
        )}
      </div>

      {/* Items */}
      {items.length === 0 ? (
        <div className="px-4 py-6 text-center text-sm text-muted-foreground">
          {emptyMessage}
        </div>
      ) : (
        <div className="divide-y divide-border/30">
          {visibleItems.map((item) => {
            const ItemIcon = item.icon;
            return (
              <button
                key={item.id}
                type="button"
                onClick={() => {
                  if (item.href) navigate(item.href);
                }}
                className={cn(
                  "flex w-full items-start gap-3 px-4 py-3 text-left transition-colors",
                  item.href && "hover:bg-muted/50 cursor-pointer",
                  !item.href && "cursor-default",
                )}
              >
                <div
                  className={cn(
                    "mt-0.5 rounded-md border bg-background p-1.5",
                    item.iconColor,
                  )}
                >
                  <ItemIcon className="h-3.5 w-3.5" />
                </div>
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2">
                    <span className="truncate text-sm font-medium text-foreground">
                      {renderDisplayValue(item.title)}
                    </span>
                    {item.badge && (
                      <span
                        className={cn(
                          "inline-flex items-center rounded-full px-2 py-0.5 text-[10px] font-medium",
                          toneBg[item.badge.tone],
                        )}
                      >
                        {renderDisplayValue(item.badge.label)}
                      </span>
                    )}
                    {item.href && (
                      <ExternalLink className="ml-auto h-3 w-3 shrink-0 text-muted-foreground/50" />
                    )}
                  </div>
                  {item.subtitle && (
                    <p className="mt-0.5 truncate text-xs text-muted-foreground">
                      {renderDisplayValue(item.subtitle)}
                    </p>
                  )}
                  {item.meta && item.meta.length > 0 && (
                    <div className="mt-1.5 flex flex-wrap gap-x-4 gap-y-0.5 text-[11px] text-muted-foreground">
                      {item.meta.map((m, idx) => (
                        <span key={idx}>
                          {m.label}:{" "}
                          <span className="font-medium text-foreground/70">
                            {renderDisplayValue(m.value)}
                          </span>
                        </span>
                      ))}
                    </div>
                  )}
                </div>
              </button>
            );
          })}
        </div>
      )}

      {/* View All Footer */}
      {(onViewAll || viewAllHref) && items.length > 0 && (
        <div className="border-t border-border/50 px-4 py-2">
          <button
            type="button"
            onClick={() => {
              if (viewAllHref) navigate(viewAllHref);
              else if (onViewAll) onViewAll();
            }}
            className="w-full text-center text-xs font-medium text-primary hover:text-primary/80 transition-colors"
          >
            View All {title}
          </button>
        </div>
      )}
    </section>
  );
}
