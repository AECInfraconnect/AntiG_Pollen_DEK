import { createContext, useContext, useState, type ReactNode } from "react";
import type { ProductMode } from "../navigation/menu";

interface ModeCtx { mode: ProductMode; setMode: (m: ProductMode) => void; }
const Ctx = createContext<ModeCtx>({ mode: "desktop_simple", setMode: () => {} });

export function ModeProvider({ children }: { children: ReactNode }) {
  const [mode, setModeState] = useState<ProductMode>(
    () => (localStorage.getItem("pollek.mode") as ProductMode) || "desktop_simple"
  );
  const setMode = (m: ProductMode) => { localStorage.setItem("pollek.mode", m); setModeState(m); };
  return <Ctx.Provider value={{ mode, setMode }}>{children}</Ctx.Provider>;
}
export const useMode = () => useContext(Ctx);

