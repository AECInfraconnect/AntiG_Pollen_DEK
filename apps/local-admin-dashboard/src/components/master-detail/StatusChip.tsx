import { cn } from "@/lib/utils";
import { statusToken, type UiStatus } from "../../lib/status";

export function StatusChip({
  status,
  label,
  className,
}: {
  status: UiStatus;
  label: string;
  className?: string;
}) {
  const s = statusToken(status);
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs font-medium",
        s.bg,
        s.text,
        className,
      )}
    >
      <span className={cn("h-1.5 w-1.5 rounded-full", s.dot)} />
      {label}
    </span>
  );
}
