import { useLocation, useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { ArrowLeft, ChevronRight, Home } from "lucide-react";
import { labelForLanguage, NAV } from "../../config/navigation";

export function Breadcrumbs() {
  const location = useLocation();
  const navigate = useNavigate();
  const { i18n, t } = useTranslation();

  // Find current nav item
  let currentNavItem = null;

  for (const group of NAV) {
    for (const item of group.items) {
      if (item.href === location.pathname) {
        currentNavItem = item;
        break;
      }
    }
    if (currentNavItem) break;
  }

  // Fallback for paths not exactly in NAV (like dynamic paths or sub-pages)
  const pathParts = location.pathname.split("/").filter(Boolean);
  let title = currentNavItem
    ? labelForLanguage(currentNavItem, i18n.language)
    : undefined;

  if (!title && pathParts.length > 0) {
    // try to match the first part
    for (const group of NAV) {
      for (const item of group.items) {
        if (item.href === `/${pathParts[0]}`) {
          title = t("breadcrumb.details", {
            title: labelForLanguage(item, i18n.language),
          });
          break;
        }
      }
    }
    if (!title) {
      // capitalize parts
      title = pathParts
        .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
        .join(" > ");
    }
  }

  const isHome = location.pathname === "/";

  return (
    <div className="mb-3 flex items-center gap-3">
      <button
        onClick={() => navigate(-1)}
        disabled={isHome}
        aria-label={t("common.goBack")}
        className="rounded-md border bg-card p-2 text-muted-foreground shadow-sm transition-colors hover:bg-muted disabled:cursor-not-allowed disabled:opacity-50"
        title={t("common.goBack")}
      >
        <ArrowLeft className="w-4 h-4" />
      </button>
      <nav className="flex items-center text-sm text-muted-foreground gap-2">
        <button
          type="button"
          className="flex cursor-pointer items-center gap-1.5 transition-colors hover:text-foreground"
          onClick={() => navigate("/")}
        >
          <Home className="w-4 h-4" />
          <span>{t("nav.home")}</span>
        </button>

        {!isHome && title && (
          <>
            <ChevronRight className="w-4 h-4" />
            <span className="text-foreground font-medium">{title}</span>
          </>
        )}
      </nav>
    </div>
  );
}
