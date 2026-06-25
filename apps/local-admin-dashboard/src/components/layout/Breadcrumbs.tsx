import { useLocation, useNavigate } from "react-router-dom";
import { ArrowLeft, ChevronRight, Home } from "lucide-react";
import { NAV } from "../../config/navigation";

export function Breadcrumbs() {
  const location = useLocation();
  const navigate = useNavigate();

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
  let title = currentNavItem?.en;
  
  if (!title && pathParts.length > 0) {
    // try to match the first part
    for (const group of NAV) {
      for (const item of group.items) {
        if (item.href === `/${pathParts[0]}`) {
          title = `${item.en} Details`;
          break;
        }
      }
    }
    if (!title) {
      // capitalize parts
      title = pathParts.map(p => p.charAt(0).toUpperCase() + p.slice(1)).join(" > ");
    }
  }

  const isHome = location.pathname === "/";

  return (
    <div className="flex items-center gap-4 mb-6">
      <button
        onClick={() => navigate(-1)}
        disabled={isHome}
        className="p-2 hover:bg-muted rounded-md border text-muted-foreground transition-colors disabled:opacity-50 disabled:cursor-not-allowed bg-card shadow-sm"
        title="Go back"
      >
        <ArrowLeft className="w-4 h-4" />
      </button>
      <nav className="flex items-center text-sm text-muted-foreground gap-2">
        <div 
          className="flex items-center gap-1.5 cursor-pointer hover:text-foreground transition-colors"
          onClick={() => navigate("/")}
        >
          <Home className="w-4 h-4" />
          <span>Home</span>
        </div>
        
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
