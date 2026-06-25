import { cn } from "@/lib/utils";

export function CardSkeleton({ count = 3 }: { count?: number }) {
  return (
    <>
      {Array.from({ length: count }).map((_, i) => (
        <div
          key={i}
          className={cn(
            "rounded-xl border border-border bg-card/40 p-4 mb-2 animate-pulse",
          )}
        >
          <div className="flex items-start gap-3">
            <div className="mt-0.5 rounded-lg h-8 w-8 bg-muted" />
            <div className="min-w-0 flex-1 space-y-2">
              <div className="flex items-center justify-between gap-2">
                <div className="h-4 w-1/2 bg-muted rounded" />
                <div className="h-4 w-16 bg-muted rounded-full" />
              </div>
              <div className="h-3 w-1/3 bg-muted rounded" />
              <div className="mt-2 flex gap-4">
                <div className="h-3 w-12 bg-muted rounded" />
                <div className="h-3 w-12 bg-muted rounded" />
              </div>
            </div>
          </div>
        </div>
      ))}
    </>
  );
}
