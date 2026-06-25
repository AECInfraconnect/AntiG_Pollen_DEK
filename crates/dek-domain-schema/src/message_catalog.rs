use crate::deployment_session::LocalizedText;

pub struct MessageCatalog;

impl MessageCatalog {
    pub fn capability_warm_check() -> LocalizedText {
        LocalizedText {
            en: "Capability Warm Check".into(),
            th: "ตรวจสอบสถานะการทำงาน".into(),
        }
    }

    pub fn capability_warm_check_detail() -> LocalizedText {
        LocalizedText {
            en: "Performing capability warm check...".into(),
            th: "กำลังตรวจสอบความพร้อมการบังคับใช้...".into(),
        }
    }

    pub fn ok() -> LocalizedText {
        LocalizedText {
            en: "OK".into(),
            th: "พร้อมใช้งาน".into(),
        }
    }

    pub fn failed() -> LocalizedText {
        LocalizedText {
            en: "Failed".into(),
            th: "ล้มเหลว".into(),
        }
    }
}
