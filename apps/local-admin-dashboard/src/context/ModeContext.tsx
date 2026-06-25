import { createContext, useContext, useState, type ReactNode } from "react";

export type AppMode = "simple" | "advanced" | "enterprise";

interface ModeCtx { mode: AppMode; setMode: (m: AppMode) => void; }
const Ctx = createContext<ModeCtx>({ mode: "simple", setMode: () => {} });

export function ModeProvider({ children }: { children: ReactNode }) {
  const [mode, setModeState] = useState<AppMode>(
    () => (localStorage.getItem("pollek.mode") as AppMode) || "simple"
  );
  const setMode = (m: AppMode) => { localStorage.setItem("pollek.mode", m); setModeState(m); };
  return <Ctx.Provider value={{ mode, setMode }}>{children}</Ctx.Provider>;
}
export const useMode = () => useContext(Ctx);
