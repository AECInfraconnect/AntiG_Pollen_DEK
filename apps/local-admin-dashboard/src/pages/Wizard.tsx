import { useState, useEffect } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { PolicyFirstApi, RegistryApi } from "../services/api";
import type {
  PolicyFeasibilityRequest,
  PolicyFeasibilityResult,
  PolicyIntent,
  ProductMode,
} from "../services/types";
import {
  ShieldCheck,
  AlertTriangle,
  ChevronDown,
  ChevronRight,
  Check,
} from "lucide-react";

export function Wizard() {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();

  const [step, setStep] = useState(1);
  const [mode] = useState<ProductMode>("desktop_simple");
  const [targets, setTargets] = useState<any[]>([]);

  // Step 1: Policy Intent
  const [policyIntent, setPolicyIntent] = useState<PolicyIntent>(
    "observe_agent_activity",
  );
  const [availableAgents, setAvailableAgents] = useState<any[]>([]);
  const [selectedAgentId, setSelectedAgentId] = useState<string>(
    searchParams.get("target") || "",
  );

  // Step 2: Control Level
  const [controlLevel, setControlLevel] = useState<string>("warn");

  // Step 3: Feasibility Result & Deploy Session
  const [feasibility, setFeasibility] =
    useState<PolicyFeasibilityResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [deploySuccess, setDeploySuccess] = useState(false);

  useEffect(() => {
    RegistryApi.listAgents().then(setAvailableAgents).catch(console.error);
  }, []);

  const handleNextStep1 = () => {
    if (!selectedAgentId) return alert("Select a target agent");
    setTargets([{ kind: "agent", id: selectedAgentId }]);
    setStep(2);
  };

  const handleNextStep2 = async () => {
    setLoading(true);
    try {
      const req: PolicyFeasibilityRequest = {
        policy_intent: policyIntent,
        requested_control_level: controlLevel,
        targets,
        mode,
      };
      const res = await PolicyFirstApi.evaluateFeasibility(req);
      setFeasibility(res);
      setStep(3);
    } catch (e) {
      console.error(e);
      alert("Failed to evaluate feasibility");
    } finally {
      setLoading(false);
    }
  };

  const handleDeploy = async () => {
    if (!feasibility) return;
    setLoading(true);
    try {
      const sessionRes = await PolicyFirstApi.createDeploymentSession({
        policy_intent: policyIntent,
        requested_control_level: controlLevel,
        target_scope: targets,
      });

      // Auto-approve the first action if required for testing
      // Real flow might have multiple actions
      if (feasibility.required_actions?.length > 0) {
        await PolicyFirstApi.approveAction(
          sessionRes.deployment_id,
          "action-1",
        );
      }
      setDeploySuccess(true);
    } catch (e) {
      console.error(e);
      alert("Deployment failed");
    } finally {
      setLoading(false);
    }
  };

  const intents: { val: PolicyIntent; label: string }[] = [
    { val: "observe_agent_activity", label: "Observe Agent Activity" },
    { val: "approve_risky_tool_calls", label: "Approve Risky Tool Calls" },
    { val: "block_specific_tools", label: "Block Specific Tools" },
    {
      val: "redact_sensitive_parameters",
      label: "Redact Sensitive Parameters",
    },
    {
      val: "block_unknown_network_destinations",
      label: "Block Unknown Network Destinations",
    },
  ];

  const levels = [
    {
      val: "strict_deny",
      label: "Strict Deny",
      desc: "Block without exception",
    },
    { val: "enforce", label: "Enforce", desc: "Block and notify user" },
    {
      val: "approval",
      label: "Require Approval",
      desc: "Prompt user for permission",
    },
    { val: "warn", label: "Warn", desc: "Allow but log warning" },
    { val: "observe", label: "Observe Only", desc: "Log activity silently" },
  ];

  return (
    <div className="space-y-6 max-w-3xl mx-auto py-8">
      <div className="flex justify-between items-center mb-8 border-b pb-4">
        <h2 className="text-2xl font-bold tracking-tight text-primary">
          Deploy Policy
        </h2>
        <div className="text-sm text-muted-foreground font-mono">
          Step {step} / 3
        </div>
      </div>

      {step === 1 && (
        <div className="space-y-6 animate-in fade-in slide-in-from-bottom-4">
          <h3 className="text-xl font-semibold">1. Choose Policy Intent</h3>

          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium mb-2">
                What do you want POLLEK to do?
              </label>
              <select
                value={policyIntent}
                onChange={(e) =>
                  setPolicyIntent(e.target.value as PolicyIntent)
                }
                className="w-full bg-background border rounded-md p-3 focus:ring-2 focus:ring-primary outline-none"
              >
                {intents.map((i) => (
                  <option key={i.val} value={i.val}>
                    {i.label}
                  </option>
                ))}
              </select>
            </div>

            <div>
              <label className="block text-sm font-medium mb-2">
                Target Agent
              </label>
              <select
                value={selectedAgentId}
                onChange={(e) => setSelectedAgentId(e.target.value)}
                className="w-full bg-background border rounded-md p-3 focus:ring-2 focus:ring-primary outline-none"
              >
                <option value="">-- Select Agent --</option>
                {availableAgents.map((a) => (
                  <option key={a.agent_id} value={a.agent_id}>
                    {a.name} ({a.agent_id})
                  </option>
                ))}
              </select>
            </div>
          </div>

          <div className="flex justify-end pt-6 border-t mt-8">
            <button
              onClick={handleNextStep1}
              className="px-6 py-2 bg-primary text-primary-foreground rounded-md shadow-sm font-medium hover:opacity-90 transition-opacity"
            >
              Continue
            </button>
          </div>
        </div>
      )}

      {step === 2 && (
        <div className="space-y-6 animate-in fade-in slide-in-from-bottom-4">
          <h3 className="text-xl font-semibold">2. Set Control Level</h3>
          <p className="text-sm text-muted-foreground mb-4">
            How strictly should POLLEK enforce this policy?
          </p>

          <div className="grid grid-cols-1 gap-3">
            {levels.map((l) => (
              <label
                key={l.val}
                className={`flex items-start gap-3 p-4 border rounded-lg cursor-pointer transition-colors ${controlLevel === l.val ? "border-primary bg-primary/5" : "hover:bg-muted/30"}`}
              >
                <input
                  type="radio"
                  name="level"
                  value={l.val}
                  checked={controlLevel === l.val}
                  onChange={() => setControlLevel(l.val)}
                  className="mt-1"
                />
                <div>
                  <div className="font-medium">{l.label}</div>
                  <div className="text-xs text-muted-foreground">{l.desc}</div>
                </div>
              </label>
            ))}
          </div>

          <div className="flex justify-between pt-6 border-t mt-8">
            <button
              onClick={() => setStep(1)}
              className="px-4 py-2 border rounded-md hover:bg-muted/50 text-sm"
            >
              Back
            </button>
            <button
              onClick={handleNextStep2}
              disabled={loading}
              className="px-6 py-2 bg-primary text-primary-foreground rounded-md shadow-sm font-medium hover:opacity-90 transition-opacity disabled:opacity-50"
            >
              {loading ? "Checking Feasibility..." : "Preview Deployment"}
            </button>
          </div>
        </div>
      )}

      {step === 3 && feasibility && !deploySuccess && (
        <div className="space-y-6 animate-in fade-in slide-in-from-bottom-4">
          <h3 className="text-xl font-semibold">3. Review & Deploy</h3>

          <div
            className={`p-4 border rounded-xl ${feasibility.status === "can_enforce_now" ? "bg-emerald-500/10 border-emerald-500/30" : "bg-amber-500/10 border-amber-500/30"}`}
          >
            <div className="flex items-start gap-3">
              {feasibility.status === "can_enforce_now" ? (
                <ShieldCheck className="w-6 h-6 text-emerald-500 mt-0.5" />
              ) : (
                <AlertTriangle className="w-6 h-6 text-amber-500 mt-0.5" />
              )}
              <div>
                <h4 className="font-semibold text-lg">
                  {feasibility.user_summary.en}
                </h4>
                <p className="text-sm mt-1 text-foreground/80">
                  {feasibility.user_detail.en}
                </p>

                {feasibility.required_actions?.length > 0 && (
                  <div className="mt-4 space-y-2">
                    <p className="text-xs font-semibold uppercase tracking-wider opacity-70">
                      Setup Required:
                    </p>
                    <ul className="text-sm list-disc list-inside">
                      {feasibility.required_actions.map((act, i) => (
                        <li key={i}>{act.label.en}</li>
                      ))}
                    </ul>
                  </div>
                )}
              </div>
            </div>
          </div>

          {feasibility.technical_plan && (
            <div className="border rounded-xl overflow-hidden">
              <button
                onClick={() => setShowAdvanced(!showAdvanced)}
                className="w-full flex items-center justify-between p-4 bg-muted/30 hover:bg-muted/50 transition-colors text-sm font-medium"
              >
                <span>Advanced Details</span>
                {showAdvanced ? (
                  <ChevronDown className="w-4 h-4" />
                ) : (
                  <ChevronRight className="w-4 h-4" />
                )}
              </button>

              {showAdvanced && (
                <div className="p-4 bg-muted/10 border-t space-y-3 font-mono text-xs">
                  <div className="grid grid-cols-2 gap-2">
                    <div className="text-muted-foreground">Control Method:</div>
                    <div className="font-medium text-primary capitalize">
                      {feasibility.technical_plan.method.replace(/_/g, " ")}
                    </div>

                    <div className="text-muted-foreground">Internal PEP:</div>
                    <div className="font-medium text-primary capitalize">
                      {feasibility.technical_plan.internal_pep.replace(
                        /_/g,
                        " ",
                      )}
                    </div>

                    <div className="text-muted-foreground">Internal PDP:</div>
                    <div className="font-medium text-primary capitalize">
                      {feasibility.technical_plan.internal_pdp.replace(
                        /_/g,
                        " ",
                      )}
                    </div>
                  </div>
                </div>
              )}
            </div>
          )}

          <div className="flex justify-between pt-6 border-t mt-8">
            <button
              onClick={() => setStep(2)}
              className="px-4 py-2 border rounded-md hover:bg-muted/50 text-sm disabled:opacity-50"
              disabled={loading}
            >
              Back
            </button>
            <button
              onClick={handleDeploy}
              disabled={loading}
              className="px-6 py-2 bg-primary text-primary-foreground rounded-md shadow-sm font-medium hover:opacity-90 transition-opacity disabled:opacity-50"
            >
              {loading ? "Deploying..." : "Confirm & Deploy"}
            </button>
          </div>
        </div>
      )}

      {deploySuccess && (
        <div className="space-y-6 animate-in fade-in slide-in-from-bottom-4 text-center py-12">
          <div className="w-16 h-16 bg-emerald-500/20 text-emerald-500 rounded-full flex items-center justify-center mx-auto mb-4">
            <Check className="w-8 h-8" />
          </div>
          <h3 className="text-2xl font-bold">Policy Deployed Successfully</h3>
          <p className="text-muted-foreground">
            The policy has been enforced on the selected targets.
          </p>
          <div className="pt-8">
            <button
              onClick={() => navigate("/agents")}
              className="px-6 py-2 bg-primary text-primary-foreground rounded-md shadow-sm font-medium hover:opacity-90 transition-opacity"
            >
              Return to Agents
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
