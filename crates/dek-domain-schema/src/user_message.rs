// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use crate::deployment_session::LocalizedText;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MessageSeverity {
    Info,
    Success,
    Warning,
    Error,
    Critical,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct UserActionHint {
    pub action_id: String,
    pub label_en: String,
    pub label_th: String,
    pub requires_admin: bool,
    pub reversible: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct UserVisibleMessage {
    pub message_id: String,
    pub severity: MessageSeverity,
    pub title_en: String,
    pub title_th: String,
    pub body_en: String,
    pub body_th: String,
    pub reason_code: String,
    pub user_action: Option<UserActionHint>,
    pub technical_detail: Option<String>,
    pub learn_more_path: Option<String>,
}

pub fn localized_reason(reason_code: &str) -> LocalizedText {
    let (en, th) = match reason_code {
        "fully_protected" => (
            "This policy is active and can block unsafe actions for this agent.",
            "นโยบายนี้ทำงานแล้วและสามารถบล็อกการกระทำที่ไม่ปลอดภัยของ Agent นี้ได้",
        ),
        "observe_only_no_local_control_method" => (
            "Pollek can monitor this activity, but cannot block it on this device yet.",
            "Pollek ติดตามกิจกรรมนี้ได้ แต่ยังบล็อกบนเครื่องนี้ไม่ได้",
        ),
        "observe_only_permission_required" => (
            "Pollek can observe this activity, but blocking requires additional permission.",
            "Pollek สังเกตกิจกรรมนี้ได้ แต่การบล็อกต้องใช้สิทธิ์เพิ่มเติม",
        ),
        "needs_os_network_extension" => (
            "To block real network egress, install or enable device-level network control.",
            "หากต้องการบล็อกเครือข่ายจริง ต้องติดตั้งหรือเปิดใช้ตัวควบคุมเครือข่ายระดับเครื่อง",
        ),
        "needs_mcp_config_change" => (
            "Pollek needs permission to update this agent's MCP configuration.",
            "Pollek ต้องได้รับอนุญาตให้แก้ไขค่า MCP configuration ของ Agent นี้",
        ),
        "needs_browser_extension" => (
            "Browser AI activity requires the Pollek browser extension.",
            "การติดตาม AI บน Browser ต้องติดตั้ง Pollek Browser Extension",
        ),
        "warm_check_failed" => (
            "Protection was not activated because the final safety check failed.",
            "ยังไม่เปิดใช้งานการป้องกัน เพราะการตรวจสอบความพร้อมขั้นสุดท้ายไม่ผ่าน",
        ),
        "simulator_only" => (
            "This is simulated traffic for testing. Real blocking is not enabled.",
            "นี่คือทราฟฟิกจำลองสำหรับทดสอบ ยังไม่ได้บล็อกทราฟฟิกจริง",
        ),
        "contract_version_mismatch" => (
            "This Local Kit and Cloud contract are not compatible.",
            "Local Kit และ Cloud Contract เวอร์ชันนี้ไม่ตรงกัน",
        ),
        _ => (
            "Pollek needs more setup before this protection can be enforced.",
            "Pollek ต้องตั้งค่าเพิ่มเติมก่อนจะบังคับใช้นโยบายนี้ได้",
        ),
    };

    LocalizedText {
        en: en.to_string(),
        th: th.to_string(),
    }
}
