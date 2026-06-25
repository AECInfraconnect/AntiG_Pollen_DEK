import { useTranslation } from "react-i18next";
import type { ControlLevel } from "../../services/types";

const LEVELS: { id: ControlLevel; icon: string; key: string }[] = [
  { id: "observe", icon: "👁️", key: "level.observe" },
  { id: "warn",    icon: "⚠️", key: "level.warn" },
  { id: "ask",     icon: "✋", key: "level.ask" },
  { id: "enforce", icon: "🛡️", key: "level.enforce" },
];

export function ControlLevelSelector({ value, onChange }:
  { value: ControlLevel; onChange: (l: ControlLevel) => void }) {
  const { t } = useTranslation();
  return (
    <div className="grid grid-cols-2 gap-3 md:grid-cols-4">
      {LEVELS.map((l) => (
        <button key={l.id} onClick={() => onChange(l.id)}
          className={`rounded-xl border p-4 text-left transition
            ${value === l.id ? "border-violet-500 bg-violet-500/10" : "border-zinc-700 hover:border-zinc-500"}`}>
          <div className="text-2xl">{l.icon}</div>
          <div className="mt-1 font-medium">{t(`${l.key}.title`)}</div>
          <div className="text-xs text-zinc-400">{t(`${l.key}.desc`)}</div>
        </button>
      ))}
    </div>
  );
}
