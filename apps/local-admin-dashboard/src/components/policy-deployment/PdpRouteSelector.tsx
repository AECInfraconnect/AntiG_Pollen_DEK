
import type { ControlMode } from "../../types/policy-presets";

export function PdpRouteSelector({
  controlMode,
  setControlMode,
}: {
  controlMode: ControlMode;
  setControlMode: (mode: ControlMode) => void;
}) {
  return (
    <div className="space-y-4">
      <h4 className="font-medium">Policy Decision Point (PDP) Route</h4>
      <div className="p-4 border rounded bg-muted/20">
        <label className="flex items-center gap-2">
          <input type="radio" checked readOnly />
          <span>Local Cedar (Primary) with Cloud Fallback</span>
        </label>
      </div>
      <h4 className="font-medium mt-4">Enforcement Mode</h4>
      <div className="grid grid-cols-2 md:grid-cols-3 gap-3">
        {["observe", "warn", "enforce"].map((lvl) => (
          <button
            key={lvl}
            onClick={() => setControlMode(lvl as ControlMode)}
            className={`p-3 rounded-lg border text-left transition-all ${
              controlMode === lvl
                ? "bg-primary/10 border-primary ring-1 ring-primary"
                : "hover:bg-muted/50"
            }`}
          >
            <div className="font-medium text-sm mb-1 capitalize">{lvl}</div>
          </button>
        ))}
      </div>
    </div>
  );
}
