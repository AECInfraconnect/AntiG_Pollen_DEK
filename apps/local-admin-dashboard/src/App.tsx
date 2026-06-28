import {
  BrowserRouter as Router,
  Navigate,
  Outlet,
  Route,
  Routes,
  useLocation,
} from "react-router-dom";
import { useEffect } from "react";
import { Toaster } from "sonner";
import { DashboardLayout } from "./components/layout/DashboardLayout";
import { ConfirmProvider } from "./components/ui/ConfirmDialog";
import { NAV } from "./config/navigation";
import { ModeProvider, useMode } from "./context/ModeContext";
import { ThemeProvider, useTheme } from "./context/ThemeContext";
import { cleanupLegacyDashboardStorage } from "./lib/storageMigrations";
import { Wizard } from "./pages/Wizard";
import { dashboardRoutes } from "./routes/dashboardRoutes";

const ModeGuard = () => {
  const { mode } = useMode();
  const { pathname } = useLocation();
  const navRule = NAV.flatMap((group) => group.items)
    .filter((item) => item.href !== "/")
    .sort((a, b) => b.href.length - a.href.length)
    .find(
      (item) => pathname === item.href || pathname.startsWith(`${item.href}/`),
    );

  if (navRule && !navRule.modes.includes(mode)) {
    return <Navigate to="/" replace />;
  }

  return <Outlet />;
};

function AppContent() {
  const { resolvedTheme } = useTheme();

  useEffect(() => {
    cleanupLegacyDashboardStorage();
  }, []);

  return (
    <ModeProvider>
      <ConfirmProvider>
        <Toaster position="top-right" theme={resolvedTheme} />
        <Router>
          <Routes>
            <Route path="/" element={<DashboardLayout />}>
              <Route element={<ModeGuard />}>
                {dashboardRoutes.map((route) =>
                  route.index ? (
                    <Route
                      key={route.key}
                      index
                      element={route.element}
                    />
                  ) : (
                    <Route
                      key={route.key}
                      path={route.path}
                      element={route.element}
                    />
                  ),
                )}
              </Route>
            </Route>
            <Route path="/wizard" element={<Wizard />} />
          </Routes>
        </Router>
      </ConfirmProvider>
    </ModeProvider>
  );
}

function App() {
  return (
    <ThemeProvider>
      <AppContent />
    </ThemeProvider>
  );
}

export default App;
