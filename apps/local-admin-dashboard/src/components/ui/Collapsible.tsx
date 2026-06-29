import React, { useState } from "react";
import { ChevronDown, ChevronRight } from "lucide-react";
import { cn } from "../../lib/utils";

export interface CollapsibleProps {
  title: string | React.ReactNode;
  children: React.ReactNode;
  defaultExpanded?: boolean;
  className?: string;
  contentClassName?: string;
}

export function Collapsible({
  title,
  children,
  defaultExpanded = false,
  className,
  contentClassName,
}: CollapsibleProps) {
  const [expanded, setExpanded] = useState(defaultExpanded);

  return (
    <div
      className={cn("border rounded-md bg-card/30 overflow-hidden", className)}
    >
      <button
        type="button"
        onClick={() => setExpanded(!expanded)}
        className="flex w-full items-center gap-2 px-4 py-2 text-sm font-medium hover:bg-muted/50 transition-colors text-left"
      >
        {expanded ? (
          <ChevronDown className="h-4 w-4 shrink-0 text-muted-foreground" />
        ) : (
          <ChevronRight className="h-4 w-4 shrink-0 text-muted-foreground" />
        )}
        <div className="min-w-0 flex-1">{title}</div>
      </button>
      {expanded && (
        <div
          className={cn(
            "px-4 pb-3 pt-1 border-t bg-background/50",
            contentClassName,
          )}
        >
          {children}
        </div>
      )}
    </div>
  );
}
