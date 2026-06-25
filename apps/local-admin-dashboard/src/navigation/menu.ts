export type ProductMode =
  | "desktop_simple"
  | "desktop_advanced"
  | "enterprise_server"
  | "sovereign_airgap";

export type NavItemId =
  | "overview"
  | "scan"
  | "offline_scan"
  | "entities"
  | "agents"
  | "recommended_policies"
  | "policies"
  | "policy_feasibility"
  | "deployments"
  | "control_methods"
  | "pep_layers"
  | "pdp_engines"
  | "bundles"
  | "timeline"
  | "telemetry"
  | "local_evidence"
  | "audit_evidence"
  | "health"
  | "keys"
  | "import_export"
  | "settings"
  | "admin_settings"
  | "help";

export interface NavItem {
  id: NavItemId;
  label: {
    en: string;
    th: string;
  };
  path: string;
  icon: string;
  modes: ProductMode[];
  requiresAdvanced?: boolean;
}

export const NAV_ITEMS: NavItem[] = [
  {
    id: "overview",
    label: { en: "Overview", th: "ภาพรวม" },
    path: "/",
    icon: "layout-dashboard",
    modes: ["desktop_simple", "desktop_advanced", "enterprise_server", "sovereign_airgap"],
  },
  {
    id: "scan",
    label: { en: "Scan This Device", th: "สแกนเครื่องนี้" },
    path: "/scan",
    icon: "radar",
    modes: ["desktop_simple", "desktop_advanced", "enterprise_server"],
  },
  {
    id: "offline_scan",
    label: { en: "Offline Scan", th: "สแกนแบบออฟไลน์" },
    path: "/offline-scan",
    icon: "hard-drive",
    modes: ["sovereign_airgap"],
  },
  {
    id: "agents",
    label: { en: "Agents", th: "Agents" },
    path: "/agents",
    icon: "bot",
    modes: ["desktop_simple", "desktop_advanced", "enterprise_server", "sovereign_airgap"],
  },
  {
    id: "recommended_policies",
    label: { en: "Recommended Policies", th: "Policy ที่แนะนำ" },
    path: "/recommended-policies",
    icon: "sparkles",
    modes: ["desktop_simple"],
  },
  {
    id: "policies",
    label: { en: "Policies", th: "นโยบาย" },
    path: "/policies",
    icon: "shield",
    modes: ["desktop_advanced", "enterprise_server", "sovereign_airgap"],
  },
  {
    id: "policy_feasibility",
    label: { en: "Policy Feasibility", th: "ความพร้อมของ Policy" },
    path: "/policy-feasibility",
    icon: "clipboard-check",
    modes: ["desktop_advanced", "enterprise_server", "sovereign_airgap"],
  },
  {
    id: "deployments",
    label: { en: "Deployments", th: "การติดตั้งใช้งาน" },
    path: "/deployments",
    icon: "server",
    modes: ["desktop_simple", "desktop_advanced", "enterprise_server", "sovereign_airgap"],
  },
  {
    id: "control_methods",
    label: { en: "Control Methods", th: "วิธีควบคุม" },
    path: "/control-methods",
    icon: "sliders-horizontal",
    modes: ["desktop_advanced"],
    requiresAdvanced: true,
  },
  {
    id: "pep_layers",
    label: { en: "PEP / Control Layers", th: "PEP / ชั้นควบคุม" },
    path: "/pep-layers",
    icon: "shield",
    modes: ["enterprise_server"],
    requiresAdvanced: true,
  },
  {
    id: "pdp_engines",
    label: { en: "PDP / Decision Engines", th: "PDP / เครื่องมือตัดสินใจ" },
    path: "/pdp-engines",
    icon: "cpu",
    modes: ["enterprise_server"],
    requiresAdvanced: true,
  },
  {
    id: "timeline",
    label: { en: "Timeline", th: "ไทม์ไลน์" },
    path: "/timeline",
    icon: "clock",
    modes: ["desktop_simple", "desktop_advanced", "enterprise_server"],
  },
  {
    id: "local_evidence",
    label: { en: "Local Evidence", th: "หลักฐานในเครื่อง" },
    path: "/local-evidence",
    icon: "database",
    modes: ["desktop_simple", "desktop_advanced", "sovereign_airgap"],
  },
  {
    id: "health",
    label: { en: "Health & Diagnostics", th: "สุขภาพระบบและการวินิจฉัย" },
    path: "/health",
    icon: "activity",
    modes: ["desktop_advanced", "enterprise_server", "sovereign_airgap"],
    requiresAdvanced: true,
  },
];

export function getNavItems(mode: ProductMode): NavItem[] {
  return NAV_ITEMS.filter((item) => item.modes.includes(mode));
}
