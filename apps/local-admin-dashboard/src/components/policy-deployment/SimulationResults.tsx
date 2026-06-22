
import { SimulationSummary } from "../presets/SimulationSummary";

export function SimulationResults({ simResult }: { simResult: any }) {
  return (
    <div className="space-y-4">
      <h4 className="font-medium">Simulation Results</h4>
      {simResult ? (
        <SimulationSummary simResult={simResult} />
      ) : (
        <div className="text-sm text-muted-foreground p-4 bg-muted/30 rounded border text-center">
          Click Next to run a dry-run simulation of the deployment.
        </div>
      )}
    </div>
  );
}
