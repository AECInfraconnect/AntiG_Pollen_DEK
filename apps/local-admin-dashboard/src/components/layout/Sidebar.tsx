import { Link, useLocation } from "react-router-dom";
import { cn } from "@/lib/utils";
import {
  LayoutDashboard,
  Server,
  Network,
  Wrench,
  ShieldAlert,
  Activity,
  FileKey,
  Users,
  Database,
  Settings as SettingsIcon,
  UserCircle,
  Search,
  Lightbulb,
  ShieldCheck,
  Zap,
  Globe,
  Puzzle,
} from "lucide-react";
import { useTranslation } from "react-i18next";

const groups = [
  {
    title: "Dashboard",
    items: [{ name: "Overview", href: "/", icon: LayoutDashboard }],
  },
  {
    title: "AI Ecosystem",
    items: [
      { name: "Agents & Models", href: "/agents", icon: Users },
      { name: "Integrations", href: "/integrations", icon: Wrench },
      { name: "Plugin Marketplace", href: "/plugin-marketplace", icon: Puzzle },
    ],
  },
  {
    title: "Security & Policies",
    items: [
      { name: "Policy Presets", href: "/policy-presets", icon: ShieldCheck },
      {
        name: "Policy Suggestions",
        href: "/policy-suggestions",
        icon: Lightbulb,
      },
      { name: "Active Policies", href: "/policies", icon: FileKey },
    ],
  },
  {
    title: "Monitoring & Audit",
    items: [
      { name: "Alerts & Shadow AI", href: "/alerts", icon: ShieldAlert },
      { name: "Audit Logs", href: "/audit", icon: Activity },
      { name: "Cost & Tokens", href: "/cost-ledger", icon: Zap },
    ],
  },
  {
    title: "System Settings",
    items: [
      { name: "Identities", href: "/identities", icon: Network },
      { name: "Entities", href: "/entities", icon: Users },
      { name: "Data Resources", href: "/resources", icon: Database },
      { name: "Simulator", href: "/simulator", icon: Activity },
      { name: "Bundles & Sync", href: "/bundles", icon: Server },
      { name: "Auto Discovery", href: "/discovery", icon: Search },
      { name: "Global Settings", href: "/settings", icon: SettingsIcon },
    ],
  },
];

export function Sidebar() {
  const location = useLocation();
  const { t } = useTranslation();

  return (
    <div className="flex h-full w-64 flex-col border-r bg-card/50 backdrop-blur-xl">
      <div className="flex h-16 items-center border-b px-6 pt-4 pb-2">
        <div className="flex items-center gap-2 w-full">
          <Globe className="h-6 w-6 text-primary flex-shrink-0" />
          <h1 className="flex flex-col leading-none">
            <span className="text-lg font-bold bg-gradient-to-r from-primary to-accent bg-clip-text text-transparent tracking-wide">
              POLLEK
            </span>
            <span className="text-[10px] uppercase font-bold tracking-wider text-muted-foreground/80 mt-0.5">
              Admin Dashboard
            </span>
          </h1>
          <span className="ml-auto rounded-md bg-primary/10 px-2 py-0.5 text-[10px] font-bold text-primary self-start mt-1">
            v2
          </span>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto py-4 [&::-webkit-scrollbar]:hidden [-ms-overflow-style:none] [scrollbar-width:none]">
        <nav className="space-y-6 px-3">
          {groups.map((group) => (
            <div key={group.title}>
              {group.title !== "Dashboard" && (
                <h3 className="px-3 text-xs font-bold uppercase tracking-wider text-muted-foreground/70 mb-2">
                  {t(group.title)}
                </h3>
              )}
              <div className="space-y-1">
                {group.items.map((item) => {
                  const isActive =
                    location.pathname === item.href ||
                    (item.href !== "/" &&
                      location.pathname.startsWith(item.href));
                  return (
                    <Link
                      key={item.name}
                      to={item.href}
                      className={cn(
                        isActive
                          ? "bg-primary/10 text-primary font-semibold shadow-[0_0_15px_rgba(124,58,237,0.15)]"
                          : "text-muted-foreground hover:bg-muted/50 hover:text-foreground hover-glow",
                        "group flex items-center rounded-md px-3 py-2 text-sm font-medium transition-all duration-300",
                      )}
                    >
                      <item.icon
                        className={cn(
                          isActive
                            ? "text-primary"
                            : "text-muted-foreground group-hover:text-foreground",
                          "mr-3 h-5 w-5 flex-shrink-0 transition-colors",
                        )}
                        aria-hidden="true"
                      />
                      {t(item.name)}
                    </Link>
                  );
                })}
              </div>
            </div>
          ))}
        </nav>
      </div>

      <div className="border-t p-4">
        <div className="flex items-center gap-3 rounded-lg bg-muted/50 p-3 hover:bg-muted/80 cursor-pointer transition-colors">
          <div className="h-8 w-8 rounded-full bg-primary/20 flex items-center justify-center">
            <UserCircle className="h-4 w-4 text-primary" />
          </div>
          <div className="flex flex-col">
            <span className="text-sm font-medium">Local Admin</span>
            <span className="text-xs text-muted-foreground">tenant: local</span>
          </div>
        </div>
      </div>
    </div>
  );
}
