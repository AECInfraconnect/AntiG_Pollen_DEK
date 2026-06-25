import { Link, useLocation } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";
import { NAV } from "../../config/navigation";
import { useMode } from "../../context/ModeContext";
import { ModeSwitcher } from "./ModeSwitcher";

export function Sidebar() {
  const { mode } = useMode();
  const { pathname } = useLocation();
  const { i18n } = useTranslation();
  const th = i18n.language === "th";

  return (
    <aside
      aria-label="Main navigation"
      className="flex h-full w-64 flex-col border-r border-border bg-card/50 backdrop-blur-xl"
    >
      <div className="flex h-16 items-center gap-2 border-b border-border px-5">
        <span className="text-lg font-semibold tracking-tight">POLLEK</span>
        <span className="rounded-md bg-primary/10 px-2 py-0.5 text-[10px] font-medium text-primary">
          LOCAL
        </span>
      </div>

      <nav className="flex-1 space-y-7 overflow-y-auto px-3 py-5">
        {NAV.map((group) => {
          const items = group.items.filter((i) => i.modes.includes(mode));
          if (!items.length) return null;
          return (
            <div key={group.id}>
              <div className="px-3 pb-2 text-xs font-medium uppercase tracking-wider text-muted-foreground">
                {th ? group.th : group.en}
              </div>
              <div className="space-y-1">
                {items.map((item) => {
                  const active =
                    pathname === item.href ||
                    (item.href !== "/" && pathname.startsWith(item.href));
                  const Icon = item.icon;
                  return (
                    <Link
                      key={item.id}
                      to={item.href}
                      aria-current={active ? "page" : undefined}
                      className={cn(
                        "relative flex items-center gap-3 rounded-lg px-3 py-2 text-sm transition focus-visible:ring-2 focus-visible:ring-primary",
                        item.primary &&
                          !active &&
                          "bg-primary text-primary-foreground hover:bg-primary/90 shadow-lg shadow-primary/20",
                        active &&
                          "bg-primary/10 text-primary before:absolute before:left-0 before:top-1.5 before:bottom-1.5 before:w-0.5 before:rounded-full before:bg-primary",
                        !active &&
                          !item.primary &&
                          "text-foreground/80 hover:bg-muted hover:text-foreground",
                      )}
                    >
                      <Icon className="h-4 w-4 shrink-0" />
                      <span className="truncate">{th ? item.th : item.en}</span>
                    </Link>
                  );
                })}
              </div>
            </div>
          );
        })}
      </nav>

      <div className="border-t border-border p-3 flex flex-col gap-2">
        <div className="flex items-center gap-3 rounded-lg bg-muted/50 p-2 cursor-pointer transition-colors hover:bg-muted/80">
          <div className="flex h-8 w-8 items-center justify-center rounded-full bg-primary/20">
            <div className="h-4 w-4 text-primary" />
          </div>
          <div className="flex flex-col">
            <span className="text-sm font-medium">Local Admin</span>
          </div>
        </div>
        <ModeSwitcher />
      </div>
    </aside>
  );
}
