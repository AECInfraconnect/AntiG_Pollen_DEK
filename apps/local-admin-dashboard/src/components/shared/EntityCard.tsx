import { Clock } from "lucide-react";
import { cn } from "@/lib/utils";

export type EntityKind =
  | "agent"
  | "user"
  | "device"
  | "mcp_server"
  | "tool"
  | "resource"
  | "policy"
  | "deployment"
  | "capability_snapshot"
  | "control_method"
  | "evidence";
export type EntityStatus =
  | "ready"
  | "active"
  | "observe_only"
  | "needs_approval"
  | "needs_setup"
  | "partial"
  | "warning"
  | "failed"
  | "unknown";
export type ChipTone = "neutral" | "success" | "warning" | "danger" | "info";

export interface EntityCardProps {
  id: string;
  kind: EntityKind;
  title: string;
  subtitle?: string;
  status: EntityStatus;
  statusLabel: string;
  summary: string;
  chips: { label: string; tone: ChipTone }[];
  metrics?: { label: string; value: string }[];
  lastUpdatedAt?: string;
  onClick?: () => void;
  selected?: boolean;
}

const statusColors: Record<EntityStatus, string> = {
  ready: "bg-green-500/10 text-green-500 border-green-500/20",
  active: "bg-blue-500/10 text-blue-500 border-blue-500/20",
  observe_only: "bg-purple-500/10 text-purple-500 border-purple-500/20",
  needs_approval: "bg-amber-500/10 text-amber-500 border-amber-500/20",
  needs_setup: "bg-slate-500/10 text-slate-500 border-slate-500/20",
  partial: "bg-yellow-500/10 text-yellow-500 border-yellow-500/20",
  warning: "bg-orange-500/10 text-orange-500 border-orange-500/20",
  failed: "bg-red-500/10 text-red-500 border-red-500/20",
  unknown: "bg-muted text-muted-foreground border-muted/20",
};

const toneColors: Record<ChipTone, string> = {
  neutral: "bg-muted text-muted-foreground",
  success: "bg-green-500/10 text-green-500 border-green-500/20",
  warning: "bg-yellow-500/10 text-yellow-500 border-yellow-500/20",
  danger: "bg-red-500/10 text-red-500 border-red-500/20",
  info: "bg-blue-500/10 text-blue-500 border-blue-500/20",
};

export function EntityCard({
  title,
  subtitle,
  status,
  statusLabel,
  summary,
  chips,
  metrics,
  lastUpdatedAt,
  onClick,
  selected,
}: EntityCardProps) {
  return (
    <div
      onClick={onClick}
      className={cn(
        "group relative overflow-hidden rounded-xl border bg-card/50 p-4 transition-all hover:shadow-md cursor-pointer",
        selected
          ? "border-primary shadow-[0_0_15px_rgba(124,58,237,0.15)] bg-primary/5"
          : "hover:border-primary/50",
      )}
    >
      <div className="flex justify-between items-start mb-2">
        <div>
          <h3 className="font-semibold text-base tracking-tight">{title}</h3>
          {subtitle && (
            <p className="text-sm text-muted-foreground">{subtitle}</p>
          )}
        </div>
        <div
          className={cn(
            "px-2 py-1 rounded-full text-xs font-medium border",
            statusColors[status],
          )}
        >
          {statusLabel}
        </div>
      </div>

      <p className="text-sm text-muted-foreground mt-3 mb-4 line-clamp-2">
        {summary}
      </p>

      {chips.length > 0 && (
        <div className="flex flex-wrap gap-1.5 mb-4">
          {chips.map((chip, idx) => (
            <span
              key={idx}
              className={cn(
                "px-2 py-0.5 rounded-md text-xs border",
                toneColors[chip.tone],
              )}
            >
              {chip.label}
            </span>
          ))}
        </div>
      )}

      {(metrics || lastUpdatedAt) && (
        <div className="mt-4 pt-4 border-t border-border/50 flex items-center justify-between text-xs text-muted-foreground">
          {metrics && metrics.length > 0 && (
            <div className="flex gap-4">
              {metrics.map((m, i) => (
                <div key={i} className="flex flex-col">
                  <span className="font-medium text-foreground">{m.value}</span>
                  <span className="text-[10px] uppercase tracking-wider">
                    {m.label}
                  </span>
                </div>
              ))}
            </div>
          )}
          {lastUpdatedAt && (
            <div className="flex items-center gap-1 ml-auto">
              <Clock className="w-3 h-3" />
              <span>{new Date(lastUpdatedAt).toLocaleTimeString()}</span>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
