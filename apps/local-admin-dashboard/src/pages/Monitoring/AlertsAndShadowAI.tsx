import { useState } from "react";
import { useSearchParams } from "react-router-dom";
import { AlertTriangle, ShieldAlert, ShieldCheck } from "lucide-react";
import { Alerts } from "../Alerts";
import { ShadowAI } from "../ShadowAI";
import { GuardIncidentFeed } from "../../features/guard/GuardIncidentCard";
import { useMode } from "../../context/ModeContext";
import { isAdvanceMode } from "../../lib/modes";

export function AlertsAndShadowAI() {
  const [searchParams] = useSearchParams();
  const { mode } = useMode();
  const showTechnicalDetails = isAdvanceMode(mode);
  const initialTab = searchParams.get("tab");
  const [activeTab, setActiveTab] = useState<"alerts" | "guard" | "shadow">(
    initialTab === "alerts" || initialTab === "guard" || initialTab === "shadow"
      ? initialTab
      : "guard",
  );
  const pageTitle = showTechnicalDetails
    ? "Prompt Guard, alerts, and Shadow AI"
    : "Prompt Guard safety center";
  const pageDescription = showTechnicalDetails
    ? "Review prompt safety incidents, policy alerts, and unregistered AI activity in one place."
    : "See when Pollek warned, redacted, or blocked risky prompts and what to do next.";
  const alertTabLabel = showTechnicalDetails
    ? "Active Alerts"
    : "System alerts";
  const shadowTabLabel = showTechnicalDetails
    ? "Shadow AI Inbox"
    : "Unknown AI apps";

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">{pageTitle}</h2>
          <p className="text-muted-foreground">{pageDescription}</p>
        </div>
      </div>

      <div className="flex space-x-1 border-b border-border/50">
        <button
          onClick={() => setActiveTab("guard")}
          className={`flex items-center gap-2 px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
            activeTab === "guard"
              ? "border-primary text-primary"
              : "border-transparent text-muted-foreground hover:text-foreground hover:border-border"
          }`}
        >
          <ShieldCheck className="h-4 w-4" />
          Prompt Guard
        </button>
        <button
          onClick={() => setActiveTab("alerts")}
          className={`flex items-center gap-2 px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
            activeTab === "alerts"
              ? "border-primary text-primary"
              : "border-transparent text-muted-foreground hover:text-foreground hover:border-border"
          }`}
        >
          <ShieldAlert className="h-4 w-4" />
          {alertTabLabel}
        </button>
        <button
          onClick={() => setActiveTab("shadow")}
          className={`flex items-center gap-2 px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
            activeTab === "shadow"
              ? "border-primary text-primary"
              : "border-transparent text-muted-foreground hover:text-foreground hover:border-border"
          }`}
        >
          <AlertTriangle className="h-4 w-4" />
          {shadowTabLabel}
        </button>
      </div>

      <div className="pt-2">
        {activeTab === "alerts" ? (
          <div className="mt-[-24px]">
            <Alerts hideHeader={true} />
          </div>
        ) : activeTab === "guard" ? (
          <GuardIncidentFeed />
        ) : (
          <div className="mt-[-24px]">
            <ShadowAI hideHeader={true} />
          </div>
        )}
      </div>
    </div>
  );
}
