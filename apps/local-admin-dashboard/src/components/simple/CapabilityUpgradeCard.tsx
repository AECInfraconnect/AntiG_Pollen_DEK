import { useTranslation } from "react-i18next";
import type { CapabilityUpgrade } from "../../services/types";

export function CapabilityUpgradeCard({
  upgrade,
}: {
  upgrade: CapabilityUpgrade;
}) {
  const { i18n, t } = useTranslation();
  const how = i18n.language === "th" ? upgrade.how_th : upgrade.how_en;
  return (
    <div className="rounded-xl border border-zinc-700 bg-zinc-800/40 p-4">
      <div className="text-sm font-medium text-zinc-200">
        🔧 {upgrade.unlocks}
      </div>
      <div className="mt-1 text-xs text-zinc-400">{how}</div>
      <div className="mt-3 flex gap-2">
        {upgrade.auto_installable && (
          <button className="rounded-lg bg-violet-600 px-3 py-1.5 text-sm font-medium text-white hover:bg-violet-500">
            {t("upgrade.install")}
          </button>
        )}
        {upgrade.download_url && (
          <a
            href={upgrade.download_url}
            target="_blank"
            rel="noreferrer"
            className="rounded-lg border border-zinc-600 px-3 py-1.5 text-sm text-zinc-200 hover:border-zinc-400"
          >
            {t("upgrade.learn_more")}
          </a>
        )}
      </div>
    </div>
  );
}
