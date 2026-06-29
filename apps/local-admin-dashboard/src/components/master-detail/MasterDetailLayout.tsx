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
  renderGroupHeader,
  toolbar,
  emptyState,
  loading,
  masterLayout = "list",
  masterListClassName,
  detailBackLabel = "Back to all records",
  itemClassName,
}: {
  items: T[];
  selectedId?: string;
  onSelect: (id: string) => void;
  idSelector: (item: T) => string;
  renderCard: (item: T, selected: boolean) => ReactNode;
  renderDetail: (item: T) => ReactNode;
  renderGroupHeader?: (item: T, index: number, prevItem: T | null) => ReactNode;
  toolbar?: ReactNode;
  emptyState?: ReactNode;
  loading?: boolean;
  masterLayout?: "list" | "grid";
  masterListClassName?: string;
  detailBackLabel?: string;
  itemClassName?: (item: T, selected: boolean) => string | undefined;
}) {
  const selected = selectedId
    ? items.find((i) => idSelector(i) === selectedId)
    : undefined;
  const masterListClass = cn(
    masterLayout === "grid"
      ? "grid gap-3 sm:grid-cols-2 2xl:grid-cols-3"
      : "space-y-2",
    masterListClassName,
  );

  if (loading) {
    return (
      <div className="space-y-4">
        {toolbar}
        <div className={masterListClass}>
          {Array.from({ length: 6 }).map((_, i) => (
            <CardSkeleton key={i} />
          ))}
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

  if (selectedId && selected) {
    return (
      <div className="space-y-4">
        <button
          type="button"
          onClick={() => onSelect("")}
          className="inline-flex items-center gap-2 rounded-lg text-sm font-medium text-muted-foreground transition-colors hover:text-foreground"
        >
          <ChevronLeft className="h-4 w-4" />
          {detailBackLabel}
        </button>
        <div className="min-w-0">{renderDetail(selected)}</div>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {toolbar}
      <div role="listbox" aria-label="Items" className={masterListClass}>
        {items.map((item, index) => {
          const id = idSelector(item);
          const prevItem = index > 0 ? items[index - 1] : null;
          const extraItemClass = itemClassName?.(item, false);
          const groupHeader = renderGroupHeader
            ? renderGroupHeader(item, index, prevItem)
            : null;

          return (
            <div
              key={`${id}-${index}`}
              className={groupHeader ? "contents" : extraItemClass}
            >
              {groupHeader && (
                <div className="col-span-full">{groupHeader}</div>
              )}
              <div
                role="option"
                tabIndex={0}
                aria-selected={false}
                onClick={() => onSelect(id)}
                onKeyDown={(event) => {
                  if (event.key === "Enter" || event.key === " ") {
                    event.preventDefault();
                    onSelect(id);
                  }
                }}
                className={cn(
                  "block h-full w-full cursor-pointer text-left focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2",
                  groupHeader && extraItemClass,
                )}
              >
                {renderCard(item, false)}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
