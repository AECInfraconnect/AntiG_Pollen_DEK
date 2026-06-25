export function CardSkeleton() {
  return (
    <div className="rounded-xl border border-border bg-card/40 p-4 shadow-sm animate-pulse">
      <div className="flex items-start gap-3">
        <div className="mt-0.5 rounded-lg p-2 bg-muted h-8 w-8" />
        <div className="min-w-0 flex-1 space-y-2">
          <div className="flex items-center justify-between gap-2">
            <div className="h-4 w-32 bg-muted rounded" />
            <div className="h-4 w-16 bg-muted rounded-full" />
          </div>
          <div className="h-3 w-48 bg-muted rounded" />
          <div className="mt-2 flex flex-wrap gap-x-4 gap-y-1">
            <div className="h-3 w-20 bg-muted/60 rounded" />
            <div className="h-3 w-24 bg-muted/60 rounded" />
          </div>
        </div>
      </div>
    </div>
  );
}
