import { Bell, Search, Moon, Sun, Languages, Menu, Monitor } from "lucide-react";
import { useTranslation } from "react-i18next";
import { ModeSwitcher } from "./ModeSwitcher";
import { useTheme } from "../../context/ThemeContext";

export function Header({
  toggleMobileMenu,
  onOpenCommandPalette,
}: {
  toggleMobileMenu?: () => void;
  onOpenCommandPalette?: () => void;
}) {
  const { i18n } = useTranslation();
  const { theme, resolvedTheme, cycleTheme } = useTheme();

  const toggleLanguage = () => {
    const nextLang = i18n.language === "en" ? "th" : "en";
    i18n.changeLanguage(nextLang);
    localStorage.setItem("i18nextLng", nextLang);
  };
  const themeLabel =
    theme === "system"
      ? `System (${resolvedTheme})`
      : theme === "dark"
        ? "Dark"
        : "Light";

  return (
    <header className="flex h-16 items-center justify-between border-b bg-card/50 px-4 md:px-6 backdrop-blur-xl shrink-0">
      <div className="flex flex-1 items-center gap-2 md:gap-4">
        {toggleMobileMenu && (
          <button
            onClick={toggleMobileMenu}
            aria-label="Open navigation"
            className="md:hidden rounded-md p-2 text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
          >
            <Menu className="h-5 w-5" />
          </button>
        )}
        <div className="relative w-full max-w-sm">
          <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
          <input
            id="global-search-input"
            type="search"
            placeholder="Search resources, policies, or agents... (Ctrl K)"
            readOnly
            onFocus={onOpenCommandPalette}
            onClick={onOpenCommandPalette}
            aria-label="Open command palette"
            className="h-9 w-full cursor-pointer rounded-md border bg-background pl-9 pr-16 text-sm outline-none transition-all hover:border-primary/60 focus:border-primary focus:ring-1 focus:ring-primary"
          />
          <kbd className="pointer-events-none absolute right-2.5 top-2.5 hidden h-4 select-none items-center gap-1 rounded border bg-muted px-1.5 font-mono text-[10px] font-medium opacity-100 sm:flex text-muted-foreground">
            Ctrl K
          </kbd>
        </div>
      </div>
      <div className="flex items-center gap-4">
        <ModeSwitcher />
        <button
          onClick={toggleLanguage}
          aria-label="Switch language"
          className="flex items-center gap-1 rounded-full p-2 text-muted-foreground hover:bg-muted hover:text-foreground transition-colors text-xs font-semibold"
        >
          <Languages className="h-5 w-5" />
          <span className="uppercase">{i18n.language}</span>
        </button>
        <button
          onClick={cycleTheme}
          aria-label={`Change theme. Current theme: ${themeLabel}`}
          title={`Theme: ${themeLabel}`}
          className="rounded-full p-2 text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
        >
          {theme === "system" ? (
            <Monitor className="h-5 w-5" />
          ) : resolvedTheme === "dark" ? (
            <Sun className="h-5 w-5" />
          ) : (
            <Moon className="h-5 w-5" />
          )}
        </button>
        <button
          aria-label="Notifications"
          className="relative rounded-full p-2 text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
        >
          <Bell className="h-5 w-5" />
          <span className="absolute right-2 top-2 h-2 w-2 rounded-full bg-destructive" />
        </button>
      </div>
    </header>
  );
}
