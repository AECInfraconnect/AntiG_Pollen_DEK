import {
  Bot,
  Database,
  FileText,
  Globe2,
  Mail,
  Plug,
  ShieldAlert,
  TerminalSquare,
  Wrench,
} from "lucide-react";
import { Link } from "react-router-dom";
import { Collapsible } from "../ui";
import {
  capabilityTone,
  categoryLabel,
} from "../../features/user-activity/userActivityModel";
import type {
  UserActivityCategory,
  UserCapabilityItem,
} from "../../features/user-activity/types";
import { cn } from "../../lib/utils";

const toneClass: Record<string, string> = {
  success: "border-emerald-500/25 bg-emerald-500/10 text-emerald-700",
  info: "border-blue-500/25 bg-blue-500/10 text-blue-700",
  warning: "border-amber-500/25 bg-amber-500/10 text-amber-700",
  neutral: "border-border bg-background text-muted-foreground",
};

const categoryIcons: Partial<Record<UserActivityCategory, typeof Bot>> = {
  files: FileText,
  web: Globe2,
  email: Mail,
  commands: TerminalSquare,
  tools: Wrench,
  plugins: Plug,
  safety: ShieldAlert,
  ai_models: Bot,
  cost: Database,
};

function controlLabel(item: UserCapabilityItem) {
  if (item.can_block) return "Can watch and block";
  if (item.can_ask_first) return "Can ask first";
  if (item.can_warn) return "Can warn";
  if (item.can_watch) return "Watch only";
  return "Needs setup";
}

function observeLabel(item: UserCapabilityItem) {
  if (item.can_watch) return "Visible now";
  if (item.status === "needs_setup") return "Needs setup";
  return "Not visible yet";
}

function setupLabel(item: UserCapabilityItem) {
  if (item.setup_action_ids.length > 0) {
    return item.setup_action_ids
      .slice(0, 2)
      .map((id) => id.replace(/[_.:-]+/g, " "))
      .join(", ");
  }
  if (item.can_block) return "Ready for local rules";
  if (item.can_watch) return "Observe first, then tighten";
  return "Connect source or app setting";
}

export function ObservePostureMatrix({
  items,
  defaultExpanded = false,
  title = "Observe and control matrix",
  subtitle = "What Pollek can see, what it can control, and where setup is still needed.",
}: {
  items: UserCapabilityItem[];
  defaultExpanded?: boolean;
  title?: string;
  subtitle?: string;
}) {
  const watchable = items.filter((item) => item.can_watch).length;
  const controllable = items.filter((item) => item.can_block).length;
  const needsSetup = items.filter((item) => item.status === "needs_setup")
    .length;

  return (
    <Collapsible
      defaultExpanded={defaultExpanded}
      className="rounded-lg bg-card/60"
      contentClassName="bg-background/60 p-4"
      title={
        <div className="flex flex-col gap-2 lg:flex-row lg:items-center lg:justify-between">
          <div>
            <div className="text-sm font-semibold">{title}</div>
            <div className="text-xs font-normal text-muted-foreground">
              {subtitle}
            </div>
          </div>
          <div className="flex flex-wrap gap-2 text-xs font-normal text-muted-foreground">
            <span className="rounded-full border bg-background px-2.5 py-1">
              {watchable} visible
            </span>
            <span className="rounded-full border bg-background px-2.5 py-1">
              {controllable} controllable
            </span>
            <span className="rounded-full border bg-background px-2.5 py-1">
              {needsSetup} setup
            </span>
          </div>
        </div>
      }
    >
      <div className="overflow-hidden rounded-lg border">
        <div className="hidden grid-cols-[1.2fr_0.9fr_0.9fr_1.3fr] gap-3 border-b bg-muted/40 px-3 py-2 text-xs font-semibold uppercase tracking-wide text-muted-foreground md:grid">
          <span>Surface</span>
          <span>Observe</span>
          <span>Control</span>
          <span>Evidence and setup</span>
        </div>
        <div className="divide-y">
          {items.map((item) => {
            const Icon = categoryIcons[item.category] ?? Bot;
            const tone = capabilityTone(item.status);
            return (
              <div
                key={item.id}
                className="grid gap-3 px-3 py-3 text-sm md:grid-cols-[1.2fr_0.9fr_0.9fr_1.3fr] md:items-center"
              >
                <div className="flex items-start gap-3">
                  <div className={cn("rounded-lg p-2", toneClass[tone])}>
                    <Icon className="h-4 w-4" />
                  </div>
                  <div className="min-w-0">
                    <div className="font-medium">
                      {item.simple_label || categoryLabel(item.category)}
                    </div>
                    <div className="mt-1 text-xs leading-5 text-muted-foreground">
                      {item.plain_description}
                    </div>
                  </div>
                </div>
                <span
                  className={cn(
                    "w-fit rounded-full border px-2.5 py-1 text-xs",
                    toneClass[item.can_watch ? "success" : tone],
                  )}
                >
                  {observeLabel(item)}
                </span>
                <span
                  className={cn(
                    "w-fit rounded-full border px-2.5 py-1 text-xs",
                    toneClass[item.can_block ? "success" : tone],
                  )}
                >
                  {controlLabel(item)}
                </span>
                <div className="text-xs leading-5 text-muted-foreground">
                  <div className="font-medium text-foreground/85">
                    {item.why}
                  </div>
                  <div className="mt-1">{setupLabel(item)}</div>
                </div>
              </div>
            );
          })}
        </div>
      </div>
      <div className="mt-3 flex flex-wrap gap-2">
        <Link
          to="/setup"
          className="inline-flex h-9 items-center gap-2 rounded-md border bg-background px-3 text-sm hover:bg-muted"
        >
          <Wrench className="h-4 w-4" />
          Setup details
        </Link>
        <Link
          to="/allowed-blocked"
          className="inline-flex h-9 items-center gap-2 rounded-md border bg-background px-3 text-sm hover:bg-muted"
        >
          <ShieldAlert className="h-4 w-4" />
          Create or review rules
        </Link>
      </div>
    </Collapsible>
  );
}
