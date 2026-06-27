import {
  Bot,
  Cloud,
  FolderOpen,
  Globe2,
  MessageCircle,
  Network,
  Plug,
  Server,
  Sparkles,
} from "lucide-react";
import type { ReferenceIntel } from "../../lib/entityReferenceIntel";

const iconMap: Record<string, any> = {
  bot: Bot,
  cloud: Cloud,
  folder: FolderOpen,
  globe: Globe2,
  message: MessageCircle,
  network: Network,
  plug: Plug,
  server: Server,
  sparkles: Sparkles,
};

export function ReferenceIntelMark({
  reference,
  size = "md",
}: {
  reference?: ReferenceIntel;
  size?: "sm" | "md";
}) {
  const visual = reference?.visualIdentity;
  if (!visual) return null;

  const Icon = iconMap[visual.icon] ?? Sparkles;
  const dimensions = size === "sm" ? "h-8 w-8 text-[10px]" : "h-10 w-10 text-xs";
  const iconSize = size === "sm" ? "h-3.5 w-3.5" : "h-4 w-4";

  return (
    <div
      className={`flex shrink-0 items-center justify-center rounded-lg font-bold shadow-sm ring-1 ring-border/50 ${dimensions}`}
      style={{
        backgroundColor: visual.background,
        color: visual.foreground,
      }}
      title={`${reference.title} visual identity. Source: ${visual.sourceLabel}`}
      aria-label={`${reference.title} visual identity`}
    >
      {visual.label ? visual.label : <Icon className={iconSize} />}
    </div>
  );
}

export function ReferenceIntelInline({
  reference,
}: {
  reference?: ReferenceIntel;
}) {
  if (!reference) return null;
  return (
    <span className="inline-flex items-center gap-1.5 align-middle">
      <ReferenceIntelMark reference={reference} size="sm" />
      <span>{reference.title}</span>
    </span>
  );
}
