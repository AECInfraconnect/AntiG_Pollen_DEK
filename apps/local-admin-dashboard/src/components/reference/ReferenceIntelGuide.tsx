import { CheckCircle2, ExternalLink, HelpCircle } from "lucide-react";
import {
  matchObserveGuideSignals,
  type ReferenceIntel,
} from "../../lib/entityReferenceIntel";
import { ReferenceIntelMark } from "./ReferenceIntelMark";
import { cn } from "@/lib/utils";

export function ReferenceIntelGuide({
  reference,
  observedTerms,
  compact = false,
}: {
  reference?: ReferenceIntel;
  observedTerms: Array<string | undefined | null>;
  compact?: boolean;
}) {
  const guide = reference?.observeGuide;
  if (!reference || !guide) return null;

  const signals = matchObserveGuideSignals(reference, observedTerms);
  const visibleSignals = compact ? signals.slice(0, 3) : signals;
  const matchedCount = signals.filter((signal) => signal.detected).length;

  return (
    <div className="rounded-md border bg-background/60 p-3">
      <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
        <div className="flex min-w-0 gap-3">
          <ReferenceIntelMark reference={reference} size="sm" />
          <div className="min-w-0">
            <div className="flex flex-wrap items-center gap-2">
              <h4 className="text-sm font-semibold">
                Known profile: {reference.title}
              </h4>
              <span className="rounded-full border px-2 py-0.5 text-[11px] text-muted-foreground">
                Definition
              </span>
              {matchedCount > 0 && (
                <span className="rounded-full border border-emerald-500/25 bg-emerald-500/10 px-2 py-0.5 text-[11px] text-emerald-700">
                  {matchedCount} matched signal(s)
                </span>
              )}
            </div>
            <p className="mt-1 text-xs leading-5 text-muted-foreground">
              {guide.summary}
            </p>
          </div>
        </div>
        <a
          href={reference.sourceUrl}
          target="_blank"
          rel="noreferrer"
          className="inline-flex shrink-0 items-center gap-1 text-xs text-primary hover:underline"
        >
          {reference.sourceLabel}
          <ExternalLink className="h-3 w-3" />
        </a>
      </div>

      <div className="mt-3 flex flex-wrap gap-1.5">
        {guide.probeKeys.slice(0, compact ? 8 : 14).map((key) => (
          <span
            key={key}
            className="rounded-full border bg-card px-2 py-0.5 text-[11px] text-muted-foreground"
          >
            {key}
          </span>
        ))}
      </div>

      <div className="mt-3 grid gap-2 lg:grid-cols-3">
        {visibleSignals.map((signal) => (
          <div
            key={signal.label}
            className={cn(
              "rounded-md border p-3 text-xs",
              signal.detected
                ? "border-emerald-500/25 bg-emerald-500/10"
                : "bg-card/70",
            )}
          >
            <div className="flex items-start gap-2">
              {signal.detected ? (
                <CheckCircle2 className="mt-0.5 h-3.5 w-3.5 shrink-0 text-emerald-600" />
              ) : (
                <HelpCircle className="mt-0.5 h-3.5 w-3.5 shrink-0 text-muted-foreground" />
              )}
              <div>
                <div className="font-medium">{signal.label}</div>
                <p className="mt-1 leading-5 text-muted-foreground">
                  {signal.meaning}
                </p>
              </div>
            </div>
            {!compact && (
              <div className="mt-2 space-y-1 text-muted-foreground">
                <p>{signal.detail}</p>
                <p>{signal.userAction}</p>
              </div>
            )}
          </div>
        ))}
      </div>

      {!compact && (
        <p className="mt-3 text-[11px] leading-5 text-muted-foreground">
          {guide.caveat}
        </p>
      )}
    </div>
  );
}
