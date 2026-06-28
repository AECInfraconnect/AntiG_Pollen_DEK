import {
  useEffect,
  useMemo,
  useRef,
  useState,
  type ComponentType,
  type KeyboardEvent,
} from "react";
import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { Search } from "lucide-react";
import { labelForLanguage, NAV } from "@/config/navigation";
import { useMode } from "@/context/ModeContext";
import { cn } from "@/lib/utils";

interface CommandItem {
  id: string;
  group: string;
  label: string;
  href: string;
  icon: ComponentType<{ className?: string }>;
}

export function CommandPalette({
  open,
  onClose,
}: {
  open: boolean;
  onClose: () => void;
}) {
  const navigate = useNavigate();
  const { mode } = useMode();
  const { i18n, t } = useTranslation();
  const inputRef = useRef<HTMLInputElement | null>(null);
  const [query, setQuery] = useState("");
  const [activeIndex, setActiveIndex] = useState(0);

  const commands = useMemo<CommandItem[]>(
    () =>
      NAV.flatMap((group) =>
        group.items
          .filter((item) => item.modes.includes(mode))
          .map((item) => ({
            id: item.id,
            group: labelForLanguage(group, i18n.language),
            label: labelForLanguage(item, i18n.language),
            href: item.href,
            icon: item.icon,
          })),
      ),
    [i18n.language, mode],
  );

  const results = useMemo(() => {
    const normalized = query.trim().toLowerCase();
    if (!normalized) return commands;
    return commands.filter((command) =>
      [command.label, command.group, command.id, command.href]
        .join(" ")
        .toLowerCase()
        .includes(normalized),
    );
  }, [commands, query]);

  useEffect(() => {
    if (!open) return;
    setQuery("");
    setActiveIndex(0);
    requestAnimationFrame(() => inputRef.current?.focus());
  }, [open]);

  useEffect(() => {
    setActiveIndex(0);
  }, [query]);

  if (!open) return null;

  const go = (item: CommandItem) => {
    navigate(item.href);
    onClose();
  };

  const onKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      setActiveIndex((index) => Math.min(index + 1, results.length - 1));
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      setActiveIndex((index) => Math.max(index - 1, 0));
    } else if (event.key === "Enter") {
      event.preventDefault();
      const item = results[activeIndex];
      if (item) go(item);
    } else if (event.key === "Escape") {
      event.preventDefault();
      onClose();
    }
  };

  return (
    <div className="fixed inset-0 z-[60] flex items-start justify-center p-4 pt-[12vh]">
      <button
        type="button"
        className="fixed inset-0 bg-background/75 backdrop-blur-sm"
        onClick={onClose}
        aria-label="Close command palette"
      />
      <div
        role="dialog"
        aria-modal="true"
        aria-label={t("command.dialogLabel")}
        className="relative z-[61] w-full max-w-xl overflow-hidden rounded-lg border border-border bg-popover shadow-2xl"
      >
        <div className="flex items-center gap-2 border-b border-border px-4">
          <Search
            className="h-4 w-4 shrink-0 text-muted-foreground"
            aria-hidden="true"
          />
          <input
            ref={inputRef}
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            onKeyDown={onKeyDown}
            role="combobox"
            aria-expanded="true"
            aria-controls="command-listbox"
            aria-activedescendant={
              results[activeIndex]
                ? `command-${results[activeIndex].id}`
                : undefined
            }
            placeholder={t("command.searchPlaceholder")}
            className="h-12 w-full bg-transparent text-sm outline-none placeholder:text-muted-foreground"
          />
          <kbd className="hidden rounded border border-border bg-muted px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground sm:block">
            Esc
          </kbd>
        </div>

        <ul
          id="command-listbox"
          role="listbox"
          className="max-h-80 overflow-y-auto p-2"
        >
          {results.length === 0 ? (
            <li className="px-3 py-8 text-center text-sm text-muted-foreground">
              {t("common.noResults")}
            </li>
          ) : (
            results.map((item, index) => {
              const Icon = item.icon;
              const active = index === activeIndex;
              return (
                <li
                  key={item.id}
                  id={`command-${item.id}`}
                  role="option"
                  aria-selected={active}
                  onMouseEnter={() => setActiveIndex(index)}
                  onClick={() => go(item)}
                  className={cn(
                    "flex cursor-pointer items-center gap-3 rounded-md px-3 py-2 text-sm",
                    active
                      ? "bg-primary/10 text-foreground"
                      : "text-muted-foreground",
                  )}
                >
                  <Icon className="h-4 w-4 shrink-0" />
                  <span className="min-w-0 flex-1 truncate">{item.label}</span>
                  <span className="truncate text-xs text-muted-foreground/80">
                    {item.group}
                  </span>
                </li>
              );
            })
          )}
        </ul>
      </div>
    </div>
  );
}
