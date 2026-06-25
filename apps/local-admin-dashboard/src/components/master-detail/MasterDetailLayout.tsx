import type { ReactNode } from "react";

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
      {toolbar}
      <div className="flex-1 min-h-0 grid gap-4 md:grid-cols-[minmax(280px,360px)_1fr]">
        <div
          role="listbox"
          aria-label="Items"
          className="space-y-2 overflow-y-auto pr-1 pb-4 no-scrollbar"
        >
          {items.map((item) => (
            <button
              key={idSelector(item)}
              role="option"
              aria-selected={
                idSelector(item) ===
                (selected ? idSelector(selected) : selectedId)
              }
              onClick={() => onSelect(idSelector(item))}
              className="block w-full text-left focus-visible:outline-none"
            >
              {renderCard(
                item,
                idSelector(item) ===
                  (selected ? idSelector(selected) : selectedId),
              )}
            </button>
          ))}
        </div>
        <div className="hidden md:flex flex-col min-h-0 bg-card/30 rounded-xl border shadow-sm">
          {selected && renderDetail(selected)}
        </div>
      </div>
    </div>
  );
}
