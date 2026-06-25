import {
  LayoutDashboard,
  ShieldCheck,
  Activity,
  Users,
  Database,
  Wrench,
  Network,
  Search,
  Lightbulb,
  FileKey,
  ShieldAlert,
  Zap,
  Server,
  Puzzle,
  Settings,
  Cpu,
  Route,
} from "lucide-react";

export interface NavItem {
  id: string;
  en: string;
  th: string;
  href: string;
  icon: any;
  modes: string[];
  primary?: boolean;
}
export interface NavGroup {
  id: string;
  en: string;
  th: string;
  items: NavItem[];
}

const ALL: string[] = ["simple", "advanced", "enterprise"];
const ADV: string[] = ["advanced", "enterprise"];
const ENT: string[] = ["enterprise"];

export const NAV: NavGroup[] = [
  {
    id: "protect",
    en: "Protect",
    th: "ปกป้อง",
    items: [
      {
        id: "home",
        en: "Home",
        th: "หน้าหลัก",
        href: "/",
        icon: LayoutDashboard,
        modes: ALL,
      },
      {
        id: "scan",
        en: "Scan & Protect",
        th: "สแกน & ปกป้อง",
        href: "/protect",
        icon: ShieldCheck,
        modes: ALL,
      },
      {
        id: "activity",
        en: "Activity",
        th: "กิจกรรม",
        href: "/activity",
        icon: Activity,
        modes: ALL,
      },
    ],
  },
  {
    id: "inventory",
    en: "Inventory",
    th: "รายการที่พบ",
    items: [
      {
        id: "agents",
        en: "Agents & Models",
        th: "Agent & โมเดล",
        href: "/agents",
        icon: Users,
        modes: ALL,
      },
      {
        id: "entities",
        en: "Entities",
        th: "เอนทิตี",
        href: "/entities",
        icon: Users,
        modes: ALL,
      },
      {
        id: "resources",
        en: "Data Resources",
        th: "แหล่งข้อมูล",
        href: "/resources",
        icon: Database,
        modes: ALL,
      },
      {
        id: "tools",
        en: "Tools",
        th: "เครื่องมือ",
        href: "/tools",
        icon: Wrench,
        modes: ADV,
      },
      {
        id: "identities",
        en: "Identities",
        th: "ตัวตน",
        href: "/identities",
        icon: Network,
        modes: ADV,
      },
      {
        id: "discovery",
        en: "Auto Discovery",
        th: "ค้นหาอัตโนมัติ",
        href: "/discovery",
        icon: Search,
        modes: ALL,
      },
    ],
  },
  {
    id: "policies",
    en: "Policies",
    th: "นโยบาย",
    items: [
      {
        id: "suggestions",
        en: "Policy Suggestions",
        th: "นโยบายแนะนำ",
        href: "/policy-suggestions",
        icon: Lightbulb,
        modes: ALL,
      },
      {
        id: "policies",
        en: "Active Policies",
        th: "นโยบายที่ใช้งาน",
        href: "/policies",
        icon: FileKey,
        modes: ALL,
      },
      {
        id: "presets",
        en: "Policy Presets",
        th: "เทมเพลตนโยบาย",
        href: "/policy-presets",
        icon: ShieldCheck,
        modes: ADV,
      },
      {
        id: "simulator",
        en: "Simulator",
        th: "จำลอง",
        href: "/simulator",
        icon: Activity,
        modes: ADV,
      },
    ],
  },
  {
    id: "monitoring",
    en: "Monitoring",
    th: "การติดตาม",
    items: [
      {
        id: "alerts",
        en: "Alerts & Shadow AI",
        th: "แจ้งเตือน & Shadow AI",
        href: "/alerts",
        icon: ShieldAlert,
        modes: ALL,
      },
      {
        id: "audit",
        en: "Audit Logs",
        th: "บันทึกตรวจสอบ",
        href: "/audit",
        icon: Activity,
        modes: ALL,
      },
      {
        id: "cost",
        en: "Cost & Tokens",
        th: "ค่าใช้จ่าย & Token",
        href: "/cost-ledger",
        icon: Zap,
        modes: ALL,
      },
      {
        id: "decisions",
        en: "Decision Logs",
        th: "บันทึกการตัดสินใจ",
        href: "/decision-logs",
        icon: FileKey,
        modes: ADV,
      },
    ],
  },
  {
    id: "system",
    en: "System",
    th: "ระบบ",
    items: [
      {
        id: "capabilities",
        en: "Capabilities",
        th: "ความสามารถเครื่อง",
        href: "/capabilities",
        icon: Cpu,
        modes: ADV,
      },
      {
        id: "plugins",
        en: "Plugin Marketplace",
        th: "ปลั๊กอิน",
        href: "/plugin-marketplace",
        icon: Puzzle,
        modes: ADV,
      },
      {
        id: "integrations",
        en: "Integrations",
        th: "การเชื่อมต่อ",
        href: "/integrations",
        icon: Wrench,
        modes: ADV,
      },
      {
        id: "bundles",
        en: "Bundles & Sync",
        th: "Bundle & Sync",
        href: "/bundles",
        icon: Server,
        modes: ENT,
      },
      {
        id: "pdp",
        en: "PDP & Routing",
        th: "PDP & เส้นทาง",
        href: "/settings/pdp",
        icon: Route,
        modes: ENT,
      },
      {
        id: "settings",
        en: "Global Settings",
        th: "ตั้งค่าทั่วไป",
        href: "/settings",
        icon: Settings,
        modes: ALL,
      },
    ],
  },
];
