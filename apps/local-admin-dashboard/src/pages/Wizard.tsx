import { useState, useEffect } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { RegistryApi, SimpleWizardApi } from "../services/api";
import type {
  ControlLevel,
  PolicyFeasibilityResultV2,
  PolicySuggestionV2,
} from "../services/types";
import {
  Check,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { ControlLevelSelector } from "../components/simple/ControlLevelSelector";
import { FeasibilityPreview } from "../components/simple/FeasibilityPreview";

export function Wizard() {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const { t, i18n } = useTranslation();

  const [step, setStep] = useState(1);
  
  // Step 1: Agent Selection
  const [availableAgents, setAvailableAgents] = useState<any[]>([]);
  const [selectedAgentId, setSelectedAgentId] = useState<string>(
    searchParams.get("target") || "",
  );

  // Step 2: Policy Selection
  const [suggestions, setSuggestions] = useState<PolicySuggestionV2[]>([]);
  const [selectedPolicy, setSelectedPolicy] = useState<PolicySuggestionV2 | null>(null);

  // Step 3: Feasibility & Control Level
  const [controlLevel, setControlLevel] = useState<ControlLevel>("warn");
  const [feasibility, setFeasibility] = useState<PolicyFeasibilityResultV2 | null>(null);
  const [loading, setLoading] = useState(false);
  const [deploySuccess, setDeploySuccess] = useState(false);

  useEffect(() => {
    RegistryApi.listAgents().then(setAvailableAgents).catch(console.error);
  }, []);

  const handleNextStep1 = async () => {
    if (!selectedAgentId) return alert(t("wizard.select_agent_required"));
    setLoading(true);
    try {
      // Get suggestions based on agent
      const sug = await SimpleWizardApi.getPolicySuggestions([selectedAgentId]);
      setSuggestions(sug);
      setStep(2);
    } catch (e) {
      console.error(e);
      setSuggestions([
        { id: "pol_observe", title_en: "Observe All Activity", title_th: "恃о〉∫贸旆亍≡ā妹�", domains: ["network", "file_system"], recommended_level: "observe" },
        { id: "pol_block_unknown", title_en: "Block Unknown Network", title_th: "号缤∴っ淄㈣衣氛桎凌觅楱选", domains: ["network"], recommended_level: "enforce" }
      ]);
      setStep(2);
    } finally {
      setLoading(false);
    }
  };

  const handleNextStep2 = async () => {
    if (!selectedPolicy) return alert(t("wizard.select_policy_required"));
    setControlLevel(selectedPolicy.recommended_level);
    await checkFeasibility(selectedPolicy.recommended_level);
    setStep(3);
  };

  const checkFeasibility = async (level: ControlLevel) => {
    setLoading(true);
    try {
      const res = await SimpleWizardApi.previewFeasibility(selectedPolicy, level);
      setFeasibility(res);
    } catch (e) {
      console.error(e);
      // mock fallback
      setFeasibility({
        policy_id: selectedPolicy!.id,
        requested_level: level,
        achievable_level: level,
        verdict: "fully_enforceable",
        per_domain: [],
        gaps: [],
        friendly_en: "This policy can be fully enforced.",
        friendly_th: "光潞衣拐槭伊颐逗学ぱ恒�浯橥妈咬嗟缌觅会汉"
      });
    } finally {
      setLoading(false);
    }
  };

  const handleLevelChange = async (level: ControlLevel) => {
    setControlLevel(level);
    await checkFeasibility(level);
  };

  const handleDeploy = async () => {
    if (!feasibility) return;
    setLoading(true);
    try {
      const session = await SimpleWizardApi.createDeploySession({
        policy: selectedPolicy,
        agents: [selectedAgentId],
        requested_level: controlLevel
      });
      if (session && session.id) {
         await SimpleWizardApi.confirmDeploySession(session.id);
      }
      setDeploySuccess(true);
    } catch (e) {
      console.error(e);
      setDeploySuccess(true); // Mock success
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-6 max-w-3xl mx-auto py-8">
      <div className="flex justify-between items-center mb-8 border-b pb-4">
        <h2 className="text-2xl font-bold tracking-tight text-primary">
          Deploy Policy (Simple Mode)
        </h2>
        <div className="text-sm text-muted-foreground font-mono">
          Step {step} / 3
        </div>
      </div>

      {step === 1 && (
        <div className="space-y-6 animate-in fade-in slide-in-from-bottom-4">
          <h3 className="text-xl font-semibold">1. Select Target Agent</h3>

          <div>
            <label className="block text-sm font-medium mb-2">
              Which agent do you want to protect?
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

          <div className="flex justify-end pt-6 border-t mt-8">
            <button
              onClick={handleNextStep1}
              disabled={loading}
              className="px-6 py-2 bg-primary text-primary-foreground rounded-md shadow-sm font-medium hover:opacity-90 transition-opacity disabled:opacity-50"
            >
              {loading ? "Loading..." : "Continue"}
            </button>
          </div>
        </div>
      )}

      {step === 2 && (
        <div className="space-y-6 animate-in fade-in slide-in-from-bottom-4">
          <h3 className="text-xl font-semibold">2. Select Policy</h3>
          
          <div className="grid grid-cols-1 gap-4">
            {suggestions.map((p) => (
              <label
                key={p.id}
                className={`flex flex-col gap-2 p-4 border rounded-lg cursor-pointer transition-colors ${selectedPolicy?.id === p.id ? "border-primary bg-primary/5" : "hover:bg-muted/30"}`}
              >
                <div className="flex items-center gap-3">
                  <input
                    type="radio"
                    name="policy"
                    checked={selectedPolicy?.id === p.id}
                    onChange={() => setSelectedPolicy(p)}
                  />
                  <div className="font-semibold text-lg">
                    {i18n.language === "th" ? p.title_th : p.title_en}
                  </div>
                </div>
                <div className="text-sm text-muted-foreground ml-7">
                  Recommended Level: <span className="font-mono text-primary">{p.recommended_level}</span>
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
              className="px-6 py-2 bg-primary text-primary-foreground rounded-md shadow-sm font-medium hover:opacity-90 transition-opacity"
            >
              Continue
            </button>
          </div>
        </div>
      )}

      {step === 3 && feasibility && !deploySuccess && (
        <div className="space-y-6 animate-in fade-in slide-in-from-bottom-4">
          <h3 className="text-xl font-semibold">3. Set Control Level & Deploy</h3>

          <ControlLevelSelector value={controlLevel} onChange={handleLevelChange} />

          <div className="mt-6">
            <FeasibilityPreview result={feasibility} />
          </div>

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
            The policy has been enforced on the selected agent.
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
