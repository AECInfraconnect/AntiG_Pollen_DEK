import i18n from "i18next";
import { initReactI18next } from "react-i18next";

const resources = {
  en: {
    translation: {
      Dashboard: "Dashboard",
      Policies: "Policies",
      Connectors: "Connectors",
      Settings: "Settings",
      Simulator: "Simulator",
      "Decision Logs": "Decision Logs",
      Language: "Language",
      step: {
        agent: "1. Select AI Agent",
        policy: "2. Select Protection Policy",
        level: "3. Confirm Control Level"
      },
      control: {
        observe: "Observe Only", observe_desc: "Log activities without blocking.",
        warn: "Warn & Ask", warn_desc: "Prompt user for approval before risky actions.",
        enforce: "Enforce & Block", enforce_desc: "Silently block all unauthorized actions."
      },
      feasibility: {
        title: "Feasibility Check",
        fully_enforceable: "This policy can be fully enforced.",
        partially_enforceable: "Some rules cannot be enforced (fallback to observe).",
        needs_upgrade: "Required capability is missing."
      },
      "common.next": "Next",
      "common.review": "Review",
      "simple.protect_now": "Protect now",
      "upgrade.install": "Install",
      "upgrade.learn_more": "How to enable",
      "mode.simple": "Simple",
      "mode.advanced": "Advanced",
      "mode.enterprise": "Enterprise",
      "wizard.select_agent_required": "Please select an agent first.",
      "wizard.select_policy_required": "Please select a policy first.",
    },
  },
  th: {
    translation: {
      Dashboard: "หน้าหลัก",
      Policies: "นโยบาย",
      Connectors: "การเชื่อมต่อ",
      Settings: "การตั้งค่า",
      Simulator: "จำลองสถานการณ์",
      "Decision Logs": "บันทึกการตัดสินใจ",
      Language: "ภาษา",
      step: {
        agent: "1. เลือก AI Agent ที่ต้องการคุม",
        policy: "2. เลือกนโยบายคุ้มครอง (Policy)",
        level: "3. ยืนยันระดับความเข้มงวด"
      },
      control: {
        observe: "แค่สังเกตการณ์ (Observe)", observe_desc: "บันทึกประวัติโดยไม่บล็อกการทำงาน",
        warn: "เตือนและถาม (Warn)", warn_desc: "แจ้งเตือนให้ผู้ใช้อนุมัติก่อนทำสิ่งที่มีความเสี่ยง",
        enforce: "บังคับและบล็อก (Enforce)", enforce_desc: "บล็อกพฤติกรรมที่ผิดกฎทันทีโดยไม่รบกวนผู้ใช้"
      },
      feasibility: {
        title: "ผลประเมินความเข้ากันได้",
        fully_enforceable: "นโยบายนี้สามารถบังคับใช้ได้ 100% บนเครื่องนี้",
        partially_enforceable: "บางกฎไม่สามารถบังคับใช้ได้ (จะทำได้แค่บันทึกประวัติ)",
        needs_upgrade: "เครื่องของคุณขาดองค์ประกอบที่จำเป็นสำหรับนโยบายนี้"
      },
      "common.next": "ถัดไป",
      "common.review": "ตรวจสอบ",
      "simple.protect_now": "ปกป้องเลย",
      "upgrade.install": "ติดตั้ง",
      "upgrade.learn_more": "วิธีเปิดใช้งาน",
      "mode.simple": "ง่าย",
      "mode.advanced": "ขั้นสูง",
      "mode.enterprise": "องค์กร",
      "wizard.select_agent_required": "กรุณาเลือก Agent ก่อน",
      "wizard.select_policy_required": "กรุณาเลือกนโยบายก่อน",
    },
  },
};

i18n.use(initReactI18next).init({
  resources,
  lng: localStorage.getItem("i18nextLng") || "en",
  fallbackLng: "en",
  interpolation: {
    escapeValue: false,
  },
});

export default i18n;
