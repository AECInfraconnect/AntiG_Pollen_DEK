import { useMode } from "../../context/ModeContext";
import { useTranslation } from "react-i18next";

export function ModeSwitcher() {
  const { mode, setMode } = useMode();
  const { t } = useTranslation();
  return (
    <select
      value={mode}
      onChange={(e) => setMode(e.target.value as any)}
      className="rounded-lg border border-zinc-700 bg-zinc-800 px-2 py-1 text-sm text-zinc-200"
    >
      <option value="simple">{t("mode.simple")}</option>
      <option value="advanced">{t("mode.advanced")}</option>
      <option value="enterprise">{t("mode.enterprise")}</option>
    </select>
  );
}
