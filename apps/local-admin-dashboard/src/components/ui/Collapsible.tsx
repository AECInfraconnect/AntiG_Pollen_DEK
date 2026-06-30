import React, { useId, useState } from "react";
import { ChevronDown, ChevronRight } from "lucide-react";
import { cn } from "../../lib/utils";

export interface CollapsibleProps {
  title: string | React.ReactNode;
  children: React.ReactNode;
  defaultExpanded?: boolean;
  headingLevel?: 2 | 3 | 4;
  className?: string;
  buttonClassName?: string;
  contentClassName?: string;
  id?: string;
}

export function Collapsible({
  title,
  children,
  defaultExpanded = false,
  headingLevel = 3,
  className,
  buttonClassName,
  contentClassName,
  id,
}: CollapsibleProps) {
  const [expanded, setExpanded] = useState(defaultExpanded);
  const generatedId = useId();
  const contentId = id ?? `collapsible-${generatedId}`;
  const buttonId = `${contentId}-button`;
  const Heading = `h${headingLevel}` as React.ElementType;

  return (
    <div
      className={cn("border rounded-md bg-card/30 overflow-hidden", className)}
      data-state={expanded ? "open" : "closed"}
    >
      <Heading className="m-0">
        <button
          id={buttonId}
          type="button"
          aria-expanded={expanded}
          aria-controls={contentId}
          onClick={() => setExpanded(!expanded)}
          className={cn(
            "flex w-full cursor-pointer items-center gap-2 px-4 py-2 text-left text-sm font-medium transition-colors hover:bg-muted/50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2",
            buttonClassName,
          )}
        >
          {expanded ? (
            <ChevronDown
              aria-hidden="true"
              className="h-4 w-4 shrink-0 text-muted-foreground"
            />
          ) : (
            <ChevronRight
              aria-hidden="true"
              className="h-4 w-4 shrink-0 text-muted-foreground"
            />
          )}
          <div className="min-w-0 flex-1">{title}</div>
        </button>
      </Heading>
      <div
        id={contentId}
        role="region"
        aria-labelledby={buttonId}
        hidden={!expanded}
        className={cn(
          "px-4 pb-3 pt-1 border-t bg-background/50",
          contentClassName,
        )}
      >
        {children}
      </div>
    </div>
  );
}
