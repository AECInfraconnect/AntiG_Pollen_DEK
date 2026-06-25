import { useState } from "react";
import { Outlet } from "react-router-dom";
import { Sidebar } from "./Sidebar";
import { Header } from "./Header";
import { FirstRunWizard } from "../FirstRunWizard";

export function DashboardLayout() {
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);

  return (
    <div className="flex h-screen w-full bg-background overflow-hidden">
      <FirstRunWizard />
      <Sidebar mobileMenuOpen={mobileMenuOpen} setMobileMenuOpen={setMobileMenuOpen} />
      <div className="flex flex-1 flex-col overflow-hidden relative">
        <Header toggleMobileMenu={() => setMobileMenuOpen(!mobileMenuOpen)} />
        <main className="flex-1 overflow-y-auto p-4 md:p-6 relative z-10">
          <Outlet />
        </main>
        {/* Decorative background glow */}
        <div className="pointer-events-none absolute -top-[20%] -right-[10%] h-[500px] w-[500px] rounded-full bg-primary/20 blur-[120px]" />
        <div className="pointer-events-none absolute -bottom-[20%] -left-[10%] h-[500px] w-[500px] rounded-full bg-accent/20 blur-[120px]" />
      </div>
    </div>
  );
}
