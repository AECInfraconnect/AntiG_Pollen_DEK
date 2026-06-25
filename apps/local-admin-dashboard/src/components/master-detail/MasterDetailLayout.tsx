import type { ReactNode } from "react";
import { ChevronLeft } from "lucide-react";
import { cn } from "@/lib/utils";
import { CardSkeleton } from "./CardSkeleton";
export function MasterDetailLayout<T>({
  items,
  selectedId,
  onSelect,
  idSelector,
  renderCard,
  renderDetail,
  toolbar,
  emptyState,
  loading,
}: {
  items: T[];
  selectedId?: string;
  onSelect: (id: string) => void;
  idSelector: (item: T) => string;
  renderCard: (item: T, selected: boolean) => ReactNode;
  renderDetail: (item: T) => ReactNode;
  toolbar?: ReactNode;
  emptyState?: ReactNode;
  loading?: boolean;
}) {
  const selected = items.find((i) => idSelector(i) === selectedId) ?? items[0];

  if (loading) {
    return (
      <div className="space-y-4 flex flex-col h-[calc(100vh-10rem)]">
        {toolbar}
        <div className="flex-1 min-h-0 grid gap-4 md:grid-cols-[minmax(280px,360px)_1fr]">
          <div className="space-y-2 overflow-y-auto pr-1 pb-4 no-scrollbar">
            {Array.from({ length: 5 }).map((_, i) => (
              <CardSkeleton key={i} />
            ))}
          </div>
          <div className="hidden md:flex flex-col min-h-0 bg-card/30 rounded-xl border shadow-sm p-6 items-center justify-center">
            <div className="animate-pulse space-y-4 w-full max-w-md">
              <div className="h-8 bg-muted rounded w-1/3 mx-auto" />
              <div className="h-32 bg-muted rounded w-full" />
            </div>
          </div>
        </div>
      </div>
    );
  }

  if (!loading && items.length === 0) {
    return (
      <div className="space-y-4">
        {toolbar}
        {emptyState}
      </div>
    );
  }

  return (
    <div className="space-y-4 flex flex-col h-[calc(100vh-10rem)]">
      {toolbar && (
        <div
          className={cn(
            "transition-all",
            selectedId ? "hidden md:block" : "block",
          )}
        >
          {toolbar}
        </div>
      )}
      <div className="flex-1 min-h-0 grid gap-4 md:grid-cols-[minmax(280px,360px)_1fr]">
        <div
          role="listbox"
          aria-label="Items"
          className={cn(
            "space-y-2 overflow-y-auto pr-1 pb-4 no-scrollbar",
            selectedId ? "hidden md:block" : "block",
          )}
        >
          {items.map((item) => {
            const id = idSelector(item);
            const isSelected =
              id === (selected ? idSelector(selected) : selectedId);

            return (
              <div
                key={id}
                role="option"
                tabIndex={0}
                aria-selected={isSelected}
                onClick={() => onSelect(id)}
                onKeyDown={(event) => {
                  if (event.key === "Enter" || event.key === " ") {
                    event.preventDefault();
                    onSelect(id);
                  }
                }}
                className="block w-full text-left focus-visible:outline-none"
              >
                {renderCard(item, isSelected)}
              </div>
            );
          })}
        </div>
        <div
          className={cn(
            "flex flex-col min-h-0 bg-card/30 rounded-xl border shadow-sm",
            selectedId ? "flex" : "hidden md:flex",
          )}
        >
          {selected && (
            <>
              <div className="md:hidden flex items-center p-2 border-b border-border bg-card">
                <button
                  onClick={() => {
                    // We need a way to clear selection. The easiest is to call onSelect with empty string.
                    onSelect("");
                  }}
                  className="flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground px-2 py-1 rounded-md"
                >
                  <ChevronLeft className="h-4 w-4" /> Back
                </button>
              </div>
              {renderDetail(selected)}
            </>
          )}
        </div>
      </div>
    </div>
  );
}
