import { useTranslation } from "react-i18next";
import type { PolicyFeasibilityResult } from "../../services/types";
import { CapabilityUpgradeCard } from "./CapabilityUpgradeCard";

const BADGE: Record<string, { icon: string; cls: string }> = {
  fully_enforceable: { icon: "🛡️", cls: "text-emerald-400 border-emerald-500/40 bg-emerald-500/10" },
  partial_observe:   { icon: "🔵", cls: "text-sky-400 border-sky-500/40 bg-sky-500/10" },
  observe_only:      { icon: "👁️", cls: "text-amber-400 border-amber-500/40 bg-amber-500/10" },
  not_applicable:    { icon: "⚪", cls: "text-zinc-400 border-zinc-600 bg-zinc-500/10" },
};

export function FeasibilityPreview({ result }: { result: PolicyFeasibilityResult }) {
  const { i18n } = useTranslation();
  const th = i18n.language === "th";
  const b = BADGE[result.verdict] || BADGE["not_applicable"];
  return (
    <div className="space-y-3">
      <div className={`rounded-xl border p-4 ${b.cls}`}>
        <div className="flex items-center gap-2 text-lg">
          <span>{b.icon}</span>
          <span className="font-semibold">{th ? result.friendly_th : result.friendly_en}</span>
        </div>
        <ul className="mt-2 space-y-1 text-sm text-zinc-300">
          {result.per_domain.map((d: any) => (
            <li key={d.domain}>• {th ? d.reason_th : d.reason_en}</li>
          ))}
        </ul>
      </div>
      {result.gaps.length > 0 && result.gaps.map((g: any) => (
        <CapabilityUpgradeCard key={g.method_id} upgrade={g} />
      ))}
    </div>
  );
}
