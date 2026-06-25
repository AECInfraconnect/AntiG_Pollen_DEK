import { Link, useLocation } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";
import { NAV } from "../../config/navigation";
import { useMode } from "../../context/ModeContext";
import { ModeSwitcher } from "./ModeSwitcher";
import { useState, useEffect } from "react";
import { ChevronLeft, X } from "lucide-react";

export function Sidebar({
  mobileMenuOpen,
  setMobileMenuOpen,
}: {
  mobileMenuOpen?: boolean;
  setMobileMenuOpen?: (open: boolean) => void;
}) {
  const { mode } = useMode();
  const { pathname } = useLocation();
  const { i18n } = useTranslation();
  const th = i18n.language === "th";

  const [collapsed, setCollapsed] = useState(() => {
    return localStorage.getItem("pollek.sidebar.collapsed") === "true";
  });

  useEffect(() => {
    localStorage.setItem("pollek.sidebar.collapsed", String(collapsed));
  }, [collapsed]);

  // Handle mobile drawer close on route change
  useEffect(() => {
    if (mobileMenuOpen && setMobileMenuOpen) {
      setMobileMenuOpen(false);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [pathname]);

  const SidebarContent = () => (
    <>
      <div
        className={cn(
          "flex h-16 shrink-0 flex-col justify-center border-b border-border transition-all",
          collapsed ? "px-2 items-center" : "px-5",
        )}
      >
        <div className="flex items-center justify-between w-full">
          {!collapsed && (
            <div>
              <div className="text-lg font-bold tracking-widest text-primary leading-none mt-1">
                POLLEK
              </div>
              <div className="text-[10px] text-muted-foreground uppercase tracking-widest mt-1">
                AI Local Enforcement Kit
              </div>
            </div>
          )}
          {collapsed && (
            <div className="text-xl font-bold tracking-widest text-primary leading-none mt-1 text-center w-full">
              P
            </div>
          )}
          {/* Mobile close button inside header area */}
          <button
            onClick={() => setMobileMenuOpen?.(false)}
            className="md:hidden rounded-md p-1.5 text-muted-foreground hover:bg-muted hover:text-foreground ml-auto"
          >
            <X className="h-5 w-5" />
          </button>
        </div>
      </div>

      <nav className="flex-1 space-y-7 overflow-y-auto px-3 py-5 no-scrollbar">
        {NAV.map((group) => {
          const items = group.items.filter((i) => i.modes.includes(mode));
          if (!items.length) return null;
          return (
            <div key={group.id} className={cn(collapsed && "flex flex-col items-center")}>
              {!collapsed && (
                <div className="px-3 pb-2 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
                  {th ? group.th : group.en}
                </div>
              )}
              {collapsed && (
                <div className="mb-2 h-px w-8 bg-border" />
              )}
              <div className="space-y-1 w-full">
                {items.map((item) => {
                  const active =
                    pathname === item.href ||
                    (item.href !== "/" && pathname.startsWith(item.href));
                  const Icon = item.icon;
                  return (
                    <Link
                      key={item.id}
                      to={item.href}
                      title={collapsed ? (th ? item.th : item.en) : undefined}
                      aria-current={active ? "page" : undefined}
                      className={cn(
                        "relative flex items-center rounded-lg py-2 text-sm transition focus-visible:ring-2 focus-visible:ring-primary",
                        collapsed ? "justify-center px-0 w-10 mx-auto" : "gap-3 px-3 w-full",
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
                      <Icon className={cn("shrink-0", collapsed ? "h-5 w-5" : "h-4 w-4")} />
                      {!collapsed && <span className="truncate">{th ? item.th : item.en}</span>}
                    </Link>
                  );
                })}
              </div>
            </div>
          );
        })}
      </nav>

      <div className="border-t border-border p-3 flex flex-col gap-2 relative">
        <button
          onClick={() => setCollapsed(!collapsed)}
          className={cn(
            "hidden md:flex absolute -right-3 -top-3 h-6 w-6 items-center justify-center rounded-full border border-border bg-background shadow-sm text-muted-foreground hover:text-foreground transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary",
            collapsed && "rotate-180"
          )}
        >
          <ChevronLeft className="h-3 w-3" />
        </button>

        {!collapsed && (
          <div className="flex items-center gap-3 rounded-lg bg-muted/50 p-2 cursor-pointer transition-colors hover:bg-muted/80">
            <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-primary/20">
              <div className="h-4 w-4 text-primary" />
            </div>
            <div className="flex flex-col min-w-0">
              <span className="text-sm font-medium truncate">Local Admin</span>
            </div>
          </div>
        )}
        {collapsed && (
          <div className="flex justify-center mb-2 mt-2">
            <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-primary/20">
              <div className="h-4 w-4 text-primary" />
            </div>
          </div>
        )}
        <div className={cn(collapsed ? "flex justify-center" : "")}>
          <ModeSwitcher collapsed={collapsed} />
        </div>
      </div>
    </>
  );

  return (
    <>
      {/* Mobile Backdrop */}
      {mobileMenuOpen && (
        <div
          className="fixed inset-0 z-40 bg-background/80 backdrop-blur-sm md:hidden"
          onClick={() => setMobileMenuOpen?.(false)}
        />
      )}
      
      {/* Sidebar Container */}
      <aside
        aria-label="Main navigation"
        className={cn(
          "fixed inset-y-0 left-0 z-50 flex h-full flex-col border-r border-border bg-card/95 backdrop-blur-xl transition-all duration-300 md:static",
          collapsed ? "w-20" : "w-64",
          mobileMenuOpen ? "translate-x-0" : "-translate-x-full md:translate-x-0"
        )}
      >
        <SidebarContent />
      </aside>
    </>
  );
}
