import { forwardRef, type HTMLAttributes } from "react";
import { cn } from "@/lib/utils";
import { statusToken, type UiStatus } from "@/lib/status";

export type BadgeVariant = "neutral" | "outline" | UiStatus;

export interface BadgeProps extends HTMLAttributes<HTMLSpanElement> {
  variant?: BadgeVariant;
  dot?: boolean;
}

export const Badge = forwardRef<HTMLSpanElement, BadgeProps>(
  ({ className, variant = "neutral", dot = false, children, ...props }, ref) => {
    const isStatus =
      variant === "ok" ||
      variant === "degraded" ||
      variant === "failed" ||
      variant === "info" ||
      variant === "idle";
    const token = isStatus ? statusToken(variant as UiStatus) : null;

    return (
      <span
        ref={ref}
        className={cn(
          "inline-flex items-center gap-1.5 rounded-full border px-2.5 py-0.5 text-xs font-medium",
          token
            ? cn(token.bg, token.text, "border-transparent ring-1", token.ring)
            : variant === "outline"
              ? "border-border bg-transparent text-foreground"
              : "border-border bg-muted text-muted-foreground",
          className,
        )}
        {...props}
      >
        {dot && token && (
          <span
            className={cn("h-1.5 w-1.5 rounded-full", token.dot)}
            aria-hidden="true"
          />
        )}
        {children}
      </span>
    );
  },
);
Badge.displayName = "Badge";
