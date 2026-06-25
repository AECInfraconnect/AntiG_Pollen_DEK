import { useMode } from "../context/ModeContext";
import { SimplePolicyWizard } from "../components/simple/SimplePolicyWizard";
import { ShieldCheck } from "lucide-react";

export function Protect() {
  const { mode } = useMode();

  // Mock agents for the wizard as an example
  const agents = [
    { id: "agent-1", label: "OpenAI Codex" },
    { id: "agent-2", label: "Claude" },
    { id: "agent-3", label: "ChatGPT (Web)" },
  ];

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between border-b pb-4">
        <div>
          <h2 className="flex items-center gap-2 text-2xl font-bold tracking-tight">
            <ShieldCheck className="h-6 w-6 text-primary" />
            {mode === "simple" ? "Protect Agents" : "Advanced Protection"}
          </h2>
          <p className="mt-1 text-muted-foreground">
            {mode === "simple"
              ? "Deploy guardrails in 3 easy steps. The system will handle the rest."
              : "Deploy guardrails with automatic feasibility planning and method selection."}
          </p>
        </div>
      </div>

      <div className="mt-8">
        <SimplePolicyWizard agents={agents} />
      </div>
    </div>
  );
}
