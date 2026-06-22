
import type { PolicyPresetV2 } from "../../types/policy-presets";
import { PepTypeSelector } from "../presets/PepTypeSelector";

export function PepCapabilityMatrix({
  preset,
  selectedPeps,
  setSelectedPeps,
}: {
  preset: PolicyPresetV2;
  selectedPeps: string[];
  setSelectedPeps: (peps: string[]) => void;
}) {
  return (
    <div className="space-y-4">
      <h4 className="font-medium">Select Policy Enforcement Points (PEP)</h4>
      <div className="text-sm text-muted-foreground mb-4">
        The capabilities below represent what the discovered agents support.
      </div>
      <PepTypeSelector
        presetId={preset.id}
        recommendedPeps={preset.recommended_pep_types}
        selectedPeps={selectedPeps}
        onChange={setSelectedPeps}
      />
    </div>
  );
}
