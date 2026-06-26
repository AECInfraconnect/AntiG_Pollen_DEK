export type PolicyFirstReasonCode =
  | "fully_protected"
  | "observe_only_no_local_control_method"
  | "observe_only_permission_required"
  | "observe_only_unsupported_os"
  | "needs_mcp_config_change"
  | "needs_browser_extension"
  | "needs_os_network_extension"
  | "needs_admin_privilege"
  | "needs_cloud_enrollment"
  | "simulator_only"
  | "warm_check_failed"
  | "contract_version_mismatch";

export const policyFirstMessages: Record<
  PolicyFirstReasonCode,
  { en: string; th: string }
> = {
  fully_protected: {
    en: "Protection is active with a real local control method.",
    th: "เปิดใช้การป้องกันด้วย control method จริงบนเครื่องนี้แล้ว",
  },
  observe_only_no_local_control_method: {
    en: "Pollek can observe this activity, but no local method can block it yet.",
    th: "Pollek สังเกตกิจกรรมนี้ได้ แต่ยังไม่มีวิธีบนเครื่องที่บล็อกได้จริง",
  },
  observe_only_permission_required: {
    en: "Observation is available. Real control needs permission or elevated setup.",
    th: "สังเกตได้แล้ว แต่การควบคุมจริงต้องได้รับสิทธิ์หรือ setup เพิ่ม",
  },
  observe_only_unsupported_os: {
    en: "This OS does not currently expose a supported local control method.",
    th: "OS นี้ยังไม่มี control method บนเครื่องที่รองรับ",
  },
  needs_mcp_config_change: {
    en: "Approve the Pollek MCP wrapper or proxy before tool-call enforcement.",
    th: "อนุมัติ Pollek MCP wrapper หรือ proxy ก่อนบังคับใช้ tool-call",
  },
  needs_browser_extension: {
    en: "Install the browser extension before browser AI sessions can be observed.",
    th: "ติดตั้ง browser extension ก่อนสังเกต AI session บน browser",
  },
  needs_os_network_extension: {
    en: "Install the OS network control method before real egress blocking.",
    th: "ติดตั้งตัวควบคุมเครือข่ายของ OS ก่อนบล็อก egress จริง",
  },
  needs_admin_privilege: {
    en: "Run the setup action with admin privileges.",
    th: "ทำ setup action ด้วยสิทธิ์ admin",
  },
  needs_cloud_enrollment: {
    en: "Cloud enrollment requires SPIFFE workload identity and OAuth authorization.",
    th: "การเชื่อม Cloud ต้องมี SPIFFE workload identity และ OAuth authorization",
  },
  simulator_only: {
    en: "This signal is simulated. Real blocking is not enabled.",
    th: "สัญญาณนี้เป็น simulation ยังไม่ได้เปิดการบล็อกจริง",
  },
  warm_check_failed: {
    en: "The control method is installed, but its warm check failed.",
    th: "ติดตั้ง control method แล้ว แต่ warm check ไม่ผ่าน",
  },
  contract_version_mismatch: {
    en: "Local and cloud contract versions are not compatible.",
    th: "contract version ของ Local และ Cloud ไม่เข้ากัน",
  },
};
