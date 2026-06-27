import { type ReactNode } from "react";
import { cn } from "@/lib/utils";
import { ContextualHelp } from "../help/ContextualHelp";

export interface EntityPageHeaderProps {
  entityType: string;
  entityName: string;
  icon: any;
  visual?: ReactNode;
  helpTopicId?: string;
  iconColor?: string;
  status?: {
    label: string;
    tone: "neutral" | "info" | "success" | "warning" | "danger";
  };
  badges?: Array<{ label: string; tone?: string }>;
  subtitle?: string;
  actions?: ReactNode;
  meta?: Array<{ label: string; value: string | ReactNode }>;
}

const statusTone: Record<string, string> = {
  neutral: "bg-muted text-muted-foreground",
  info: "bg-blue-500/10 text-blue-600 dark:text-blue-400",
  success: "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400",
  warning: "bg-amber-500/10 text-amber-600 dark:text-amber-400",
  danger: "bg-red-500/10 text-red-600 dark:text-red-400",
};

const statusDot: Record<string, string> = {
  neutral: "bg-muted-foreground",
  info: "bg-blue-500",
  success: "bg-emerald-500",
  warning: "bg-amber-500",
  danger: "bg-red-500",
};

/**
 * Compact record header — Salesforce Lightning style.
 * Shows: Icon + EntityType label + EntityName + Status badge + Action buttons.
 * No large stat cards or KPI strips.
 */
export function EntityPageHeader({
  entityType,
  entityName,
  icon: Icon,
  visual,
  helpTopicId,
  iconColor = "text-primary",
  status,
  badges = [],
  subtitle,
  actions,
  meta = [],
}: EntityPageHeaderProps) {
  return (
    <div className="border-b border-border pb-4">
      <div className="flex items-center gap-3">
        {/* Icon */}
        {visual ?? (
          <div
            className={cn(
              "flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-primary/10",
              iconColor,
            )}
          >
            <Icon className="h-5 w-5" />
          </div>
        )}

        {/* Name block */}
        <div className="min-w-0 flex-1">
          <div className="text-[11px] font-medium uppercase tracking-wider text-muted-foreground">
            <span className="inline-flex items-center gap-1.5">
              {entityType}
              <ContextualHelp topicId={helpTopicId} />
            </span>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <h1 className="text-lg font-bold leading-tight text-foreground">
              {entityName}
            </h1>
            {status && (
              <span
                className={cn(
                  "inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[11px] font-medium",
                  statusTone[status.tone],
                )}
              >
                <span
                  className={cn("h-1.5 w-1.5 rounded-full", statusDot[status.tone])}
                />
                {status.label}
              </span>
            )}
            {badges.map((badge, idx) => (
              <span
                key={idx}
                className="inline-flex items-center rounded border border-border bg-muted/40 px-1.5 py-0.5 text-[10px] font-medium text-muted-foreground"
              >
                {badge.label}
              </span>
            ))}
          </div>
          {subtitle && (
            <p className="mt-0.5 text-xs text-muted-foreground truncate max-w-lg">
              {subtitle}
            </p>
          )}
        </div>

        {/* Actions — right aligned */}
        {actions && (
          <div className="flex shrink-0 items-center gap-2">{actions}</div>
        )}
      </div>

      {/* Inline meta — small, below the header line */}
      {meta.length > 0 && (
        <div className="mt-2 ml-[52px] flex flex-wrap gap-x-4 gap-y-0.5 text-[11px] text-muted-foreground">
          {meta.map((m, idx) => (
            <span key={idx}>
              {m.label}: <span className="font-medium text-foreground/70">{m.value}</span>
            </span>
          ))}
        </div>
      )}
    </div>
  );
}
