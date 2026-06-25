import { cn } from "@/lib/utils";
import { statusToken, type UiStatus } from "../../lib/status";

export function EntityCard({
  title,
  subtitle,
  icon: Icon,
  status,
  statusLabel,
  meta = [],
  selected,
}: {
  title: string;
  subtitle?: string;
  icon: any;
  status: UiStatus;
  statusLabel: string;
  meta?: { label: string; value: React.ReactNode }[];
  selected: boolean;
}) {
  const s = statusToken(status);
  return (
    <div
      className={cn(
        "rounded-xl border border-border bg-card/60 p-4 backdrop-blur-sm transition-all duration-200",
        "hover:border-primary/40 hover:bg-card hover:shadow-sm",
        selected &&
          "ring-1 ring-primary/50 border-primary/50 bg-card shadow-md",
      )}
    >
      <div className="flex items-start gap-3">
        <div className={cn("mt-0.5 rounded-lg p-2", s.bg)}>
          <Icon className={cn("h-4 w-4", s.text)} />
        </div>
        <div className="min-w-0 flex-1">
          <div className="flex items-center justify-between gap-2">
            <span className="truncate font-medium">{title}</span>
            <span
              className={cn(
                "flex items-center gap-1.5 rounded-full px-2 py-0.5 text-[11px] font-medium transition-colors",
                s.bg,
                s.text,
              )}
            >
              <span className={cn("h-1.5 w-1.5 rounded-full", s.dot)} />
              {statusLabel}
            </span>
          </div>
          {subtitle && (
            <div className="truncate text-xs text-muted-foreground mt-0.5">
              {subtitle}
            </div>
          )}
          {meta.length > 0 && (
            <div className="mt-3 flex flex-wrap gap-x-4 gap-y-1 text-[11px] text-muted-foreground">
              {meta.map((m, idx) => (
                <span key={idx} className="flex items-center gap-1">
                  {m.label}:{" "}
                  <span className="text-foreground/80 font-medium">
                    {m.value}
                  </span>
                </span>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
