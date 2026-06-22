
import type { PolicyPresetV2 } from "../../types/policy-presets";

export function PolicyGoalSelector({ preset }: { preset: PolicyPresetV2 }) {
  return (
    <div className="space-y-4">
      <h4 className="font-medium">Goal Overview</h4>
      <p className="text-sm text-muted-foreground">
        {preset.long_description}
      </p>
    </div>
  );
}
