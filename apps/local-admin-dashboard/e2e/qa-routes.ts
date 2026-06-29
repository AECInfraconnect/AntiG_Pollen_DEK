export type QaMode = "desktop_simple" | "desktop_advanced" | "enterprise_cloud";

export interface QaRoute {
  path: string;
  name: string;
  modes: QaMode[];
  seedScan?: boolean;
}

const ALL: QaMode[] = [
  "desktop_simple",
  "desktop_advanced",
  "enterprise_cloud",
];
const ADV: QaMode[] = ["desktop_advanced", "enterprise_cloud"];
const ENT: QaMode[] = ["enterprise_cloud"];

export const qaRoutes: QaRoute[] = [
  { path: "/", name: "Overview", modes: ALL },
  { path: "/scan", name: "Find AI Apps", modes: ALL, seedScan: true },
  { path: "/my-ai-apps", name: "My AI Apps", modes: ALL, seedScan: true },
  { path: "/activity", name: "AI Activity", modes: ALL, seedScan: true },
  {
    path: "/observe-coverage",
    name: "Observe Coverage",
    modes: ALL,
    seedScan: true,
  },
  { path: "/alerts", name: "Prompt Guard", modes: ALL, seedScan: true },
  { path: "/protect", name: "Create Rule", modes: ALL, seedScan: true },
  {
    path: "/allowed-blocked",
    name: "Allowed & Blocked",
    modes: ALL,
    seedScan: true,
  },
  { path: "/data-apps", name: "Data & Apps", modes: ALL, seedScan: true },
  {
    path: "/cost-ledger",
    name: "AI Usage & Cost",
    modes: ALL,
    seedScan: true,
  },
  { path: "/setup", name: "Setup", modes: ALL, seedScan: true },
  { path: "/history", name: "History", modes: ALL, seedScan: true },
  { path: "/plugin-marketplace", name: "Plugins", modes: ALL },
  { path: "/settings", name: "Settings", modes: ALL },

  { path: "/agents", name: "Agents & Models", modes: ADV, seedScan: true },
  { path: "/tools", name: "Tools & Resources", modes: ADV, seedScan: true },
  { path: "/identities", name: "Identities", modes: ADV, seedScan: true },
  {
    path: "/entity-graph",
    name: "Relationship Map",
    modes: ADV,
    seedScan: true,
  },
  { path: "/policies", name: "Policies", modes: ADV, seedScan: true },
  { path: "/policy-presets", name: "Policy Presets", modes: ADV },
  { path: "/deployments", name: "Deployments", modes: ADV, seedScan: true },
  { path: "/simulator", name: "Simulator", modes: ADV },
  {
    path: "/activity-timeline",
    name: "Activity Timeline",
    modes: ADV,
    seedScan: true,
  },
  { path: "/health", name: "Health", modes: ADV },
  { path: "/capabilities", name: "Capabilities", modes: ADV },
  { path: "/integrations", name: "Integrations", modes: ADV },
  { path: "/bundles", name: "Bundles & Sync", modes: ENT },
];

export const qaModes: QaMode[] = ALL;
