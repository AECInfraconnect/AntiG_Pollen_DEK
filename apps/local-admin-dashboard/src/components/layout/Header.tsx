import { Bell, Search, Moon, Sun, Languages } from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

export function Header() {
  const { i18n } = useTranslation();
  const [isDark, setIsDark] = useState(true);

  useEffect(() => {
    if (isDark) {
      document.documentElement.classList.add("dark");
    } else {
      document.documentElement.classList.remove("dark");
    }
  }, [isDark]);

  const toggleLanguage = () => {
    const nextLang = i18n.language === "en" ? "th" : "en";
    i18n.changeLanguage(nextLang);
    localStorage.setItem("i18nextLng", nextLang);
  };

  return (
    <header className="flex h-16 items-center justify-between border-b bg-card/50 px-6 backdrop-blur-xl">
      <div className="flex flex-1 items-center gap-4">
        <div className="relative w-96">
          <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
          <input
            type="search"
            placeholder="Search resources, policies, or agents..."
            className="h-9 w-full rounded-md border bg-background pl-9 pr-4 text-sm outline-none focus:border-primary focus:ring-1 focus:ring-primary transition-all"
          />
        </div>
      </div>
      <div className="flex items-center gap-4">
        <button
          onClick={toggleLanguage}
          className="flex items-center gap-1 rounded-full p-2 text-muted-foreground hover:bg-muted hover:text-foreground transition-colors text-xs font-semibold"
        >
          <Languages className="h-5 w-5" />
          <span className="uppercase">{i18n.language}</span>
        </button>
        <button
          onClick={() => setIsDark(!isDark)}
          className="rounded-full p-2 text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
        >
          {isDark ? <Sun className="h-5 w-5" /> : <Moon className="h-5 w-5" />}
        </button>
        <button className="relative rounded-full p-2 text-muted-foreground hover:bg-muted hover:text-foreground transition-colors">
          <Bell className="h-5 w-5" />
          <span className="absolute right-2 top-2 h-2 w-2 rounded-full bg-destructive" />
        </button>
      </div>
    </header>
  );
}
