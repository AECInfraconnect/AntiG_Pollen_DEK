import { cn } from "@/lib/utils";
import { statusToken, type UiStatus } from "../../lib/status";

export function EntityCard({
  title,
  subtitle,
  summary,
  icon: Icon,
  status,
  statusLabel,
  meta = [],
  actions = [],
  selected,
}: {
  title: string;
  subtitle?: string;
  summary?: string;
  icon: any;
  status: UiStatus;
  statusLabel: string;
  meta?: { label: string; value: React.ReactNode }[];
  actions?: {
    label: string;
    icon?: any;
    primary?: boolean;
    disabled?: boolean;
    onClick: (event: React.MouseEvent<HTMLButtonElement>) => void;
  }[];
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
          {summary && (
            <p className="mt-2 line-clamp-2 text-xs leading-5 text-muted-foreground">
              {summary}
            </p>
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
          {actions.length > 0 && (
            <div className="mt-3 flex flex-wrap gap-2">
              {actions.map((action, idx) => {
                const ActionIcon = action.icon;
                return (
                  <button
                    key={idx}
                    type="button"
                    disabled={action.disabled}
                    onKeyDown={(event) => event.stopPropagation()}
                    onClick={(event) => {
                      event.stopPropagation();
                      action.onClick(event);
                    }}
                    className={cn(
                      "inline-flex h-8 items-center justify-center whitespace-nowrap gap-1.5 rounded-md px-3 text-xs font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
                      action.primary
                        ? "bg-primary text-primary-foreground hover:bg-primary/90"
                        : "border border-input bg-background hover:bg-accent hover:text-accent-foreground",
                      action.disabled && "cursor-not-allowed opacity-50",
                    )}
                  >
                    {ActionIcon && <ActionIcon className="h-3.5 w-3.5" />}
                    {action.label}
                  </button>
                );
              })}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
