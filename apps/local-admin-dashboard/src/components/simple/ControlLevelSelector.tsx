import { Bell, Eye, Hand, ShieldCheck } from "lucide-react";
import type { ControlLevel } from "../../services/types";

const LEVELS: Array<{
  id: ControlLevel;
  icon: any;
  title: string;
  description: string;
  advancedLabel: string;
}> = [
  {
    id: "observe",
    icon: Eye,
    title: "Watch only",
    description: "Record what happens without interrupting the AI app.",
    advancedLabel: "Observe",
  },
  {
    id: "warn",
    icon: Bell,
    title: "Warn me",
    description: "Let it continue, but make risky actions visible.",
    advancedLabel: "Warn",
  },
  {
    id: "ask",
    icon: Hand,
    title: "Ask first",
    description: "Pause and ask before this kind of action continues.",
    advancedLabel: "Require approval",
  },
  {
    id: "enforce",
    icon: ShieldCheck,
    title: "Block automatically",
    description: "Stop this action when the local setup can enforce it.",
    advancedLabel: "Enforce",
  },
];

export function ControlLevelSelector({
  value,
  onChange,
}: {
  value: ControlLevel;
  onChange: (level: ControlLevel) => void;
}) {
  return (
    <div className="grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-4">
      {LEVELS.map((level) => {
        const Icon = level.icon;
        const active = value === level.id;
        return (
          <button
            key={level.id}
            type="button"
            onClick={() => onChange(level.id)}
            className={`rounded-lg border p-4 text-left transition ${
              active
                ? "border-primary bg-primary/10 text-foreground"
                : "border-border bg-background hover:bg-muted"
            }`}
          >
            <div className="flex items-center gap-2">
              <span className="rounded-md bg-primary/10 p-2 text-primary">
                <Icon className="h-4 w-4" />
              </span>
              <div>
                <div className="text-sm font-semibold">{level.title}</div>
                <div className="text-[11px] text-muted-foreground">
                  {level.advancedLabel}
                </div>
              </div>
            </div>
            <p className="mt-3 text-xs leading-5 text-muted-foreground">
              {level.description}
            </p>
          </button>
        );
      })}
    </div>
  );
}
