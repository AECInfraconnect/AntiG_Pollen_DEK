import { Link, useLocation } from "react-router-dom";
import { cn } from "@/lib/utils";
import * as LucideIcons from "lucide-react";
import { useTranslation } from "react-i18next";
import { useMode } from "../../context/ModeContext";
import { getNavItems } from "../../navigation/menu";

export function Sidebar() {
  const location = useLocation();
  const { mode } = useMode();
  // We do not have i18n fully set up with keys that match menu.ts dynamic label, 
  // so we will just use label.en or label.th based on current language
  const { i18n } = useTranslation();
  const currentLang = (i18n.language || "en").startsWith("th") ? "th" : "en";

  const navItems = getNavItems(mode);

  return (
    <div className="flex h-full w-64 flex-col border-r bg-card/50 backdrop-blur-xl">
      <div className="flex h-20 items-center justify-center border-b px-6 py-4">
        <img
          src="/POLLEK_LOGO.png"
          alt="Pollek Local Enforcement Kit"
          className="h-full w-auto object-contain mix-blend-multiply dark:mix-blend-screen dark:brightness-200 dark:contrast-200"
        />
      </div>

      <div
        className="flex-1 overflow-y-auto py-4 no-scrollbar"
        style={{ msOverflowStyle: "none", scrollbarWidth: "none" }}
      >
        <nav className="space-y-2 px-3">
          {navItems.map((item) => {
            const isActive =
              location.pathname === item.path ||
              (item.path !== "/" && location.pathname.startsWith(item.path));
            
            // @ts-ignore - dynamic icon access
            const IconComp = LucideIcons[item.icon.split("-").map(p => p.charAt(0).toUpperCase() + p.slice(1)).join("")] || LucideIcons.Circle;

            return (
              <Link
                key={item.id}
                to={item.path}
                className={cn(
                  isActive
                    ? "bg-primary/10 text-primary font-semibold shadow-[0_0_15px_rgba(124,58,237,0.15)]"
                    : "text-muted-foreground hover:bg-muted/50 hover:text-foreground hover-glow",
                  "group flex items-center rounded-md px-3 py-2 text-sm font-medium transition-all duration-300",
                )}
              >
                <IconComp
                  className={cn(
                    isActive
                      ? "text-primary"
                      : "text-muted-foreground group-hover:text-foreground",
                    "mr-3 h-5 w-5 flex-shrink-0 transition-colors",
                  )}
                  aria-hidden="true"
                />
                {item.label[currentLang as "en" | "th"]}
              </Link>
            );
          })}
        </nav>
      </div>

      <div className="border-t p-4">
        <div className="flex items-center gap-3 rounded-lg bg-muted/50 p-3 hover:bg-muted/80 cursor-pointer transition-colors">
          <div className="h-8 w-8 rounded-full bg-primary/20 flex items-center justify-center">
            <LucideIcons.UserCircle className="h-4 w-4 text-primary" />
          </div>
          <div className="flex flex-col">
            <span className="text-sm font-medium">Local Admin</span>
            <span className="text-xs text-muted-foreground">mode: {mode.replace("_", " ")}</span>
          </div>
        </div>
      </div>
    </div>
  );
}

