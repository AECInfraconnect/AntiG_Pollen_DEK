import { useState } from "react";
import { cn } from "@/lib/utils";
import type { UiStatus } from "../../lib/status";
import { StatusChip } from "./StatusChip";

export interface DetailTab {
  id: string;
  label: string;
  content: React.ReactNode;
}

export interface DetailAction {
  label: string;
  primary?: boolean;
  danger?: boolean;
  onClick: () => void;
  icon?: any;
  disabled?: boolean;
}

export function DetailPane({
  title,
  subtitle,
  status,
  statusLabel,
  tabs,
  actions = [],
  children,
}: {
  title: string;
  subtitle?: string;
  status: UiStatus;
  statusLabel: string;
  tabs?: DetailTab[];
  actions?: DetailAction[];
  children?: React.ReactNode;
}) {
  const [activeTab, setActiveTab] = useState(tabs?.[0]?.id);

  return (
    <div className="flex flex-col h-full rounded-xl bg-card/40 overflow-hidden">
      <div className="border-b px-6 py-5 flex flex-wrap items-start justify-between gap-4">
        <div className="min-w-0 flex-1">
          <h3 className="text-xl font-semibold tracking-tight break-words">{title}</h3>
          {subtitle && (
            <p className="text-sm text-muted-foreground mt-1 break-words">{subtitle}</p>
          )}
          <div className="mt-3 flex items-center gap-3">
            <StatusChip status={status} label={statusLabel} />
          </div>
        </div>
        {actions.length > 0 && (
          <div className="flex flex-wrap items-center gap-2 shrink-0">
            {actions.map((act, i) => {
              const Icon = act.icon;
              return (
                <button
                  key={i}
                  onClick={act.onClick}
                  disabled={act.disabled}
                  className={cn(
                    "inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 h-9 px-4 py-2",
                    act.primary
                      ? "bg-primary text-primary-foreground hover:bg-primary/90 shadow-sm"
                      : act.danger
                        ? "bg-destructive text-destructive-foreground hover:bg-destructive/90 shadow-sm"
                        : "border border-input bg-background hover:bg-accent hover:text-accent-foreground",
                    act.disabled && "opacity-50 cursor-not-allowed",
                  )}
                >
                  {Icon && <Icon className="mr-2 h-4 w-4" />}
                  {act.label}
                </button>
              );
            })}
          </div>
        )}
      </div>

      {tabs && tabs.length > 0 ? (
        <>
          <div className="px-6 border-b bg-card/20 pt-2">
            <nav className="-mb-px flex space-x-6" aria-label="Tabs">
              {tabs.map((tab) => (
                <button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id)}
                  className={cn(
                    "whitespace-nowrap border-b-2 py-3 px-1 text-sm font-medium transition-colors outline-none focus-visible:ring-2 focus-visible:ring-primary rounded-t-sm",
                    activeTab === tab.id
                      ? "border-primary text-primary"
                      : "border-transparent text-muted-foreground hover:border-muted-foreground/30 hover:text-foreground",
                  )}
                >
                  {tab.label}
                </button>
              ))}
            </nav>
          </div>
          <div className="flex-1 overflow-y-auto p-6 no-scrollbar">
            {tabs.find((t) => t.id === activeTab)?.content}
            {children}
          </div>
        </>
      ) : (
        <div className="flex-1 overflow-y-auto p-6 no-scrollbar">
          {children}
        </div>
      )}
    </div>
  );
}
