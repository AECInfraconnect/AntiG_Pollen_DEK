import { CircleHelp } from "lucide-react";
import { getHelpTopic } from "../../lib/contextualHelp";

export function ContextualHelp({
  topicId,
  align = "left",
}: {
  topicId?: string;
  align?: "left" | "right";
}) {
  const topic = getHelpTopic(topicId);
  if (!topic) return null;

  return (
    <span className="group/help relative inline-flex align-middle">
      <button
        type="button"
        aria-label={`Help: ${topic.title}`}
        className="inline-flex h-6 w-6 items-center justify-center rounded-full border border-border/70 bg-background/80 text-muted-foreground transition-colors hover:border-primary/40 hover:text-primary focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
      >
        <CircleHelp className="h-3.5 w-3.5" />
      </button>
      <span
        role="tooltip"
        aria-hidden="true"
        className={`pointer-events-none absolute top-8 z-50 hidden w-80 max-w-[min(20rem,calc(100vw-2rem))] rounded-lg border border-border bg-popover p-3 text-left text-xs text-popover-foreground shadow-xl group-hover/help:block group-focus-within/help:block ${
          align === "right" ? "right-0" : "left-0"
        }`}
      >
        <span className="block text-sm font-semibold">{topic.title}</span>
        <span className="mt-1 block leading-5 text-muted-foreground">
          {topic.summary}
        </span>
        <span className="mt-2 block space-y-1">
          {topic.guidance.slice(0, 3).map((line) => (
            <span key={line} className="block leading-5">
              - {line}
            </span>
          ))}
        </span>
        <span className="mt-2 block border-t border-border/50 pt-2 text-[11px] text-muted-foreground">
          Docs: {topic.sourceDoc}
          {topic.sourceAnchor ? `#${topic.sourceAnchor}` : ""}
        </span>
      </span>
    </span>
  );
}
