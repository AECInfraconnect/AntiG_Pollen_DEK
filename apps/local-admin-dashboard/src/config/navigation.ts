import {
  Activity,
  Bot,
  Cpu,
  FileKey,
  FolderTree,
  LayoutDashboard,
  Network,
  Puzzle,
  Route,
  ScanSearch,
  Server,
  Settings,
  ShieldAlert,
  ShieldCheck,
  SlidersHorizontal,
  Users,
  Zap,
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

const ALL: string[] = ["desktop_simple", "desktop_advanced", "enterprise_cloud"];
const ADV: string[] = ["desktop_advanced", "enterprise_cloud"];
const ENT: string[] = ["enterprise_cloud"];

export const NAV: NavGroup[] = [
  {
    id: "home",
    en: "Home",
    th: "หน้าหลัก",
    items: [
      {
        id: "overview",
        en: "Overview",
        th: "ภาพรวม",
        href: "/",
        icon: LayoutDashboard,
        modes: ALL,
        primary: true,
      },
    ],
  },
  {
    id: "registry",
    en: "Registry",
    th: "ทะเบียน",
    items: [
      {
        id: "agents",
        en: "Agents & Models",
        th: "เอเจนต์และโมเดล",
        href: "/agents",
        icon: Bot,
        modes: ALL,
      },
      {
        id: "tools-resources",
        en: "Tools & Resources",
        th: "เครื่องมือและทรัพยากร",
        href: "/tools",
        icon: FolderTree,
        modes: ALL,
      },
      {
        id: "identities",
        en: "Identities",
        th: "ตัวตน",
        href: "/identities",
        icon: Users,
        modes: ADV,
      },
      {
        id: "entity-graph",
        en: "Entity Graph",
        th: "กราฟความสัมพันธ์",
        href: "/entity-graph",
        icon: Network,
        modes: ADV,
      },
    ],
  },
  {
    id: "governance",
    en: "Governance",
    th: "การกำกับดูแล",
    items: [
      {
        id: "policies",
        en: "Policies",
        th: "นโยบาย",
        href: "/policies",
        icon: FileKey,
        modes: ALL,
      },
      {
        id: "policy-presets",
        en: "Policy Presets",
        th: "เทมเพลตนโยบาย",
        href: "/policy-presets",
        icon: ShieldCheck,
        modes: ADV,
      },
      {
        id: "deployments",
        en: "Deployments",
        th: "การบังคับใช้",
        href: "/deployments",
        icon: SlidersHorizontal,
        modes: ALL,
      },
      {
        id: "simulator",
        en: "Simulator",
        th: "จำลองสถานการณ์",
        href: "/simulator",
        icon: Route,
        modes: ADV,
      },
    ],
  },
  {
    id: "observe",
    en: "Observe",
    th: "ตรวจสอบ",
    items: [
      {
        id: "activity",
        en: "Activity",
        th: "กิจกรรม",
        href: "/activity-timeline",
        icon: Activity,
        modes: ALL,
      },
      {
        id: "alerts",
        en: "Alerts & Shadow AI",
        th: "แจ้งเตือน",
        href: "/alerts",
        icon: ShieldAlert,
        modes: ALL,
      },
      {
        id: "cost",
        en: "Cost & Tokens",
        th: "ค่าใช้จ่าย",
        href: "/cost-ledger",
        icon: Zap,
        modes: ALL,
      },
      {
        id: "health",
        en: "Health",
        th: "สุขภาพระบบ",
        href: "/health",
        icon: Cpu,
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
        id: "scan",
        en: "Scan & Discover",
        th: "สแกนและค้นหา",
        href: "/scan",
        icon: ScanSearch,
        modes: ALL,
      },
      {
        id: "integrations",
        en: "Integrations",
        th: "การเชื่อมต่อ",
        href: "/integrations",
        icon: Puzzle,
        modes: ADV,
      },
      {
        id: "bundles",
        en: "Bundles & Sync",
        th: "แพ็กเกจและซิงค์",
        href: "/bundles",
        icon: Server,
        modes: ENT,
      },
      {
        id: "settings",
        en: "Settings",
        th: "ตั้งค่า",
        href: "/settings",
        icon: Settings,
        modes: ALL,
      },
    ],
  },
];
