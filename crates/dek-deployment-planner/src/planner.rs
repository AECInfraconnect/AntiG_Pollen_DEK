// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 AEC Infraconnect

use dek_domain_schema::{
    control_level::ControlLevel,
    deployment_session::LocalizedText,
    feasibility::{
        ControlMethod, ControlMethodPlan, Enforceability, PolicyFeasibilityRequest, ProductMode,
    },
};

pub struct ControlLevelNegotiation {
    pub requested: ControlLevel,
    pub effective: ControlLevel,
    pub downgraded: bool,
    pub reason: LocalizedText,
    pub requires_user_confirmation: bool,
}

pub fn negotiate_control_level(
    requested: ControlLevel,
    enforceability: &Enforceability,
) -> ControlLevelNegotiation {
    let effective = match requested {
        ControlLevel::StrictDeny if enforceability.can_strict_deny => ControlLevel::StrictDeny,
        ControlLevel::StrictDeny if enforceability.can_enforce => ControlLevel::Enforce,
        ControlLevel::Enforce if enforceability.can_enforce => ControlLevel::Enforce,
        ControlLevel::Enforce if enforceability.can_require_approval => ControlLevel::Approval,
        ControlLevel::Approval if enforceability.can_require_approval => ControlLevel::Approval,
        ControlLevel::Warn if enforceability.can_warn => ControlLevel::Warn,
        _ if enforceability.can_observe => ControlLevel::Observe,
        _ => ControlLevel::Observe,
    };

    let downgraded = effective != requested;

    ControlLevelNegotiation {
        requested,
        effective,
        downgraded,
        reason: if downgraded {
            LocalizedText {
                en: "This device cannot fully enforce the requested level yet. POLLEK will use the strongest available safe mode.".into(),
                th: "เครื่องนี้ยัง enforce ตามระดับที่ขอได้ไม่เต็มที่ ระบบจะใช้ระดับที่ปลอดภัยและทำได้จริงที่สุด".into(),
            }
        } else {
            LocalizedText {
                en: "The requested control level is supported on this device.".into(),
                th: "เครื่องนี้รองรับ control level ที่เลือก".into(),
            }
        },
        requires_user_confirmation: downgraded,
    }
}

pub fn score_plan(req: &PolicyFeasibilityRequest, plan: &ControlMethodPlan) -> i32 {
    let mut score = 0;

    if plan.enforceability.can_enforce {
        score += 100;
    }
    if plan.enforceability.can_require_approval {
        score += 70;
    }
    if plan.enforceability.can_warn {
        score += 50;
    }
    if plan.enforceability.can_observe {
        score += 20;
    }

    if matches!(req.mode, ProductMode::DesktopSimple) {
        score += match plan.method {
            ControlMethod::AgentToolControl => 40,
            ControlMethod::AgentConfigWrapper => 30,
            ControlMethod::LocalApiControl => 25,
            ControlMethod::BrowserActivityMonitor => 15,
            ControlMethod::NetworkControl => -10,
            ControlMethod::ProcessObservation => 10,
            ControlMethod::ObserveOnly => 0,
        };
    }

    if matches!(req.mode, ProductMode::EnterpriseServer) {
        if matches!(plan.method, ControlMethod::NetworkControl) {
            score += 40;
        }
    }

    score
}
