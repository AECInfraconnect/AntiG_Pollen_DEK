import {
  Activity,
  Bot,
  Cpu,
  Database,
  FileKey,
  FolderSearch,
  FolderTree,
  History,
  LayoutDashboard,
  ListChecks,
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
  Wrench,
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

const ALL: string[] = [
  "desktop_simple",
  "desktop_advanced",
  "enterprise_cloud",
];
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
    id: "activity",
    en: "AI Activity",
    th: "กิจกรรม AI",
    items: [
      {
        id: "scan",
        en: "Find AI Apps",
        th: "ค้นหา AI",
        href: "/scan",
        icon: ScanSearch,
        modes: ALL,
      },
      {
        id: "my-ai-apps",
        en: "My AI Apps",
        th: "AI ของฉัน",
        href: "/my-ai-apps",
        icon: Bot,
        modes: ALL,
      },
      {
        id: "ai-activity",
        en: "AI Activity",
        th: "กิจกรรม AI",
        href: "/activity",
        icon: Activity,
        modes: ALL,
      },
      {
        id: "create-rule",
        en: "Create Rule",
        th: "สร้างกฎ",
        href: "/protect",
        icon: ShieldCheck,
        modes: ALL,
      },
      {
        id: "allowed-blocked",
        en: "Allowed & Blocked",
        th: "อนุญาตและห้าม",
        href: "/allowed-blocked",
        icon: ListChecks,
        modes: ALL,
      },
      {
        id: "data-apps",
        en: "Data & Apps",
        th: "ไฟล์ เว็บ แอป",
        href: "/data-apps",
        icon: Database,
        modes: ALL,
      },
      {
        id: "setup",
        en: "Setup",
        th: "ตั้งค่า",
        href: "/setup",
        icon: Wrench,
        modes: ALL,
      },
      {
        id: "history",
        en: "History",
        th: "ประวัติย้อนหลัง",
        href: "/history",
        icon: History,
        modes: ALL,
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
        modes: ADV,
      },
      {
        id: "tools-resources",
        en: "Tools & Resources",
        th: "เครื่องมือและทรัพยากร",
        href: "/tools",
        icon: FolderTree,
        modes: ADV,
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
        en: "Relationship Map",
        th: "แผนผังความสัมพันธ์",
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
        modes: ADV,
      },
      {
        id: "policy-presets",
        en: "Policy Presets",
        th: "เทมเพลตกฎ",
        href: "/policy-presets",
        icon: ShieldCheck,
        modes: ADV,
      },
      {
        id: "deployments",
        en: "Deployments",
        th: "การใช้งานกฎ",
        href: "/deployments",
        icon: SlidersHorizontal,
        modes: ADV,
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
    en: "Advanced Observe",
    th: "ตรวจสอบขั้นสูง",
    items: [
      {
        id: "activity-timeline",
        en: "Activity Timeline",
        th: "ไทม์ไลน์กิจกรรม",
        href: "/activity-timeline",
        icon: Activity,
        modes: ADV,
      },
      {
        id: "alerts",
        en: "Alerts & Shadow AI",
        th: "แจ้งเตือน",
        href: "/alerts",
        icon: ShieldAlert,
        modes: ADV,
      },
      {
        id: "cost",
        en: "Cost & Tokens",
        th: "ค่าใช้จ่าย",
        href: "/cost-ledger",
        icon: Zap,
        modes: ADV,
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
        id: "capabilities",
        en: "Capabilities",
        th: "ความสามารถระบบ",
        href: "/capabilities",
        icon: FolderSearch,
        modes: ADV,
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
        th: "แพ็กเกจและซิงก์",
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
