import { useMode } from "../../context/ModeContext";
import { useTranslation } from "react-i18next";
import { Settings2 } from "lucide-react";

export function ModeSwitcher({ collapsed }: { collapsed?: boolean }) {
  const { mode, setMode } = useMode();
  const { t } = useTranslation();

  if (collapsed) {
    return (
      <div
        className="flex h-8 w-8 items-center justify-center rounded-lg border border-border bg-card text-muted-foreground"
        title={t(`mode.${mode}`)}
      >
        <Settings2 className="h-4 w-4" />
      </div>
    );
  }

  return (
    <select
      value={mode}
      onChange={(e) => setMode(e.target.value as any)}
      className="w-full rounded-lg border border-border bg-card px-2 py-1.5 text-sm text-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-primary"
    >
      <option value="simple">{t("mode.simple")}</option>
      <option value="advanced">{t("mode.advanced")}</option>
      <option value="enterprise">{t("mode.enterprise")}</option>
    </select>
  );
}
